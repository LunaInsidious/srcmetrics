//! JavaScript, TypeScript and TSX mappings. The grammars share their node types; TypeScript
//! only adds type-level nodes, which are `other` for the metrics.

use super::Mapping;
use crate::ir::NodeKind::{self, *};

const KINDS: &[(&str, NodeKind)] = &[
    ("statement_block", Block),
    ("expression_statement", Statement),
    ("lexical_declaration", Declaration),
    ("variable_declaration", Declaration),
    ("if_statement", Branch),
    ("else_clause", Else),
    ("for_statement", Loop),
    ("for_in_statement", Loop),
    ("while_statement", Loop),
    ("do_statement", Loop),
    ("switch_case", Case),
    ("catch_clause", Catch),
    ("return_statement", Return),
    ("break_statement", Jump),
    ("continue_statement", Jump),
    ("throw_statement", Jump),
    ("call_expression", Call),
    ("new_expression", Call),
    ("assignment_expression", Assignment),
    ("augmented_assignment_expression", Assignment),
    ("binary_expression", Binary),
    ("unary_expression", Unary),
    ("update_expression", Unary),
    ("ternary_expression", Conditional),
    ("member_expression", Expression),
    ("subscript_expression", Expression),
    ("parenthesized_expression", Expression),
    ("identifier", Identifier),
    ("number", Literal),
    ("string", Literal),
    ("template_string", Literal),
    ("regex", Literal),
    ("true", Literal),
    ("false", Literal),
    ("null", Literal),
    ("undefined", Literal),
    ("import_statement", Import),
    ("function_declaration", Function),
    ("function_expression", Function),
    ("generator_function_declaration", Function),
    ("generator_function", Function),
    ("arrow_function", Function),
    ("method_definition", Function),
];

const LITERALS: &[&str] = &[
    "number",
    "string",
    "template_string",
    "regex",
    "true",
    "false",
    "null",
    "undefined",
];

const IDENTIFIERS: &[&str] = &[
    "identifier",
    "property_identifier",
    "private_property_identifier",
    "type_identifier",
    "shorthand_property_identifier",
    "shorthand_property_identifier_pattern",
    "statement_identifier",
];

pub static TYPESCRIPT: Mapping = Mapping {
    language: "typescript",
    extensions: &["ts", "mts", "cts"],
    grammar: || tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
    grammar_crate: ("tree-sitter-typescript", "0.23.2"),
    kinds: KINDS,
    callee_fields: &["function", "constructor"],
    logical_operators: &["&&", "||"],
    else_field: None,
    default_case_keyword: None,
    comments: &["comment"],
    literals: LITERALS,
    interpolations: &["template_substitution"],
    identifiers: IDENTIFIERS,
    name_fields: &["name", "pattern"],
    decorators: &["decorator"],
    docstring: None,
    parameter_fields: &["parameters", "parameter"],
    ignored_parameters: &[],
};

pub static TSX: Mapping = Mapping {
    language: "tsx",
    extensions: &["tsx"],
    grammar: || tree_sitter_typescript::LANGUAGE_TSX.into(),
    ..TYPESCRIPT
};

pub static JAVASCRIPT: Mapping = Mapping {
    language: "javascript",
    extensions: &["js", "mjs", "cjs", "jsx"],
    grammar: || tree_sitter_javascript::LANGUAGE.into(),
    grammar_crate: ("tree-sitter-javascript", "0.25.0"),
    ..TYPESCRIPT
};
