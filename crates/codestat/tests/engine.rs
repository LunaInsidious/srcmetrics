//! Metric Engine invariants and cross-language equivalence (PLAN.md §6, §9, §19).

use codestat::ir::Program;
use codestat::lang::adapter_for_path;
use codestat::metrics::{self, MetricValue, Metrics, Scope};
use std::collections::HashSet;

fn parse(name: &str) -> codestat::ir::File {
    let path = format!("{}/../../tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    let source = std::fs::read_to_string(&path).unwrap();
    adapter_for_path(&path)
        .unwrap()
        .to_ir(&path, &source)
        .unwrap()
}

/// Extensions of the `equivalence/classify.*` fixtures: the same algorithm in every language.
const EQUIVALENCE_LANGUAGES: [&str; 6] = ["c", "py", "ts", "go", "java", "js"];

fn equivalence_program() -> Program {
    Program {
        files: EQUIVALENCE_LANGUAGES
            .map(|ext| parse(&format!("equivalence/classify.{ext}")))
            .into(),
    }
}

#[test]
fn metric_ids_are_unique() {
    let mut seen = HashSet::new();
    for d in metrics::definitions() {
        assert!(seen.insert(d.id), "duplicate metric id {}", d.id);
    }
}

#[test]
fn emitted_metrics_match_their_definitions_scopes() {
    let result = metrics::compute(&equivalence_program());
    let check = |scope: Scope, emitted: &Metrics| {
        let expected: HashSet<_> = metrics::definitions()
            .into_iter()
            .filter(|d| d.scopes.contains(&scope))
            .map(|d| d.id)
            .collect();
        let actual: HashSet<_> = emitted.keys().copied().collect();
        assert_eq!(actual, expected, "{scope:?}");
    };
    check(Scope::Project, &result.project);
    for file in &result.files {
        check(Scope::File, &file.metrics);
        for function in &file.functions {
            check(Scope::Function, function);
        }
    }
}

#[test]
fn real_files_produce_no_errors() {
    let result = metrics::compute(&equivalence_program());
    for file in &result.files {
        for (id, value) in file.metrics.iter().chain(file.functions.iter().flatten()) {
            assert!(!matches!(value, MetricValue::Error(_)), "{id}: {value:?}");
        }
    }
}

/// The same algorithm written in C, Python and TypeScript yields the same
/// language-independent structural metrics.
#[test]
fn same_algorithm_same_metrics_across_languages() {
    let result = metrics::compute(&equivalence_program());
    let ids = ["complexity.cyclomatic", "nesting.max_depth"];
    let per_language: Vec<Vec<Vec<&MetricValue>>> = result
        .files
        .iter()
        .map(|f| {
            f.functions
                .iter()
                .map(|m| ids.iter().map(|id| &m[id]).collect())
                .collect()
        })
        .collect();
    for (i, ext) in EQUIVALENCE_LANGUAGES.iter().enumerate().skip(1) {
        assert_eq!(per_language[0], per_language[i], "c vs {ext}");
    }
    let classify = &result.files[0].functions[0];
    assert_eq!(
        classify["complexity.cyclomatic"],
        MetricValue::Available(7.0)
    );
    assert_eq!(classify["nesting.max_depth"], MetricValue::Available(2.0));
    assert_eq!(
        result.files[0].metrics["size.function_count"],
        MetricValue::Available(2.0)
    );
}

/// PLAN.md §19-2: the Metric Engine refers to the Common IR only.
#[test]
fn metric_engine_does_not_depend_on_parsers_or_languages() {
    let dir = format!("{}/src/metrics", env!("CARGO_MANIFEST_DIR"));
    let forbidden = [
        "tree_sitter",
        "crate::lang",
        "\"python\"",
        "\"typescript\"",
        "\"c\"",
    ];
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let text = std::fs::read_to_string(&path).unwrap();
        for word in forbidden {
            assert!(!text.contains(word), "{} mentions {word}", path.display());
        }
    }
}

/// Real C code has `else if` ladders with hundreds of arms; in the IR each arm nests one level
/// deeper (Branch -> Else -> Branch ...). Metrics must stay linear and must not recurse per level.
#[test]
fn long_else_if_ladders_are_handled_in_linear_time() {
    let arms = 20_000;
    let mut source = String::from("int f(int x) {\n  if (x == 0) { x = 1; }\n");
    for i in 1..arms {
        source += &format!("  else if (x == {i}) {{ x = 1; }}\n");
    }
    source += "  return x;\n}\n";
    let file = adapter_for_path("ladder.c")
        .unwrap()
        .to_ir("ladder.c", &source)
        .unwrap();
    let started = std::time::Instant::now();
    let result = metrics::compute(&Program { files: vec![file] });
    let f = &result.files[0].functions[0];
    assert_eq!(f["nesting.max_depth"], MetricValue::Available(1.0));
    assert_eq!(
        f["complexity.path_count"],
        MetricValue::Available((arms + 1) as f64)
    );
    assert_eq!(
        f["complexity.cyclomatic"],
        MetricValue::Available((arms + 1) as f64)
    );
    assert!(
        started.elapsed().as_secs() < 5,
        "took {:?}",
        started.elapsed()
    );
}
