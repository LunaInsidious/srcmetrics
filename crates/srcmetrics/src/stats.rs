//! Statistical analysis of analysis results (PLAN.md §20 Phase 4, ADR-0018).

use crate::metrics::{self, Scope};
use crate::result::{AnalysisResult, FileResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One unit of analysis (a file or a function) with its metric values.
#[derive(Debug, Clone, PartialEq)]
pub struct Unit {
    pub language: String,
    pub metrics: BTreeMap<String, Option<f64>>,
}

/// The unit of statistical analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitScope {
    File,
    Function,
}

impl UnitScope {
    fn scope(self) -> Scope {
        match self {
            UnitScope::File => Scope::File,
            UnitScope::Function => Scope::Function,
        }
    }
}

/// The files, or the functions, of the successfully analyzed files of `results`.
pub fn units(results: &[AnalysisResult], scope: UnitScope) -> Vec<Unit> {
    let mut units = vec![];
    for file in results.iter().flat_map(|r| &r.files) {
        if let FileResult::Ok {
            language,
            metrics,
            functions,
            ..
        } = file
        {
            let unit = |m: &BTreeMap<String, Option<f64>>| Unit {
                language: language.clone(),
                metrics: m.clone(),
            };
            match scope {
                UnitScope::File => units.push(unit(&metrics.metrics)),
                UnitScope::Function => {
                    units.extend(functions.iter().map(|f| unit(&f.metrics.metrics)))
                }
            }
        }
    }
    units
}

/// Metric ids defined at `scope`, in definition order.
pub fn metric_ids(scope: UnitScope) -> Vec<&'static str> {
    metrics::definitions()
        .into_iter()
        .filter(|d| d.scopes.contains(&scope.scope()))
        .map(|d| d.id)
        .collect()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Summary {
    /// Units with a value.
    pub n: usize,
    /// Units whose value is null.
    pub missing: usize,
    pub mean: Option<f64>,
    /// Sample standard deviation (n - 1).
    pub sd: Option<f64>,
    pub min: Option<f64>,
    pub q1: Option<f64>,
    pub median: Option<f64>,
    pub q3: Option<f64>,
    pub max: Option<f64>,
}

impl Summary {
    pub fn of(values: &[f64], missing: usize) -> Summary {
        let mut sorted = values.to_vec();
        sorted.sort_by(f64::total_cmp);
        let n = sorted.len();
        let mean = (n > 0).then(|| sorted.iter().sum::<f64>() / n as f64);
        let sd = mean
            .filter(|_| n > 1)
            .map(|m| (sorted.iter().map(|v| (v - m).powi(2)).sum::<f64>() / (n - 1) as f64).sqrt());
        let q = |p| quantile(&sorted, p);
        Summary {
            n,
            missing,
            mean,
            sd,
            min: sorted.first().copied(),
            q1: q(0.25),
            median: q(0.5),
            q3: q(0.75),
            max: sorted.last().copied(),
        }
    }
}

/// Quantile of sorted values by linear interpolation (R type 7).
fn quantile(sorted: &[f64], p: f64) -> Option<f64> {
    let last = sorted.len().checked_sub(1)?;
    let h = p * last as f64;
    let (lower, upper) = (h.floor() as usize, h.ceil() as usize);
    Some(sorted[lower] + (h - lower as f64) * (sorted[upper] - sorted[lower]))
}

/// Pearson correlation; `None` for fewer than 3 pairs or zero variance.
pub fn pearson(x: &[f64], y: &[f64]) -> Option<f64> {
    let n = x.len();
    if n < 3 {
        return None;
    }
    let (mx, my) = (
        x.iter().sum::<f64>() / n as f64,
        y.iter().sum::<f64>() / n as f64,
    );
    let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
    for (a, b) in x.iter().zip(y) {
        sxy += (a - mx) * (b - my);
        sxx += (a - mx).powi(2);
        syy += (b - my).powi(2);
    }
    (sxx > 0.0 && syy > 0.0).then(|| sxy / (sxx * syy).sqrt())
}

/// Spearman rank correlation (Pearson of average ranks).
pub fn spearman(x: &[f64], y: &[f64]) -> Option<f64> {
    pearson(&ranks(x), &ranks(y))
}

