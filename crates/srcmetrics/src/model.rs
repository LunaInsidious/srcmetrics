//! Experimental readability model: ridge regression fitted on user-provided labels (ADR-0019).
//! There are no built-in weights: a model exists only when trained on labels (design principle P5).

use crate::result::{AnalysisResult, FileResult};
use crate::stats::{UnitScope, metric_ids};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// A model error, with a message saying what to do next.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelError(pub String);

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ModelError {}

/// A training example: the metrics of a file or function and its label.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub metrics: BTreeMap<String, Option<f64>>,
    pub score: f64,
}

/// A label from the labels CSV (`path,function,score`); `function` is empty for a file label.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Label {
    pub path: String,
    #[serde(deserialize_with = "empty_as_none")]
    pub function: Option<String>,
    pub score: f64,
}

fn empty_as_none<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    let s = String::deserialize(d)?;
    Ok((!s.is_empty()).then_some(s))
}

pub fn read_labels(csv_text: &str) -> Result<Vec<Label>, ModelError> {
    let mut reader = csv::Reader::from_reader(csv_text.as_bytes());
    let headers = reader
        .headers()
        .map_err(|e| ModelError(format!("labels CSV: {e}")))?
        .clone();
    if headers.iter().collect::<Vec<_>>() != ["path", "function", "score"] {
        return Err(ModelError(
            "labels CSV must have the header `path,function,score` (function empty for file labels)".into(),
        ));
    }
    reader
        .deserialize()
        .enumerate()
        .map(|(i, r)| {
            let row = i + 2;
            let label: Label = r.map_err(|e| ModelError(format!("labels CSV row {row}: {e}")))?;
            if !label.score.is_finite() {
                return Err(ModelError(format!(
                    "labels CSV row {row}: score must be a finite number, got {}",
                    label.score
                )));
            }
            Ok(label)
        })
        .collect()
}

pub const EXPERIMENTAL_NOTICE: &str = "Experimental: fitted only on the labels provided by the user. \
                                       srcmetrics has no built-in readability weights.";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub notice: String,
    /// The unit the model scores (files or functions).
    pub scope: UnitScope,
    pub features: Vec<String>,
    /// Feature means and standard deviations of the training data (for standardization).
    pub means: Vec<f64>,
    pub sds: Vec<f64>,
    /// Coefficients of the standardized features.
    pub coefficients: Vec<f64>,
    pub intercept: f64,
    pub lambda: f64,
    /// Number of labelled units used for fitting.
    pub n: usize,
    /// Coefficient of determination on the training data.
    pub r_squared: f64,
    /// Root mean squared error of 5-fold cross-validation; `None` with fewer than 5 labels.
    pub cv_rmse: Option<f64>,
    /// Features not used, with the reason.
    pub excluded_features: BTreeMap<String, String>,
}

const FOLDS: usize = 5;

/// Fits a model on the units of `results` that have a label. All labels must be of one scope
/// (all file labels or all function labels) and must match exactly one unit.
pub fn train(
    results: &[AnalysisResult],
    labels: &[Label],
    requested: Option<&[String]>,
    lambda: f64,
) -> Result<Model, ModelError> {
    let scope = match (
        labels.iter().all(|l| l.function.is_none()),
        labels.iter().all(|l| l.function.is_some()),
    ) {
        (true, _) => UnitScope::File,
        (_, true) => UnitScope::Function,
        _ => {
            return Err(ModelError(
                "labels mix file and function labels; use one kind per model".into(),
            ));
        }
    };
    let rows = labels
        .iter()
        .map(|label| {
            Ok(Row {
                metrics: find_unit(results, label)?.clone(),
                score: label.score,
            })
        })
        .collect::<Result<Vec<_>, ModelError>>()?;
    fit(&rows, scope, &metric_ids(scope), requested, lambda)
}

