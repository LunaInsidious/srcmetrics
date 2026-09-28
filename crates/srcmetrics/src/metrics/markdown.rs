//! Markdown rendering of the metric definitions: one document for `srcmetrics metrics --format
//! markdown`, and the metric reference pages of the documentation site (`docs/metrics/` in English,
//! `docs/ja/metrics/` in Japanese; ADR-0025).

use super::definition::{Applicability, DEFINITION_VERSION, MetricDefinition, Scope};

/// Language of the rendered documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En,
    Ja,
}

/// The texts of a definition in one language.
struct Texts {
    name: &'static str,
    description: &'static str,
    definition: &'static str,
    input: &'static str,
    calculation: &'static str,
    limitations: &'static str,
}

fn texts(d: &MetricDefinition, lang: Lang) -> Texts {
    match lang {
        Lang::En => Texts {
            name: d.name,
            description: d.description,
            definition: d.definition,
            input: d.input,
            calculation: d.calculation,
            limitations: d.limitations,
        },
        Lang::Ja => Texts {
            name: d.ja.name,
            description: d.ja.description,
            definition: d.ja.definition,
            input: d.ja.input,
            calculation: d.ja.calculation,
            limitations: d.ja.limitations,
        },
    }
}

/// All definitions as one Markdown document (English).
pub fn to_markdown(definitions: &[&MetricDefinition]) -> String {
    let mut out =
        format!("# Metric definitions\n\nMetric definition version: `{DEFINITION_VERSION}`\n\n");
    for d in definitions {
        out += &section(d, Lang::En);
    }
    out
}

/// The metric reference pages as (file name, content): `index.md` with an overview table, and one
/// page per metric group (the id prefix, e.g. `complexity.md`).
pub fn reference_pages(definitions: &[&MetricDefinition], lang: Lang) -> Vec<(String, String)> {
    let mut pages = vec![("index.md".to_string(), overview(definitions, lang))];
    for group in groups(definitions) {
        let mut page = format!("{GENERATED}\n# {}\n\n", title(group, lang));
        for d in definitions.iter().filter(|d| group_of(d) == group) {
            page += &section(d, lang);
        }
        pages.push((format!("{group}.md"), page));
    }
    pages
}

const GENERATED: &str = "<!-- Generated from crates/srcmetrics/src/metrics. Do not edit; \
                         run `UPDATE_DOCS=1 cargo test -p srcmetrics --test docs`. -->\n";

fn overview(definitions: &[&MetricDefinition], lang: Lang) -> String {
    let (title, intro, header) = match lang {
        Lang::En => (
            "Metrics",
            "Every metric is reported separately under its id, at the scopes listed below. A value that \
             cannot be computed is `null`, with the reason in `unavailable`; it is never 0.\n\n\
             Metric definition version: `{version}` (recorded in every analysis result).",
            "| Metric | Name | Scopes | Unit |",
        ),
        Lang::Ja => (
            "メトリクス定義",
            "各メトリクスは、下の表のスコープで、ID ごとに個別に出力されます。計算できない値は 0 ではなく `null` \
             になり、理由が `unavailable` に入ります。\n\n\
             メトリクス定義のバージョン：`{version}`（すべての解析結果に記録されます）。",
            "| メトリクス | 名前 | スコープ | 単位 |",
        ),
    };
    let mut out = format!(
        "{GENERATED}\n# {title}\n\n{}\n\n{header}\n|---|---|---|---|\n",
        intro.replace("{version}", DEFINITION_VERSION)
    );
    for d in definitions {
        out += &format!(
            "| [`{}`](./{}#{}) | {} | {} | {} |\n",
            d.id,
            group_of(d),
            anchor(d.id),
            cell(texts(d, lang).name),
            cell(&scopes(d)),
            cell(d.unit)
        );
    }
    out
}

fn section(d: &MetricDefinition, lang: Lang) -> String {
    let t = texts(d, lang);
    let labels = match lang {
        Lang::En => [
            "Item",
            "Value",
            "Definition",
            "Scope",
            "Input",
            "Calculation",
            "Unit",
            "Language Applicability",
            "Limitations",
            "Reference",
        ],
        Lang::Ja => [
            "項目",
            "内容",
            "定義",
            "スコープ",
            "入力",
            "計算方法",
            "単位",
            "言語依存性",
            "制約",
            "参考文献",
        ],
    };
    let rows = [
        t.definition.to_string(),
        scopes(d),
        t.input.to_string(),
        t.calculation.to_string(),
        d.unit.to_string(),
        applicability(d.applicability, lang).to_string(),
        t.limitations.to_string(),
        d.reference.to_string(),
    ];
    let mut out = format!(
        "## {} {{#{}}}\n\n`{}` — {}\n\n| {} | {} |\n|---|---|\n",
        t.name,
        anchor(d.id),
        d.id,
        t.description,
        labels[0],
        labels[1]
    );
    for (label, value) in labels[2..].iter().zip(rows) {
        out += &format!("| {label} | {} |\n", cell(&value));
    }
    out + "\n"
}

fn applicability(a: Applicability, lang: Lang) -> &'static str {
    match (lang, a) {
        (Lang::En, _) => a.as_str(),
        (Lang::Ja, Applicability::LanguageIndependent) => "言語に依存しない",
        (Lang::Ja, Applicability::PartiallyLanguageDependent) => "部分的に言語に依存する",
        (Lang::Ja, Applicability::LanguageSpecific) => "特定の言語に固有",
    }
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

fn title(group: &str, lang: Lang) -> String {
    let japanese = match group {
        "size" => "規模",
        "complexity" => "複雑さ",
        "nesting" => "ネスト",
        "halstead" => "Halstead",
        "function" => "関数",
        "duplication" => "重複",
        "dependency" => "依存関係",
        "documentation" => "ドキュメント",
        "maintainability" => "保守性",
        "derived" => "派生メトリクス",
        other => panic!("metric group {other} has no Japanese title; add it here"),
    };
    match lang {
        Lang::Ja => japanese.to_string(),
        Lang::En => {
            let mut chars = group.chars();
            let first = chars.next().expect("group names are not empty");
            first.to_uppercase().chain(chars).collect()
        }
    }
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
