//! Size Metrics (PLAN.md §8.1).
//!
//! Size metrics are textual: at function scope they include the text of nested functions.

use super::common::{LineClass, is_statement, line_classes};
use super::{
    Applicability::*, Calculator, FileMetrics, MetricDefinition, MetricValue, Metrics,
    ProgramMetrics, Scope::*,
};
use crate::ir::{File, Function, Program, TokenKind};

pub struct SizeCalculator;

const LINES: &str = "File source text, Token ranges";

static DEFINITIONS: &[MetricDefinition] = &[
    MetricDefinition {
        id: "size.loc",
        name: "LOC",
        description: "Physical lines of code.",
        definition: "Number of lines in the file.",
        scopes: &[File, Project],
        input: "File source text",
        calculation: "Count of lines; a trailing newline does not start a new line. Project: sum over files.",
        unit: "lines",
        applicability: LanguageIndependent,
        limitations: "",
        reference: "",
    },
    MetricDefinition {
        id: "size.sloc",
        name: "SLOC",
        description: "Source lines of code.",
        definition: "Lines occupied by at least one non-comment token.",
        scopes: &[Function, File, Project],
        input: LINES,
        calculation: "A token spanning several lines (e.g. a multi-line string) occupies each of them. \
                      Function: lines of the function's range (including nested functions). Project: sum.",
        unit: "lines",
        applicability: LanguageIndependent,
        limitations: "",
        reference: "",
    },
    MetricDefinition {
        id: "size.comment_loc",
        name: "Comment LOC",
        description: "Lines containing only comments.",
        definition: "Lines occupied by a comment token and by no other token.",
        scopes: &[File, Project],
        input: LINES,
        calculation: "Lines with code and a trailing comment are SLOC, not Comment LOC. Project: sum.",
        unit: "lines",
        applicability: LanguageIndependent,
        limitations: "Documentation strings that are string literals (e.g. Python docstrings) count as SLOC.",
        reference: "",
    },
    MetricDefinition {
        id: "size.blank_loc",
        name: "Blank LOC",
        description: "Blank lines.",
        definition: "Whitespace-only lines not occupied by any token.",
        scopes: &[File, Project],
        input: LINES,
        calculation: "Blank lines inside a multi-line comment or string are not blank. Project: sum.",
        unit: "lines",
        applicability: LanguageIndependent,
        limitations: "",
        reference: "",
    },
    MetricDefinition {
        id: "size.comment_ratio",
        name: "Comment Ratio",
        description: "Share of comment lines.",
        definition: "Comment LOC / LOC.",
        scopes: &[File, Project],
        input: "size.comment_loc, size.loc",
        calculation: "not_applicable when LOC is 0. Project: ratio of the sums.",
        unit: "ratio",
        applicability: LanguageIndependent,
        limitations: "",
        reference: "",
    },
    MetricDefinition {
        id: "size.statement_count",
        name: "Statement Count",
        description: "Number of statements.",
        definition: "Nodes of kind statement, declaration, branch, loop, return or jump.",
        scopes: &[Function, File, Project],
        input: "Node kinds",
        calculation: "Function scope includes nested functions. Project: sum.",
        unit: "count",
        applicability: PartiallyLanguageDependent,
        limitations: "Languages differ in what is a statement (e.g. C for-loop initializer declarations count; Python has no equivalent).",
        reference: "",
    },
    MetricDefinition {
        id: "size.token_count",
        name: "Token Count",
        description: "Number of non-comment tokens.",
        definition: "Tokens of every kind except comment.",
        scopes: &[Function, File, Project],
        input: "Tokens",
        calculation: "A string literal is one token. Function scope includes nested functions. Project: sum.",
        unit: "count",
        applicability: PartiallyLanguageDependent,
        limitations: "Token granularity follows each grammar (e.g. C `#include` is one token).",
        reference: "",
    },
    MetricDefinition {
        id: "size.function_count",
        name: "Function Count",
        description: "Number of functions.",
        definition: "Functions in the IR, including methods, nested and anonymous functions.",
        scopes: &[File, Project],
        input: "Functions",
        calculation: "Project: sum.",
        unit: "count",
        applicability: LanguageIndependent,
        limitations: "",
        reference: "",
    },
    MetricDefinition {
        id: "size.function_length",
        name: "Function Length",
        description: "Lines spanned by a function.",
        definition: "Last line - first line + 1 of the function's source range.",
        scopes: &[Function],
        input: "Function source range",
        calculation: "Includes the signature, blank and comment lines, and nested functions.",
        unit: "lines",
        applicability: LanguageIndependent,
        limitations: "",
        reference: "",
    },
    MetricDefinition {
        id: "size.avg_function_length",
        name: "Average Function Length",
        description: "Mean Function Length.",
        definition: "Mean of size.function_length over all functions.",
        scopes: &[File, Project],
        input: "size.function_length",
        calculation: "not_applicable when there are no functions.",
        unit: "lines",
        applicability: LanguageIndependent,
        limitations: "",
        reference: "",
    },
    MetricDefinition {
        id: "size.max_function_length",
        name: "Maximum Function Length",
        description: "Longest Function Length.",
        definition: "Maximum of size.function_length over all functions.",
        scopes: &[File, Project],
        input: "size.function_length",
        calculation: "not_applicable when there are no functions.",
        unit: "lines",
        applicability: LanguageIndependent,
        limitations: "",
        reference: "",
    },
];

