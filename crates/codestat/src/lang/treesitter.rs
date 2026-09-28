//! Generic tree-sitter -> Common IR converter driven by per-language `Mapping` tables (ADR-0004).

use super::LanguageAdapter;
use crate::error::AnalysisError;
use crate::ir::{
    File, Function, FunctionId, Node, NodeId, NodeKind, Parameter, Position, SourceRange, Token,
    TokenKind,
};
use std::collections::HashMap;
use tree_sitter::{Parser, Point};

/// Declarative description of one language. Everything language-specific lives here.
pub struct Mapping {
    pub language: &'static str,
    pub extensions: &'static [&'static str],
    pub grammar: fn() -> tree_sitter::Language,
    /// Grammar crate name and version, reported as the parser version (PLAN.md §17).
    /// Checked against Cargo.lock by tests.
    pub grammar_crate: (&'static str, &'static str),
    /// tree-sitter node type -> IR node kind. Unlisted named nodes become `Other`.
    pub kinds: &'static [(&'static str, NodeKind)],
    /// Fields of a call node holding the callee, tried in order (ADR-0012).
    pub callee_fields: &'static [&'static str],
    /// Operators that turn a `Binary` node into a `Logical` (short-circuit) node.
    pub logical_operators: &'static [&'static str],
    /// Keyword that marks a `Case` node as the default label (not a decision point).
    pub default_case_keyword: Option<&'static str>,
    /// For grammars without an else-clause node (Go, Java): the field of a `Branch` node holding
    /// its else part. A child in this field that is not itself a `Branch` becomes an `Else`.
    pub else_field: Option<&'static str>,
    /// Node types that become a single comment token.
    pub comments: &'static [&'static str],
    /// Node types that become literal tokens; their subtree is not walked, except for interpolations.
    pub literals: &'static [&'static str],
    /// Node types of code embedded in a literal (e.g. `${...}`, f-string `{...}`), walked as code.
    pub interpolations: &'static [&'static str],
    /// Leaf node types that become identifier tokens.
    pub identifiers: &'static [&'static str],
    /// Field names followed, in order, to find the identifier naming a function or parameter.
    pub name_fields: &'static [&'static str],
    /// Node types written between a function's documentation and the function itself
    /// (Python decorators, Rust attributes); skipped when looking for documentation (ADR-0013).
    pub decorators: &'static [&'static str],
    /// Literal node type that documents a function when it is the first statement of its body
    /// (Python docstrings). Takes precedence over a preceding comment (ADR-0013).
    pub docstring: Option<&'static str>,
    /// Parameter node texts that are not parameters (e.g. C `f(void)`).
    pub ignored_parameters: &'static [&'static str],
}

/// tree-sitter runtime version. Checked against Cargo.lock by tests.
pub const TREE_SITTER_VERSION: &str = "0.27.0";

pub struct TreeSitterAdapter {
    mapping: &'static Mapping,
}

impl TreeSitterAdapter {
    pub const fn new(mapping: &'static Mapping) -> Self {
        TreeSitterAdapter { mapping }
    }
}

impl LanguageAdapter for TreeSitterAdapter {
    fn language(&self) -> &'static str {
        self.mapping.language
    }

    fn extensions(&self) -> &'static [&'static str] {
        self.mapping.extensions
    }

    fn parser_version(&self) -> String {
        let (name, version) = self.mapping.grammar_crate;
        format!("tree-sitter {TREE_SITTER_VERSION} / {name} {version}")
    }

    fn to_ir(&self, path: &str, source: &str) -> Result<File, AnalysisError> {
        let mut parser = Parser::new();
        parser
            .set_language(&(self.mapping.grammar)())
            .map_err(|e| AnalysisError::IrConversion {
                path: path.into(),
                message: format!("incompatible tree-sitter grammar: {e}"),
            })?;
        let tree = parser
            .parse(source, None)
            .ok_or_else(|| AnalysisError::IrConversion {
                path: path.into(),
                message: "tree-sitter returned no tree".into(),
            })?;
        let root = tree.root_node();
        if root.has_error() {
            return Err(syntax_error(path, root));
        }
        Converter {
            mapping: self.mapping,
            source,
            nodes: vec![],
            tokens: vec![],
            ts_to_ir: HashMap::new(),
        }
        .convert(path, root)
    }
}

/// Finds the first ERROR or MISSING node and reports its position.
fn syntax_error(path: &str, root: tree_sitter::Node) -> AnalysisError {
    let mut cursor = root.walk();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if node.is_error() || node.is_missing() {
            let p = node.start_position();
            let message = if node.is_missing() {
                format!("missing `{}`", node.kind())
            } else {
                "unexpected syntax".to_string()
            };
            return AnalysisError::Parse {
                path: path.into(),
                line: p.row + 1,
                column: p.column,
                message,
            };
        }
        let children: Vec<_> = node
            .children(&mut cursor)
            .filter(|c| c.has_error() || c.is_missing())
            .collect();
        stack.extend(children.into_iter().rev());
    }
    unreachable!("syntax_error is only called when the tree has an error")
}

