//! codestat command-line interface.

use clap::{Parser, Subcommand, ValueEnum};
use codestat::analyze::analyze;
use codestat::metrics;
use codestat::result::{AnalysisResult, FileResult};
use codestat::stats::{self, UnitScope};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    version,
    about = "Language-independent source code readability metrics"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Analyze a file or directory and print the result.
    Analyze {
        /// File or directory to analyze. Directories are walked respecting .gitignore.
        path: PathBuf,
        /// Project name recorded in the result (default: the directory name).
        #[arg(long)]
        project: Option<String>,
        /// Write the result to this file instead of stdout.
        #[arg(long, short)]
        output: Option<PathBuf>,
        /// JSON has every value and the reason for each null; CSV has one row per project/file/function.
        #[arg(long, value_enum, default_value_t = ResultFormat::Json)]
        format: ResultFormat,
    },
    /// Descriptive statistics, per-language baselines and correlations of analysis results (JSON).
    Stats {
        /// Result files written by `codestat analyze` (JSON).
        #[arg(required = true)]
        results: Vec<PathBuf>,
        /// Unit of analysis.
        #[arg(long, value_enum, default_value_t = Unit::File)]
        scope: Unit,
    },
    /// List the metric definitions.
    Metrics {
        #[arg(long, value_enum, default_value_t = DefinitionFormat::Json)]
        format: DefinitionFormat,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum ResultFormat {
    Json,
    Csv,
}

#[derive(Clone, Copy, ValueEnum)]
enum Unit {
    File,
    Function,
}

impl From<Unit> for UnitScope {
    fn from(unit: Unit) -> Self {
        match unit {
            Unit::File => UnitScope::File,
            Unit::Function => UnitScope::Function,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum DefinitionFormat {
    Json,
    Markdown,
}

fn main() -> ExitCode {
    match run(Cli::parse().command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(command: Command) -> Result<(), String> {
    match command {
        Command::Analyze {
            path,
            project,
            output,
            format,
        } => {
            let result = analyze(&path, project.as_deref()).map_err(|e| e.to_string())?;
            warn_failed_files(&result);
            let text = match format {
                ResultFormat::Json => to_json(&result)?,
                ResultFormat::Csv => codestat::csv::to_csv(&result),
            };
            write(output, &text)
        }
        Command::Stats { results, scope } => {
            let results = load_results(&results)?;
            let scope = UnitScope::from(scope);
            let report =
                stats::Report::of(&stats::units(&results, scope), &stats::metric_ids(scope));
            write(None, &to_json(&report)?)
        }
        Command::Metrics { format } => {
            let definitions = metrics::definitions();
            let text = match format {
                DefinitionFormat::Json => to_json(&definitions)?,
                DefinitionFormat::Markdown => metrics::to_markdown(&definitions),
            };
            write(None, &text)
        }
    }
}

fn load_results(paths: &[PathBuf]) -> Result<Vec<AnalysisResult>, String> {
    paths
        .iter()
        .map(|p| {
            let text = std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()))?;
            serde_json::from_str(&text).map_err(|e| {
                format!("{}: not an analysis result ({e}); create one with `codestat analyze <PATH> -o result.json`", p.display())
            })
        })
        .collect()
}

fn to_json(value: &impl serde::Serialize) -> Result<String, String> {
    serde_json::to_string_pretty(value)
        .map(|s| s + "\n")
        .map_err(|e| format!("cannot serialize result: {e}"))
}

fn write(path: Option<PathBuf>, text: &str) -> Result<(), String> {
    match path {
        Some(p) => std::fs::write(&p, text).map_err(|e| {
            format!(
                "{}: {e}; check that the directory exists and is writable",
                p.display()
            )
        }),
        None => {
            print!("{text}");
            Ok(())
        }
    }
}

/// Failed files are part of the result; also report them on stderr so they are not missed.
fn warn_failed_files(result: &AnalysisResult) {
    let failed: Vec<_> = result
        .files
        .iter()
        .filter_map(|f| match f {
            FileResult::Error { error, .. } => Some(error),
            FileResult::Ok { .. } => None,
        })
        .collect();
    if !failed.is_empty() {
        eprintln!(
            "warning: {} file(s) could not be analyzed (status \"error\" in the result):",
            failed.len()
        );
        for error in failed {
            eprintln!("  {error}");
        }
    }
}
