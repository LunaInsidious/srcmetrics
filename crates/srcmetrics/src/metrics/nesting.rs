//! Nesting Metrics: maximum and average nesting depth.

use super::common::{is_continuation, is_nesting, is_statement, nesting_levels};
use super::{
    Applicability::*, Calculator, Ja, MetricDefinition, MetricValue, Metrics, ProgramMetrics,
    Scope::*, per_file,
};
use crate::ir::{File, Node, Program};

pub struct NestingCalculator;

static DEFINITIONS: &[MetricDefinition] = &[
    MetricDefinition {
        id: "nesting.max_depth",
        name: "Maximum Nesting Depth",
        description: "Deepest nesting of control structures.",
        definition: "Maximum over control structures (branch, loop, case, catch) of 1 + the number of \
                     control structures enclosing it.",
        scopes: &[Function, File, Project],
        input: "Node kinds and parent links",
        calculation: "`else if` / `elif` continue their if-chain and do not add a level. Nesting restarts \
                      at function boundaries. 0 when there is no control structure. File / Project: maximum.",
        unit: "levels",
        applicability: LanguageIndependent,
        limitations: "A branch directly inside a branch without a block (C `if (a) if (b) x;`) is treated \
                      as an if-chain continuation.",
        reference: "",
        ja: Ja {
            name: "最大ネスト深さ",
            description: "制御構造の最も深い入れ子。",
            definition: "制御構造（branch, loop, case, catch）ごとの「1 + それを囲む制御構造の数」の最大値。",
            input: "ノードの種類と親子関係",
            calculation: "`else if` / `elif` は if の連鎖の続きで、レベルを増やさない。関数の境界でネストはリセットする。制御構造がなければ 0。ファイル / プロジェクト：最大値。",
            limitations: "ブロックを挟まずに分岐の中に直接ある分岐（C の `if (a) if (b) x;`）は、if の連鎖の続きとして扱う。",
        },
    },
    MetricDefinition {
        id: "nesting.avg_depth",
        name: "Average Nesting Depth",
        description: "Mean nesting level of statements.",
        definition: "Mean over statements of the number of control structures enclosing the statement.",
        scopes: &[Function, File, Project],
        input: "Node kinds and parent links",
        calculation: "Statements as in size.statement_count. not_applicable when there are no statements. \
                      File / Project: mean over all their statements.",
        unit: "levels",
        applicability: LanguageIndependent,
        limitations: "Inherits the statement differences of size.statement_count.",
        reference: "",
        ja: Ja {
            name: "平均ネスト深さ",
            description: "文のネストのレベルの平均。",
            definition: "文ごとの「それを囲む制御構造の数」の平均。",
            input: "ノードの種類と親子関係",
            calculation: "文の定義は size.statement_count と同じ。文がなければ not_applicable。ファイル / プロジェクト：含まれるすべての文の平均。",
            limitations: "size.statement_count と同じく、文の数え方の言語差を受け継ぐ。",
        },
    },
];

impl Calculator for NestingCalculator {
    fn definitions(&self) -> &'static [MetricDefinition] {
        DEFINITIONS
    }

    fn compute(&self, program: &Program) -> ProgramMetrics {
        let mut result = per_file(
            program,
            levels,
            |file, levels| Summary::of(file, levels, file.nodes.iter()).metrics(),
            |file, levels, function| {
                Summary::of(file, levels, file.function_nodes(function)).metrics()
            },
        );
        let project = program
            .files
            .iter()
            .map(|f| Summary::of(f, &levels(f), f.nodes.iter()))
            .fold(Summary::default(), Summary::add);
        result.project = project.metrics();
        result
    }
}

fn levels(file: &File) -> Vec<usize> {
    nesting_levels(file, |n| is_nesting(n.kind))
}

#[derive(Default)]
struct Summary {
    max_depth: usize,
    level_sum: usize,
    statements: usize,
}

