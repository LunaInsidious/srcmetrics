//! Markdown rendering of the metric definitions: one document for `srcmetrics metrics --format
//! markdown`, and the metric reference pages of the documentation site (docs/metrics/).

use super::definition::{DEFINITION_VERSION, MetricDefinition, Scope};

/// All definitions as one Markdown document.
pub fn to_markdown(definitions: &[&MetricDefinition]) -> String {
    let mut out =
        format!("# Metric definitions\n\nMetric definition version: `{DEFINITION_VERSION}`\n\n");
    for d in definitions {
        out += &section(d);
    }
    out
}

/// The metric reference pages as (file name, content): `index.md` with an overview table, and one
/// page per metric group (the id prefix, e.g. `complexity.md`).
pub fn reference_pages(definitions: &[&MetricDefinition]) -> Vec<(String, String)> {
    let mut pages = vec![("index.md".to_string(), overview(definitions))];
    for group in groups(definitions) {
        let mut page = format!("{GENERATED}\n# {}\n\n", title(group));
        for d in definitions.iter().filter(|d| group_of(d) == group) {
            page += &section(d);
        }
        pages.push((format!("{group}.md"), page));
    }
    pages
}

const GENERATED: &str = "<!-- Generated from crates/srcmetrics/src/metrics. Do not edit; \
                         run `UPDATE_DOCS=1 cargo test -p srcmetrics --test docs`. -->\n";

fn overview(definitions: &[&MetricDefinition]) -> String {
    let mut out = format!(
        "{GENERATED}\n# Metrics\n\n\
         Every metric is reported separately under its id, at the scopes listed below. A value that \
         cannot be computed is `null`, with the reason in `unavailable`; it is never 0.\n\n\
         Metric definition version: `{DEFINITION_VERSION}` (recorded in every analysis result).\n\n\
         | Metric | Name | Scopes | Unit |\n|---|---|---|---|\n"
    );
    for d in definitions {
        out += &format!(
            "| [`{}`](./{}#{}) | {} | {} | {} |\n",
            d.id,
            group_of(d),
            anchor(d.id),
            cell(d.name),
            cell(&scopes(d)),
            cell(d.unit)
        );
    }
    out
}

fn section(d: &MetricDefinition) -> String {
    format!(
        "## {} {{#{}}}\n\n`{}` — {}\n\n| Item | Value |\n|---|---|\n\
         | Definition | {} |\n| Scope | {} |\n| Input | {} |\n| Calculation | {} |\n| Unit | {} |\n\
         | Language Applicability | {} |\n| Limitations | {} |\n| Reference | {} |\n\n",
        d.name,
        anchor(d.id),
        d.id,
        d.description,
        cell(d.definition),
        cell(&scopes(d)),
        cell(d.input),
        cell(d.calculation),
        cell(d.unit),
        d.applicability.as_str(),
        cell(d.limitations),
        cell(d.reference),
    )
}

fn group_of(d: &MetricDefinition) -> &'static str {
    d.id.split('.')
        .next()
        .expect("metric ids have a group prefix")
}

/// Groups in first-appearance order.
fn groups(definitions: &[&MetricDefinition]) -> Vec<&'static str> {
    let mut groups: Vec<&str> = vec![];
    for d in definitions {
        if !groups.contains(&group_of(d)) {
            groups.push(group_of(d));
        }
    }
    groups
}

fn title(group: &str) -> String {
    let mut chars = group.chars();
    let first = chars.next().expect("group names are not empty");
    first.to_uppercase().chain(chars).collect()
}

fn anchor(id: &str) -> String {
    id.replace(['.', '_'], "-")
}

fn scopes(d: &MetricDefinition) -> String {
    d.scopes
        .iter()
        .map(Scope::as_str)
        .collect::<Vec<_>>()
        .join(", ")
}

/// A table cell: pipes escaped, "-" for empty values.
fn cell(s: &str) -> String {
    if s.is_empty() {
        "-".to_string()
    } else {
        s.replace('|', "\\|")
    }
}
