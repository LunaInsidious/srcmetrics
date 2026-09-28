//! CLI end-to-end tests.

use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    format!("{}/../../tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"))
}

fn srcmetrics(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_srcmetrics"))
        .args(args)
        .output()
        .unwrap()
}

fn json(output: &Output) -> serde_json::Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn analyze_prints_json_result() {
    let value = json(&srcmetrics(&["analyze", &fixture("equivalence")]));
    let files = value["files"].as_array().unwrap().len();
    assert!(files > 0);
    assert_eq!(value["run"]["project"], "equivalence");
    assert_eq!(
        value["project"]["metrics"]["size.function_count"],
        2.0 * files as f64
    );
}

#[test]
fn analyze_accepts_project_name_and_output_file() {
    let dir = format!("{}/../../target/cli-test", env!("CARGO_MANIFEST_DIR"));
    std::fs::create_dir_all(&dir).unwrap();
    let out = format!("{dir}/result.json");
    let output = srcmetrics(&[
        "analyze",
        &fixture("equivalence"),
        "--project",
        "demo",
        "--output",
        &out,
    ]);
    assert!(output.status.success());
    let value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&out).unwrap()).unwrap();
    assert_eq!(value["run"]["project"], "demo");
}

#[test]
fn analyze_warns_about_files_that_failed() {
    let output = srcmetrics(&["analyze", &fixture("mixed")]);
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("1 file(s) could not be analyzed"),
        "{stderr}"
    );
    assert!(stderr.contains("broken.c"), "{stderr}");
}

#[test]
fn analyze_fails_with_a_message_for_a_missing_path() {
    let output = srcmetrics(&["analyze", &fixture("does-not-exist")]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("check that the path exists"), "{stderr}");
}

#[test]
fn analyze_can_print_csv() {
    let output = srcmetrics(&["analyze", &fixture("mixed"), "--format", "csv"]);
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.starts_with("scope,path,language,function,start_line,end_line,status,error,"));
    assert!(
        text.lines()
            .any(|l| l.starts_with("function,ok.py,python,ok,1,2,ok,")),
        "{text}"
    );
}

/// Analyzes `fixture` into a JSON result file under target/ and returns its path.
fn analyzed(fixture_name: &str) -> String {
    let dir = format!("{}/../../target/cli-test", env!("CARGO_MANIFEST_DIR"));
    std::fs::create_dir_all(&dir).unwrap();
    let out = format!("{dir}/{}.json", fixture_name.replace('/', "_"));
    assert!(
        srcmetrics(&["analyze", &fixture(fixture_name), "-o", &out])
            .status
            .success()
    );
    out
}

#[test]
fn stats_summarizes_functions_by_language() {
    let result = analyzed("equivalence");
    let value = json(&srcmetrics(&["stats", &result, "--scope", "function"]));
    let languages = value["by_language"].as_object().unwrap().len();
    assert_eq!(value["units"], 2 * languages);
    // The same algorithm in every language: cyclomatic complexity has no spread across languages.
    assert_eq!(value["metrics"]["complexity.cyclomatic"]["median"], 4.5);
    assert!(value["correlations"].as_array().unwrap().len() > 10);
}

#[test]
fn stats_rejects_a_file_that_is_not_a_result() {
    let output = srcmetrics(&["stats", &fixture("mixed/ok.py")]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("srcmetrics analyze"));
}

#[test]
fn model_train_then_predict() {
    let result = analyzed("equivalence");
    let dir = format!("{}/../../target/cli-test", env!("CARGO_MANIFEST_DIR"));
    let parsed = json(&srcmetrics(&["stats", &result]));
    assert!(parsed["units"].as_u64().unwrap() >= 5);
    let files: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&result).unwrap()).unwrap();
    let mut labels = String::from("path,function,score\n");
    for (i, f) in files["files"].as_array().unwrap().iter().enumerate() {
        labels += &format!("{},,{}\n", f["path"].as_str().unwrap(), i);
    }
    let labels_path = format!("{dir}/labels.csv");
    std::fs::write(&labels_path, labels).unwrap();
    let model_path = format!("{dir}/model.json");
    let output = srcmetrics(&[
        "model",
        "train",
        "--labels",
        &labels_path,
        "-o",
        &model_path,
        &result,
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let model: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&model_path).unwrap()).unwrap();
    assert!(
        model["notice"]
            .as_str()
            .unwrap()
            .starts_with("Experimental")
    );
    let predictions = json(&srcmetrics(&[
        "model",
        "predict",
        "--model",
        &model_path,
        &result,
    ]));
    assert_eq!(
        predictions.as_array().unwrap().len(),
        files["files"].as_array().unwrap().len()
    );
}

#[test]
fn report_writes_a_self_contained_html_file() {
    let result = analyzed("equivalence");
    let out = format!(
        "{}/../../target/cli-test/report.html",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = srcmetrics(&["report", &result, "-o", &out]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let html = std::fs::read_to_string(&out).unwrap();
    assert!(html.contains("class=\"heatmap\""));
}
