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
    /// Operators that turn a `Binary` node into a `Logical` (short-circuit) node.
    pub logical_operators: &'static [&'static str],
    /// Keyword that marks a `Case` node as the default label (not a decision point).
    pub default_case_keyword: Option<&'static str>,
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
        let mut stack: Vec<(tree_sitter::Node, Option<NodeId>)> = vec![(root, None)];
        while let Some((ts, parent)) = stack.pop() {
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
                let id = self.push_node(ts, parent);
                if self.nodes[id.0].kind == NodeKind::Function {
                    function_nodes.push((ts, id));
                }
                Some(id)
            } else {
                parent
            };
            stack.extend(children.into_iter().rev().map(|c| (c, ir_parent)));
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

    fn push_node(&mut self, ts: tree_sitter::Node, parent: Option<NodeId>) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(Node {
            id,
            kind: self.node_kind(ts),
            parent,
            children: vec![],
            range: range(ts),
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

    fn is_logical(&self, ts: tree_sitter::Node) -> bool {
        ts.child_by_field_name("operator")
            .is_some_and(|op| self.mapping.logical_operators.contains(&self.text(op)))
    }

    fn is_default_case(&self, ts: tree_sitter::Node) -> bool {
        let first = ts.child(0).map(|c| c.kind());
        self.mapping
            .default_case_keyword
            .is_some_and(|kw| first == Some(kw))
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
    fn push_leaf(&mut self, ts: tree_sitter::Node<'a>) -> Vec<tree_sitter::Node<'a>> {
        if ts.child_count() == 0 && ts.start_byte() < ts.end_byte() {
            self.push_token(self.classify_leaf(ts), ts.byte_range(), range(ts));
        }
        let mut cursor = ts.walk();
        ts.children(&mut cursor).collect()
    }

    /// Emits one literal token per stretch of text between interpolations, and returns the
    /// interpolation children, which are walked as ordinary code.
    fn push_literal(&mut self, ts: tree_sitter::Node<'a>) -> Vec<tree_sitter::Node<'a>> {
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
        interpolations
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
            doc: None,
        }
    }

    /// Parameters come from a `parameters` list field, or a single `parameter` field
    /// (e.g. `x => x` in JavaScript). A function with neither has no parameters.
    fn parameters(&self, ts: tree_sitter::Node) -> Vec<Parameter> {
        let nodes: Vec<_> = if let Some(list) = self.find_field(ts, "parameters") {
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
            .map(|p| Parameter {
                name: self.name_of(p),
                range: range(p),
            })
            .collect()
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

    /// Follows name fields until reaching an identifier. When the chain ends on a node
    /// without a name field (e.g. Python `x: int`, `*args`), its first identifier child is the name.
    fn name_of(&self, ts: tree_sitter::Node) -> Option<String> {
        let is_identifier = |n: &tree_sitter::Node| self.mapping.identifiers.contains(&n.kind());
        let last = std::iter::successors(Some(ts), |n| self.next_name_node(*n)).last()?;
        let name = if is_identifier(&last) {
            Some(last)
        } else {
            let mut cursor = last.walk();
            last.named_children(&mut cursor).find(is_identifier)
        };
        name.map(|n| self.text(n).to_string())
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
