//! Derived metrics (ADR-0015, design principle P8): computed from standard metric values only.

use super::{MetricSpec, MetricValue, Metrics, Scope};

/// A derived metric: its id and scopes, and how to compute it from the standard metrics of a scope.
/// Formulas are documented in docs/metrics/ (ADR-0026).
struct Derivation {
    spec: MetricSpec,
    compute: fn(&Metrics) -> MetricValue,
}

static DERIVATIONS: &[Derivation] = &[
    Derivation {
        spec: MetricSpec {
            id: "maintainability.index",
            scopes: &[Scope::Function, Scope::File],
        },
        compute: maintainability_index,
    },
    Derivation {
        spec: MetricSpec {
            id: "derived.cyclomatic_per_function",
            scopes: &[Scope::File, Scope::Project],
        },
        compute: |m| divide(m, "complexity.cyclomatic", "size.function_count"),
    },
    Derivation {
        spec: MetricSpec {
            id: "derived.tokens_per_loc",
            scopes: &[Scope::File, Scope::Project],
        },
        compute: |m| divide(m, "size.token_count", "size.loc"),
    },
    Derivation {
        spec: MetricSpec {
            id: "derived.statements_per_function",
            scopes: &[Scope::File, Scope::Project],
        },
        compute: |m| divide(m, "size.statement_count", "size.function_count"),
    },
    Derivation {
        spec: MetricSpec {
            id: "derived.duplicate_tokens_per_sloc",
            scopes: &[Scope::File, Scope::Project],
        },
        compute: |m| divide(m, "duplication.duplicate_token_count", "size.sloc"),
    },
];

pub(super) fn specs() -> impl Iterator<Item = &'static MetricSpec> {
    DERIVATIONS.iter().map(|d| &d.spec)
}

/// Adds the derived metrics defined for `scope` to `metrics`.
pub(super) fn derive(scope: Scope, metrics: &mut Metrics) {
    for d in DERIVATIONS
        .iter()
        .filter(|d| d.spec.scopes.contains(&scope))
    {
        let value = (d.compute)(metrics);
        metrics.insert(d.spec.id, value);
    }
}

/// An input value, or the value the derived metric takes when the input is unavailable.
fn input(metrics: &Metrics, id: &str) -> Result<f64, MetricValue> {
    match metrics
        .get(id)
        .unwrap_or_else(|| panic!("derived metric input {id} is not computed at this scope"))
    {
        MetricValue::Available(v) => Ok(*v),
        MetricValue::Error(e) => Err(MetricValue::Error(format!("{id}: {e}"))),
        _ => Err(MetricValue::NotApplicable),
    }
}

fn divide(metrics: &Metrics, numerator: &str, denominator: &str) -> MetricValue {
    match [input(metrics, numerator), input(metrics, denominator)] {
        [Ok(n), Ok(d)] => MetricValue::ratio(n, d),
        inputs => unavailable(inputs),
    }
}

fn maintainability_index(metrics: &Metrics) -> MetricValue {
    let inputs = [
        input(metrics, "halstead.volume"),
        input(metrics, "complexity.cyclomatic"),
        input(metrics, "size.sloc"),
    ];
    match inputs {
        [Ok(v), Ok(cc), Ok(sloc)] if v > 0.0 && sloc > 0.0 => {
            MetricValue::Available(171.0 - 5.2 * v.ln() - 0.23 * cc - 16.2 * sloc.ln())
        }
        [Ok(_), Ok(_), Ok(_)] => MetricValue::NotApplicable,
        inputs => unavailable(inputs),
    }
}

/// The value of a derived metric with an unavailable input: an input error if any, else not applicable.
fn unavailable<const N: usize>(inputs: [Result<f64, MetricValue>; N]) -> MetricValue {
    inputs
        .into_iter()
        .filter_map(Result::err)
        .min_by_key(|e| !matches!(e, MetricValue::Error(_)))
        .expect("called only when some input is unavailable")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::MetricValue::*;

    fn metrics(values: &[(&'static str, MetricValue)]) -> Metrics {
        values.iter().cloned().collect()
    }

    #[test]
    fn maintainability_index_uses_volume_cyclomatic_and_sloc() {
        let mut m = metrics(&[
            ("halstead.volume", Available(1000.0)),
            ("complexity.cyclomatic", Available(5.0)),
            ("size.sloc", Available(40.0)),
        ]);
        derive(Scope::Function, &mut m);
        let expected = 171.0 - 5.2 * 1000f64.ln() - 0.23 * 5.0 - 16.2 * 40f64.ln();
        assert_eq!(m["maintainability.index"], Available(expected));
    }

    #[test]
    fn maintainability_index_needs_positive_volume_and_sloc() {
        let mut m = metrics(&[
            ("halstead.volume", Available(0.0)),
            ("complexity.cyclomatic", Available(1.0)),
            ("size.sloc", Available(3.0)),
        ]);
        derive(Scope::Function, &mut m);
        assert_eq!(m["maintainability.index"], NotApplicable);
    }

    #[test]
    fn unavailable_inputs_propagate() {
        let mut m = metrics(&[
            ("halstead.volume", NotApplicable),
            ("complexity.cyclomatic", Available(1.0)),
            ("size.sloc", Error("bad line".into())),
        ]);
        derive(Scope::Function, &mut m);
        assert_eq!(
            m["maintainability.index"],
            Error("size.sloc: bad line".into())
        );
    }

    #[test]
    fn normalized_values_at_file_scope() {
        let mut m = metrics(&[
            ("halstead.volume", Available(100.0)),
            ("complexity.cyclomatic", Available(12.0)),
            ("size.sloc", Available(50.0)),
            ("size.loc", Available(80.0)),
            ("size.function_count", Available(4.0)),
            ("size.token_count", Available(400.0)),
            ("size.statement_count", Available(20.0)),
            ("duplication.duplicate_token_count", Available(25.0)),
        ]);
        derive(Scope::File, &mut m);
        assert_eq!(m["derived.cyclomatic_per_function"], Available(3.0));
        assert_eq!(m["derived.tokens_per_loc"], Available(5.0));
        assert_eq!(m["derived.statements_per_function"], Available(5.0));
        assert_eq!(m["derived.duplicate_tokens_per_sloc"], Available(0.5));
        assert!(m.contains_key("maintainability.index"));
    }

    #[test]
    fn zero_denominators_are_not_applicable() {
        let mut m = metrics(&[
            ("complexity.cyclomatic", Available(0.0)),
            ("size.function_count", Available(0.0)),
            ("size.loc", Available(0.0)),
            ("size.sloc", Available(0.0)),
            ("size.token_count", Available(0.0)),
            ("size.statement_count", Available(0.0)),
            ("duplication.duplicate_token_count", Available(0.0)),
        ]);
        derive(Scope::Project, &mut m);
        assert_eq!(m["derived.cyclomatic_per_function"], NotApplicable);
        assert_eq!(m["derived.tokens_per_loc"], NotApplicable);
        assert!(!m.contains_key("maintainability.index"));
    }
}
