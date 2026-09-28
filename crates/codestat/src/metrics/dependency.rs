//! Dependency Metrics (PLAN.md §8.6, ADR-0012).

use super::{
    Applicability::*, Calculator, FileMetrics, MetricDefinition, Metrics, ProgramMetrics, Scope::*,
};
use crate::ir::{NodeKind, Program};
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub struct DependencyCalculator;

const NAME_BASED: &str = "Calls are resolved by callee name only (no types, scopes or imports).";

static DEFINITIONS: &[MetricDefinition] = &[
    MetricDefinition {
        id: "dependency.fan_out",
        name: "Fan-out",
        description: "Number of distinct functions a function calls.",
        definition: "Distinct callee names of the call nodes in the function.",
        scopes: &[Function],
        input: "Call nodes and their callee labels",
        calculation: "Excludes calls made by nested functions. Includes callees defined outside the project. \
                      Calls without a callee name (e.g. `f()()`) are not counted.",
        unit: "count",
        applicability: PartiallyLanguageDependent,
        limitations: NAME_BASED,
        reference: "Henry, S. & Kafura, D. (1981). Software Structure Metrics Based on Information Flow. IEEE TSE SE-7(5).",
    },
    MetricDefinition {
        id: "dependency.fan_in",
        name: "Fan-in",
        description: "Number of distinct project functions that call a function.",
        definition: "Distinct functions in the project having a call whose callee name is this function's name.",
        scopes: &[Function],
        input: "Call nodes and their callee labels, function names",
        calculation: "Functions with the same name share the value. Anonymous functions have 0.",
        unit: "count",
        applicability: PartiallyLanguageDependent,
        limitations: "Name-based: same-named methods of different classes are not distinguished, which \
                      overestimates fan-in.",
        reference: "Henry, S. & Kafura, D. (1981). Software Structure Metrics Based on Information Flow. IEEE TSE SE-7(5).",
    },
    MetricDefinition {
        id: "dependency.call_depth",
        name: "Call Depth",
        description: "Longest chain of calls through project functions.",
        definition: "Longest path, in edges, from the function's name in the project call graph of \
                     function names, with strongly connected components (recursion) collapsed.",
        scopes: &[Function],
        input: "Call nodes and their callee labels, function names",
        calculation: "Nodes are the names of project functions; a name calls the union of what its \
                      functions call. Edges inside a strongly connected component are not counted. An \
                      anonymous function has 1 + the deepest name it calls. 0 when no project function is called.",
        unit: "calls",
        applicability: PartiallyLanguageDependent,
        limitations: "Name-based: same-named functions share one value.",
        reference: "",
    },
    MetricDefinition {
        id: "dependency.dependency_count",
        name: "Dependency Count",
        description: "Number of import / include declarations.",
        definition: "Nodes of kind import.",
        scopes: &[File, Project],
        input: "Import nodes",
        calculation: "Each imported item that the grammar represents as a separate declaration counts once \
                      (e.g. each Go import spec). Project: sum.",
        unit: "count",
        applicability: PartiallyLanguageDependent,
        limitations: "Import granularity differs between languages (Python `from a import b, c` is one import).",
        reference: "",
    },
];

impl Calculator for DependencyCalculator {
    fn definitions(&self) -> &'static [MetricDefinition] {
        DEFINITIONS
    }

    fn compute(&self, program: &Program) -> ProgramMetrics {
        let graph = CallGraph::of(program);
        let depth = graph.call_depths();
        let mut next = 0;
        let files = program
            .files
            .iter()
            .map(|file| {
                let functions = file
                    .functions
                    .iter()
                    .map(|_| {
                        let f = next;
                        next += 1;
                        Metrics::from([
                            ("dependency.fan_out", graph.callee_names[f].len().into()),
                            ("dependency.fan_in", graph.fan_in(f).into()),
                            ("dependency.call_depth", depth[f].into()),
                        ])
                    })
                    .collect();
                FileMetrics {
                    metrics: Metrics::from([("dependency.dependency_count", imports(file).into())]),
                    functions,
                }
            })
            .collect();
        let total: usize = program.files.iter().map(imports).sum();
        ProgramMetrics {
            project: Metrics::from([("dependency.dependency_count", total.into())]),
            files,
        }
    }
}

