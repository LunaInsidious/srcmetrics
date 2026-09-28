//! End-to-end analysis: directory -> result with run metadata (PLAN.md §11, §17, ADR-0010).

use codestat::analyze::analyze;
use codestat::error::AnalysisError;
use codestat::result::FileResult;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(format!(
        "{}/../../tests/fixtures/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
}

fn paths(files: &[FileResult]) -> Vec<&str> {
    files.iter().map(|f| f.path()).collect()
}

#[test]
fn analyzes_supported_files_in_sorted_order() {
    // equivalence/ holds one `classify.<ext>` (two functions) per supported language.
    let result = analyze(&fixture("equivalence"), None).unwrap();
    let files = paths(&result.files);
    let mut sorted = files.clone();
    sorted.sort();
    assert_eq!(files, sorted);
    assert!(
        files.iter().all(|p| p.starts_with("classify.")),
        "{files:?}"
    );
    assert_eq!(
        result.project.metrics["size.function_count"],
        Some(2.0 * files.len() as f64)
    );
}

#[test]
fn records_run_metadata() {
    let result = analyze(&fixture("equivalence"), Some("demo")).unwrap();
    let run = &result.run;
    assert_eq!(run.project, "demo");
    assert_eq!(
        run.metric_definition_version,
        codestat::metrics::DEFINITION_VERSION
    );
    assert_eq!(run.tool_version, env!("CARGO_PKG_VERSION"));
    assert!(
        run.parsers["c"].starts_with("tree-sitter "),
        "{:?}",
        run.parsers
    );
    assert_eq!(
        run.parsers.len(),
        result.files.len(),
        "one language per fixture"
    );
    assert!(run.timestamp.ends_with('Z'), "{}", run.timestamp);
}

#[test]
fn project_name_defaults_to_the_directory_name() {
    let result = analyze(&fixture("equivalence"), None).unwrap();
    assert_eq!(result.run.project, "equivalence");
}

#[test]
fn parse_failures_are_recorded_and_excluded_from_project_totals() {
    let result = analyze(&fixture("mixed"), None).unwrap();
    assert_eq!(
        paths(&result.files),
        vec!["broken.c", "ok.py", "sub/empty.py"]
    );
    match &result.files[0] {
        FileResult::Error {
            error, language, ..
        } => {
            assert!(error.contains("parse error"), "{error}");
            assert_eq!(language.as_deref(), Some("c"));
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(result.project.metrics["size.function_count"], Some(1.0));
}

#[test]
fn unavailable_values_are_null_with_a_reason() {
    let result = analyze(&fixture("mixed"), None).unwrap();
    let FileResult::Ok { metrics, .. } = &result.files[2] else {
        panic!()
    };
    assert_eq!(metrics.metrics["size.comment_ratio"], None);
    assert_eq!(metrics.unavailable["size.comment_ratio"], "not_applicable");
    assert_eq!(metrics.metrics["size.loc"], Some(0.0));
}

#[test]
fn functions_carry_name_and_lines() {
    let result = analyze(&fixture("mixed"), None).unwrap();
    let FileResult::Ok { functions, .. } = &result.files[1] else {
        panic!()
    };
    assert_eq!(functions[0].name.as_deref(), Some("ok"));
    assert_eq!((functions[0].start_line, functions[0].end_line), (1, 2));
    assert_eq!(
        functions[0].metrics.metrics["function.parameter_count"],
        Some(1.0)
    );
}

#[test]
fn explicit_unsupported_file_is_an_error() {
    let err = analyze(&fixture("mixed/README.md"), None).unwrap_err();
    assert!(
        matches!(err, AnalysisError::UnsupportedLanguage { .. }),
        "{err:?}"
    );
}

#[test]
fn single_file_can_be_analyzed() {
    let result = analyze(&fixture("mixed/ok.py"), None).unwrap();
    assert_eq!(paths(&result.files), vec!["ok.py"]);
}

#[test]
fn result_round_trips_through_json() {
    let result = analyze(&fixture("mixed"), None).unwrap();
    let json = serde_json::to_string(&result).unwrap();
    let back: codestat::result::AnalysisResult = serde_json::from_str(&json).unwrap();
    assert_eq!(back, result);
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["files"][0]["status"], "error");
    assert_eq!(value["files"][1]["status"], "ok");
}

#[test]
fn source_text_can_be_analyzed_without_the_file_system() {
    let result =
        codestat::analyze::analyze_source("snippet.py", "def f(a):\n    return a\n").unwrap();
    assert_eq!(paths(&result.files), vec!["snippet.py"]);
    assert_eq!(result.run.project, "snippet");
    assert_eq!(
        (
            result.run.repository.as_deref(),
            result.run.commit.as_deref()
        ),
        (None, None)
    );
    assert_eq!(result.project.metrics["size.function_count"], Some(1.0));
    let err = codestat::analyze::analyze_source("broken.py", "def (:\n").unwrap_err();
    assert!(matches!(err, AnalysisError::Parse { .. }), "{err:?}");
}