/// The metrics of the unit a label refers to.
fn find_unit<'a>(
    results: &'a [AnalysisResult],
    label: &Label,
) -> Result<&'a BTreeMap<String, Option<f64>>, ModelError> {
    let file = results
        .iter()
        .flat_map(|r| &r.files)
        .find(|f| f.path() == label.path)
        .ok_or_else(|| {
            ModelError(format!(
                "label for {}: no such file in the analysis results",
                label.path
            ))
        })?;
    let FileResult::Ok {
        metrics, functions, ..
    } = file
    else {
        return Err(ModelError(format!(
            "label for {}: the file failed to analyze",
            label.path
        )));
    };
    let Some(name) = &label.function else {
        return Ok(&metrics.metrics);
    };
    let mut matching = functions
        .iter()
        .filter(|f| f.name.as_deref() == Some(name.as_str()));
    match (matching.next(), matching.next()) {
        (Some(f), None) => Ok(&f.metrics.metrics),
        (None, _) => Err(ModelError(format!(
            "label for {}: no function named {name}",
            label.path
        ))),
        (Some(_), Some(_)) => Err(ModelError(format!(
            "label for {}: several functions are named {name}; label a file with unique function names",
            label.path
        ))),
    }
}

/// A predicted score for a file or function.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Prediction {
    pub path: String,
    pub function: Option<String>,
    pub start_line: Option<usize>,
    pub score: Option<f64>,
    /// Why `score` is null.
    pub unavailable: Option<String>,
}

/// Scores every unit of the model's scope in `results`.
pub fn predict_all(model: &Model, results: &[AnalysisResult]) -> Vec<Prediction> {
    let mut predictions = vec![];
    for file in results.iter().flat_map(|r| &r.files) {
        let (path, metrics, functions) = match file {
            FileResult::Ok {
                path,
                metrics,
                functions,
                ..
            } => (path, metrics, functions),
            FileResult::Error { path, error, .. } => {
                predictions.push(Prediction {
                    path: path.clone(),
                    function: None,
                    start_line: None,
                    score: None,
                    unavailable: Some(format!("the file failed to analyze: {error}")),
                });
                continue;
            }
        };
        let predict = |function: Option<String>, start_line, m| {
            let (score, unavailable) = match model.predict(m) {
                Ok(s) => (Some(s), None),
                Err(reason) => (None, Some(reason)),
            };
            Prediction {
                path: path.clone(),
                function,
                start_line,
                score,
                unavailable,
            }
        };
        match model.scope {
            UnitScope::File => predictions.push(predict(None, None, &metrics.metrics)),
            UnitScope::Function => predictions.extend(
                functions
                    .iter()
                    .map(|f| predict(f.name.clone(), Some(f.start_line), &f.metrics.metrics)),
            ),
        }
    }
    predictions
}

/// Fits a ridge regression. `candidates` are the metric ids considered when `requested` is `None`;
/// unusable candidates (missing for a labelled unit, or constant) are excluded and reported.
/// Requested features must all be usable.
pub fn fit(
    rows: &[Row],
    scope: UnitScope,
    candidates: &[&str],
    requested: Option<&[String]>,
    lambda: f64,
) -> Result<Model, ModelError> {
    if rows.len() < 2 {
        return Err(ModelError(format!(
            "{} labelled unit(s); at least 2 are needed to fit a model",
            rows.len()
        )));
    }
    if rows.iter().all(|r| r.score == rows[0].score) {
        return Err(ModelError(
            "all labels are equal; there is nothing to learn".into(),
        ));
    }
    if lambda.is_nan() || lambda < 0.0 {
        return Err(ModelError(format!("lambda must be >= 0, got {lambda}")));
    }
    let (features, excluded) = select_features(rows, candidates, requested)?;
    let x: Vec<Vec<f64>> = rows
        .iter()
        .map(|r| {
            features
                .iter()
                .map(|f| r.metrics[f].expect("usable"))
                .collect()
        })
        .collect();
    let y: Vec<f64> = rows.iter().map(|r| r.score).collect();
    let fitted = Ridge::fit(&x, &y, lambda)?;
    let r_squared = r_squared(
        &y,
        &x.iter().map(|xi| fitted.predict(xi)).collect::<Vec<_>>(),
    );
    let cv_rmse = (rows.len() >= FOLDS)
        .then(|| cross_validate(&x, &y, lambda))
        .transpose()?;
    Ok(Model {
        notice: EXPERIMENTAL_NOTICE.into(),
        scope,
        features,
        means: fitted.means,
        sds: fitted.sds,
        coefficients: fitted.coefficients,
        intercept: fitted.intercept,
        lambda,
        n: rows.len(),
        r_squared,
        cv_rmse,
        excluded_features: excluded,
    })
}

