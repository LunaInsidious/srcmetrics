//! Complexity Metrics (PLAN.md §8.2).

use super::common::is_decision;
use super::{
    Applicability::*, Calculator, MetricDefinition, Metrics, ProgramMetrics, Scope::*, per_file,
};
use crate::ir::{File, Function, Node, Program};

pub struct ComplexityCalculator;

static DEFINITIONS: &[MetricDefinition] = &[MetricDefinition {
    id: "complexity.cyclomatic",
    name: "Cyclomatic Complexity",
    description: "Number of linearly independent paths (McCabe).",
    definition: "1 + number of decision points in a function.",
    scopes: &[Function, File, Project],
    input: "Node kinds: branch, loop, case, catch, logical, conditional",
    calculation: "Function: 1 + decision nodes, excluding nested functions. Each `else if` / `elif`, \
                  each short-circuit operator (&&, ||, and, or), each ternary and each non-default case \
                  label is one decision. File: sum over its functions + decisions in top-level code. \
                  Project: sum over files.",
    unit: "count",
    applicability: PartiallyLanguageDependent,
    limitations: "Which constructs are decisions follows each language Mapping (e.g. Python comprehension \
                  `for`/`if` clauses count; Python `case _` counts as a case).",
    reference: "McCabe, T. J. (1976). A Complexity Measure. IEEE TSE SE-2(4).",
}];

impl Calculator for ComplexityCalculator {
    fn definitions(&self) -> &'static [MetricDefinition] {
        DEFINITIONS
    }

    fn compute(&self, program: &Program) -> ProgramMetrics {
        let mut result = per_file(program, file_metrics, function_metrics);
        let total: usize = program.files.iter().map(file_cyclomatic).sum();
        result.project.insert("complexity.cyclomatic", total.into());
        result
    }
}

fn decisions<'a>(nodes: impl Iterator<Item = &'a Node>) -> usize {
    nodes.filter(|n| is_decision(n.kind)).count()
}

fn cyclomatic(file: &File, function: &Function) -> usize {
    1 + decisions(file.function_nodes(function))
}

fn function_metrics(file: &File, function: &Function) -> Metrics {
    Metrics::from([("complexity.cyclomatic", cyclomatic(file, function).into())])
}

fn file_cyclomatic(file: &File) -> usize {
    let functions: usize = file.functions.iter().map(|f| cyclomatic(file, f)).sum();
    functions + decisions(file.top_level_nodes())
}

fn file_metrics(file: &File) -> Metrics {
    Metrics::from([("complexity.cyclomatic", file_cyclomatic(file).into())])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::NodeKind::*;
    use crate::ir::Program;
    use crate::ir::builder::*;
    use crate::metrics::{Calculator, MetricValue};

    fn v(x: f64) -> MetricValue {
        MetricValue::Available(x)
    }

    /// f: if (a && b) {} else if (c) {} ; while {} ; return x ? 1 : 2
    /// g: straight-line code
    /// top level: one branch
    fn sample() -> Program {
        let mut b = FileBuilder::new("");
        let root = b.root();
        let f = b.function(root, "f", 0, lines(1, 5));
        let branch = b.node(f, Branch);
        b.node(branch, Logical);
        let els = b.node(branch, Else);
        b.node(els, Branch);
        b.node(f, Loop);
        let ret = b.node(f, Return);
        b.node(ret, Conditional);
        let g = b.function(root, "g", 0, lines(6, 7));
        b.node(g, Statement);
        b.node(root, Branch);
        Program {
            files: vec![b.build()],
        }
    }

    #[test]
    fn function_cyclomatic_counts_decision_points_plus_one() {
        let result = ComplexityCalculator.compute(&sample());
        assert_eq!(
            result.files[0].functions[0]["complexity.cyclomatic"],
            v(6.0)
        );
        assert_eq!(
            result.files[0].functions[1]["complexity.cyclomatic"],
            v(1.0)
        );
    }

    #[test]
    fn nested_function_decisions_belong_to_the_nested_function() {
        let mut b = FileBuilder::new("");
        let root = b.root();
        let outer = b.function(root, "outer", 0, lines(1, 5));
        let inner = b.function(outer, "inner", 0, lines(2, 4));
        b.node(inner, Loop);
        let result = ComplexityCalculator.compute(&Program {
            files: vec![b.build()],
        });
        assert_eq!(
            result.files[0].functions[0]["complexity.cyclomatic"],
            v(1.0)
        );
        assert_eq!(
            result.files[0].functions[1]["complexity.cyclomatic"],
            v(2.0)
        );
    }

    #[test]
    fn file_cyclomatic_sums_functions_and_top_level_decisions() {
        let result = ComplexityCalculator.compute(&sample());
        assert_eq!(result.files[0].metrics["complexity.cyclomatic"], v(8.0));
    }

    #[test]
    fn project_cyclomatic_sums_files() {
        let mut program = sample();
        program.files.push(program.files[0].clone());
        assert_eq!(
            ComplexityCalculator.compute(&program).project["complexity.cyclomatic"],
            v(16.0)
        );
    }
}
