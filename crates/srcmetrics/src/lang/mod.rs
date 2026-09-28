//! Language Adapters: source code -> Common IR (ADR-0004).
//!
//! All language-specific knowledge lives in the `Mapping` tables of the
//! per-language modules. The Metric Engine never imports this module.

mod c;
mod cpp;
mod ecmascript;
mod go;
mod java;
mod python;
mod rust;
mod treesitter;

use crate::error::AnalysisError;
use crate::ir::File;
use std::path::Path;

pub use treesitter::{Mapping, TREE_SITTER_VERSION, TreeSitterAdapter};

pub trait LanguageAdapter: Sync {
    /// Language id written to the IR and to results (e.g. "c", "python").
    fn language(&self) -> &'static str;
    fn extensions(&self) -> &'static [&'static str];
    /// Parser name and version, recorded with results for reproducibility (design principle P10).
    fn parser_version(&self) -> String;
    fn to_ir(&self, path: &str, source: &str) -> Result<File, AnalysisError>;
}

static ADAPTERS: &[TreeSitterAdapter] = &[
    TreeSitterAdapter::new(&c::MAPPING),
    TreeSitterAdapter::new(&python::MAPPING),
    TreeSitterAdapter::new(&ecmascript::TYPESCRIPT),
    TreeSitterAdapter::new(&ecmascript::TSX),
    TreeSitterAdapter::new(&ecmascript::JAVASCRIPT),
    TreeSitterAdapter::new(&go::MAPPING),
    TreeSitterAdapter::new(&java::MAPPING),
    TreeSitterAdapter::new(&rust::MAPPING),
    TreeSitterAdapter::new(&cpp::MAPPING),
];

pub fn adapters() -> impl Iterator<Item = &'static dyn LanguageAdapter> {
    ADAPTERS.iter().map(|a| a as &dyn LanguageAdapter)
}

pub fn supported_extensions() -> Vec<&'static str> {
    adapters()
        .flat_map(|a| a.extensions().iter().copied())
        .collect()
}

/// Selects the adapter by file extension. Unknown extensions are an error: the
/// language is never guessed (ADR-0005).
pub fn adapter_for_path(path: &str) -> Result<&'static dyn LanguageAdapter, AnalysisError> {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    adapters()
        .find(|a| a.extensions().contains(&ext))
        .ok_or_else(|| AnalysisError::UnsupportedLanguage {
            path: path.to_string(),
        })
}