fn imports(file: &crate::ir::File) -> usize {
    file.nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Import)
        .count()
}

/// Name-based call graph (ADR-0012). Functions are numbered in file and function order; the
/// graph's nodes are the distinct names of project functions, so same-named functions share a
/// node and every edge is a distinct (caller name, callee name) pair.
struct CallGraph<'a> {
    /// Per function: its name and the distinct names it calls.
    names: Vec<Option<&'a str>>,
    callee_names: Vec<BTreeSet<&'a str>>,
    /// Function name -> functions calling that name.
    callers: HashMap<&'a str, BTreeSet<usize>>,
}

impl<'a> CallGraph<'a> {
    fn of(program: &'a Program) -> CallGraph<'a> {
        let mut names = vec![];
        let mut callee_names = vec![];
        for file in &program.files {
            for function in &file.functions {
                names.push(function.name.as_deref());
                callee_names.push(
                    file.function_nodes(function)
                        .filter(|n| n.kind == NodeKind::Call)
                        .filter_map(|n| n.label.as_deref())
                        .collect::<BTreeSet<_>>(),
                );
            }
        }
        let defined: BTreeSet<&str> = names.iter().flatten().copied().collect();
        let mut callers: HashMap<&str, BTreeSet<usize>> = HashMap::new();
        for (caller, callees) in callee_names.iter().enumerate() {
            for callee in callees.iter().filter(|c| defined.contains(*c)) {
                callers.entry(callee).or_default().insert(caller);
            }
        }
        CallGraph {
            names,
            callee_names,
            callers,
        }
    }

    fn fan_in(&self, f: usize) -> usize {
        self.names[f]
            .and_then(|n| self.callers.get(n))
            .map_or(0, BTreeSet::len)
    }

    /// Call depth of every function: the longest path over the condensation of the name graph.
    /// An anonymous function is not a node (nothing calls it): 1 + the deepest name it calls.
    fn call_depths(&self) -> Vec<usize> {
        let mut index: BTreeMap<&str, usize> = BTreeMap::new();
        for name in self.names.iter().flatten() {
            let next = index.len();
            index.entry(name).or_insert(next);
        }
        let callees_of = |f: usize| {
            self.callee_names[f]
                .iter()
                .filter_map(|c| index.get(c).copied())
        };
        let mut edges: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); index.len()];
        for (f, name) in self.names.iter().enumerate() {
            if let Some(name) = name {
                edges[index[name]].extend(callees_of(f));
            }
        }
        let edges: Vec<Vec<usize>> = edges.into_iter().map(|e| e.into_iter().collect()).collect();
        let depth = name_depths(&edges);
        (0..self.names.len())
            .map(|f| match self.names[f] {
                Some(name) => depth[index[name]],
                None => callees_of(f).map(|c| depth[c] + 1).max().unwrap_or(0),
            })
            .collect()
    }
}

/// Longest path from each node over the condensation of the graph (edges inside a strongly
/// connected component are not counted).
fn name_depths(edges: &[Vec<usize>]) -> Vec<usize> {
    let component = strongly_connected_components(edges);
    let count = component.iter().max().map_or(0, |c| c + 1);
    let mut members = vec![vec![]; count];
    for (v, c) in component.iter().enumerate() {
        members[*c].push(v);
    }
    // Tarjan numbers components in reverse topological order: callees come first.
    let mut depth = vec![0; count];
    for c in 0..count {
        for &v in &members[c] {
            for &w in &edges[v] {
                if component[w] != c {
                    depth[c] = depth[c].max(depth[component[w]] + 1);
                }
            }
        }
    }
    component.iter().map(|c| depth[*c]).collect()
}

