//! Metric definitions (PLAN.md §9).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Function,
    File,
    Project,
}

/// PLAN.md §6.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Applicability {
    LanguageIndependent,
    PartiallyLanguageDependent,
    LanguageSpecific,
}

#[derive(Debug, Clone, PartialEq)]
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
