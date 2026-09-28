//! Documentation Metrics (ADR-0013).
//!
//! Comment LOC and Comment Ratio are provided by the Size calculator (`size.comment_loc`,
//! `size.comment_ratio`).

use super::{Calculator, MetricSpec, MetricValue, Metrics, ProgramMetrics, Scope::*, per_file};
use crate::ir::{File, Program};

pub struct DocumentationCalculator;

static SPECS: &[MetricSpec] = &[
    MetricSpec {
        id: "documentation.doc_loc",
        scopes: &[Function],
    },
    MetricSpec {
        id: "documentation.documented_function_count",
        scopes: &[File, Project],
    },
    MetricSpec {
        id: "documentation.documentation_ratio",
        scopes: &[File, Project],
    },
];

impl Calculator for DocumentationCalculator {
    fn specs(&self) -> &'static [MetricSpec] {
        SPECS
    }

    fn compute(&self, program: &Program) -> ProgramMetrics {
        let mut result = per_file(
            program,
            |_| (),
            |file, _| totals(&[file]),
            |_, _, function| {
                let lines = function.doc.map_or(0, |d| d.line_count());
                Metrics::from([("documentation.doc_loc", lines.into())])
            },
        );
        result.project = totals(&program.files.iter().collect::<Vec<_>>());
        result
    }
}

fn totals(files: &[&File]) -> Metrics {
    let functions = files.iter().flat_map(|f| &f.functions);
    let total = functions.clone().count();
    let documented = functions.filter(|f| f.doc.is_some()).count();
    Metrics::from([
        ("documentation.documented_function_count", documented.into()),
        (
            "documentation.documentation_ratio",
            MetricValue::ratio(documented as f64, total as f64),
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::builder::*;
    use crate::metrics::{Calculator, MetricValue};

    fn v(x: f64) -> MetricValue {
        MetricValue::Available(x)
    }

    fn sample() -> Program {
        let mut b = FileBuilder::new("");
        let root = b.root();
        b.function(root, "f", 0, lines(3, 5));
        b.function(root, "g", 0, lines(6, 7));
        let mut file = b.build();
        file.functions[0].doc = Some(lines(1, 2));
        Program { files: vec![file] }
    }

    #[test]
    fn function_doc_loc() {
        let r = DocumentationCalculator.compute(&sample());
        assert_eq!(r.files[0].functions[0]["documentation.doc_loc"], v(2.0));
        assert_eq!(r.files[0].functions[1]["documentation.doc_loc"], v(0.0));
    }

    #[test]
    fn file_and_project_counts_and_ratio() {
        let mut program = sample();
        program.files.push(FileBuilder::new("").build());
        let r = DocumentationCalculator.compute(&program);
        assert_eq!(
            r.files[0].metrics["documentation.documented_function_count"],
            v(1.0)
        );
        assert_eq!(
            r.files[0].metrics["documentation.documentation_ratio"],
            v(0.5)
        );
        assert_eq!(
            r.files[1].metrics["documentation.documentation_ratio"],
            MetricValue::NotApplicable
        );
        assert_eq!(r.project["documentation.documentation_ratio"], v(0.5));
    }
}