/// 1-based ranks; tied values get the average of their ranks.
pub fn ranks(values: &[f64]) -> Vec<f64> {
    let mut order: Vec<usize> = (0..values.len()).collect();
    order.sort_by(|a, b| values[*a].total_cmp(&values[*b]));
    let mut ranks = vec![0.0; values.len()];
    let mut start = 0;
    while start < order.len() {
        let end = start
            + order[start..]
                .iter()
                .take_while(|i| values[**i] == values[order[start]])
                .count();
        let average = (start + end + 1) as f64 / 2.0;
        for i in &order[start..end] {
            ranks[*i] = average;
        }
        start = end;
    }
    ranks
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Correlation {
    pub a: String,
    pub b: String,
    /// Units where both values are present.
    pub n: usize,
    pub pearson: Option<f64>,
    pub spearman: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Report {
    pub units: usize,
    pub metrics: BTreeMap<String, Summary>,
    /// Language -> metric -> summary (language baselines).
    pub by_language: BTreeMap<String, BTreeMap<String, Summary>>,
    /// Every pair of metrics, in id order.
    pub correlations: Vec<Correlation>,
}

impl Report {
    pub fn of(units: &[Unit], ids: &[&str]) -> Report {
        // Column per metric: one value (or None) per unit.
        let columns: Vec<Vec<Option<f64>>> = ids
            .iter()
            .map(|id| {
                units
                    .iter()
                    .map(|u| u.metrics.get(*id).copied().flatten())
                    .collect()
            })
            .collect();
        let summarize = |rows: &[usize]| -> BTreeMap<String, Summary> {
            ids.iter()
                .zip(&columns)
                .map(|(id, column)| {
                    let values: Vec<f64> = rows.iter().filter_map(|r| column[*r]).collect();
                    (
                        id.to_string(),
                        Summary::of(&values, rows.len() - values.len()),
                    )
                })
                .collect()
        };
        let mut languages: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
        for (row, u) in units.iter().enumerate() {
            languages.entry(&u.language).or_default().push(row);
        }
        let mut correlations = vec![];
        for (i, a) in ids.iter().enumerate() {
            for (j, b) in ids.iter().enumerate().skip(i + 1) {
                let (x, y): (Vec<f64>, Vec<f64>) = columns[i]
                    .iter()
                    .zip(&columns[j])
                    .filter_map(|(x, y)| Some(((*x)?, (*y)?)))
                    .unzip();
                correlations.push(Correlation {
                    a: a.to_string(),
                    b: b.to_string(),
                    n: x.len(),
                    pearson: pearson(&x, &y),
                    spearman: spearman(&x, &y),
                });
            }
        }
        Report {
            units: units.len(),
            metrics: summarize(&(0..units.len()).collect::<Vec<_>>()),
            by_language: languages
                .into_iter()
                .map(|(l, rows)| (l.to_string(), summarize(&rows)))
                .collect(),
            correlations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Option<f64>, b: f64) {
        let a = a.expect("value");
        assert!((a - b).abs() < 1e-9, "{a} != {b}");
    }

    #[test]
    fn describes_values_with_type7_quantiles() {
        let s = Summary::of(&[4.0, 1.0, 3.0, 2.0], 1);
        assert_eq!((s.n, s.missing), (4, 1));
        close(s.mean, 2.5);
        close(s.sd, (5.0f64 / 3.0).sqrt());
        close(s.min, 1.0);
        close(s.q1, 1.75);
        close(s.median, 2.5);
        close(s.q3, 3.25);
        close(s.max, 4.0);
    }

    #[test]
    fn small_samples_have_null_statistics() {
        let empty = Summary::of(&[], 2);
        assert_eq!((empty.n, empty.mean, empty.median), (0, None, None));
        let one = Summary::of(&[5.0], 0);
        assert_eq!((one.mean, one.sd), (Some(5.0), None));
    }

    #[test]
    fn pearson_and_spearman() {
        close(pearson(&[1.0, 2.0, 3.0, 4.0], &[2.0, 4.0, 6.0, 8.0]), 1.0);
        close(pearson(&[1.0, 2.0, 3.0], &[3.0, 2.0, 1.0]), -1.0);
        assert_eq!(pearson(&[1.0, 2.0, 3.0], &[5.0, 5.0, 5.0]), None);
        assert_eq!(pearson(&[1.0, 2.0], &[1.0, 2.0]), None);
        // Monotonic but not linear: Spearman is 1.
        close(
            spearman(&[1.0, 2.0, 3.0, 4.0], &[1.0, 4.0, 9.0, 100.0]),
            1.0,
        );
    }

    #[test]
    fn ranks_average_ties() {
        assert_eq!(ranks(&[10.0, 20.0, 10.0, 30.0]), vec![1.5, 3.0, 1.5, 4.0]);
    }

    fn unit(language: &str, values: &[(&str, Option<f64>)]) -> Unit {
        Unit {
            language: language.to_string(),
            metrics: values.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
        }
    }

    #[test]
    fn report_uses_pairwise_complete_observations_and_groups_by_language() {
        let units = vec![
            unit("c", &[("a", Some(1.0)), ("b", Some(2.0))]),
            unit("c", &[("a", Some(2.0)), ("b", Some(4.0))]),
            unit("python", &[("a", Some(3.0)), ("b", None)]),
            unit("python", &[("a", Some(4.0)), ("b", Some(8.0))]),
        ];
        let report = Report::of(&units, &["a", "b"]);
        assert_eq!(report.units, 4);
        assert_eq!((report.metrics["b"].n, report.metrics["b"].missing), (3, 1));
        close(report.by_language["python"]["a"].mean, 3.5);
        let ab = &report.correlations[0];
        assert_eq!((ab.a.as_str(), ab.b.as_str(), ab.n), ("a", "b", 3));
        close(ab.pearson, 1.0);
        close(ab.spearman, 1.0);
    }
}
