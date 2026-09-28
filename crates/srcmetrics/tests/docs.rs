//! docs/METRICS.md must match the metric definitions in code (AGENTS.md: docs are part of the task).

use srcmetrics::metrics;

#[test]
fn metrics_doc_is_up_to_date() {
    let path = format!("{}/../../docs/METRICS.md", env!("CARGO_MANIFEST_DIR"));
    let generated = metrics::to_markdown(&metrics::definitions());
    if std::env::var_os("UPDATE_DOCS").is_some() {
        std::fs::write(&path, &generated).unwrap();
    }
    let current = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        current == generated,
        "docs/METRICS.md is out of date; run `UPDATE_DOCS=1 cargo test -p srcmetrics --test docs`"
    );
}
