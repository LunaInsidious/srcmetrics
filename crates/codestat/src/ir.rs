//! Common Intermediate Representation (PLAN.md §7, ADR-0003).
//!
//! The IR is the only input of the Metric Engine. It carries no language-specific
//! information other than the language id string.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    /// 1-based line number.
    pub line: usize,
    /// 0-based byte column.
    pub column: usize,
    /// 0-based byte offset from the start of the file.
    pub offset: usize,
}

/// Half-open source range `[start, end)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceRange {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeKind {
    Block,
    Statement,
    Expression,
    Declaration,
    Branch,
    Else,
    Loop,
    Case,
    Catch,
    Return,
    Jump,
    Call,
    Assignment,
    Binary,
    Logical,
    Conditional,
    Unary,
    Identifier,
    Literal,
    Import,
    Function,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub range: SourceRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Keyword,
    Identifier,
    Literal,
    Operator,
    Punctuation,
    Comment,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub range: SourceRange,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: Option<String>,
    pub range: SourceRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId(pub usize);

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub id: FunctionId,
    /// `None` for anonymous functions (lambdas, closures).
    pub name: Option<String>,
    pub parameters: Vec<Parameter>,
    /// The `NodeKind::Function` node of this function.
    pub node: NodeId,
    pub body: Option<NodeId>,
    pub range: SourceRange,
    /// Range of the documentation attached to this function, if any.
    pub doc: Option<SourceRange>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct File {
    pub path: String,
    pub language: String,
    pub source: String,
    pub nodes: Vec<Node>,
    pub root: NodeId,
    pub tokens: Vec<Token>,
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Program {
    pub files: Vec<File>,
}

impl SourceRange {
    pub fn first_line(&self) -> usize {
        self.start.line
    }

    /// Last line that contains at least one character of the range.
    /// A range ending at column 0 (e.g. a comment that includes its newline)
    /// does not occupy the line it ends on.
    pub fn last_line(&self) -> usize {
        if self.end.column == 0 && self.end.line > self.start.line {
            self.end.line - 1
        } else {
            self.end.line
        }
    }

    pub fn line_count(&self) -> usize {
        self.last_line() - self.first_line() + 1
    }
}

impl File {
    pub fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id.0]
    }

    /// Ancestors of `id`, nearest first.
    pub fn ancestors(&self, id: NodeId) -> impl Iterator<Item = &Node> {
        std::iter::successors(self.node(id).parent, |p| self.node(*p).parent).map(|p| self.node(p))
    }

    /// Descendants of `id` in pre-order, excluding `id` itself.
    /// Subtrees rooted at nodes for which `prune` returns true are skipped entirely.
    pub fn descendants_pruned<'a>(
        &'a self,
        id: NodeId,
        prune: impl Fn(&Node) -> bool + 'a,
    ) -> impl Iterator<Item = &'a Node> + 'a {
        let mut stack: Vec<NodeId> = self.node(id).children.iter().rev().copied().collect();
        std::iter::from_fn(move || {
            while let Some(next) = stack.pop() {
                let node = self.node(next);
                if prune(node) {
                    continue;
                }
                stack.extend(node.children.iter().rev().copied());
                return Some(node);
            }
            None
        })
    }

    /// Nodes that belong to `function` itself: its descendants, excluding nested functions.
    pub fn function_nodes<'a>(
        &'a self,
        function: &Function,
    ) -> impl Iterator<Item = &'a Node> + 'a {
        self.descendants_pruned(function.node, |n| n.kind == NodeKind::Function)
    }

    /// Nodes outside of any function (module / top-level code).
    pub fn top_level_nodes(&self) -> impl Iterator<Item = &Node> {
        self.descendants_pruned(self.root, |n| n.kind == NodeKind::Function)
    }

    /// Tokens whose range lies within `range`.
    pub fn tokens_in(&self, range: SourceRange) -> impl Iterator<Item = &Token> {
        self.tokens.iter().filter(move |t| {
            t.range.start.offset >= range.start.offset && t.range.end.offset <= range.end.offset
        })
    }
}

#[cfg(test)]
pub mod builder {
    //! Hand-built IR for Metric Engine unit tests, independent of any parser (PLAN.md §13.4).
    use super::*;

    pub fn pos(line: usize) -> Position {
        Position {
            line,
            column: 0,
            offset: 0,
        }
    }

    pub fn lines(first: usize, last: usize) -> SourceRange {
        SourceRange {
            start: pos(first),
            end: Position {
                line: last,
                column: 1,
                offset: 0,
            },
        }
    }