/// The usable features, and the excluded candidates with the reason. Requested features must all
/// be usable; unusable candidates are excluded.
fn select_features(
    rows: &[Row],
    candidates: &[&str],
    requested: Option<&[String]>,
) -> Result<(Vec<String>, BTreeMap<String, String>), ModelError> {
    let names: Vec<&str> = match requested {
        Some(r) => r.iter().map(String::as_str).collect(),
        None => candidates.to_vec(),
    };
    let mut features = vec![];
    let mut excluded = BTreeMap::new();
    for name in names {
        match unusable(rows, name) {
            None => features.push(name.to_string()),
            Some(reason) if requested.is_some() => {
                return Err(ModelError(format!(
                    "feature {name} cannot be used: {reason}; choose other features"
                )));
            }
            Some(reason) => {
                excluded.insert(name.to_string(), reason);
            }
        }
    }
    if features.is_empty() {
        return Err(ModelError(
            "no usable features; label more units or choose features explicitly".into(),
        ));
    }
    Ok((features, excluded))
}

/// Why a feature cannot be used for these rows, if it cannot.
fn unusable(rows: &[Row], name: &str) -> Option<String> {
    let values: Vec<Option<f64>> = rows
        .iter()
        .map(|r| r.metrics.get(name).copied().flatten())
        .collect();
    let missing = values.iter().filter(|v| v.is_none()).count();
    if missing > 0 {
        return Some(format!("missing for {missing} labelled unit(s)"));
    }
    let first = values[0];
    values
        .iter()
        .all(|v| *v == first)
        .then(|| "constant over the labelled units".to_string())
}

impl Model {
    /// Loads a model written by `srcmetrics model train`, checking that its vectors are consistent.
    pub fn from_json(text: &str) -> Result<Model, ModelError> {
        let model: Model = serde_json::from_str(text).map_err(|e| {
            ModelError(format!(
                "not a model ({e}); create one with `srcmetrics model train`"
            ))
        })?;
        let lengths = [
            model.features.len(),
            model.means.len(),
            model.sds.len(),
            model.coefficients.len(),
        ];
        if lengths.iter().any(|l| *l != lengths[0]) {
            return Err(ModelError(format!(
                "not a model: features, means, sds and coefficients lengths differ ({lengths:?}); \
                 create one with `srcmetrics model train`"
            )));
        }
        Ok(model)
    }

    /// Predicted score, or why it cannot be computed.
    pub fn predict(&self, metrics: &BTreeMap<String, Option<f64>>) -> Result<f64, String> {
        let x = self
            .features
            .iter()
            .map(|f| {
                metrics
                    .get(f)
                    .copied()
                    .flatten()
                    .ok_or_else(|| format!("{f} is null"))
            })
            .collect::<Result<Vec<f64>, String>>()?;
        Ok(self.as_ridge().predict(&x))
    }

    fn as_ridge(&self) -> Ridge {
        Ridge {
            means: self.means.clone(),
            sds: self.sds.clone(),
            coefficients: self.coefficients.clone(),
            intercept: self.intercept,
        }
    }
}

/// Ridge regression on standardized features; the intercept is not penalized.
struct Ridge {
    means: Vec<f64>,
    sds: Vec<f64>,
    coefficients: Vec<f64>,
    intercept: f64,
}

impl Ridge {
    fn fit(x: &[Vec<f64>], y: &[f64], lambda: f64) -> Result<Ridge, ModelError> {
        let (n, p) = (x.len() as f64, x[0].len());
        let means: Vec<f64> = (0..p)
            .map(|j| x.iter().map(|r| r[j]).sum::<f64>() / n)
            .collect();
        let sds: Vec<f64> = (0..p)
            .map(|j| (x.iter().map(|r| (r[j] - means[j]).powi(2)).sum::<f64>() / n).sqrt())
            .collect();
        // A feature constant within this data carries no information: its standardized column is 0.
        let z: Vec<Vec<f64>> = x
            .iter()
            .map(|r| {
                (0..p)
                    .map(|j| {
                        if sds[j] > 0.0 {
                            (r[j] - means[j]) / sds[j]
                        } else {
                            0.0
                        }
                    })
                    .collect()
            })
            .collect();
        let intercept = y.iter().sum::<f64>() / n;
        // Normal equations: (Z'Z + lambda I) beta = Z'(y - mean(y)).
        let mut a = vec![vec![0.0; p]; p];
        let mut b = vec![0.0; p];
        for (zi, yi) in z.iter().zip(y) {
            for j in 0..p {
                b[j] += zi[j] * (yi - intercept);
                for k in 0..p {
                    a[j][k] += zi[j] * zi[k];
                }
            }
        }
        for (j, row) in a.iter_mut().enumerate() {
            row[j] += lambda;
        }
        let coefficients = solve(a, b).ok_or_else(|| {
            ModelError(
                "the features are linearly dependent; use a larger lambda or fewer features".into(),
            )
        })?;
        Ok(Ridge {
            means,
            sds,
            coefficients,
            intercept,
        })
    }

