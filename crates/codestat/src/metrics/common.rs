//! Helpers shared by calculators. IR-only.

use crate::ir::{File, NodeKind, TokenKind};

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
            let slot = &mut classes[line - 1];
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