struct Converter<'a> {
    mapping: &'static Mapping,
    source: &'a str,
    nodes: Vec<Node>,
    tokens: Vec<Token>,
    /// tree-sitter node id -> IR node id, for resolving function bodies.
    ts_to_ir: HashMap<usize, NodeId>,
}

impl<'a> Converter<'a> {
    /// Walks the concrete syntax tree once, iteratively (deep expressions must not overflow the stack).
    /// Named nodes become IR nodes; leaves and comments become tokens. A literal becomes literal
    /// tokens for its text, and only its interpolated code (e.g. `${...}`) is walked further.
    fn convert(mut self, path: &str, root: tree_sitter::Node<'a>) -> Result<File, AnalysisError> {
        let mut function_nodes = vec![];
        let mut stack: Vec<(tree_sitter::Node, Option<NodeId>, Option<&str>)> =
            vec![(root, None, None)];
        while let Some((ts, parent, field)) = stack.pop() {
            let kind = ts.kind();
            if self.mapping.comments.contains(&kind) {
                self.push_token(TokenKind::Comment, ts.byte_range(), range(ts));
                continue;
            }
            let literal = self.mapping.literals.contains(&kind);
            let children = if literal {
                self.push_literal(ts)
            } else {
                self.push_leaf(ts)
            };
            let ir_parent = if ts.is_named() {
                let id = self.push_node(ts, parent, field);
                if self.nodes[id.0].kind == NodeKind::Function {
                    function_nodes.push((ts, id));
                }
                Some(id)
            } else {
                parent
            };
            stack.extend(children.into_iter().rev().map(|(c, f)| (c, ir_parent, f)));
        }
        // Interpolated code is tokenized after its enclosing literal's text; restore source order.
        self.tokens.sort_by_key(|t| t.range.start.offset);
        let functions = function_nodes
            .into_iter()
            .enumerate()
            .map(|(i, (ts, id))| self.function(FunctionId(i), ts, id))
            .collect();
        Ok(File {
            path: path.into(),
            language: self.mapping.language.into(),
            source: self.source.into(),
            nodes: self.nodes,
            root: NodeId(0),
            tokens: self.tokens,
            functions,
        })
    }

    fn push_node(
        &mut self,
        ts: tree_sitter::Node,
        parent: Option<NodeId>,
        field: Option<&str>,
    ) -> NodeId {
        let id = NodeId(self.nodes.len());
        let mut kind = self.node_kind(ts);
        let parent_is_branch = parent.is_some_and(|p| self.nodes[p.0].kind == NodeKind::Branch);
        if parent_is_branch
            && kind != NodeKind::Branch
            && field.is_some()
            && field == self.mapping.else_field
        {
            kind = NodeKind::Else;
        }
        let label = match kind {
            NodeKind::Call => self.callee_name(ts),
            NodeKind::Logical => ts
                .child_by_field_name("operator")
                .map(|op| self.text(op).to_string()),
            _ => None,
        };
        self.nodes.push(Node {
            id,
            kind,
            parent,
            children: vec![],
            range: range(ts),
            label,
        });
        if let Some(p) = parent {
            self.nodes[p.0].children.push(id);
        }
        self.ts_to_ir.insert(ts.id(), id);
        id
    }

    fn node_kind(&self, ts: tree_sitter::Node) -> NodeKind {
        let kind = self
            .mapping
            .kinds
            .iter()
            .find(|(name, _)| *name == ts.kind())
            .map(|(_, k)| *k)
            .unwrap_or(NodeKind::Other);
        match kind {
            NodeKind::Binary if self.is_logical(ts) => NodeKind::Logical,
            NodeKind::Case if self.is_default_case(ts) => NodeKind::Other,
            k => k,
        }
    }

    /// The last identifier leaf of the callee (`f(x)` -> f, `a.b.c()` -> c, `std::sort()` -> sort).
    fn callee_name(&self, call: tree_sitter::Node) -> Option<String> {
        let callee = self
            .mapping
            .callee_fields
            .iter()
            .find_map(|f| call.child_by_field_name(f))?;
        let mut cursor = callee.walk();
        let mut last = None;
        let mut stack = vec![callee];
        while let Some(n) = stack.pop() {
            if n.child_count() == 0 && self.is_identifier(&n) {
                last = Some(n);
            }
            let children: Vec<_> = n.children(&mut cursor).collect();
            stack.extend(children.into_iter().rev());
        }
        last.map(|n| self.text(n).to_string())
    }

