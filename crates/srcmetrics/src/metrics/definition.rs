//! Metric definitions (design principle P6).

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    Function,
    File,
    Project,
}

/// How far a metric's definition depends on the language (design principle P6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Applicability {
    LanguageIndependent,
    PartiallyLanguageDependent,
    LanguageSpecific,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MetricDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    /// What the metric means.
    pub definition: &'static str,
    pub scopes: &'static [Scope],
    /// IR elements the metric reads.
    pub input: &'static str,
    /// How the value is computed.
    pub calculation: &'static str,
    pub unit: &'static str,
    pub applicability: Applicability,
    pub limitations: &'static str,
    pub reference: &'static str,
}

/// Version of the metric definitions (design principle P10). Bump when any definition or calculation changes.
pub const DEFINITION_VERSION: &str = "0.1.0";

impl Scope {
    pub fn as_str(&self) -> &'static str {
        match self {
            Scope::Function => "function",
            Scope::File => "file",
            Scope::Project => "project",
        }
    }
}

impl Applicability {
    pub fn as_str(&self) -> &'static str {
        match self {
            Applicability::LanguageIndependent => "language_independent",
            Applicability::PartiallyLanguageDependent => "partially_language_dependent",
            Applicability::LanguageSpecific => "language_specific",
        }
    }
}
