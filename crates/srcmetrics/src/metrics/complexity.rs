//! Complexity Metrics (ADR-0007, ADR-0011).

use super::common::{is_continuation, is_decision};
use super::{
    Applicability::*, Calculator, MetricDefinition, Metrics, ProgramMetrics, Scope::*, per_file,
};
use crate::ir::{File, Function, Node, NodeId, NodeKind, Program};

pub struct ComplexityCalculator;

const ALL_SCOPES: &[super::Scope] = &[Function, File, Project];

const fn count(id: &'static str, name: &'static str, definition: &'static str) -> MetricDefinition {
    MetricDefinition {
        id,
        name,
        description: definition,
        definition,
        scopes: ALL_SCOPES,
        input: "Node kinds",
        calculation: "Function: excluding nested functions. File: the whole file. Project: sum over files.",
        unit: "count",
        applicability: PartiallyLanguageDependent,
        limitations: "Which constructs map to each node kind follows each language Mapping.",
        reference: "",
    }
}

static DEFINITIONS: &[MetricDefinition] = &[
    MetricDefinition {
        id: "complexity.cyclomatic",
        name: "Cyclomatic Complexity",
        description: "Number of linearly independent paths (McCabe).",
        definition: "1 + number of decision points in a function.",
        scopes: ALL_SCOPES,
        input: "Node kinds: branch, loop, case, catch, logical, conditional",
        calculation: "Function: 1 + decision nodes, excluding nested functions. Each `else if` / `elif`, \
                      each short-circuit operator (&&, ||, and, or), each ternary and each non-default case \
                      label is one decision. File: sum over its functions + decisions in top-level code. \
                      Project: sum over files.",
        unit: "count",
        applicability: PartiallyLanguageDependent,
        limitations: "Which constructs are decisions follows each language Mapping (e.g. Python comprehension \
                      `for`/`if` clauses count; Python `case _:` and Rust `_ =>` count as cases).",
        reference: "McCabe, T. J. (1976). A Complexity Measure. IEEE TSE SE-2(4).",
    },
    count(
        "complexity.branch_count",
        "Branch Count",
        "Number of branch nodes (if, else if, elif) plus non-default case labels.",
    ),
    count(
        "complexity.conditional_count",
        "Conditional Count",
        "Number of conditional (ternary) expressions.",
    ),
    count("complexity.loop_count", "Loop Count", "Number of loops."),
    count(
        "complexity.return_count",
        "Return Count",
        "Number of return statements.",
    ),
    count(
        "complexity.jump_count",
        "Jump Count",
        "Number of jumps: break, continue, goto and throw / raise.",
    ),
    MetricDefinition {
        id: "complexity.path_count",
        name: "Number of Paths",
        description: "Acyclic execution paths through a function.",
        definition: "Number of paths through the function when each loop runs zero times or once.",
        scopes: &[Function],
        input: "Node kinds and tree structure",
        calculation: "Children in sequence multiply. An if-chain is the sum of its arms, +1 without a final \
                      else. A loop or ternary is its children's product + 1. Consecutive case labels or catch \
                      clauses are the sum of their paths + 1. Nested functions count as 1.",
        unit: "count",
        applicability: PartiallyLanguageDependent,
        limitations: "Not Nejmeh's NPATH: short-circuit operators and early exits (return, jump) do not \
                      change the count.",
        reference: "Nejmeh, B. A. (1988). NPATH: a measure of execution path complexity. CACM 31(2) (related, not identical).",
    },
];

impl Calculator for ComplexityCalculator {
    fn definitions(&self) -> &'static [MetricDefinition] {
        DEFINITIONS
    }

    fn compute(&self, program: &Program) -> ProgramMetrics {
        let mut result = per_file(
            program,
            path_counts,
            |file, _| Counts::of(file.nodes.iter()).metrics(),
            |file, paths, function| function_metrics(file, paths, function),
        );
        let project = program
            .files
            .iter()
            .map(|f| Counts::of(f.nodes.iter()))
            .fold(Counts::default(), Counts::add);
        result.project = project.metrics();
        result
    }
}

