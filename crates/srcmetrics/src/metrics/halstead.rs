//! Halstead Metrics (ADR-0008).

use super::{
    Applicability::*, Calculator, MetricDefinition, MetricValue, Metrics, ProgramMetrics, Scope::*,
    per_file,
};
use crate::ir::{Program, Token, TokenKind};
use std::collections::HashSet;

pub struct HalsteadCalculator;

const SCOPES: &[super::Scope] = &[Function, File, Project];
const INPUT: &str = "Tokens (kind and text)";
const CLASSIFICATION: &str = "Operators: keyword and operator tokens and opening brackets ( [ {. \
                              Operands: identifier and literal tokens. Commas, semicolons, closing brackets \
                              and comments are not counted. Distinct = same token text.";
const LIMITATIONS: &str = "Keywords, including type keywords such as `int`, are operators. \
                           A string literal is one operand.";
const REFERENCE: &str = "Halstead, M. H. (1977). Elements of Software Science. Elsevier.";

const fn halstead(
    id: &'static str,
    name: &'static str,
    description: &'static str,
    definition: &'static str,
    unit: &'static str,
) -> MetricDefinition {
    MetricDefinition {
        id,
        name,
        description,
        definition,
        scopes: SCOPES,
        input: INPUT,
        calculation: CLASSIFICATION,
        unit,
        applicability: PartiallyLanguageDependent,
        limitations: LIMITATIONS,
        reference: REFERENCE,
    }
}

static DEFINITIONS: &[MetricDefinition] = &[
    halstead(
        "halstead.unique_operators",
        "Unique Operators",
        "n1.",
        "Number of distinct operators.",
        "count",
    ),
    halstead(
        "halstead.unique_operands",
        "Unique Operands",
        "n2.",
        "Number of distinct operands.",
        "count",
    ),
    halstead(
        "halstead.total_operators",
        "Total Operators",
        "N1.",
        "Number of operator occurrences.",
        "count",
    ),
    halstead(
        "halstead.total_operands",
        "Total Operands",
        "N2.",
        "Number of operand occurrences.",
        "count",
    ),
    halstead(
        "halstead.vocabulary",
        "Vocabulary",
        "n.",
        "n = n1 + n2.",
        "count",
    ),
    halstead(
        "halstead.length",
        "Program Length",
        "N.",
        "N = N1 + N2.",
        "count",
    ),
    halstead(
        "halstead.volume",
        "Volume",
        "V.",
        "V = N * log2(n); not_applicable when n = 0.",
        "bits",
    ),
    halstead(
        "halstead.difficulty",
        "Difficulty",
        "D.",
        "D = (n1 / 2) * (N2 / n2); not_applicable when n2 = 0.",
        "ratio",
    ),
    halstead("halstead.effort", "Effort", "E.", "E = D * V.", "count"),
    halstead(
        "halstead.time",
        "Estimated Program Time",
        "T.",
        "T = E / 18.",
        "seconds",
    ),
    halstead(
        "halstead.bugs",
        "Estimated Bugs",
        "B.",
        "B = V / 3000.",
        "count",
    ),
];

impl Calculator for HalsteadCalculator {
    fn definitions(&self) -> &'static [MetricDefinition] {
        DEFINITIONS
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
