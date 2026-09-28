//! CSV output (ADR-0016): one row per project / file / function.

use crate::metrics;
use crate::result::{AnalysisResult, FileResult, MetricsOutput};

const FIXED_COLUMNS: [&str; 8] = [
    "scope",
    "path",
    "language",
    "function",
    "start_line",
    "end_line",
    "status",
    "error",
];

/// Renders the result as CSV. Unavailable values are empty cells (the reasons are in the JSON output).
pub fn to_csv(result: &AnalysisResult) -> String {
    let ids: Vec<&str> = metrics::definitions().iter().map(|d| d.id).collect();
    let row = |fixed: [&str; 8], values: Option<&MetricsOutput>| -> String {
        let value = |id: &str| values.and_then(|v| v.metrics.get(id).copied().flatten());
        let metric_cells = ids
            .iter()
            .map(|id| value(id).map(|x| x.to_string()).unwrap_or_default());
        fixed
            .iter()
            .map(|c| cell(c))
            .chain(metric_cells)
            .collect::<Vec<_>>()
            .join(",")
    };
    let header = FIXED_COLUMNS
        .iter()
        .chain(&ids)
        .map(|c| cell(c))
        .collect::<Vec<_>>()
        .join(",");
    let mut lines = vec![
        header,
        row(
            ["project", "", "", "", "", "", "", ""],
            Some(&result.project),
        ),
    ];
    for file in &result.files {
        match file {
            FileResult::Ok {
                path,
                language,
                metrics,
                functions,
            } => {
                lines.push(row(
                    ["file", path, language, "", "", "", "ok", ""],
                    Some(metrics),
                ));
                for f in functions {
                    let (start, end) = (f.start_line.to_string(), f.end_line.to_string());
                    let name = f.name.as_deref().unwrap_or("");
                    lines.push(row(
                        ["function", path, language, name, &start, &end, "ok", ""],
                        Some(&f.metrics),
                    ));
                }
            }
            FileResult::Error {
                path,
                language,
                error,
            } => {
                let language = language.as_deref().unwrap_or("");
                lines.push(row(
                    ["file", path, language, "", "", "", "error", error],
                    None,
                ));
            }
        }
    }
    lines.join("\n") + "\n"
}

/// A CSV cell, quoted when it contains a separator, quote or line break.
fn cell(text: &str) -> String {
    if text.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", text.replace('"', "\"\""))
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyze::analyze;
    use std::path::Path;

    fn fixture(name: &str) -> String {
        let path = format!("{}/../../tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
        to_csv(&analyze(Path::new(&path), None).unwrap())
    }

    #[test]
    fn header_has_fixed_columns_then_every_metric_id() {
        let csv = fixture("mixed");
        let header: Vec<&str> = csv.lines().next().unwrap().split(',').collect();
        assert_eq!(
            header[..8],
            [
                "scope",
                "path",
                "language",
                "function",
                "start_line",
                "end_line",
                "status",
                "error"
            ]
        );
        let ids: Vec<_> = crate::metrics::definitions().iter().map(|d| d.id).collect();
        assert_eq!(header[8..], ids[..]);
    }

    #[test]
    fn rows_are_project_then_files_each_followed_by_its_functions() {
        let csv = fixture("mixed");
        let rows: Vec<Vec<&str>> = csv
            .lines()
            .skip(1)
            .map(|l| l.splitn(9, ',').take(8).collect())
            .collect();
        let scopes: Vec<_> = rows.iter().map(|r| (r[0], r[1], r[3])).collect();
        assert_eq!(
            scopes,
            vec![
                ("project", "", ""),
                ("file", "broken.c", ""),
                ("file", "ok.py", ""),
                ("function", "ok.py", "ok"),
                ("file", "sub/empty.py", ""),
            ]
        );
        assert_eq!(rows[1][6], "error");
        assert_eq!(rows[3][4..7], ["1", "2", "ok"]);
    }

    #[test]
    fn unavailable_values_are_empty_cells() {
        let csv = fixture("mixed");
        let header: Vec<&str> = csv.lines().next().unwrap().split(',').collect();
        let column = header
            .iter()
            .position(|h| *h == "size.comment_ratio")
            .unwrap();
        let empty_file = csv
            .lines()
            .find(|l| l.starts_with("file,sub/empty.py"))
            .unwrap();
        assert_eq!(empty_file.split(',').nth(column), Some(""));
    }

    #[test]
    fn cells_with_separators_or_quotes_are_quoted() {
        assert_eq!(cell("plain"), "plain");
        assert_eq!(cell("a,b"), "\"a,b\"");
        assert_eq!(cell("say \"hi\""), "\"say \"\"hi\"\"\"");
        assert_eq!(cell("two\nlines"), "\"two\nlines\"");
    }
}
