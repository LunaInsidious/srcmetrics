//! Metric definitions (PLAN.md §9).

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    Function,
    File,
    Project,
}

/// PLAN.md §6.1.
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

/// Version of the metric definitions (PLAN.md §17). Bump when any definition or calculation changes.
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

/// Renders the metric definition document (docs/METRICS.md).
pub fn to_markdown(definitions: &[&MetricDefinition]) -> String {
    let mut out = format!(
        "# メトリクス定義書\n\n\
         このファイルは `crates/codestat/src/metrics` の定義から生成される。直接編集しないこと。\n\
         再生成: `UPDATE_DOCS=1 cargo test -p codestat --test docs`\n\n\
         Metric Definition Version: `{DEFINITION_VERSION}`\n\n"
    );
    for d in definitions {
        let scopes: Vec<_> = d.scopes.iter().map(Scope::as_str).collect();
        // Table cells: escape pipes, show "-" for empty values.
        let cell = |s: &str| {
            if s.is_empty() {
                "-".to_string()
            } else {
                s.replace('|', "\\|")
            }
        };
        out += &format!(
            "## `{}` — {}\n\n{}\n\n| Item | Value |\n|---|---|\n\
             | Definition | {} |\n| Scope | {} |\n| Input | {} |\n| Calculation | {} |\n| Unit | {} |\n\
             | Language Applicability | {} |\n| Limitations | {} |\n| Reference | {} |\n\n",
            d.id,
            d.name,
            d.description,
            cell(d.definition),
            cell(&scopes.join(", ")),
            cell(d.input),
            cell(d.calculation),
            cell(d.unit),
            d.applicability.as_str(),
            cell(d.limitations),
            cell(d.reference),
        );
    }
    out
}