    pub struct FileBuilder {
        file: File,
    }

    impl FileBuilder {
        pub fn new(source: &str) -> Self {
            let root = Node {
                id: NodeId(0),
                kind: NodeKind::Block,
                parent: None,
                children: vec![],
                range: lines(1, source.lines().count().max(1)),
            };
            FileBuilder {
                file: File {
                    path: "test".into(),
                    language: "test".into(),
                    source: source.into(),
                    nodes: vec![root],
                    root: NodeId(0),
                    tokens: vec![],
                    functions: vec![],
                },
            }
        }

        pub fn root(&self) -> NodeId {
            self.file.root
        }

        pub fn node(&mut self, parent: NodeId, kind: NodeKind) -> NodeId {
            let range = self.file.node(parent).range;
            self.node_at(parent, kind, range)
        }

        pub fn node_at(&mut self, parent: NodeId, kind: NodeKind, range: SourceRange) -> NodeId {
            let id = NodeId(self.file.nodes.len());
            self.file.nodes.push(Node {
                id,
                kind,
                parent: Some(parent),
                children: vec![],
                range,
            });
            self.file.nodes[parent.0].children.push(id);
            id
        }

        /// Adds a function node under `parent` and registers it as a Function.
        pub fn function(
            &mut self,
            parent: NodeId,
            name: &str,
            params: usize,
            range: SourceRange,
        ) -> NodeId {
            let node = self.node_at(parent, NodeKind::Function, range);
            let body = self.node_at(node, NodeKind::Block, range);
            self.file.functions.push(Function {
                id: FunctionId(self.file.functions.len()),
                name: Some(name.into()),
                parameters: (0..params)
                    .map(|_| Parameter { name: None, range })
                    .collect(),
                node,
                body: Some(body),
                range,
                doc: None,
            });
            body
        }

        pub fn token(&mut self, kind: TokenKind, text: &str, line: usize) -> &mut Self {
            let offset = self.file.tokens.len();
            let start = Position {
                line,
                column: 0,
                offset,
            };
            let end = Position {
                line,
                column: text.len(),
                offset: offset + 1,
            };
            self.file.tokens.push(Token {
                kind,
                text: text.into(),
                range: SourceRange { start, end },
            });
            self
        }

        pub fn build(self) -> File {
            self.file
        }
    }
}

#[cfg(test)]
mod tests {
    use super::builder::*;
    use super::*;

    #[test]
    fn range_ending_at_column_zero_does_not_occupy_that_line() {
        let r = SourceRange {
            start: Position {
                line: 3,
                column: 4,
                offset: 0,
            },
            end: Position {
                line: 4,
                column: 0,
                offset: 0,
            },
        };
        assert_eq!(r.last_line(), 3);
        assert_eq!(r.line_count(), 1);
    }

    #[test]
    fn range_line_count_is_inclusive() {
        assert_eq!(lines(2, 5).line_count(), 4);
    }

    #[test]
    fn ancestors_are_nearest_first() {
        let mut b = FileBuilder::new("");
        let root = b.root();
        let a = b.node(root, NodeKind::Loop);
        let c = b.node(a, NodeKind::Branch);
        let file = b.build();
        let kinds: Vec<_> = file.ancestors(c).map(|n| n.kind).collect();
        assert_eq!(kinds, vec![NodeKind::Loop, NodeKind::Block]);
    }

    #[test]
    fn function_nodes_exclude_nested_functions() {
        let mut b = FileBuilder::new("");
        let root = b.root();
        let outer = b.function(root, "outer", 0, lines(1, 5));
        b.node(outer, NodeKind::Branch);
        let inner = b.function(outer, "inner", 0, lines(2, 3));
        b.node(inner, NodeKind::Loop);
        let file = b.build();
        let kinds: Vec<_> = file
            .function_nodes(&file.functions[0])
            .map(|n| n.kind)
            .collect();
        assert_eq!(kinds, vec![NodeKind::Block, NodeKind::Branch]);
    }

    #[test]
    fn top_level_nodes_exclude_functions() {
        let mut b = FileBuilder::new("");
        let root = b.root();
        b.node(root, NodeKind::Import);
        let body = b.function(root, "f", 0, lines(1, 2));
        b.node(body, NodeKind::Loop);
        let file = b.build();
        let kinds: Vec<_> = file.top_level_nodes().map(|n| n.kind).collect();
        assert_eq!(kinds, vec![NodeKind::Import]);
    }
}
