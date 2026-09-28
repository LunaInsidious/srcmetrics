//! CLI end-to-end tests.

use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    format!("{}/../../tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"))
}

fn codestat(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_codestat"))
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
    let value = json(&codestat(&["analyze", &fixture("equivalence")]));
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
    let output = codestat(&[
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
    let output = codestat(&["analyze", &fixture("mixed")]);
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
    let output = codestat(&["analyze", &fixture("does-not-exist")]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("check that the path exists"), "{stderr}");
}

#[test]
fn metrics_lists_definitions_as_json() {
    let value = json(&codestat(&["metrics"]));
    let ids: Vec<_> = value
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["id"].as_str().unwrap().to_string())
        .collect();
    assert!(ids.contains(&"complexity.cyclomatic".to_string()));
    let cyclomatic = value
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["id"] == "complexity.cyclomatic")
        .unwrap();
    assert_eq!(
        cyclomatic["scopes"],
        serde_json::json!(["function", "file", "project"])
    );
    assert_eq!(cyclomatic["applicability"], "partially_language_dependent");
}

#[test]
fn metrics_markdown_matches_the_definition_document() {
    let output = codestat(&["metrics", "--format", "markdown"]);
    let doc = std::fs::read_to_string(format!(
        "{}/../../docs/METRICS.md",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    assert_eq!(String::from_utf8(output.stdout).unwrap(), doc);
}

#[test]
fn analyze_can_print_csv() {
    let output = codestat(&["analyze", &fixture("mixed"), "--format", "csv"]);
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.starts_with("scope,path,language,function,start_line,end_line,status,error,"));
    assert!(
        text.lines()
            .any(|l| l.starts_with("function,ok.py,python,ok,1,2,ok,")),
        "{text}"
    );
}
