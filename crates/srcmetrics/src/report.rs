//! Self-contained HTML report (PLAN.md §20 Phase 4, ADR-0020): no scripts, no external resources.

use crate::result::AnalysisResult;
use crate::stats::{self, Report, Unit, UnitScope};
use std::fmt::Write;

const HISTOGRAM_BINS: usize = 20;

pub fn to_html(result: &AnalysisResult) -> String {
    let results = std::slice::from_ref(result);
    let files = stats::units(results, UnitScope::File);
    let functions = stats::units(results, UnitScope::Function);
    let (file_ids, function_ids) = (
        stats::metric_ids(UnitScope::File),
        stats::metric_ids(UnitScope::Function),
    );
    let file_report = Report::of(&files, &file_ids);
    let function_report = Report::of(&functions, &function_ids);

    let mut html = format!(
        "<!DOCTYPE html>\n<html lang=\"en\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
         <title>srcmetrics report</title><style>{STYLE}</style></head><body>\n<h1>srcmetrics report: {}</h1>\n",
        escape(&result.run.project)
    );
    html += &run_table(result);
    html += "<h2>Project metrics</h2>\n";
    html += &table(
        &["metric", "value"],
        result
            .project
            .metrics
            .iter()
            .map(|(id, v)| vec![id.clone(), number(*v)]),
    );
    html += "<h2>File medians by language</h2>\n";
    html += &language_medians(&file_report, &file_ids);
    html += "<h2>Function metrics: distributions</h2>\n";
    html += &histograms(&functions, &function_ids);
    html += "<h2>File metrics: distributions</h2>\n";
    html += &histograms(&files, &file_ids);
    html += "<h2>Function metrics: Spearman correlations</h2>\n";
    html += &heatmap(&function_report, &function_ids);
    html += "</body></html>\n";
    html
}

const STYLE: &str = "body{font-family:system-ui,sans-serif;margin:16px;background:#fff;color:#222}\
table{border-collapse:collapse;margin:8px 0;font-size:13px}td,th{border:1px solid #ccc;padding:2px 8px;text-align:right}\
td:first-child,th:first-child{text-align:left}.charts{display:flex;flex-wrap:wrap;gap:12px}\
figure{margin:0}figcaption{font-size:12px}svg text{font-size:10px;fill:currentColor}.bar{fill:#4a7fb5}\
@media (prefers-color-scheme:dark){body{background:#1b1b1b;color:#ddd}td,th{border-color:#555}}";

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn number(value: Option<f64>) -> String {
    value.map_or("null".to_string(), |v| {
        if v.fract() == 0.0 {
            format!("{v}")
        } else {
            format!("{v:.3}")
        }
    })
}

fn table(header: &[&str], rows: impl Iterator<Item = Vec<String>>) -> String {
    let mut html = String::from("<table><tr>");
    for h in header {
        write!(html, "<th>{}</th>", escape(h)).unwrap();
    }
    html += "</tr>\n";
    for row in rows {
        html += "<tr>";
        for cell in row {
            write!(html, "<td>{}</td>", escape(&cell)).unwrap();
        }
        html += "</tr>\n";
    }
    html + "</table>\n"
}

fn run_table(result: &AnalysisResult) -> String {
    let run = &result.run;
    let optional = |v: &Option<String>| v.clone().unwrap_or_else(|| "null".into());
    let mut rows = vec![
        vec!["repository".into(), optional(&run.repository)],
        vec!["commit".into(), optional(&run.commit)],
        vec!["tool_version".into(), run.tool_version.clone()],
        vec![
            "metric_definition_version".into(),
            run.metric_definition_version.clone(),
        ],
        vec!["timestamp".into(), run.timestamp.clone()],
    ];
    rows.extend(
        run.parsers
            .iter()
            .map(|(language, version)| vec![format!("parser: {language}"), version.clone()]),
    );
    table(&["run", ""], rows.into_iter())
}

fn language_medians(report: &Report, ids: &[&str]) -> String {
    let languages: Vec<&String> = report.by_language.keys().collect();
    let header: Vec<&str> = std::iter::once("metric")
        .chain(languages.iter().map(|l| l.as_str()))
        .collect();
    let rows = ids.iter().map(|id| {
        let medians = languages
            .iter()
            .map(|l| number(report.by_language[*l][*id].median));
        std::iter::once(id.to_string()).chain(medians).collect()
    });
    table(&header, rows)
}

/// Equal-width bins `(low, high, count)` over the range of `values`; one bin when all are equal.
fn bins(values: &[f64], count: usize) -> Vec<(f64, f64, usize)> {
    let (min, max) = values
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| {
            (lo.min(*v), hi.max(*v))
        });
    if min == max {
        return vec![(min, max, values.len())];
    }
    let width = (max - min) / count as f64;
    let mut counts = vec![0; count];
    for v in values {
        counts[(((v - min) / width) as usize).min(count - 1)] += 1;
    }
    counts
        .into_iter()
        .enumerate()
        .map(|(i, c)| (min + i as f64 * width, min + (i + 1) as f64 * width, c))
        .collect()
}

