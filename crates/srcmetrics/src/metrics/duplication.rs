//! Duplication Metrics (ADR-0009).

use super::{
    Applicability::*, Calculator, MetricDefinition, MetricValue, Metrics, ProgramMetrics, Scope::*,
};
use crate::ir::{File, Program, TokenKind};
use std::collections::HashMap;

pub struct DuplicationCalculator;

/// Minimum clone length in normalized tokens (ADR-0009).
const MIN_TOKENS: usize = 50;

const CALCULATION: &str = "Tokens are normalized (identifier -> $id, literal -> $lit, comments removed). \
                           Every window of 50 consecutive normalized tokens that occurs at least twice marks \
                           its tokens as duplicated; a maximal run of duplicated tokens is one block. \
                           File: clones within the file only. Project: clones across all files.";
const LIMITATIONS: &str =
    "Repetitive token sequences (e.g. long array literals) are reported as duplicates.";

const fn duplication(
    id: &'static str,
    name: &'static str,
    definition: &'static str,
    unit: &'static str,
) -> MetricDefinition {
    MetricDefinition {
        id,
        name,
        description: definition,
        definition,
        scopes: &[File, Project],
        input: "Tokens (kind and text)",
        calculation: CALCULATION,
        unit,
        applicability: LanguageIndependent,
        limitations: LIMITATIONS,
        reference: "",
    }
}

static DEFINITIONS: &[MetricDefinition] = &[
    duplication(
        "duplication.duplicate_block_count",
        "Duplicate Block Count",
        "Number of duplicated token runs.",
        "count",
    ),
    duplication(
        "duplication.duplicate_token_count",
        "Duplicate Token Count",
        "Number of duplicated tokens.",
        "count",
    ),
    duplication(
        "duplication.duplication_ratio",
        "Duplication Ratio",
        "Duplicate Token Count / non-comment tokens; not_applicable when there are no tokens.",
        "ratio",
    ),
    duplication(
        "duplication.max_duplicate_length",
        "Maximum Duplicate Length",
        "Longest duplicated token run; 0 when there is none.",
        "tokens",
    ),
];

impl Calculator for DuplicationCalculator {
    fn definitions(&self) -> &'static [MetricDefinition] {
        DEFINITIONS
    }

    fn compute(&self, program: &Program) -> ProgramMetrics {
        let sequences = normalize(&program.files);
        let files = sequences
            .iter()
            .zip(&program.files)
            .map(|(seq, file)| super::FileMetrics {
                metrics: metrics(&duplicated(std::slice::from_ref(seq))),
                functions: vec![Metrics::new(); file.functions.len()],
            })
            .collect();
        ProgramMetrics {
            project: metrics(&duplicated(&sequences)),
            files,
        }
    }
}

/// Normalized token ids per file (ADR-0009).
fn normalize(files: &[File]) -> Vec<Vec<u32>> {
    let mut ids: HashMap<&str, u32> = HashMap::new();
    files
        .iter()
        .map(|f| {
            f.tokens
                .iter()
                .filter(|t| t.kind != TokenKind::Comment)
                .map(|t| {
                    let key = match t.kind {
                        TokenKind::Identifier => "$id",
                        TokenKind::Literal => "$lit",
                        _ => t.text.as_str(),
                    };
                    let next = ids.len() as u32;
                    *ids.entry(key).or_insert(next)
                })
                .collect()
        })
        .collect()
}

/// For each sequence, which tokens lie in a window that occurs at least twice across `sequences`.
fn duplicated(sequences: &[Vec<u32>]) -> Vec<Vec<bool>> {
    let mut windows: HashMap<&[u32], Vec<(usize, usize)>> = HashMap::new();
    for (s, seq) in sequences.iter().enumerate() {
        for (start, window) in seq.windows(MIN_TOKENS).enumerate() {
            windows.entry(window).or_default().push((s, start));
        }
    }
    let mut flags: Vec<Vec<bool>> = sequences.iter().map(|s| vec![false; s.len()]).collect();
    for occurrences in windows.values().filter(|o| o.len() > 1) {
        for &(s, start) in occurrences {
            flags[s][start..start + MIN_TOKENS].fill(true);
        }
    }
    flags
}

