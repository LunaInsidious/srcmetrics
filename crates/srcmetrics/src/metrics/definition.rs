//! The metric registry: ids and the scopes they are reported at (ADR-0026). What each metric
//! means is documented in docs/metrics/ (English) and docs/ja/metrics/ (Japanese), not in code.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Function,
    File,
    Project,
}

/// A metric id and the scopes it is reported at.
#[derive(Debug, Clone, PartialEq)]
pub struct MetricSpec {
    pub id: &'static str,
    pub scopes: &'static [Scope],
}

/// Version of the metric calculations (design principle P10), recorded in every result. Bump when
/// any calculation changes.
pub const DEFINITION_VERSION: &str = "0.1.0";