fn function_metrics(file: &File, paths: &[f64], function: &Function) -> Metrics {
    let counts = Counts {
        functions: 1,
        ..Counts::of(file.function_nodes(function))
    };
    let mut m = counts.metrics();
    m.insert("complexity.path_count", paths[function.node.0].into());
    m
}

/// Node-kind tallies. Cyclomatic = functions + decisions: 1 + decisions per function, and
/// top-level decisions add to the file total.
#[derive(Default, Clone, Copy)]
struct Counts {
    functions: usize,
    decisions: usize,
    branches: usize,
    conditionals: usize,
    loops: usize,
    returns: usize,
    jumps: usize,
}

impl Counts {
    fn of<'a>(nodes: impl Iterator<Item = &'a Node>) -> Counts {
        let mut c = Counts::default();
        for n in nodes {
            c.decisions += is_decision(n.kind) as usize;
            match n.kind {
                NodeKind::Function => c.functions += 1,
                NodeKind::Branch | NodeKind::Case => c.branches += 1,
                NodeKind::Conditional => c.conditionals += 1,
                NodeKind::Loop => c.loops += 1,
                NodeKind::Return => c.returns += 1,
                NodeKind::Jump => c.jumps += 1,
                _ => {}
            }
        }
        c
    }

    fn add(self, o: Counts) -> Counts {
        Counts {
            functions: self.functions + o.functions,
            decisions: self.decisions + o.decisions,
            branches: self.branches + o.branches,
            conditionals: self.conditionals + o.conditionals,
            loops: self.loops + o.loops,
            returns: self.returns + o.returns,
            jumps: self.jumps + o.jumps,
        }
    }

    fn metrics(&self) -> Metrics {
        Metrics::from([
            (
                "complexity.cyclomatic",
                (self.functions + self.decisions).into(),
            ),
            ("complexity.branch_count", self.branches.into()),
            ("complexity.conditional_count", self.conditionals.into()),
            ("complexity.loop_count", self.loops.into()),
            ("complexity.return_count", self.returns.into()),
            ("complexity.jump_count", self.jumps.into()),
        ])
    }
}

/// Number of acyclic paths through every node (ADR-0011), indexed by `NodeId`.
///
/// Computed in reverse arena order, i.e. children before parents (the arena is pre-order), so
/// deep trees such as long `else if` ladders take linear time and no recursion.
fn path_counts(file: &File) -> Vec<f64> {
    let mut paths = vec![1.0; file.nodes.len()];
    // For branches: (sum of the paths of the chain's arms from this branch on, chain ends with else).
    let mut arms = vec![(0.0, false); file.nodes.len()];
    for node in file.nodes.iter().rev() {
        let i = node.id.0;
        paths[i] = match node.kind {
            NodeKind::Branch => {
                arms[i] = chain_arms(file, &paths, &arms, node);
                arms[i].0 + if arms[i].1 { 0.0 } else { 1.0 }
            }
            NodeKind::Loop | NodeKind::Conditional => sequence(file, &paths, &node.children) + 1.0,
            _ => sequence(file, &paths, &node.children),
        };
    }
    paths
}

/// Arms of the if-chain starting at `branch`: the sum of their paths and whether the chain ends
/// with an `else`. Handles both chain shapes: C-style (`else { if ... }`) and Python-style (`elif` children).
fn chain_arms(file: &File, paths: &[f64], arms: &[(f64, bool)], branch: &Node) -> (f64, bool) {
    let is_link = |n: &Node| n.kind == NodeKind::Else || is_continuation(file, n);
    let children = || branch.children.iter().map(|c| file.node(*c));
    let then: Vec<NodeId> = children().filter(|c| !is_link(c)).map(|c| c.id).collect();
    let (mut sum, mut has_else) = (sequence(file, paths, &then), false);
    for link in children().filter(|c| is_link(c)) {
        let next = if link.kind == NodeKind::Else {
            link.children
                .iter()
                .copied()
                .find(|c| is_continuation(file, file.node(*c)))
        } else {
            Some(link.id)
        };
        match next {
            Some(next) => {
                sum += arms[next.0].0;
                has_else |= arms[next.0].1;
            }
            None => {
                sum += paths[link.id.0];
                has_else = true;
            }
        }
    }
    (sum, has_else)
}

