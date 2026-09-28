//! Function Metrics.
//!
//! Function Length, Statement Count, Complexity, Maximum Nesting Depth and Return Count are
//! provided at function scope by the Size, Complexity and Nesting calculators.

use super::{Calculator, MetricSpec, MetricValue, Metrics, ProgramMetrics, Scope::*, per_file};
use crate::ir::{File, Function, Node, NodeKind, Program};

pub struct FunctionCalculator;

static SPECS: &[MetricSpec] = &[
    MetricSpec {
        id: "function.parameter_count",
        scopes: &[Function],
    },
    MetricSpec {
        id: "function.avg_parameter_count",
        scopes: &[File, Project],
    },
    MetricSpec {
        id: "function.max_parameter_count",
        scopes: &[File, Project],
    },
    MetricSpec {
        id: "function.expression_count",
        scopes: &[Function, File, Project],
    },
    MetricSpec {
        id: "function.call_count",
        scopes: &[Function, File, Project],
    },
];

impl Calculator for FunctionCalculator {
    fn specs(&self) -> &'static [MetricSpec] {
        SPECS
    }

    fn compute(&self, program: &Program) -> ProgramMetrics {
        let mut result = per_file(
            program,
            |_| (),
            |f, _| file_metrics(&[f]),
            |f, _, func| function_metrics(f, func),
        );
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
    m.insert("function.avg_parameter_count", MetricValue::mean(&counts));
    m.insert("function.max_parameter_count", MetricValue::max(&counts));
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