fn histograms(units: &[Unit], ids: &[&str]) -> String {
    let mut html = String::from("<div class=\"charts\">\n");
    for id in ids {
        let values: Vec<f64> = units
            .iter()
            .filter_map(|u| u.metrics.get(*id).copied().flatten())
            .collect();
        if values.is_empty() {
            continue;
        }
        let bins = bins(&values, HISTOGRAM_BINS);
        let tallest = bins.iter().map(|b| b.2).max().unwrap_or(0).max(1) as f64;
        let (width, height) = (240.0, 100.0);
        let bar = width / bins.len() as f64;
        write!(
            html,
            "<figure><svg class=\"histogram\" width=\"{width}\" height=\"{}\">",
            height + 14.0
        )
        .unwrap();
        for (i, (low, high, count)) in bins.iter().enumerate() {
            let h = height * *count as f64 / tallest;
            write!(
                html,
                "<rect class=\"bar\" x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{h:.1}\"><title>{} - {}: {count}</title></rect>",
                i as f64 * bar,
                height - h,
                (bar - 1.0).max(1.0),
                number(Some(*low)),
                number(Some(*high)),
            )
            .unwrap();
        }
        let (min, max) = (bins[0].0, bins[bins.len() - 1].1);
        writeln!(
            html,
            "<text x=\"0\" y=\"{}\">{}</text><text x=\"{width}\" y=\"{}\" text-anchor=\"end\">{}</text></svg>\
             <figcaption>{} (n={})</figcaption></figure>",
            height + 12.0,
            number(Some(min)),
            height + 12.0,
            number(Some(max)),
            escape(id),
            values.len()
        )
        .unwrap();
    }
    html + "</div>\n"
}

fn heatmap(report: &Report, ids: &[&str]) -> String {
    let cell = 12.0;
    let label = 190.0;
    let size = label + cell * ids.len() as f64;
    let mut html = format!("<svg class=\"heatmap\" width=\"{size}\" height=\"{size}\">");
    let index = |id: &str| {
        ids.iter()
            .position(|x| *x == id)
            .expect("correlations use these ids")
    };
    for (i, id) in ids.iter().enumerate() {
        let offset = label + i as f64 * cell + cell - 2.0;
        write!(
            html,
            "<text x=\"{}\" y=\"{offset}\" text-anchor=\"end\">{}</text>",
            label - 4.0,
            escape(id)
        )
        .unwrap();
        write!(
            html,
            "<text transform=\"translate({offset},{}) rotate(-90)\">{}</text>",
            label - 4.0,
            escape(id)
        )
        .unwrap();
    }
    for i in 0..ids.len() {
        let at = label + i as f64 * cell;
        write!(html, "<rect x=\"{at}\" y=\"{at}\" width=\"{cell}\" height=\"{cell}\" fill=\"rgba(128,128,128,0.6)\"/>").unwrap();
    }
    for c in &report.correlations {
        let (i, j) = (index(&c.a), index(&c.b));
        for (row, column) in [(i, j), (j, i)] {
            let fill = match c.spearman {
                Some(r) if r >= 0.0 => format!("rgba(200,60,60,{:.2})", r),
                Some(r) => format!("rgba(60,90,200,{:.2})", -r),
                None => "rgba(128,128,128,0.25)".into(),
            };
            write!(
                html,
                "<rect x=\"{}\" y=\"{}\" width=\"{cell}\" height=\"{cell}\" fill=\"{fill}\"><title>{} × {}: {} (n={})</title></rect>",
                label + column as f64 * cell,
                label + row as f64 * cell,
                escape(&c.a),
                escape(&c.b),
                number(c.spearman),
                c.n
            )
            .unwrap();
        }
    }
    html + "</svg>\n"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyze::analyze;
    use std::path::Path;

    fn report(project: &str) -> String {
        let path = format!(
            "{}/../../tests/fixtures/equivalence",
            env!("CARGO_MANIFEST_DIR")
        );
        to_html(&analyze(Path::new(&path), Some(project)).unwrap())
    }

    #[test]
    fn contains_metadata_tables_histograms_and_heatmap() {
        let html = report("demo");
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("<h1>srcmetrics report: demo</h1>"));
        assert!(html.contains("metric_definition_version"));
        assert!(html.contains("<td>complexity.cyclomatic</td>"));
        assert!(html.contains("class=\"histogram\""));
        assert!(html.contains("class=\"heatmap\""));
    }

    #[test]
    fn escapes_text() {
        let html = report("<script>alert(1)</script>");
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn loads_no_external_resources() {
        let html = report("demo");
        for needle in ["<script", "<link", "src=", "@import", "url("] {
            assert!(!html.contains(needle), "{needle}");
        }
    }

    #[test]
    fn histogram_bins_cover_the_range() {
        assert_eq!(
            bins(&[1.0, 2.0, 2.0, 10.0], 3),
            vec![(1.0, 4.0, 3), (4.0, 7.0, 0), (7.0, 10.0, 1)]
        );
        assert_eq!(bins(&[5.0, 5.0], 3), vec![(5.0, 5.0, 2)]);
    }
}
