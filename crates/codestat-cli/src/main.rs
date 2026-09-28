//! codestat command-line interface.

mod serve;

use clap::{Parser, Subcommand, ValueEnum};
use codestat::analyze::analyze;
use codestat::result::{AnalysisResult, FileResult};
use codestat::stats::{self, UnitScope};
use codestat::{metrics, model};
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
    /// Write a self-contained HTML report (tables, histograms, correlation heatmap) of a result.
    Report {
        /// Result file written by `codestat analyze` (JSON).
        result: PathBuf,
        /// Write the report to this file instead of stdout.
        #[arg(long, short)]
        output: Option<PathBuf>,
    },
    /// Experimental readability model fitted on your own labels (no built-in weights).
    Model {
        #[command(subcommand)]
        command: ModelCommand,
    },
    /// Serve the HTTP API and a minimal web UI (local use; no authentication).
    Serve {
        /// Address to listen on.
        #[arg(long, default_value = "127.0.0.1")]
        bind: std::net::IpAddr,
        /// Port to listen on (0 picks a free port).
        #[arg(long, default_value_t = 8080)]
        port: u16,
    },
    /// List the metric definitions.
    Metrics {
        #[arg(long, value_enum, default_value_t = DefinitionFormat::Json)]
        format: DefinitionFormat,
    },
}

#[derive(Subcommand)]
enum ModelCommand {
    /// Fit a ridge regression on labelled files or functions and write the model (JSON).
    Train {
        /// Labels CSV with the header `path,function,score` (function empty for file labels).
        #[arg(long)]
        labels: PathBuf,
        /// Comma-separated metric ids to use (default: every usable metric of the labels' scope).
        #[arg(long, value_delimiter = ',')]
        features: Option<Vec<String>>,
        /// Ridge regularization strength (>= 0).
        #[arg(long, default_value_t = 1.0)]
        lambda: f64,
        /// Write the model to this file instead of stdout.
        #[arg(long, short)]
        output: Option<PathBuf>,
        /// Result files written by `codestat analyze` (JSON).
        #[arg(required = true)]
        results: Vec<PathBuf>,
    },
    /// Score every file or function of analysis results with a trained model (JSON).
    Predict {
        #[arg(long)]
        model: PathBuf,
        #[arg(required = true)]
        results: Vec<PathBuf>,
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
        Command::Model {
            command:
                ModelCommand::Train {
                    labels,
                    features,
                    lambda,
                    output,
                    results,
                },
        } => {
            let results = load_results(&results)?;
            let text = read(&labels)?;
            let labels =
                model::read_labels(&text).map_err(|e| format!("{}: {e}", labels.display()))?;
            let trained = model::train(&results, &labels, features.as_deref(), lambda)
                .map_err(|e| e.to_string())?;
            write(output, &to_json(&trained)?)
        }
        Command::Model {
            command:
                ModelCommand::Predict {
                    model: path,
                    results,
                },
        } => {
            let trained = model::Model::from_json(&read(&path)?)
                .map_err(|e| format!("{}: {e}", path.display()))?;
            write(
                None,
                &to_json(&model::predict_all(&trained, &load_results(&results)?))?,
            )
        }
        Command::Report { result, output } => {
            let result = load_results(std::slice::from_ref(&result))?.remove(0);
            write(output, &codestat::report::to_html(&result))
        }
        Command::Serve { bind, port } => serve::serve(std::net::SocketAddr::new(bind, port)),
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

fn read(path: &PathBuf) -> Result<String, String> {
    std::fs::read_to_string(path)
        .map_err(|e| format!("{}: {e}; check that the file exists", path.display()))
}

fn load_results(paths: &[PathBuf]) -> Result<Vec<AnalysisResult>, String> {
    paths
        .iter()
        .map(|p| {
            let text = read(p)?;
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