    fn is_logical(&self, ts: tree_sitter::Node) -> bool {
        ts.child_by_field_name("operator")
            .is_some_and(|op| self.mapping.logical_operators.contains(&self.text(op)))
    }

    /// A case whose first token is the default keyword (C `default:`, Java `default ->`).
    fn is_default_case(&self, ts: tree_sitter::Node) -> bool {
        let first_leaf = std::iter::successors(Some(ts), |n| n.child(0)).last();
        self.mapping
            .default_case_keyword
            .is_some_and(|kw| first_leaf.is_some_and(|leaf| leaf.kind() == kw))
    }

    fn push_token(&mut self, kind: TokenKind, bytes: std::ops::Range<usize>, range: SourceRange) {
        self.tokens.push(Token {
            kind,
            text: self.source[bytes].into(),
            range,
        });
    }

    /// Tokenizes a leaf (zero-width leaves such as an empty file's root carry no text and are
    /// skipped) and returns the children to walk.
    fn push_leaf(&mut self, ts: tree_sitter::Node<'a>) -> Vec<Child<'a>> {
        if ts.child_count() == 0 && ts.start_byte() < ts.end_byte() {
            self.push_token(self.classify_leaf(ts), ts.byte_range(), range(ts));
        }
        children_with_fields(ts)
    }

    /// Emits one literal token per stretch of text between interpolations, and returns the
    /// interpolation children, which are walked as ordinary code.
    fn push_literal(&mut self, ts: tree_sitter::Node<'a>) -> Vec<Child<'a>> {
        let mut cursor = ts.walk();
        let interpolations: Vec<_> = ts
            .named_children(&mut cursor)
            .filter(|c| self.mapping.interpolations.contains(&c.kind()))
            .collect();
        let start = |n: &tree_sitter::Node| position(n.start_position(), n.start_byte());
        let end = |n: &tree_sitter::Node| position(n.end_position(), n.end_byte());
        let mut text_from = start(&ts);
        let mut text = vec![];
        for c in &interpolations {
            text.push(SourceRange {
                start: text_from,
                end: start(c),
            });
            text_from = end(c);
        }
        text.push(SourceRange {
            start: text_from,
            end: end(&ts),
        });
        for r in text.into_iter().filter(|r| r.start.offset < r.end.offset) {
            self.push_token(TokenKind::Literal, r.start.offset..r.end.offset, r);
        }
        interpolations.into_iter().map(|c| (c, None)).collect()
    }

    /// Generic token classification rule (ADR-0004).
    fn classify_leaf(&self, ts: tree_sitter::Node) -> TokenKind {
        let text = self.text(ts);
        if self.mapping.identifiers.contains(&ts.kind()) {
            TokenKind::Identifier
        } else if text.chars().any(|c| c.is_alphabetic() || c == '_') {
            TokenKind::Keyword
        } else if matches!(text, "," | ";" | "(" | ")" | "[" | "]" | "{" | "}") {
            TokenKind::Punctuation
        } else {
            TokenKind::Operator
        }
    }

    fn function(&self, id: FunctionId, ts: tree_sitter::Node, node: NodeId) -> Function {
        Function {
            id,
            name: self.name_of(ts),
            parameters: self.parameters(ts),
            node,
            body: ts
                .child_by_field_name("body")
                .map(|b| self.ts_to_ir[&b.id()]),
            range: range(ts),
            doc: self.docstring(ts).or_else(|| self.preceding_comments(ts)),
        }
    }

    /// A docstring: the body's first statement consisting of a single docstring literal.
    fn docstring(&self, ts: tree_sitter::Node) -> Option<SourceRange> {
        let kind = self.mapping.docstring?;
        let first = ts.child_by_field_name("body")?.named_child(0)?;
        let is_statement = self
            .mapping
            .kinds
            .contains(&(first.kind(), NodeKind::Statement));
        let literal = first
            .named_child(0)
            .filter(|l| first.named_child_count() == 1 && l.kind() == kind)?;
        is_statement.then(|| range(literal))
    }

    /// The block of comment tokens directly above the function (or above its decorators): no blank
    /// line in between, and not a trailing comment of a preceding line of code.
    fn preceding_comments(&self, ts: tree_sitter::Node) -> Option<SourceRange> {
        let first = std::iter::successors(Some(ts), |n| {
            n.prev_named_sibling()
                .filter(|p| self.mapping.decorators.contains(&p.kind()))
        })
        .last()?;
        let end = self
            .tokens
            .partition_point(|t| t.range.start.offset < first.start_byte());
        let mut start = end;
        let mut next_line = first.start_position().row + 1;
        while start > 0 {
            let comment = &self.tokens[start - 1];
            let adjacent = comment.range.last_line() + 1 >= next_line;
            let trailing = start >= 2 && {
                let before = &self.tokens[start - 2];
                before.kind != TokenKind::Comment
                    && before.range.last_line() == comment.range.first_line()
            };
            if comment.kind != TokenKind::Comment || !adjacent || trailing {
                break;
            }
            next_line = comment.range.first_line();
            start -= 1;
        }
        (start < end).then(|| SourceRange {
            start: self.tokens[start].range.start,
            end: self.tokens[end - 1].range.end,
        })
    }

    /// Parameters come from a `parameters` list field, or a single `parameter` field
    /// (e.g. `x => x` in JavaScript). A function with neither has no parameters.
    fn parameters(&self, ts: tree_sitter::Node) -> Vec<Parameter> {
        let nodes: Vec<_> = if let Some(single) = self
            .find_field(ts, "parameters")
            .filter(|p| self.is_identifier(p))
        {
            vec![single] // Java `x -> ...`
        } else if let Some(list) = self.find_field(ts, "parameters") {
            let mut cursor = list.walk();
            list.named_children(&mut cursor)
                .filter(|p| !self.mapping.comments.contains(&p.kind()))
                .collect()
        } else {
            ts.child_by_field_name("parameter").into_iter().collect()
        };
        nodes
            .into_iter()
            .filter(|p| !self.mapping.ignored_parameters.contains(&self.text(*p)))
            .flat_map(|p| self.declared_parameters(p))
            .collect()
    }

    /// One parameter per name when a declaration names several (Go `a, b int`), else one.
    fn declared_parameters(&self, p: tree_sitter::Node) -> Vec<Parameter> {
        let mut cursor = p.walk();
        let names: Vec<_> = p.children_by_field_name("name", &mut cursor).collect();
        if names.len() > 1 {
            names
                .into_iter()
                .map(|n| Parameter {
                    name: Some(self.text(n).to_string()),
                    range: range(n),
                })
                .collect()
        } else {
            vec![Parameter {
                name: self.parameter_name(p),
                range: range(p),
            }]
        }
    }

    /// Looks up `field` on `ts`, then along its name-field chain (e.g. C's declarator nesting).
    fn find_field<'t>(
        &self,
        ts: tree_sitter::Node<'t>,
        field: &str,
    ) -> Option<tree_sitter::Node<'t>> {
        std::iter::successors(Some(ts), |n| self.next_name_node(*n))
            .find_map(|n| n.child_by_field_name(field))
    }

