//! Analysis result: the serialized output format (ADR-0010, design principle P10).

use crate::metrics::{MetricValue, Metrics};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub run: RunInfo,
    pub project: MetricsOutput,
    pub files: Vec<FileResult>,
}

/// Identifies what was analyzed and with which versions (design principle P10).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunInfo {
    pub project: String,
    /// `None` when the analyzed path is not in a git repository with an `origin` remote.
    pub repository: Option<String>,
    /// `None` when the analyzed path is not in a git repository.
    pub commit: Option<String>,
    pub tool_version: String,
    pub metric_definition_version: String,
    /// Language id -> parser version, for the languages present in the result.
    pub parsers: BTreeMap<String, String>,
    /// RFC 3339, UTC.
    pub timestamp: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum FileResult {
    Ok {
        path: String,
        language: String,
        #[serde(flatten)]
        metrics: MetricsOutput,
        functions: Vec<FunctionResult>,
    },
    Error {
        path: String,
        /// `None` when the language could not be determined.
        language: Option<String>,
        error: String,
    },
}

impl FileResult {
    pub fn path(&self) -> &str {
        match self {
            FileResult::Ok { path, .. } | FileResult::Error { path, .. } => path,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionResult {
    pub name: Option<String>,
    pub start_line: usize,
    pub end_line: usize,
    #[serde(flatten)]
    pub metrics: MetricsOutput,
}

/// Metric values as numbers or `null`, with the reason for every `null` (ADR-0010).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct MetricsOutput {
    pub metrics: BTreeMap<String, Option<f64>>,
    pub unavailable: BTreeMap<String, String>,
}

impl From<&Metrics> for MetricsOutput {
    fn from(metrics: &Metrics) -> Self {
        let mut out = MetricsOutput::default();
        for (id, value) in metrics {
            let number = match value {
                MetricValue::Available(v) if v.is_finite() => Some(*v),
                other => {
                    out.unavailable
                        .insert(id.to_string(), unavailable_reason(other));
                    None
                }
            };
            out.metrics.insert(id.to_string(), number);
        }
        out
    }
}

fn unavailable_reason(value: &MetricValue) -> String {
    match value {
        MetricValue::Available(v) => format!("error: value {v} is not finite"),
        MetricValue::NotApplicable => "not_applicable".into(),
        MetricValue::Unsupported => "unsupported".into(),
        MetricValue::Error(e) => format!("error: {e}"),
    }
}