fn metrics(flags: &[Vec<bool>]) -> Metrics {
    let runs: Vec<usize> = flags
        .iter()
        .flat_map(|f| {
            f.chunk_by(|a, b| a == b)
                .filter(|run| run[0])
                .map(<[bool]>::len)
        })
        .collect();
    let duplicated: usize = runs.iter().sum();
    let total: usize = flags.iter().map(Vec::len).sum();
    Metrics::from([
        ("duplication.duplicate_block_count", runs.len().into()),
        ("duplication.duplicate_token_count", duplicated.into()),
        (
            "duplication.duplication_ratio",
            MetricValue::ratio(duplicated as f64, total as f64),
        ),
        (
            "duplication.max_duplicate_length",
            runs.iter().max().copied().unwrap_or(0).into(),
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::builder::*;
    use crate::ir::{File, Program, TokenKind::*};
    use crate::metrics::{Calculator, MetricValue};

    fn v(x: f64) -> MetricValue {
        MetricValue::Available(x)
    }

    /// Appends a non-repetitive fragment of `2 * pairs` tokens: `kw0 <id> kw1 <id> ...`.
    /// Identifier names differ per `variant`, so copies are Type-2 clones.
    fn fragment(b: &mut FileBuilder, line: usize, pairs: usize, variant: &str) {
        fragment_from(b, line, 0, pairs, variant);
    }

    fn file_with(build: impl FnOnce(&mut FileBuilder)) -> File {
        let mut b = FileBuilder::new("");
        build(&mut b);
        b.build()
    }

    fn file_metrics(file: File) -> crate::metrics::Metrics {
        DuplicationCalculator
            .compute(&Program { files: vec![file] })
            .files[0]
            .metrics
            .clone()
    }

    #[test]
    fn detects_renamed_copies_within_a_file() {
        let m = file_metrics(file_with(|b| {
            fragment(b, 1, 30, "a");
            b.token(Punctuation, ";", 2);
            fragment(b, 3, 30, "b");
        }));
        assert_eq!(m["duplication.duplicate_token_count"], v(120.0));
        assert_eq!(m["duplication.duplicate_block_count"], v(2.0));
        assert_eq!(m["duplication.max_duplicate_length"], v(60.0));
        assert_eq!(m["duplication.duplication_ratio"], v(120.0 / 121.0));
    }

    #[test]
    fn repeats_shorter_than_the_minimum_are_not_duplicates() {
        let m = file_metrics(file_with(|b| {
            fragment(b, 1, 24, "a");
            fragment(b, 2, 24, "b");
        }));
        assert_eq!(m["duplication.duplicate_token_count"], v(0.0));
        assert_eq!(m["duplication.duplicate_block_count"], v(0.0));
        assert_eq!(m["duplication.max_duplicate_length"], v(0.0));
    }

    #[test]
    fn comments_do_not_break_a_clone() {
        let m = file_metrics(file_with(|b| {
            fragment(b, 1, 30, "a");
            fragment(b, 2, 15, "b");
            b.token(Comment, "// note", 2);
            fragment_from(b, 3, 15, 30, "b");
        }));
        assert_eq!(m["duplication.duplicate_token_count"], v(120.0));
    }

    /// Like `fragment`, but numbering keywords from `from` so it continues a split fragment.
    fn fragment_from(b: &mut FileBuilder, line: usize, from: usize, to: usize, variant: &str) {
        for i in from..to {
            b.token(Keyword, &format!("kw{i}"), line);
            b.token(Identifier, &format!("{variant}{i}"), line);
        }
    }

    #[test]
    fn file_scope_ignores_other_files_and_project_scope_does_not() {
        let a = file_with(|b| fragment(b, 1, 30, "a"));
        let b = file_with(|b| fragment(b, 1, 30, "b"));
        let result = DuplicationCalculator.compute(&Program { files: vec![a, b] });
        assert_eq!(
            result.files[0].metrics["duplication.duplicate_token_count"],
            v(0.0)
        );
        assert_eq!(
            result.project["duplication.duplicate_token_count"],
            v(120.0)
        );
        assert_eq!(result.project["duplication.duplicate_block_count"], v(2.0));
        assert_eq!(result.project["duplication.duplication_ratio"], v(1.0));
    }

    #[test]
    fn empty_file_has_no_ratio() {
        let m = file_metrics(FileBuilder::new("").build());
        assert_eq!(m["duplication.duplicate_token_count"], v(0.0));
        assert_eq!(
            m["duplication.duplication_ratio"],
            MetricValue::NotApplicable
        );
    }
}