    /// Follows name fields until reaching an identifier (function names, and parameters with name fields).
    fn name_of(&self, ts: tree_sitter::Node) -> Option<String> {
        std::iter::successors(Some(ts), |n| self.next_name_node(*n))
            .find(|n| self.is_identifier(n))
            .map(|n| self.text(n).to_string())
    }

    /// A parameter's name: via name fields, or else the last child that is an identifier or has a
    /// name (Python `x: int` and `*args`, Java `String... xs`, where a type may come first).
    fn parameter_name(&self, p: tree_sitter::Node) -> Option<String> {
        let end = std::iter::successors(Some(p), |n| self.next_name_node(*n)).last()?;
        if self.is_identifier(&end) {
            return Some(self.text(end).to_string());
        }
        let mut cursor = end.walk();
        let named: Vec<_> = end.named_children(&mut cursor).collect();
        named.into_iter().rev().find_map(|c| {
            if self.is_identifier(&c) {
                Some(self.text(c).to_string())
            } else {
                self.name_of(c).filter(|_| self.next_name_node(c).is_some())
            }
        })
    }

    fn is_identifier(&self, n: &tree_sitter::Node) -> bool {
        self.mapping.identifiers.contains(&n.kind())
    }

    fn next_name_node<'t>(&self, ts: tree_sitter::Node<'t>) -> Option<tree_sitter::Node<'t>> {
        self.mapping
            .name_fields
            .iter()
            .find_map(|f| ts.child_by_field_name(f))
    }

    fn text(&self, ts: tree_sitter::Node) -> &'a str {
        &self.source[ts.byte_range()]
    }
}

/// A child node and the field it is in.
type Child<'a> = (tree_sitter::Node<'a>, Option<&'a str>);

fn children_with_fields(ts: tree_sitter::Node<'_>) -> Vec<Child<'_>> {
    let mut cursor = ts.walk();
    let mut children = vec![];
    if cursor.goto_first_child() {
        loop {
            children.push((cursor.node(), cursor.field_name()));
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
    children
}

fn position(p: Point, offset: usize) -> Position {
    Position {
        line: p.row + 1,
        column: p.column,
        offset,
    }
}

fn range(ts: tree_sitter::Node) -> SourceRange {
    SourceRange {
        start: position(ts.start_position(), ts.start_byte()),
        end: position(ts.end_position(), ts.end_byte()),
    }
}
