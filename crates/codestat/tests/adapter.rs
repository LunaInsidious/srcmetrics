//! Language Adapter tests: source code -> Common IR, per language.

use codestat::error::AnalysisError;
use codestat::ir::{File, NodeKind, TokenKind};
use codestat::lang::adapter_for_path;

fn fixture(name: &str) -> (String, String) {
    let path = format!("{}/../../tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    let source = std::fs::read_to_string(&path).unwrap();
    (path, source)
}

fn parse(name: &str) -> File {
    let (path, source) = fixture(name);
    adapter_for_path(&path)
        .unwrap()
        .to_ir(&path, &source)
        .unwrap()
}

fn count(file: &File, function: usize, kind: NodeKind) -> usize {
    file.function_nodes(&file.functions[function])
        .filter(|n| n.kind == kind)
        .count()
}

fn names_and_arity(file: &File) -> Vec<(String, usize)> {
    file.functions
        .iter()
        .map(|f| (f.name.clone().unwrap(), f.parameters.len()))
        .collect()
}

const LANGS: [&str; 4] = ["c", "py", "ts", "js"];

#[test]
fn extracts_functions_with_names_and_parameters() {
    for ext in LANGS {
        let file = parse(&format!("equivalence/classify.{ext}"));
        assert_eq!(
            names_and_arity(&file),
            vec![("classify".to_string(), 2), ("max2".to_string(), 2)],
            "{ext}"
        );
    }
}

#[test]
fn parameter_names_are_extracted() {
    for ext in LANGS {
        let file = parse(&format!("equivalence/classify.{ext}"));
        let names: Vec<_> = file.functions[0]
            .parameters
            .iter()
            .map(|p| p.name.clone().unwrap())
            .collect();
        assert_eq!(names, vec!["values", "n"], "{ext}");
    }
}

#[test]
fn maps_control_structures_to_common_kinds() {
    for ext in LANGS {
        let file = parse(&format!("equivalence/classify.{ext}"));
        assert_eq!(count(&file, 0, NodeKind::Branch), 2, "{ext} branch");
        assert_eq!(count(&file, 0, NodeKind::Loop), 2, "{ext} loop");
        assert_eq!(count(&file, 0, NodeKind::Logical), 1, "{ext} logical");
        assert_eq!(
            count(&file, 0, NodeKind::Conditional),
            1,
            "{ext} conditional"
        );
        assert_eq!(count(&file, 0, NodeKind::Return), 1, "{ext} return");
        assert_eq!(count(&file, 0, NodeKind::Jump), 1, "{ext} jump");
        assert_eq!(count(&file, 1, NodeKind::Return), 2, "{ext} max2 return");
    }
}

#[test]
fn maps_imports() {
    for ext in LANGS {
        let file = parse(&format!("equivalence/classify.{ext}"));
        assert_eq!(
            file.top_level_nodes()
                .filter(|n| n.kind == NodeKind::Import)
                .count(),
            1,
            "{ext}"
        );
    }
}

#[test]
fn classifies_tokens() {
    let file = parse("equivalence/classify.c");
    let line: Vec<_> = file
        .tokens
        .iter()
        .filter(|t| t.range.start.line == 5)
        .map(|t| (t.kind, t.text.as_str()))
        .collect();
    assert_eq!(
        line,
        vec![
            (TokenKind::Keyword, "int"),
            (TokenKind::Identifier, "score"),
            (TokenKind::Operator, "="),
            (TokenKind::Literal, "0"),
            (TokenKind::Punctuation, ";"),
        ]
    );
}

#[test]
fn string_literals_and_comments_are_single_tokens() {
    let file = parse("equivalence/classify.py");
    let docstring: Vec<_> = file
        .tokens
        .iter()
        .filter(|t| t.range.start.line == 5)
        .collect();
    assert_eq!(docstring.len(), 1);
    assert_eq!(docstring[0].kind, TokenKind::Literal);
    assert_eq!(
        file.tokens
            .iter()
            .filter(|t| t.kind == TokenKind::Comment)
            .count(),
        1
    );
}

#[test]
fn preprocessor_directive_is_a_keyword() {
    let file = parse("equivalence/classify.c");
    assert_eq!(file.tokens[0].kind, TokenKind::Keyword);
    assert_eq!(file.tokens[0].text, "#include");
}

#[test]
fn source_ranges_are_one_based_lines() {
    let file = parse("equivalence/classify.c");
    let f = &file.functions[0];
    assert_eq!((f.range.first_line(), f.range.last_line()), (4, 19));
}

#[test]
fn syntax_error_is_reported_with_position() {
    let err = adapter_for_path("broken.c")
        .unwrap()
        .to_ir("broken.c", "int f( {\n")
        .unwrap_err();
    assert!(
        matches!(err, AnalysisError::Parse { line: 1, .. }),
        "{err:?}"
    );
}

#[test]
fn unknown_extension_is_an_error() {
    let err = adapter_for_path("notes.txt").err().unwrap();
    assert!(
        matches!(err, AnalysisError::UnsupportedLanguage { .. }),
        "{err:?}"
    );
}

fn parse_str(path: &str, source: &str) -> File {
    adapter_for_path(path).unwrap().to_ir(path, source).unwrap()
}

fn param_names(file: &File, function: usize) -> Vec<Option<String>> {
    file.functions[function]
        .parameters
        .iter()
        .map(|p| p.name.clone())
        .collect()
}

#[test]
fn c_void_parameter_list_has_no_parameters() {
    let file = parse_str("a.c", "int f(void) { return 0; }\n");
    assert!(file.functions[0].parameters.is_empty());
}

#[test]
fn c_pointer_returning_function_is_named() {
    let file = parse_str("a.c", "char *dup(const char *s) { return 0; }\n");
    assert_eq!(file.functions[0].name.as_deref(), Some("dup"));
    assert_eq!(param_names(&file, 0), vec![Some("s".to_string())]);
}

#[test]
fn c_default_label_is_not_a_case() {
    let file = parse_str(
        "a.c",
        "int f(int x) { switch (x) { case 1: return 1; case 2: return 2; default: return 0; } }\n",
    );
    assert_eq!(count(&file, 0, NodeKind::Case), 2);
}

#[test]
fn python_parameter_forms_are_named() {
    let file = parse_str(
        "a.py",
        "def f(a, b: int, c=1, d: int = 2, *args, e, **kw):\n    pass\n",
    );
    let expected = ["a", "b", "c", "d", "args", "e", "kw"].map(|s| Some(s.to_string()));
    assert_eq!(param_names(&file, 0), expected);
}

#[test]
fn python_keyword_only_separator_is_not_a_parameter() {
    let file = parse_str("a.py", "def f(a, *, b, /):\n    pass\n");
    assert_eq!(file.functions[0].parameters.len(), 2);
}

#[test]
fn nested_and_anonymous_functions_are_separate_functions() {
    let file = parse_str(
        "a.py",
        "def outer():\n    g = lambda x: x + 1\n    return g\n",
    );
    assert_eq!(file.functions.len(), 2);
    assert_eq!(file.functions[1].name, None);
    assert_eq!(param_names(&file, 1), vec![Some("x".to_string())]);
}

#[test]
fn typescript_arrow_function_with_single_parameter() {
    let file = parse_str("a.ts", "const inc = x => x + 1;\n");
    assert_eq!(param_names(&file, 0), vec![Some("x".to_string())]);
}

#[test]
fn typescript_methods_are_named_functions() {
    let file = parse_str(
        "a.ts",
        "class A {\n  run(a: number, b?: string): void {}\n}\n",
    );
    assert_eq!(names_and_arity(&file), vec![("run".to_string(), 2)]);
}

#[test]
fn tsx_is_supported() {
    let file = parse_str("a.tsx", "const App = () => <div>{1}</div>;\n");
    assert_eq!(file.language, "tsx");
    assert_eq!(file.functions.len(), 1);
}

#[test]
fn parser_versions_match_cargo_lock() {
    let lock = std::fs::read_to_string(format!("{}/../../Cargo.lock", env!("CARGO_MANIFEST_DIR")))
        .unwrap();
    let locked = |name: &str| {
        let entry = format!("name = \"{name}\"\nversion = \"");
        let start = lock
            .find(&entry)
            .unwrap_or_else(|| panic!("{name} not in Cargo.lock"))
            + entry.len();
        lock[start..].split('"').next().unwrap().to_string()
    };
    for adapter in codestat::lang::adapters() {
        let version = adapter.parser_version();
        let (runtime, grammar) = version.split_once(" / ").unwrap();
        assert_eq!(runtime, format!("tree-sitter {}", locked("tree-sitter")));
        let (name, v) = grammar.split_once(' ').unwrap();
        assert_eq!(v, locked(name), "{}", adapter.language());
    }
}

#[test]
fn empty_file_has_no_tokens() {
    for path in ["a.c", "a.py", "a.ts"] {
        let file = parse_str(path, "");
        assert!(file.tokens.is_empty(), "{path}: {:?}", file.tokens);
    }
}

fn kind_count(file: &File, kind: NodeKind) -> usize {
    file.nodes.iter().filter(|n| n.kind == kind).count()
}

#[test]
fn code_inside_template_literals_is_analyzed() {
    let file = parse_str("a.ts", "const s = `x${a ? f() : g(() => 1)}y`;\n");
    assert_eq!(kind_count(&file, NodeKind::Conditional), 1);
    assert_eq!(kind_count(&file, NodeKind::Call), 2);
    assert_eq!(file.functions.len(), 1);
    let literals: Vec<_> = file
        .tokens
        .iter()
        .filter(|t| t.kind == TokenKind::Literal)
        .map(|t| t.text.as_str())
        .collect();
    assert_eq!(literals, vec!["`x", "1", "y`"]);
}

#[test]
fn code_inside_python_f_strings_is_analyzed() {
    let file = parse_str("a.py", "s = f\"v={compute(x) if x else 0}!\"\n");
    assert_eq!(kind_count(&file, NodeKind::Conditional), 1);
    assert_eq!(kind_count(&file, NodeKind::Call), 1);
    assert!(
        file.tokens
            .iter()
            .any(|t| t.kind == TokenKind::Identifier && t.text == "compute")
    );
}

#[test]
fn tokens_are_in_source_order() {
    let file = parse_str("a.ts", "const s = `a${b}c${d}e`;\n");
    let offsets: Vec<_> = file.tokens.iter().map(|t| t.range.start.offset).collect();
    let mut sorted = offsets.clone();
    sorted.sort();
    assert_eq!(offsets, sorted);
}

#[test]
fn go_functions_and_parameters() {
    let file = parse("equivalence/classify.go");
    assert_eq!(
        names_and_arity(&file),
        vec![("classify".to_string(), 2), ("max2".to_string(), 2)]
    );
    let file = parse_str("a.go", "package p\nfunc f(a, b int, c ...string) {}\n");
    assert_eq!(
        param_names(&file, 0),
        ["a", "b", "c"].map(|s| Some(s.to_string()))
    );
}

#[test]
fn else_without_an_else_clause_node_is_an_else() {
    // Go has no else_clause node: the else block is the if statement's `alternative`.
    let file = parse_str(
        "a.go",
        "package p\nfunc f(x int) {\n\tif x > 0 {\n\t} else if x < 0 {\n\t} else {\n\t}\n}\n",
    );
    assert_eq!(count(&file, 0, NodeKind::Branch), 2);
    assert_eq!(count(&file, 0, NodeKind::Else), 1);
}

#[test]
fn go_switch_default_is_not_a_case_and_closures_are_functions() {
    let file = parse_str(
        "a.go",
        "package p\nfunc f(a int) {\n\tswitch a {\n\tcase 1:\n\tcase 2:\n\tdefault:\n\t}\n\tg := func(x int) int { return x }\n\t_ = g\n}\n",
    );
    assert_eq!(count(&file, 0, NodeKind::Case), 2);
    assert_eq!(file.functions.len(), 2);
}

#[test]
fn java_functions_parameters_and_cases() {
    let file = parse("equivalence/classify.java");
    assert_eq!(
        names_and_arity(&file),
        vec![("classify".to_string(), 2), ("max2".to_string(), 2)]
    );
    let file = parse_str(
        "A.java",
        "class A {\n  A(int x) {}\n  void f(int x, String... xs) {\n    switch (x) { case 1: break; case 2: break; default: }\n    switch (x) { case 1 -> g(); default -> h(); }\n    Runnable r = y -> {};\n  }\n}\n",
    );
    let names: Vec<_> = file.functions.iter().map(|f| f.name.clone()).collect();
    assert_eq!(names, vec![Some("A".into()), Some("f".into()), None]);
    assert_eq!(
        param_names(&file, 1),
        ["x", "xs"].map(|s| Some(s.to_string()))
    );
    assert_eq!(param_names(&file, 2), vec![Some("y".to_string())]);
    assert_eq!(count(&file, 1, NodeKind::Case), 3);
}

#[test]
fn jsx_is_supported() {
    let file = parse_str(
        "a.jsx",
        "const App = (props) => <div>{props.x ? 1 : 2}</div>;\n",
    );
    assert_eq!(file.language, "javascript");
    assert_eq!(kind_count(&file, NodeKind::Conditional), 1);
}

#[test]
fn rust_functions_closures_and_match_arms() {
    let file = parse("equivalence/classify.rs");
    assert_eq!(
        names_and_arity(&file),
        vec![("classify".to_string(), 2), ("max2".to_string(), 2)]
    );
    let file = parse_str(
        "a.rs",
        "impl A {\n    fn m(&self, x: i32) -> i32 {\n        let c = |y| y + 1;\n        match x { 1 => 2, _ => c(3) }\n    }\n}\n",
    );
    assert_eq!(file.functions.len(), 2);
    assert_eq!(param_names(&file, 0), vec![None, Some("x".to_string())]);
    assert_eq!(param_names(&file, 1), vec![Some("y".to_string())]);
    assert_eq!(count(&file, 0, NodeKind::Case), 2);
}
