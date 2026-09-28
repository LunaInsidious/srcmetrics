//! Metric Engine (PLAN.md §5.4, §8). Calculators read only the Common IR.

mod common;
mod definition;
mod size;

use crate::ir::{File, Function, Program};
use std::collections::BTreeMap;

pub use definition::{Applicability, MetricDefinition, Scope};

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
    fn merge(&mut self, other: ProgramMetrics) {
        self.project.extend(other.project);
        self.files
            .resize_with(other.files.len(), FileMetrics::default);
        for (mine, theirs) in self.files.iter_mut().zip(other.files) {
            mine.metrics.extend(theirs.metrics);
            mine.functions
                .resize_with(theirs.functions.len(), Metrics::default);
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
pub(crate) fn per_file(
    program: &Program,
    file_metrics: impl Fn(&File) -> Metrics,
    function_metrics: impl Fn(&File, &Function) -> Metrics,
) -> ProgramMetrics {
    let files = program
        .files
        .iter()
        .map(|file| FileMetrics {
            metrics: file_metrics(file),
            functions: file
                .functions
                .iter()
                .map(|f| function_metrics(file, f))
                .collect(),
        })
        .collect();
    ProgramMetrics {
        project: Metrics::new(),
        files,
    }
}

pub fn calculators() -> Vec<Box<dyn Calculator>> {
    vec![Box::new(size::SizeCalculator)]
}

pub fn definitions() -> Vec<&'static MetricDefinition> {
    calculators()
        .iter()
        .flat_map(|c| c.definitions().iter())
        .collect()
}

/// Runs every calculator over the program.
pub fn compute(program: &Program) -> ProgramMetrics {
    let mut result = ProgramMetrics::default();
    for calculator in calculators() {
        result.merge(calculator.compute(program));
    }
    result
}
