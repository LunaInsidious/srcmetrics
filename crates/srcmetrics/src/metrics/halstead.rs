//! Halstead Metrics (ADR-0008).

use super::{Calculator, MetricSpec, MetricValue, Metrics, ProgramMetrics, Scope::*, per_file};
use crate::ir::{Program, Token, TokenKind};
use std::collections::HashSet;

pub struct HalsteadCalculator;

static SPECS: &[MetricSpec] = &[
    MetricSpec {
        id: "halstead.unique_operators",
        scopes: &[Function, File, Project],
    },
    MetricSpec {
        id: "halstead.unique_operands",
        scopes: &[Function, File, Project],
    },
    MetricSpec {
        id: "halstead.total_operators",
        scopes: &[Function, File, Project],
    },
    MetricSpec {
        id: "halstead.total_operands",
        scopes: &[Function, File, Project],
    },
    MetricSpec {
        id: "halstead.vocabulary",
        scopes: &[Function, File, Project],
    },
    MetricSpec {
        id: "halstead.length",
        scopes: &[Function, File, Project],
    },
    MetricSpec {
        id: "halstead.volume",
        scopes: &[Function, File, Project],
    },
    MetricSpec {
        id: "halstead.difficulty",
        scopes: &[Function, File, Project],
    },
    MetricSpec {
        id: "halstead.effort",
        scopes: &[Function, File, Project],
    },
    MetricSpec {
        id: "halstead.time",
        scopes: &[Function, File, Project],
    },
    MetricSpec {
        id: "halstead.bugs",
        scopes: &[Function, File, Project],
    },
];

impl Calculator for HalsteadCalculator {
    fn specs(&self) -> &'static [MetricSpec] {
        SPECS
    }

    fn compute(&self, program: &Program) -> ProgramMetrics {
        let mut result = per_file(
            program,
            |_| (),
            |file, _| Counts::of(&file.tokens).metrics(),
            |file, _, function| Counts::of(file.tokens_in(function.range)).metrics(),
        );
        let all: Vec<&Token> = program.files.iter().flat_map(|f| &f.tokens).collect();
        result.project = Counts::of(all).metrics();
        result
    }
}

fn is_operand(t: &Token) -> bool {
    matches!(t.kind, TokenKind::Identifier | TokenKind::Literal)
}

fn is_operator(t: &Token) -> bool {
    match t.kind {
        TokenKind::Keyword | TokenKind::Operator => true,
        TokenKind::Punctuation => matches!(t.text.as_str(), "(" | "[" | "{"),
        _ => false,
    }
}

#[derive(Default)]
struct Counts<'a> {
    operators: HashSet<&'a str>,
    operands: HashSet<&'a str>,
    total_operators: usize,
    total_operands: usize,
}

impl<'a> Counts<'a> {
    fn of(tokens: impl IntoIterator<Item = &'a Token>) -> Counts<'a> {
        let mut c = Counts::default();
        for t in tokens {
            if is_operand(t) {
                c.operands.insert(&t.text);
                c.total_operands += 1;
            } else if is_operator(t) {
                c.operators.insert(&t.text);
                c.total_operators += 1;
            }
        }
        c
    }

    fn metrics(&self) -> Metrics {
        let (n1, n2) = (self.operators.len() as f64, self.operands.len() as f64);
        let (big_n1, big_n2) = (self.total_operators as f64, self.total_operands as f64);
        let (n, big_n) = (n1 + n2, big_n1 + big_n2);
        let volume = (n > 0.0).then(|| big_n * n.log2());
        let difficulty = (n2 > 0.0).then(|| n1 / 2.0 * (big_n2 / n2));
        let effort = volume.zip(difficulty).map(|(v, d)| v * d);
        let value = |x: Option<f64>| x.map_or(MetricValue::NotApplicable, MetricValue::Available);
        Metrics::from([
            ("halstead.unique_operators", n1.into()),
            ("halstead.unique_operands", n2.into()),
            ("halstead.total_operators", big_n1.into()),
            ("halstead.total_operands", big_n2.into()),
            ("halstead.vocabulary", n.into()),
            ("halstead.length", big_n.into()),
            ("halstead.volume", value(volume)),
            ("halstead.difficulty", value(difficulty)),
            ("halstead.effort", value(effort)),
            ("halstead.time", value(effort.map(|e| e / 18.0))),
            ("halstead.bugs", value(volume.map(|v| v / 3000.0))),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::builder::*;
    use crate::ir::{Program, TokenKind::*};
    use crate::metrics::{Calculator, MetricValue};

    fn close(value: &MetricValue, expected: f64) {
        match value {
            MetricValue::Available(v) => assert!((v - expected).abs() < 1e-9, "{v} != {expected}"),
            other => panic!("{other:?}"),
        }
    }

    /// `x = f(x, 1); // c`
    fn sample() -> Program {
        let mut b = FileBuilder::new("x = f(x, 1); // c\n");
        let root = b.root();
        b.function(root, "g", 0, lines(1, 1));
        b.token(Identifier, "x", 1)
            .token(Operator, "=", 1)
            .token(Identifier, "f", 1);
        b.token(Punctuation, "(", 1)
            .token(Identifier, "x", 1)
            .token(Punctuation, ",", 1);
        b.token(Literal, "1", 1)
            .token(Punctuation, ")", 1)
            .token(Punctuation, ";", 1);
        b.token(Comment, "// c", 1);
        Program {
            files: vec![b.build()],
        }
    }

    #[test]
    fn classifies_operators_and_operands() {
        let m = &HalsteadCalculator.compute(&sample()).files[0].metrics;
        // operators: "=", "("  operands: x, f, x, 1
        close(&m["halstead.unique_operators"], 2.0);
        close(&m["halstead.total_operators"], 2.0);
        close(&m["halstead.unique_operands"], 3.0);
        close(&m["halstead.total_operands"], 4.0);
        close(&m["halstead.vocabulary"], 5.0);
        close(&m["halstead.length"], 6.0);
    }

    #[test]
    fn derived_measures_follow_halstead_formulas() {
        let m = &HalsteadCalculator.compute(&sample()).files[0].metrics;
        let volume = 6.0 * 5f64.log2();
        let difficulty = (2.0 / 2.0) * (4.0 / 3.0);
        close(&m["halstead.volume"], volume);
        close(&m["halstead.difficulty"], difficulty);
        close(&m["halstead.effort"], difficulty * volume);
        close(&m["halstead.time"], difficulty * volume / 18.0);
        close(&m["halstead.bugs"], volume / 3000.0);
    }

    #[test]
    fn function_scope_uses_tokens_in_the_function_range() {
        let m = &HalsteadCalculator.compute(&sample()).files[0].functions[0];
        close(&m["halstead.length"], 6.0);
    }

    #[test]
    fn project_counts_distinct_tokens_across_files() {
        let mut program = sample();
        program.files.push(program.files[0].clone());
        let m = HalsteadCalculator.compute(&program).project;
        close(&m["halstead.unique_operands"], 3.0);
        close(&m["halstead.total_operands"], 8.0);
    }

    #[test]
    fn empty_input_has_no_derived_measures() {
        let program = Program {
            files: vec![FileBuilder::new("").build()],
        };
        let m = &HalsteadCalculator.compute(&program).files[0].metrics;
        close(&m["halstead.length"], 0.0);
        assert_eq!(m["halstead.volume"], MetricValue::NotApplicable);
        assert_eq!(m["halstead.difficulty"], MetricValue::NotApplicable);
        assert_eq!(m["halstead.effort"], MetricValue::NotApplicable);
    }
}