    fn predict(&self, x: &[f64]) -> f64 {
        let standardized = x.iter().enumerate().map(|(j, v)| {
            if self.sds[j] > 0.0 {
                (v - self.means[j]) / self.sds[j]
            } else {
                0.0
            }
        });
        self.intercept
            + standardized
                .zip(&self.coefficients)
                .map(|(z, c)| z * c)
                .sum::<f64>()
    }
}

/// Solves `a x = b` by Gaussian elimination with partial pivoting; `None` if `a` is singular.
fn solve(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Option<Vec<f64>> {
    let p = b.len();
    for col in 0..p {
        let pivot = (col..p).max_by(|i, j| a[*i][col].abs().total_cmp(&a[*j][col].abs()))?;
        if a[pivot][col].abs() < 1e-12 {
            return None;
        }
        a.swap(col, pivot);
        b.swap(col, pivot);
        let (upper, lower) = a.split_at_mut(col + 1);
        let pivot_row = &upper[col];
        for (offset, row) in lower.iter_mut().enumerate() {
            let factor = row[col] / pivot_row[col];
            for (value, pivot_value) in row[col..].iter_mut().zip(&pivot_row[col..]) {
                *value -= factor * pivot_value;
            }
            b[col + 1 + offset] -= factor * b[col];
        }
    }
    let mut x = vec![0.0; p];
    for row in (0..p).rev() {
        let sum: f64 = (row + 1..p).map(|k| a[row][k] * x[k]).sum();
        x[row] = (b[row] - sum) / a[row][row];
    }
    Some(x)
}

/// R²; the labels are not all equal (checked by `fit`), so the total sum of squares is positive.
fn r_squared(y: &[f64], predicted: &[f64]) -> f64 {
    let mean = y.iter().sum::<f64>() / y.len() as f64;
    let total: f64 = y.iter().map(|v| (v - mean).powi(2)).sum();
    let residual: f64 = y.iter().zip(predicted).map(|(v, p)| (v - p).powi(2)).sum();
    1.0 - residual / total
}

/// k-fold cross-validation RMSE; unit i belongs to fold i mod k (deterministic).
fn cross_validate(x: &[Vec<f64>], y: &[f64], lambda: f64) -> Result<f64, ModelError> {
    let mut squared = 0.0;
    for fold in 0..FOLDS {
        let (train, test): (Vec<usize>, Vec<usize>) = (0..y.len()).partition(|i| i % FOLDS != fold);
        let pick = |rows: &[usize]| -> (Vec<Vec<f64>>, Vec<f64>) {
            (
                rows.iter().map(|i| x[*i].clone()).collect(),
                rows.iter().map(|i| y[*i]).collect(),
            )
        };
        let (train_x, train_y) = pick(&train);
        let model = Ridge::fit(&train_x, &train_y, lambda)?;
        squared += test
            .iter()
            .map(|i| (model.predict(&x[*i]) - y[*i]).powi(2))
            .sum::<f64>();
    }
    Ok((squared / y.len() as f64).sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(values: &[(&str, Option<f64>)], score: f64) -> Row {
        Row {
            metrics: values.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            score,
        }
    }

    /// score = 2a - b + 3 exactly.
    fn linear_rows() -> Vec<Row> {
        let points = [
            (1.0, 5.0),
            (2.0, 1.0),
            (3.0, 4.0),
            (4.0, 2.0),
            (5.0, 7.0),
            (6.0, 3.0),
            (7.0, 6.0),
        ];
        points
            .iter()
            .map(|(a, b)| {
                row(
                    &[("a", Some(*a)), ("b", Some(*b)), ("c", Some(1.0))],
                    2.0 * a - b + 3.0,
                )
            })
            .collect()
    }

    #[test]
    fn fits_a_linear_relation_and_reports_its_quality() {
        let model = fit(
            &linear_rows(),
            UnitScope::File,
            &["a", "b", "c"],
            None,
            1e-9,
        )
        .unwrap();
        assert_eq!(model.features, vec!["a", "b"]);
        assert_eq!(
            model.excluded_features["c"],
            "constant over the labelled units"
        );
        assert!((model.r_squared - 1.0).abs() < 1e-6, "{}", model.r_squared);
        let predicted = model.predict(&linear_rows()[0].metrics).unwrap();
        assert!((predicted - (2.0 - 5.0 + 3.0)).abs() < 1e-6, "{predicted}");
        assert!(model.cv_rmse.unwrap() < 1e-3);
    }

    #[test]
    fn larger_lambda_shrinks_coefficients() {
        let small = fit(&linear_rows(), UnitScope::File, &["a", "b"], None, 0.01).unwrap();
        let large = fit(&linear_rows(), UnitScope::File, &["a", "b"], None, 100.0).unwrap();
        assert!(large.coefficients[0].abs() < small.coefficients[0].abs());
    }

    #[test]
    fn features_missing_for_some_labelled_unit_are_excluded_automatically() {
        let mut rows = linear_rows();
        rows[0].metrics.insert("d".into(), None);
        for r in &mut rows[1..] {
            r.metrics.insert("d".into(), Some(1.0 + r.score));
        }
        let model = fit(&rows, UnitScope::File, &["a", "b", "d"], None, 1.0).unwrap();
        assert_eq!(
            model.excluded_features["d"],
            "missing for 1 labelled unit(s)"
        );
    }

    #[test]
    fn explicitly_requested_features_must_be_usable() {
        let mut rows = linear_rows();
        rows[0].metrics.insert("a".into(), None);
        let err = fit(
            &rows,
            UnitScope::File,
            &["a", "b"],
            Some(&["a".to_string()]),
            1.0,
        )
        .unwrap_err();
        assert!(err.0.contains("a"), "{}", err.0);
    }

    #[test]
    fn too_few_labels_is_an_error_and_small_sets_have_no_cross_validation() {
        assert!(fit(&linear_rows()[..1], UnitScope::File, &["a", "b"], None, 1.0).is_err());
        let model = fit(&linear_rows()[..4], UnitScope::File, &["a", "b"], None, 1.0).unwrap();
        assert_eq!(model.cv_rmse, None);
    }

    #[test]
    fn prediction_needs_every_feature() {
        let model = fit(&linear_rows(), UnitScope::File, &["a", "b"], None, 1.0).unwrap();
        let metrics = row(&[("a", Some(1.0)), ("b", None)], 0.0).metrics;
        assert_eq!(model.predict(&metrics).unwrap_err(), "b is null");
    }

    #[test]
    fn reads_labels_csv() {
        let labels =
            read_labels("path,function,score\nsrc/a.py,,3.5\n\"src/b,c.py\",run,1\n").unwrap();
        assert_eq!(
            labels[0],
            Label {
                path: "src/a.py".into(),
                function: None,
                score: 3.5
            }
        );
        assert_eq!(labels[1].function.as_deref(), Some("run"));
        assert_eq!(labels[1].path, "src/b,c.py");
        assert!(read_labels("path,score\na,1\n").is_err());
        assert!(read_labels("path,function,score\na,,high\n").is_err());
    }

    #[test]
    fn non_finite_label_scores_are_rejected() {
        for bad in ["NaN", "inf", "1e400"] {
            let err = read_labels(&format!("path,function,score\na.py,,{bad}\n")).unwrap_err();
            assert!(
                err.0.contains("row 2") && err.0.contains("finite"),
                "{bad}: {}",
                err.0
            );
        }
    }

    #[test]
    fn models_with_inconsistent_vectors_are_rejected_when_loaded() {
        let model = fit(&linear_rows(), UnitScope::File, &["a", "b"], None, 1.0).unwrap();
        let mut value = serde_json::to_value(&model).unwrap();
        assert_eq!(Model::from_json(&value.to_string()).unwrap(), model);
        value["sds"] = serde_json::json!([1.0]);
        let err = Model::from_json(&value.to_string()).unwrap_err();
        assert!(err.0.contains("lengths"), "{}", err.0);
        assert!(Model::from_json("{}").is_err());
    }
}
