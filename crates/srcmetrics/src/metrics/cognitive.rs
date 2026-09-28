//! Cognitive Complexity (ADR-0014).

use super::common::{is_continuation, is_nesting, nesting_levels};
use super::{
    Applicability::*, Calculator, MetricDefinition, Metrics, ProgramMetrics, Scope::*, per_file,
};
use crate::ir::{File, Function, Node, NodeKind, Program};

pub struct CognitiveCalculator;

static DEFINITIONS: &[MetricDefinition] = &[MetricDefinition {
    id: "complexity.cognitive",
    name: "Cognitive Complexity",
    description: "How hard a function's control flow is to understand (SonarSource).",
    definition: "Sum of increments for breaks in linear flow, weighted by nesting.",
    scopes: &[Function, File, Project],
    input: "Node kinds, parent links, call and logical labels",
    calculation: "if-chain head, loop, catch, ternary, and each run of case labels (a switch): 1 + nesting \
                  level. else if / elif and else: 1. Each sequence of like logical operators: 1. A call \
                  to the function's own name (recursion): 1. Nesting levels are opened by branches, loops, \
                  cases, catches and ternaries. File: sum over functions. Project: sum over files.",
    unit: "count",
    applicability: PartiallyLanguageDependent,
    limitations: "Nested functions (lambdas) are measured separately instead of adding to the enclosing \
                  function. Labelled break / continue and goto add nothing (jumps have no labels in the IR).",
    reference: "Campbell, G. A. (2018). Cognitive Complexity: A new way of measuring understandability. SonarSource.",
}];

impl Calculator for CognitiveCalculator {
    fn definitions(&self) -> &'static [MetricDefinition] {
        DEFINITIONS
    }

    fn compute(&self, program: &Program) -> ProgramMetrics {
        let mut result = per_file(
            program,
            levels,
            |file, levels| {
                Metrics::from([("complexity.cognitive", file_total(file, levels).into())])
            },
            |file, levels, function| {
                Metrics::from([(
                    "complexity.cognitive",
                    cognitive(file, levels, function).into(),
                )])
            },
        );
        let total: usize = program
            .files
            .iter()
            .map(|f| file_total(f, &levels(f)))
            .sum();
        result.project.insert("complexity.cognitive", total.into());
        result
    }
}

fn levels(file: &File) -> Vec<usize> {
    nesting_levels(file, |n| {
        is_nesting(n.kind) || n.kind == NodeKind::Conditional
    })
}

fn file_total(file: &File, levels: &[usize]) -> usize {
    file.functions
        .iter()
        .map(|f| cognitive(file, levels, f))
        .sum()
}

fn cognitive(file: &File, levels: &[usize], function: &Function) -> usize {
    file.function_nodes(function)
        .map(|n| increment(file, levels, function, n))
        .sum()
}

fn increment(file: &File, levels: &[usize], function: &Function, node: &Node) -> usize {
    let nested = 1 + levels[node.id.0];
    let parent = node.parent.map(|p| file.node(p));
    match node.kind {
        NodeKind::Branch if is_continuation(file, node) => 1,
        NodeKind::Branch | NodeKind::Loop | NodeKind::Catch | NodeKind::Conditional => nested,
        // An `else` wrapping an `else if` is counted by that branch.
        NodeKind::Else => usize::from(
            !node
                .children
                .iter()
                .any(|c| is_continuation(file, file.node(*c))),
        ),
        NodeKind::Case => {
            let siblings = &parent.expect("a case has a parent").children;
            let position = siblings
                .iter()
                .position(|c| *c == node.id)
                .expect("a node is among its parent's children");
            let starts_switch =
                position == 0 || file.node(siblings[position - 1]).kind != NodeKind::Case;
            if starts_switch { nested } else { 0 }
        }
        NodeKind::Logical => usize::from(
            !parent.is_some_and(|p| p.kind == NodeKind::Logical && p.label == node.label),
        ),
        NodeKind::Call => usize::from(node.label.is_some() && node.label == function.name),
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::NodeKind::*;
    use crate::ir::builder::*;
    use crate::ir::{NodeId, Program};
    use crate::metrics::{Calculator, MetricValue};

    fn cognitive_of(build: impl FnOnce(&mut FileBuilder, NodeId)) -> MetricValue {
        let mut b = FileBuilder::new("");
        let root = b.root();
        let body = b.function(root, "f", 0, lines(1, 1));
        build(&mut b, body);
        let r = CognitiveCalculator.compute(&Program {
            files: vec![b.build()],
        });
        r.files[0].functions[0]["complexity.cognitive"].clone()
    }

    fn v(x: f64) -> MetricValue {
        MetricValue::Available(x)
    }

    #[test]
    fn nesting_increments_structures() {
        // for { if { while {} } }  => 1 + 2 + 3
        let c = cognitive_of(|b, f| {
            let lp = b.node(f, Loop);
            let br = b.node(lp, Branch);
            b.node(br, Loop);
        });
        assert_eq!(c, v(6.0));
    }

    #[test]
    fn else_if_and_else_are_flat_increments() {
        // C-style: if {} else if {} else {}  => 1 + 1 + 1
        let c = cognitive_of(|b, f| {
            let head = b.node(f, Branch);
            let e = b.node(head, Else);
            let cont = b.node(e, Branch);
            b.node(cont, Else);
        });
        assert_eq!(c, v(3.0));
        // Python-style: if: elif: else:  => 1 + 1 + 1
        let c = cognitive_of(|b, f| {
            let head = b.node(f, Branch);
            b.node(head, Branch);
            b.node(head, Else);
        });
        assert_eq!(c, v(3.0));
    }

    #[test]
    fn a_switch_counts_once() {
        // loop { switch { case, case, case } } => 1 + (1 + 1)
        let c = cognitive_of(|b, f| {
            let lp = b.node(f, Loop);
            let block = b.node(lp, Block);
            for _ in 0..3 {
                b.node(block, Case);
            }
        });
        assert_eq!(c, v(3.0));
    }

    #[test]
    fn sequences_of_like_logical_operators_count_once() {
        // a && b && c || d  => 2
        let c = cognitive_of(|b, f| {
            let or = b.labelled(f, Logical, "||");
            let and1 = b.labelled(or, Logical, "&&");
            b.labelled(and1, Logical, "&&");
        });
        assert_eq!(c, v(2.0));
    }

    #[test]
    fn recursion_and_catch_and_ternary() {
        // try {} catch { f() } ; x ? y : z  => catch 1, recursion 1, ternary 1
        let c = cognitive_of(|b, f| {
            let catch = b.node(f, Catch);
            b.labelled(catch, Call, "f");
            b.labelled(f, Call, "g");
            b.node(f, Conditional);
        });
        assert_eq!(c, v(3.0));
    }

    #[test]
    fn nested_functions_are_measured_separately() {
        let mut b = FileBuilder::new("");
        let root = b.root();
        let outer = b.function(root, "outer", 0, lines(1, 3));
        let lp = b.node(outer, Loop);
        let inner = b.function(lp, "inner", 0, lines(2, 2));
        b.node(inner, Branch);
        let r = CognitiveCalculator.compute(&Program {
            files: vec![b.build()],
        });
        assert_eq!(r.files[0].functions[0]["complexity.cognitive"], v(1.0));
        assert_eq!(r.files[0].functions[1]["complexity.cognitive"], v(1.0));
        assert_eq!(r.files[0].metrics["complexity.cognitive"], v(2.0));
        assert_eq!(r.project["complexity.cognitive"], v(2.0));
    }
}