/// Paths through children executed in order. Runs of consecutive `case` or `catch` siblings are
/// alternatives (sum + 1); everything else multiplies. Nested functions count as 1.
fn sequence(file: &File, paths: &[f64], children: &[NodeId]) -> f64 {
    let kind = |id: &NodeId| file.node(*id).kind;
    let is_alternative = |k: NodeKind| matches!(k, NodeKind::Case | NodeKind::Catch);
    children
        .chunk_by(|a, b| is_alternative(kind(a)) && kind(a) == kind(b))
        .map(|run| match kind(&run[0]) {
            k if is_alternative(k) => run.iter().map(|c| paths[c.0]).sum::<f64>() + 1.0,
            NodeKind::Function => 1.0,
            _ => paths[run[0].0],
        })
        .product()
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

    #[test]
    fn counts_control_structures() {
        let result = ComplexityCalculator.compute(&sample());
        let f = &result.files[0].functions[0];
        assert_eq!(f["complexity.branch_count"], v(2.0));
        assert_eq!(f["complexity.conditional_count"], v(1.0));
        assert_eq!(f["complexity.loop_count"], v(1.0));
        assert_eq!(f["complexity.return_count"], v(1.0));
        assert_eq!(f["complexity.jump_count"], v(0.0));
        let file = &result.files[0].metrics;
        assert_eq!(file["complexity.branch_count"], v(3.0));
    }

    fn paths_of(build: impl FnOnce(&mut FileBuilder, crate::ir::NodeId)) -> MetricValue {
        let mut b = FileBuilder::new("");
        let root = b.root();
        let body = b.function(root, "f", 0, lines(1, 1));
        build(&mut b, body);
        let result = ComplexityCalculator.compute(&Program {
            files: vec![b.build()],
        });
        result.files[0].functions[0]["complexity.path_count"].clone()
    }

    #[test]
    fn straight_line_code_has_one_path() {
        assert_eq!(
            paths_of(|b, f| {
                b.node(f, Statement);
            }),
            v(1.0)
        );
    }

    #[test]
    fn if_without_else_has_two_paths() {
        assert_eq!(
            paths_of(|b, f| {
                b.node(f, Branch);
            }),
            v(2.0)
        );
    }

    #[test]
    fn c_style_else_if_chain_sums_its_arms() {
        // if {} else if {} else {}
        let p = paths_of(|b, f| {
            let head = b.node(f, Branch);
            let e = b.node(head, Else);
            let cont = b.node(e, Branch);
            b.node(cont, Else);
        });
        assert_eq!(p, v(3.0));
    }

    #[test]
    fn python_style_elif_chain_sums_its_arms() {
        // if: elif: elif:   (no else)
        let p = paths_of(|b, f| {
            let head = b.node(f, Branch);
            b.node(head, Branch);
            b.node(head, Branch);
        });
        assert_eq!(p, v(4.0));
    }

    #[test]
    fn sequential_structures_multiply_and_nested_ones_combine() {
        // if {} ; while { if {} }
        let p = paths_of(|b, f| {
            b.node(f, Branch);
            let lp = b.node(f, Loop);
            b.node(lp, Branch);
        });
        assert_eq!(p, v(2.0 * 3.0));
    }

    #[test]
    fn case_labels_are_alternatives() {
        // switch { case: if {} ; case: }
        let p = paths_of(|b, f| {
            let switch = b.node(f, Block);
            let first = b.node(switch, Case);
            b.node(first, Branch);
            b.node(switch, Case);
        });
        assert_eq!(p, v(2.0 + 1.0 + 1.0));
    }

    #[test]
    fn catch_clauses_are_alternatives_to_the_normal_path() {
        let p = paths_of(|b, f| {
            let try_ = b.node(f, Other);
            b.node(try_, Block);
            b.node(try_, Catch);
            b.node(try_, Catch);
        });
        assert_eq!(p, v(3.0));
    }

    #[test]
    fn ternary_adds_an_alternative() {
        assert_eq!(
            paths_of(|b, f| {
                b.node(f, Conditional);
            }),
            v(2.0)
        );
    }
}