impl Calculator for SizeCalculator {
    fn definitions(&self) -> &'static [MetricDefinition] {
        DEFINITIONS
    }

    fn compute(&self, program: &Program) -> ProgramMetrics {
        let classified: Vec<Classified> = program.files.iter().map(|f| (f, classify(f))).collect();
        let files = classified
            .iter()
            .map(|(file, lines)| FileMetrics {
                metrics: totals(&[(file, lines.clone())]),
                functions: file
                    .functions
                    .iter()
                    .map(|f| function_metrics(file, lines, f))
                    .collect(),
            })
            .collect();
        ProgramMetrics {
            project: totals(&classified),
            files,
        }
    }
}

/// A file with its line classes, or the reason they are unavailable.
type Classified<'a> = (&'a File, Result<Vec<LineClass>, String>);

fn classify(file: &File) -> Result<Vec<LineClass>, String> {
    line_classes(file).map_err(|e| format!("{}: {e}", file.path))
}

fn count(lines: &[LineClass], class: LineClass) -> usize {
    lines.iter().filter(|c| **c == class).count()
}

/// Line metrics summed over files; an error in any file makes them all errors.
fn line_metrics(files: &[Classified]) -> Metrics {
    let ids = [
        "size.loc",
        "size.sloc",
        "size.comment_loc",
        "size.blank_loc",
    ];
    let mut sums = [0usize; 4];
    for (_, lines) in files {
        match lines {
            Ok(lines) => {
                let counts = [
                    lines.len(),
                    count(lines, LineClass::Code),
                    count(lines, LineClass::Comment),
                    count(lines, LineClass::Blank),
                ];
                sums.iter_mut().zip(counts).for_each(|(s, c)| *s += c);
            }
            Err(e) => {
                let error = MetricValue::Error(e.clone());
                return ids
                    .iter()
                    .chain(["size.comment_ratio"].iter())
                    .map(|id| (*id, error.clone()))
                    .collect();
            }
        }
    }
    let mut m: Metrics = ids
        .iter()
        .zip(sums)
        .map(|(id, v)| (*id, v.into()))
        .collect();
    m.insert(
        "size.comment_ratio",
        MetricValue::ratio(sums[2] as f64, sums[0] as f64),
    );
    m
}

/// File or project totals: sums over `files`, with ratios and function statistics recomputed.
fn totals(files: &[Classified]) -> Metrics {
    let mut m = line_metrics(files);
    let statements: usize = files
        .iter()
        .map(|(f, _)| f.nodes.iter().filter(|n| is_statement(n.kind)).count())
        .sum();
    let tokens: usize = files.iter().map(|(f, _)| code_tokens(&f.tokens)).sum();
    let lengths: Vec<usize> = files
        .iter()
        .flat_map(|(f, _)| &f.functions)
        .map(|f| f.range.line_count())
        .collect();
    m.insert("size.statement_count", statements.into());
    m.insert("size.token_count", tokens.into());
    m.insert("size.function_count", lengths.len().into());
    m.insert("size.avg_function_length", MetricValue::mean(&lengths));
    m.insert("size.max_function_length", MetricValue::max(&lengths));
    m
}

fn function_metrics(
    file: &File,
    lines: &Result<Vec<LineClass>, String>,
    function: &Function,
) -> Metrics {
    let statements = file
        .descendants_pruned(function.node, |_| false)
        .filter(|n| is_statement(n.kind))
        .count();
    let r = function.range;
    let sloc = match lines {
        Ok(lines) => count(&lines[r.first_line() - 1..r.last_line()], LineClass::Code).into(),
        Err(e) => MetricValue::Error(e.clone()),
    };
    Metrics::from([
        ("size.function_length", r.line_count().into()),
        ("size.sloc", sloc),
        ("size.statement_count", statements.into()),
        ("size.token_count", code_tokens(file.tokens_in(r)).into()),
    ])
}

