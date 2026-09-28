//! Analysis errors (PLAN.md §14, ADR-0005).

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum AnalysisError {
    /// No Language Adapter handles the file extension.
    UnsupportedLanguage { path: String },
    /// The parser could not build a valid syntax tree.
    Parse {
        path: String,
        line: usize,
        column: usize,
        message: String,
    },
    /// A file or directory could not be read.
    Io { path: String, message: String },
    /// The syntax tree could not be converted to the Common IR.
    IrConversion { path: String, message: String },
}

impl fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnalysisError::UnsupportedLanguage { path } => write!(
                f,
                "{path}: unsupported file extension; supported extensions are: {}",
                crate::lang::supported_extensions().join(", ")
            ),
            AnalysisError::Parse {
                path,
                line,
                column,
                message,
            } => write!(
                f,
                "{path}:{line}:{column}: parse error: {message}; fix the syntax or check that the extension matches the language"
            ),
            AnalysisError::Io { path, message } => {
                write!(
                    f,
                    "{path}: {message}; check that the path exists and is readable"
                )
            }
            AnalysisError::IrConversion { path, message } => {
                write!(
                    f,
                    "{path}: IR conversion error: {message}; please report this with the input file"
                )
            }
        }
    }
}

impl std::error::Error for AnalysisError {}
