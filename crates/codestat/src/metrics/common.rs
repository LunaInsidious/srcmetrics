//! Helpers shared by calculators. IR-only.

use crate::ir::{File, Node, NodeKind, TokenKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LineClass {
    Code,
    Comment,
    Blank,
}

/// Classifies every line of the file (index 0 = line 1).
///
/// A line is `Code` if any non-comment token occupies it, else `Comment` if a comment token
/// occupies it, else `Blank` if it is whitespace only. A non-blank line occupied by no token
/// means the IR is inconsistent with the source, which is reported instead of guessed.
pub(crate) fn line_classes(file: &File) -> Result<Vec<LineClass>, String> {
    let source_lines: Vec<&str> = file.source.lines().collect();
    let mut classes: Vec<Option<LineClass>> = vec![None; source_lines.len()];
    for token in &file.tokens {
        let class = if token.kind == TokenKind::Comment {
            LineClass::Comment
        } else {
            LineClass::Code
        };
        for line in token.range.first_line()..=token.range.last_line() {
            let slot = classes.get_mut(line - 1).ok_or_else(|| {
                format!(
                    "token {:?} is on line {line}, beyond the end of the source",
                    token.text
                )
            })?;
            if *slot != Some(LineClass::Code) {
                *slot = Some(class);
            }
        }
    }
    classes
        .into_iter()
        .zip(&source_lines)
        .enumerate()
        .map(|(i, (class, text))| match class {
            Some(c) => Ok(c),
            None if text.trim().is_empty() => Ok(LineClass::Blank),
            None => Err(format!("line {} has text that no token covers", i + 1)),
        })
        .collect()
}

/// Statement-like node kinds (PLAN.md §8.1 Statement Count).
pub(crate) fn is_statement(kind: NodeKind) -> bool {
    use NodeKind::*;
    matches!(
        kind,
        Statement | Declaration | Branch | Loop | Return | Jump
    )
}

/// Decision points of Cyclomatic Complexity.
pub(crate) fn is_decision(kind: NodeKind) -> bool {
    use NodeKind::*;
    matches!(kind, Branch | Loop | Case | Catch | Logical | Conditional)
}

/// Control structures that open a nesting level.
pub(crate) fn is_nesting(kind: NodeKind) -> bool {
    use NodeKind::*;
    matches!(kind, Branch | Loop | Case | Catch)
}

/// A branch continuing an if-chain (`else if`, `elif`): a branch whose parent is an `else`
/// or another branch. It is part of its chain head's level, not a deeper one.
pub(crate) fn is_continuation(file: &File, node: &Node) -> bool {
    node.kind == NodeKind::Branch
        && node
            .parent
            .is_some_and(|p| matches!(file.node(p).kind, NodeKind::Else | NodeKind::Branch))
}

/// Number of nesting levels enclosing `node` within its function (or top-level code).
/// A continuation branch has the level of its chain head.
pub(crate) fn nesting_level(file: &File, node: &Node) -> usize {
    file.ancestors(chain_head(file, node).id)
        .take_while(|a| a.kind != NodeKind::Function)
        .filter(|a| is_nesting(a.kind) && !is_continuation(file, a))
        .count()
}

/// The first branch of the if-chain `node` continues; `node` itself if it is not a continuation.
fn chain_head<'a>(file: &'a File, node: &'a Node) -> &'a Node {
    let mut n = node;
    while is_continuation(file, n) {
        n = file
            .ancestors(n.id)
            .find(|a| a.kind == NodeKind::Branch)
            .expect("a continuation branch always has a branch ancestor");
    }
    n
}