/// Tarjan's algorithm, iterative (call chains can be long). Returns the component of each node;
/// components are numbered in reverse topological order (a component's successors have smaller numbers).
fn strongly_connected_components(edges: &[Vec<usize>]) -> Vec<usize> {
    const UNVISITED: usize = usize::MAX;
    let n = edges.len();
    let (mut index, mut low, mut component) = (vec![UNVISITED; n], vec![0; n], vec![UNVISITED; n]);
    let (mut on_stack, mut stack) = (vec![false; n], vec![]);
    let (mut next_index, mut next_component) = (0, 0);
    for start in 0..n {
        if index[start] != UNVISITED {
            continue;
        }
        let mut frames = vec![(start, 0)];
        index[start] = next_index;
        low[start] = next_index;
        next_index += 1;
        stack.push(start);
        on_stack[start] = true;
        while let Some(&(v, edge)) = frames.last() {
            if let Some(&w) = edges[v].get(edge) {
                frames.last_mut().expect("frames is not empty").1 += 1;
                if index[w] == UNVISITED {
                    index[w] = next_index;
                    low[w] = next_index;
                    next_index += 1;
                    stack.push(w);
                    on_stack[w] = true;
                    frames.push((w, 0));
                } else if on_stack[w] {
                    low[v] = low[v].min(index[w]);
                }
                continue;
            }
            frames.pop();
            if let Some(&(parent, _)) = frames.last() {
                low[parent] = low[parent].min(low[v]);
            }
            if low[v] == index[v] {
                while let Some(w) = stack.pop() {
                    on_stack[w] = false;
                    component[w] = next_component;
                    if w == v {
                        break;
                    }
                }
                next_component += 1;
            }
        }
    }
    component
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::NodeKind::*;
    use crate::ir::builder::*;
    use crate::ir::{File, NodeId, Program};
    use crate::metrics::{Calculator, MetricValue};

    fn v(x: f64) -> MetricValue {
        MetricValue::Available(x)
    }

    fn calls(b: &mut FileBuilder, body: NodeId, callees: &[&str]) {
        for callee in callees {
            b.labelled(body, Call, callee);
        }
    }

    /// file 0: f -> g, h(external), h ; g -> f, k ; k ; plus two imports
    /// file 1: m -> k ; anonymous lambda inside m -> f
    fn sample() -> Program {
        let mut b = FileBuilder::new("");
        let root = b.root();
        b.node(root, Import);
        b.node(root, Import);
        let f = b.function(root, "f", 0, lines(1, 1));
        calls(&mut b, f, &["g", "h", "h"]);
        let g = b.function(root, "g", 0, lines(2, 2));
        calls(&mut b, g, &["f", "k"]);
        b.function(root, "k", 0, lines(3, 3));
        let first: File = b.build();

        let mut b = FileBuilder::new("");
        let root = b.root();
        let m = b.function(root, "m", 0, lines(1, 3));
        calls(&mut b, m, &["k"]);
        let lambda = b.function(m, "", 0, lines(2, 2));
        calls(&mut b, lambda, &["f"]);
        let mut second = b.build();
        second.functions[1].name = None;
        Program {
            files: vec![first, second],
        }
    }

    fn function_metric(
        result: &ProgramMetrics,
        file: usize,
        function: usize,
        id: &str,
    ) -> MetricValue {
        result.files[file].functions[function][id].clone()
    }

    #[test]
    fn fan_out_counts_distinct_callee_names() {
        let r = DependencyCalculator.compute(&sample());
        assert_eq!(function_metric(&r, 0, 0, "dependency.fan_out"), v(2.0));
        assert_eq!(function_metric(&r, 0, 2, "dependency.fan_out"), v(0.0));
        // Calls in the nested lambda belong to the lambda.
        assert_eq!(function_metric(&r, 1, 0, "dependency.fan_out"), v(1.0));
    }

    #[test]
    fn fan_in_counts_distinct_calling_functions_across_the_project() {
        let r = DependencyCalculator.compute(&sample());
        assert_eq!(function_metric(&r, 0, 0, "dependency.fan_in"), v(2.0)); // g, lambda
        assert_eq!(function_metric(&r, 0, 2, "dependency.fan_in"), v(2.0)); // g, m
        assert_eq!(function_metric(&r, 1, 0, "dependency.fan_in"), v(0.0));
    }

    #[test]
    fn call_depth_collapses_recursion() {
        let r = DependencyCalculator.compute(&sample());
        // f <-> g form one component that calls k.
        assert_eq!(function_metric(&r, 0, 0, "dependency.call_depth"), v(1.0));
        assert_eq!(function_metric(&r, 0, 1, "dependency.call_depth"), v(1.0));
        assert_eq!(function_metric(&r, 0, 2, "dependency.call_depth"), v(0.0));
        // lambda -> f -> k
        assert_eq!(function_metric(&r, 1, 1, "dependency.call_depth"), v(2.0));
        // m -> k (m does not include its lambda's calls)
        assert_eq!(function_metric(&r, 1, 0, "dependency.call_depth"), v(1.0));
    }

    #[test]
    fn dependency_count_counts_imports() {
        let r = DependencyCalculator.compute(&sample());
        assert_eq!(r.files[0].metrics["dependency.dependency_count"], v(2.0));
        assert_eq!(r.files[1].metrics["dependency.dependency_count"], v(0.0));
        assert_eq!(r.project["dependency.dependency_count"], v(2.0));
    }

    #[test]
    fn deep_call_chains_do_not_recurse() {
        let n = 100_000;
        let mut b = FileBuilder::new("");
        let root = b.root();
        for i in 0..n {
            let body = b.function(root, &format!("f{i}"), 0, lines(1, 1));
            if i + 1 < n {
                b.labelled(body, Call, &format!("f{}", i + 1));
            }
        }
        let r = DependencyCalculator.compute(&Program {
            files: vec![b.build()],
        });
        assert_eq!(
            function_metric(&r, 0, 0, "dependency.call_depth"),
            v((n - 1) as f64)
        );
    }

    #[test]
    fn same_named_functions_share_one_call_graph_node() {
        // Two `run`s: one calls helper, one calls nothing. Resolved by name, both reach helper.
        let mut b = FileBuilder::new("");
        let root = b.root();
        let first = b.function(root, "run", 0, lines(1, 1));
        calls(&mut b, first, &["helper"]);
        b.function(root, "run", 0, lines(2, 2));
        b.function(root, "helper", 0, lines(3, 3));
        let r = DependencyCalculator.compute(&Program {
            files: vec![b.build()],
        });
        assert_eq!(function_metric(&r, 0, 0, "dependency.call_depth"), v(1.0));
        assert_eq!(function_metric(&r, 0, 1, "dependency.call_depth"), v(1.0));
        assert_eq!(function_metric(&r, 0, 1, "dependency.fan_out"), v(0.0));
    }

    #[test]
    fn many_calls_to_a_common_name_stay_linear() {
        // 5,000 functions named `get`, each called from 5,000 callers: 25M function pairs.
        let n = 5_000;
        let mut b = FileBuilder::new("");
        let root = b.root();
        for _ in 0..n {
            b.function(root, "get", 0, lines(1, 1));
        }
        for i in 0..n {
            let body = b.function(root, &format!("caller{i}"), 0, lines(1, 1));
            calls(&mut b, body, &["get"]);
        }
        let started = std::time::Instant::now();
        let r = DependencyCalculator.compute(&Program {
            files: vec![b.build()],
        });
        assert_eq!(function_metric(&r, 0, 0, "dependency.fan_in"), v(n as f64));
        assert_eq!(function_metric(&r, 0, n, "dependency.call_depth"), v(1.0));
        assert!(
            started.elapsed().as_secs() < 2,
            "took {:?}",
            started.elapsed()
        );
    }
}
