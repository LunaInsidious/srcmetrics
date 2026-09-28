//! Every metric id in the code is documented in the English and Japanese metric reference pages,
//! and the pages document no other ids (ADR-0026). The explanations themselves are written by hand.

use srcmetrics::metrics;
use std::collections::BTreeSet;

/// Metric ids documented in the pages of `dir`: each metric section starts with a line
/// "`<id>` — <description>".
fn documented_ids(dir: &str) -> BTreeSet<String> {
    let dir = format!("{}/../../{dir}", env!("CARGO_MANIFEST_DIR"));
    let mut ids = BTreeSet::new();
    for entry in std::fs::read_dir(&dir).unwrap() {
        let text = std::fs::read_to_string(entry.unwrap().path()).unwrap();
        for line in text.lines() {
            if let Some(id) = line
                .strip_prefix('`')
                .and_then(|rest| rest.split_once("` — "))
                .map(|(id, _)| id)
            {
                assert!(
                    ids.insert(id.to_string()),
                    "{dir}: {id} is documented twice"
                );
            }
        }
    }
    ids
}

fn code_ids() -> BTreeSet<String> {
    metrics::specs().iter().map(|s| s.id.to_string()).collect()
}

#[test]
fn english_metric_reference_documents_every_metric() {
    assert_eq!(documented_ids("docs/metrics"), code_ids());
}

#[test]
fn japanese_metric_reference_documents_every_metric() {
    assert_eq!(documented_ids("docs/ja/metrics"), code_ids());
}
