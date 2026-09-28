//! Function Metrics (PLAN.md §8.4).
//!
//! Function Length, Statement Count, Complexity, Maximum Nesting Depth and Return Count are
//! provided at function scope by the Size, Complexity and Nesting calculators.

use super::{
    Applicability::*, Calculator, MetricDefinition, MetricValue, Metrics, ProgramMetrics, Scope::*,
    per_file,
};
use crate::ir::{File, Function, Node, NodeKind, Program};

pub struct FunctionCalculator;

static DEFINITIONS: &[MetricDefinition] = &[
    MetricDefinition {
        id: "function.parameter_count",
        name: "Parameter Count",
        description: "Number of declared parameters.",
        definition: "Parameters of the function in the IR.",
        scopes: &[Function],
        input: "Function parameters",
        calculation: "Variadic parameters (e.g. `*args`) count as one. Separators such as Python `*` and `/` \
                      and C `(void)` are not parameters.",
        unit: "count",
        applicability: PartiallyLanguageDependent,
        limitations: "Explicit receivers (Python `self`) count; implicit ones (`this`) do not.",
        reference: "",
    },
    MetricDefinition {
        id: "function.avg_parameter_count",
        name: "Average Parameter Count",
        description: "Mean Parameter Count over functions.",
        definition: "Mean of function.parameter_count.",
        scopes: &[File, Project],
        input: "Function parameters",
        calculation: "not_applicable when there are no functions.",
        unit: "count",
        applicability: PartiallyLanguageDependent,
        limitations: "See function.parameter_count.",
        reference: "",
    },
    MetricDefinition {
        id: "function.max_parameter_count",
        name: "Maximum Parameter Count",
        description: "Largest Parameter Count over functions.",
        definition: "Maximum of function.parameter_count.",
        scopes: &[File, Project],
        input: "Function parameters",
        calculation: "not_applicable when there are no functions.",
        unit: "count",
        applicability: PartiallyLanguageDependent,
        limitations: "See function.parameter_count.",
        reference: "",
    },
    MetricDefinition {
        id: "function.expression_count",
        name: "Expression Count",
        description: "Number of expressions, including sub-expressions.",
        definition: "Nodes of kind expression, call, assignment, binary, logical, conditional or unary.",
        scopes: &[Function, File, Project],
        input: "Node kinds",
        calculation: "Function: excluding nested functions. File: the whole file. Project: sum. \
                      Identifiers and literals are not expressions on their own.",
        unit: "count",
        applicability: PartiallyLanguageDependent,
        limitations: "Expression forms not listed in a language Mapping (e.g. lambdas' bodies are nested \
                      functions; unmapped expression types are `other`) are not counted.",
        reference: "",
    },
    MetricDefinition {
        id: "function.call_count",
        name: "Call Count",
        description: "Number of call sites.",
        definition: "Nodes of kind call (function calls and constructor calls).",
        scopes: &[Function, File, Project],
        input: "Node kinds",
        calculation: "Function: excluding nested functions. File: the whole file. Project: sum.",
        unit: "count",
        applicability: LanguageIndependent,
        limitations: "",
        reference: "",
    },
];

impl Calculator for FunctionCalculator {
    fn definitions(&self) -> &'static [MetricDefinition] {
        DEFINITIONS
    }

    fn compute(&self, program: &Program) -> ProgramMetrics {
        let mut result = per_file(program, |f| file_metrics(&[f]), function_metrics);
        result.project = file_metrics(&program.files.iter().collect::<Vec<_>>());
        result
    }
}

fn is_expression(kind: NodeKind) -> bool {
    use NodeKind::*;
    matches!(
        kind,
        Expression | Call | Assignment | Binary | Logical | Conditional | Unary
    )
}

fn node_metrics<'a>(nodes: impl Iterator<Item = &'a Node>) -> Metrics {
    let (mut expressions, mut calls) = (0usize, 0usize);
    for n in nodes {
        expressions += is_expression(n.kind) as usize;
        calls += (n.kind == NodeKind::Call) as usize;
    }
    Metrics::from([
        ("function.expression_count", expressions.into()),
        ("function.call_count", calls.into()),
    ])
}

fn function_metrics(file: &File, function: &Function) -> Metrics {
    let mut m = node_metrics(file.function_nodes(function));
    m.insert("function.parameter_count", function.parameters.len().into());
    m
}

fn file_metrics(files: &[&File]) -> Metrics {
    let mut m = node_metrics(files.iter().flat_map(|f| &f.nodes));
    let counts: Vec<usize> = files
        .iter()
        .flat_map(|f| &f.functions)
        .map(|f| f.parameters.len())
        .collect();
    let total: usize = counts.iter().sum();
    m.insert(
        "function.avg_parameter_count",
        MetricValue::ratio(total as f64, counts.len() as f64),
    );
    m.insert(
        "function.max_parameter_count",
        counts
            .iter()
            .max()
            .map_or(MetricValue::NotApplicable, |&x| x.into()),
    );
    m
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

    /// f(a, b, c): x = g(1 + 2); lambda(y) { h() }
    /// k(): (none)
    fn sample() -> Program {
        let mut b = FileBuilder::new("");
        let root = b.root();
        let f = b.function(root, "f", 3, lines(1, 3));
        let stmt = b.node(f, Statement);
        let assign = b.node(stmt, Assignment);
        b.node(assign, Identifier);
        let call = b.node(assign, Call);
        b.node(call, Binary);
        let lambda = b.function(f, "lambda", 1, lines(2, 2));
        b.node(lambda, Call);
        b.function(root, "k", 0, lines(4, 4));
        b.node(root, Call);
        Program {
            files: vec![b.build()],
        }
    }

    #[test]
    fn function_scope_counts_exclude_nested_functions() {
        let result = FunctionCalculator.compute(&sample());
        let f = &result.files[0].functions[0];
        assert_eq!(f["function.parameter_count"], v(3.0));
        assert_eq!(f["function.expression_count"], v(3.0));
        assert_eq!(f["function.call_count"], v(1.0));
    }

    #[test]
    fn file_scope_counts_the_whole_file_and_parameter_statistics() {
        let m = &FunctionCalculator.compute(&sample()).files[0].metrics;
        assert_eq!(m["function.call_count"], v(3.0));
        assert_eq!(m["function.expression_count"], v(5.0));
        assert_eq!(m["function.avg_parameter_count"], v(4.0 / 3.0));
        assert_eq!(m["function.max_parameter_count"], v(3.0));
    }

    #[test]
    fn project_aggregates_files() {
        let mut program = sample();
        program.files.push(program.files[0].clone());
        let m = FunctionCalculator.compute(&program).project;
        assert_eq!(m["function.call_count"], v(6.0));
        assert_eq!(m["function.avg_parameter_count"], v(4.0 / 3.0));
    }

    #[test]
    fn parameter_statistics_need_functions() {
        let program = Program {
            files: vec![FileBuilder::new("").build()],
        };
        let m = &FunctionCalculator.compute(&program).files[0].metrics;
        assert_eq!(
            m["function.avg_parameter_count"],
            MetricValue::NotApplicable
        );
        assert_eq!(
            m["function.max_parameter_count"],
            MetricValue::NotApplicable
        );
    }
}
