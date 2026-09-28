//! The metric reference pages of the documentation site (docs/metrics/, docs/ja/metrics/) must
//! match the metric definitions in code (AGENTS.md: docs are part of the task; ADR-0025).

use srcmetrics::metrics::{self, Lang};
use std::collections::BTreeSet;

fn check(dir: &str, lang: Lang) {
    let dir = format!("{}/../../{dir}", env!("CARGO_MANIFEST_DIR"));
    let pages = metrics::reference_pages(&metrics::definitions(), lang);
    if std::env::var_os("UPDATE_DOCS").is_some() {
        std::fs::create_dir_all(&dir).unwrap();
        for (name, content) in &pages {
            std::fs::write(format!("{dir}/{name}"), content).unwrap();
        }
    }
    let hint = "run `UPDATE_DOCS=1 cargo test -p srcmetrics --test docs`";
    for (name, content) in &pages {
        let current = std::fs::read_to_string(format!("{dir}/{name}")).unwrap_or_default();
        assert!(&current == content, "{dir}/{name} is out of date; {hint}");
    }
    let expected: BTreeSet<String> = pages.into_iter().map(|(name, _)| name).collect();
    let actual: BTreeSet<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        actual, expected,
        "{dir} has pages for metric groups that no longer exist"
    );
}

#[test]
fn english_metric_reference_is_up_to_date() {
    check("docs/metrics", Lang::En);
}

#[test]
fn japanese_metric_reference_is_up_to_date() {
    check("docs/ja/metrics", Lang::Ja);
}