impl Summary {
    fn of<'a>(file: &'a File, levels: &[usize], nodes: impl Iterator<Item = &'a Node>) -> Summary {
        let mut s = Summary::default();
        for node in nodes {
            let level = levels[node.id.0];
            if is_nesting(node.kind) && !is_continuation(file, node) {
                s.max_depth = s.max_depth.max(level + 1);
            }
            if is_statement(node.kind) {
                s.level_sum += level;
                s.statements += 1;
            }
        }
        s
    }

    fn add(self, other: Summary) -> Summary {
        Summary {
            max_depth: self.max_depth.max(other.max_depth),
            level_sum: self.level_sum + other.level_sum,
            statements: self.statements + other.statements,
        }
    }

    fn metrics(&self) -> Metrics {
        Metrics::from([
            ("nesting.max_depth", self.max_depth.into()),
            (
                "nesting.avg_depth",
                MetricValue::ratio(self.level_sum as f64, self.statements as f64),
            ),
        ])
    }
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

    /// ```text
    /// f:
    ///   s0                      level 0
    ///   for:                    level 0, depth 1
    ///     if:                   level 1, depth 2
    ///       s1                  level 2
    ///     else if:              continuation: level 1
    ///       while:              level 2, depth 3
    ///         s2                level 3
    ///     else:
    ///       s3                  level 2
    /// ```
    fn sample() -> Program {
        let mut b = FileBuilder::new("");
        let root = b.root();
        let f = b.function(root, "f", 0, lines(1, 9));
        b.node(f, Statement);
        let lp = b.node(f, Loop);
        let body = b.node(lp, Block);
        let head = b.node(body, Branch);
        b.node(head, Statement);
        let els = b.node(head, Else);
        let elif = b.node(els, Branch);
        let inner = b.node(elif, Loop);
        b.node(inner, Statement);
        let last_else = b.node(elif, Else);
        b.node(last_else, Statement);
        Program {
            files: vec![b.build()],
        }
    }

    #[test]
    fn max_depth_counts_enclosing_control_structures() {
        let m = &NestingCalculator.compute(&sample()).files[0].functions[0];
        assert_eq!(m["nesting.max_depth"], v(3.0));
    }

    #[test]
    fn avg_depth_is_mean_statement_level() {
        // statements: s0=0, for=0, if=1, s1=2, else-if=1, while=2, s2=3, s3=2
        let m = &NestingCalculator.compute(&sample()).files[0].functions[0];
        assert_eq!(m["nesting.avg_depth"], v(11.0 / 8.0));
    }

    #[test]
    fn python_style_elif_is_a_continuation() {
        // if: (elif as direct child branch: s)
        let mut b = FileBuilder::new("");
        let root = b.root();
        let f = b.function(root, "f", 0, lines(1, 4));
        let head = b.node(f, Branch);
        let elif = b.node(head, Branch);
        b.node(elif, Statement);
        let m = &NestingCalculator
            .compute(&Program {
                files: vec![b.build()],
            })
            .files[0]
            .functions[0];
        assert_eq!(m["nesting.max_depth"], v(1.0));
    }

    #[test]
    fn nesting_restarts_inside_nested_functions() {
        let mut b = FileBuilder::new("");
        let root = b.root();
        let outer = b.function(root, "outer", 0, lines(1, 5));
        let lp = b.node(outer, Loop);
        let inner = b.function(lp, "inner", 0, lines(2, 4));
        b.node(inner, Branch);
        let result = NestingCalculator.compute(&Program {
            files: vec![b.build()],
        });
        assert_eq!(result.files[0].functions[0]["nesting.max_depth"], v(1.0));
        assert_eq!(result.files[0].functions[1]["nesting.max_depth"], v(1.0));
    }

    #[test]
    fn function_without_statements_has_no_average() {
        let mut b = FileBuilder::new("");
        let root = b.root();
        b.function(root, "f", 0, lines(1, 1));
        let m = &NestingCalculator
            .compute(&Program {
                files: vec![b.build()],
            })
            .files[0]
            .functions[0];
        assert_eq!(m["nesting.max_depth"], v(0.0));
        assert_eq!(m["nesting.avg_depth"], MetricValue::NotApplicable);
    }

    #[test]
    fn file_and_project_aggregate_all_statements() {
        let mut program = sample();
        program.files.push(program.files[0].clone());
        let result = NestingCalculator.compute(&program);
        assert_eq!(result.files[0].metrics["nesting.max_depth"], v(3.0));
        assert_eq!(result.project["nesting.max_depth"], v(3.0));
        assert_eq!(result.project["nesting.avg_depth"], v(11.0 / 8.0));
    }
}
