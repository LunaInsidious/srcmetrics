//! Language-independent source code metrics.
//!
//! `srcmetrics` parses source code with [tree-sitter](https://tree-sitter.github.io/), converts it
//! to a small language-independent intermediate representation ([`ir`]), and computes metrics
//! from that representation only ([`metrics`]). Every metric is reported separately, with its
//! definition ([`metrics::definitions`]); none are combined into a single readability score.
//!
//! Supported languages: C, C++, Go, Java, JavaScript (JSX), Python, Rust, TypeScript (TSX).
//!
//! # Analyze source text
//!
//! ```
//! let result = srcmetrics::analyze::analyze_source(
//!     "example.py",
//!     "def greet(name):\n    if name:\n        return 'Hello, ' + name\n    return 'Hello'\n",
//! )?;
//! let srcmetrics::result::FileResult::Ok { functions, .. } = &result.files[0] else { unreachable!() };
//! assert_eq!(functions[0].name.as_deref(), Some("greet"));
//! assert_eq!(functions[0].metrics.metrics["complexity.cyclomatic"], Some(2.0));
//! # Ok::<(), srcmetrics::error::AnalysisError>(())
//! ```
//!
//! # Analyze a directory
//!
//! [`analyze::analyze`] walks a directory (respecting `.gitignore`), analyzes every file with a
//! supported extension and records run metadata (commit, parser and metric definition versions).
//! Files that fail to parse are kept in the result with `status: "error"` instead of being dropped.
//!
//! ```no_run
//! let result = srcmetrics::analyze::analyze(std::path::Path::new("src"), None)?;
//! println!("{}", serde_json::to_string_pretty(&result).unwrap());
//! # Ok::<(), srcmetrics::error::AnalysisError>(())
//! ```
//!
//! A metric that cannot be computed (e.g. an average over zero functions) is `null` in
//! [`result::MetricsOutput::metrics`], with the reason in [`result::MetricsOutput::unavailable`];
//! it is never reported as 0.
pub mod analyze;
pub mod csv;
pub mod error;
pub mod ir;
pub mod lang;
pub mod metrics;
pub mod model;
pub mod report;
pub mod result;
pub mod stats;
