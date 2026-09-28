//! Metric Engine (PLAN.md §5.4, §8). Calculators read only the Common IR.

mod cognitive;
mod common;
mod complexity;
mod definition;
mod dependency;
mod derived;
mod documentation;
mod duplication;
mod function;
mod halstead;
mod nesting;
mod size;

use crate::ir::{File, Function, Program};
use std::collections::BTreeMap;

pub use definition::{Applicability, DEFINITION_VERSION, MetricDefinition, Scope, to_markdown};

/// A metric value, or the reason it has none (ADR-0005). Never conflates "0" with "not computed".
#[derive(Debug, Clone, PartialEq)]
pub enum MetricValue {
    Available(f64),
    NotApplicable,
    Unsupported,
    Error(String),
}

impl MetricValue {
    /// `NotApplicable` when the denominator is zero.
    pub fn ratio(numerator: f64, denominator: f64) -> MetricValue {
        if denominator == 0.0 {
            MetricValue::NotApplicable
        } else {
            MetricValue::Available(numerator / denominator)
        }
    }
}

impl MetricValue {
    /// Mean of `values`; `NotApplicable` when empty.
    pub fn mean(values: &[usize]) -> MetricValue {
        MetricValue::ratio(values.iter().sum::<usize>() as f64, values.len() as f64)
    }

    /// Maximum of `values`; `NotApplicable` when empty.
    pub fn max(values: &[usize]) -> MetricValue {
        values
            .iter()
            .max()
            .map_or(MetricValue::NotApplicable, |&x| x.into())
    }
}

impl From<usize> for MetricValue {
    fn from(v: usize) -> Self {
        MetricValue::Available(v as f64)
    }
}

impl From<f64> for MetricValue {
    fn from(v: f64) -> Self {
        MetricValue::Available(v)
    }
}

/// Metric id -> value. Ordered for reproducible output (PLAN.md §13.1).
pub type Metrics = BTreeMap<&'static str, MetricValue>;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FileMetrics {
    pub metrics: Metrics,
    /// Parallel to `File::functions`.
    pub functions: Vec<Metrics>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ProgramMetrics {
    pub project: Metrics,
    /// Parallel to `Program::files`.
    pub files: Vec<FileMetrics>,
}

impl ProgramMetrics {
    /// Adds `other`'s values. Both must describe the same program (same file and function counts).
    fn merge(&mut self, other: ProgramMetrics) {
        self.project.extend(other.project);
        if self.files.is_empty() {
            self.files = other.files;
            return;
        }
        assert_eq!(
            self.files.len(),
            other.files.len(),
            "calculators must report every file"
        );
        for (mine, theirs) in self.files.iter_mut().zip(other.files) {
            assert_eq!(
                mine.functions.len(),
                theirs.functions.len(),
                "calculators must report every function"
            );
            mine.metrics.extend(theirs.metrics);
            for (f, g) in mine.functions.iter_mut().zip(theirs.functions) {
                f.extend(g);
            }
        }
    }
}

/// An independent metric calculator (PLAN.md §12.2).
pub trait Calculator {
    fn definitions(&self) -> &'static [MetricDefinition];
    fn compute(&self, program: &Program) -> ProgramMetrics;
}

/// Builds `ProgramMetrics` for calculators whose file and function values depend only on that file.
/// `prepare` computes per-file data shared by the file and all its functions (computed once).
pub(crate) fn per_file<T>(
    program: &Program,
    prepare: impl Fn(&File) -> T,
    file_metrics: impl Fn(&File, &T) -> Metrics,
    function_metrics: impl Fn(&File, &T, &Function) -> Metrics,
) -> ProgramMetrics {
    let files = program
        .files
        .iter()
        .map(|file| {
            let prepared = prepare(file);
            FileMetrics {
                metrics: file_metrics(file, &prepared),
                functions: file
                    .functions
                    .iter()
                    .map(|f| function_metrics(file, &prepared, f))
                    .collect(),
            }
        })
        .collect();
    ProgramMetrics {
        project: Metrics::new(),
        files,
    }
}

pub fn calculators() -> Vec<Box<dyn Calculator>> {
    vec![
        Box::new(size::SizeCalculator),
        Box::new(complexity::ComplexityCalculator),
        Box::new(cognitive::CognitiveCalculator),
        Box::new(nesting::NestingCalculator),
        Box::new(halstead::HalsteadCalculator),
        Box::new(function::FunctionCalculator),
        Box::new(duplication::DuplicationCalculator),
        Box::new(dependency::DependencyCalculator),
        Box::new(documentation::DocumentationCalculator),
    ]
}

/// Definitions of all standard metrics followed by the derived ones.
pub fn definitions() -> Vec<&'static MetricDefinition> {
    let standard: Vec<_> = calculators()
        .iter()
        .flat_map(|c| c.definitions().iter())
        .collect();
    standard.into_iter().chain(derived::definitions()).collect()
}

/// Runs every calculator over the program, then adds the derived metrics to every scope (ADR-0015).
pub fn compute(program: &Program) -> ProgramMetrics {
    let mut result = ProgramMetrics::default();
    for calculator in calculators() {
        result.merge(calculator.compute(program));
    }
    derived::derive(Scope::Project, &mut result.project);
    for file in &mut result.files {
        derived::derive(Scope::File, &mut file.metrics);
        for function in &mut file.functions {
            derived::derive(Scope::Function, function);
        }
    }
    result
}
