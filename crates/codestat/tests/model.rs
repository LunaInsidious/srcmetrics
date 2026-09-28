//! Training and prediction on analysis results (ADR-0019).

use codestat::analyze::analyze;
use codestat::model::{Label, predict_all, train};
use codestat::result::AnalysisResult;
use std::path::PathBuf;

fn result(name: &str) -> AnalysisResult {
    analyze(
        &PathBuf::from(format!(
            "{}/../../tests/fixtures/{name}",
            env!("CARGO_MANIFEST_DIR")
        )),
        None,
    )
    .unwrap()
}

fn label(path: &str, function: Option<&str>, score: f64) -> Label {
    Label {
        path: path.into(),
        function: function.map(str::to_string),
        score,
    }
}

#[test]
fn trains_on_function_labels_and_predicts_every_function() {
    let results = [result("equivalence")];
    let paths: Vec<String> = results[0]
        .files
        .iter()
        .map(|f| f.path().to_string())
        .collect();
    let labels: Vec<Label> = paths
        .iter()
        .enumerate()
        .flat_map(|(i, p)| {
            [
                label(p, Some("classify"), 2.0 + i as f64 * 0.1),
                label(p, Some("max2"), 8.0),
            ]
        })
        .collect();
    let model = train(&results, &labels, None, 1.0).unwrap();
    assert_eq!(model.n, labels.len());
    assert!(
        model
            .features
            .contains(&"complexity.cyclomatic".to_string())
    );
    assert!(model.r_squared > 0.9, "{}", model.r_squared);
    let predictions = predict_all(&model, &results);
    assert_eq!(predictions.len(), labels.len());
    assert!(predictions.iter().all(|p| p.score.is_some()));
}

#[test]
fn labels_must_match_exactly_one_unit() {
    let results = [result("model"), result("mixed")];
    let err = |labels: &[Label]| train(&results, labels, None, 1.0).unwrap_err().0;
    assert!(
        err(&[
            label("dup.py", Some("run"), 1.0),
            label("ok.py", Some("ok"), 2.0)
        ])
        .contains("several functions")
    );
    assert!(
        err(&[label("nope.py", None, 1.0), label("ok.py", None, 2.0)]).contains("no such file")
    );
    assert!(
        err(&[label("broken.c", None, 1.0), label("ok.py", None, 2.0)])
            .contains("failed to analyze")
    );
    assert!(err(&[label("ok.py", None, 1.0), label("ok.py", Some("ok"), 2.0)]).contains("mix"));
}

#[test]
fn predictions_report_files_that_failed_to_analyze() {
    let results = [result("equivalence")];
    let labels: Vec<Label> = results[0]
        .files
        .iter()
        .enumerate()
        .map(|(i, f)| label(f.path(), None, i as f64))
        .collect();
    let model = train(&results, &labels, None, 1.0).unwrap();
    let predictions = predict_all(&model, &[result("mixed")]);
    let broken = predictions.iter().find(|p| p.path == "broken.c").unwrap();
    assert_eq!(broken.score, None);
    assert!(
        broken
            .unavailable
            .as_ref()
            .unwrap()
            .contains("failed to analyze")
    );
}