fn code_tokens(tokens: &[crate::ir::Token]) -> usize {
    tokens
        .iter()
        .filter(|t| t.kind != TokenKind::Comment)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::builder::*;
    use crate::ir::{NodeKind, Program, TokenKind::*};
    use crate::metrics::{Calculator, MetricValue, ProgramMetrics};

    /// ```text
    /// 1 int x; // c
    /// 2
    /// 3 /* a
    /// 4    b */
    /// 5 void f() {
    /// 6   g();
    /// 7 }
    /// ```
    fn sample() -> Program {
        let source = "int x; // c\n\n/* a\n   b */\nvoid f() {\n  g();\n}\n";
        let mut b = FileBuilder::new(source);
        let root = b.root();
        b.node(root, NodeKind::Declaration);
        let body = b.function(root, "f", 0, lines(5, 7));
        b.node(body, NodeKind::Statement);
        b.token(Keyword, "int", 1)
            .token(Identifier, "x", 1)
            .token(Punctuation, ";", 1)
            .token(Comment, "// c", 1);
        b.token_span(Comment, "/* a b */", 3, 4);
        b.token(Keyword, "void", 5)
            .token(Identifier, "f", 5)
            .token(Punctuation, "(", 5);
        b.token(Punctuation, ")", 5).token(Punctuation, "{", 5);
        b.token(Identifier, "g", 6)
            .token(Punctuation, "(", 6)
            .token(Punctuation, ")", 6);
        b.token(Punctuation, ";", 6);
        b.token(Punctuation, "}", 7);
        Program {
            files: vec![b.build()],
        }
    }

    fn run(program: &Program) -> ProgramMetrics {
        SizeCalculator.compute(program)
    }

    fn v(x: f64) -> MetricValue {
        MetricValue::Available(x)
    }

    #[test]
    fn file_line_counts() {
        let m = &run(&sample()).files[0].metrics;
        assert_eq!(m["size.loc"], v(7.0));
        assert_eq!(m["size.sloc"], v(4.0));
        assert_eq!(m["size.comment_loc"], v(2.0));
        assert_eq!(m["size.blank_loc"], v(1.0));
        assert_eq!(m["size.comment_ratio"], v(2.0 / 7.0));
    }

    #[test]
    fn file_structure_counts() {
        let m = &run(&sample()).files[0].metrics;
        assert_eq!(m["size.statement_count"], v(2.0));
        assert_eq!(m["size.token_count"], v(13.0));
        assert_eq!(m["size.function_count"], v(1.0));
        assert_eq!(m["size.avg_function_length"], v(3.0));
        assert_eq!(m["size.max_function_length"], v(3.0));
    }

    #[test]
    fn function_counts() {
        let m = &run(&sample()).files[0].functions[0];
        assert_eq!(m["size.function_length"], v(3.0));
        assert_eq!(m["size.sloc"], v(3.0));
        assert_eq!(m["size.statement_count"], v(1.0));
        assert_eq!(m["size.token_count"], v(10.0));
    }

    #[test]
    fn project_sums_files_and_recomputes_ratios() {
        let mut program = sample();
        program.files.push(program.files[0].clone());
        let m = run(&program).project;
        assert_eq!(m["size.loc"], v(14.0));
        assert_eq!(m["size.comment_ratio"], v(4.0 / 14.0));
        assert_eq!(m["size.function_count"], v(2.0));
        assert_eq!(m["size.max_function_length"], v(3.0));
    }

    #[test]
    fn empty_file_has_no_ratio_and_no_function_averages() {
        let program = Program {
            files: vec![FileBuilder::new("").build()],
        };
        let m = &run(&program).files[0].metrics;
        assert_eq!(m["size.loc"], v(0.0));
        assert_eq!(m["size.comment_ratio"], MetricValue::NotApplicable);
        assert_eq!(m["size.avg_function_length"], MetricValue::NotApplicable);
        assert_eq!(m["size.max_function_length"], MetricValue::NotApplicable);
    }

    #[test]
    fn text_not_covered_by_tokens_is_an_error_not_a_guess() {
        let program = Program {
            files: vec![FileBuilder::new("???\n").build()],
        };
        let m = &run(&program).files[0].metrics;
        assert!(
            matches!(m["size.sloc"], MetricValue::Error(_)),
            "{:?}",
            m["size.sloc"]
        );
    }
}
