//! Analysis pipeline: path -> IR -> metrics -> `AnalysisResult` (ADR-0010).

use crate::error::AnalysisError;
use crate::ir::{File, Program};
use crate::lang::{LanguageAdapter, adapter_for_path, adapters};
use crate::metrics::{self, DEFINITION_VERSION};
use crate::result::{AnalysisResult, FileResult, FunctionResult, MetricsOutput, RunInfo};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Analyzes a file or a directory. Directories are walked respecting `.gitignore`; files with
/// unsupported extensions are not analysis targets. An explicitly given unsupported file is an error.
/// `project` defaults to the name of the analyzed directory (or file).
pub fn analyze(root: &Path, project: Option<&str>) -> Result<AnalysisResult, AnalysisError> {
    let root = &root.canonicalize().map_err(|e| io_error(root, &e))?;
    let project = match project {
        Some(p) => p.to_string(),
        None => default_project_name(root)?,
    };
    let targets = targets(root)?;
    let mut parsed: Vec<Result<File, ParseFailure>> = Vec::new();
    for (path, relative) in &targets {
        let adapter = adapter_for_path(&relative.to_string_lossy())?;
        parsed.push(parse(adapter, path, relative));
    }
    let program = Program {
        files: parsed
            .iter()
            .filter_map(|r| r.as_ref().ok())
            .cloned()
            .collect(),
    };
    let computed = metrics::compute(&program);

    let mut computed_files = program.files.iter().zip(computed.files);
    let files = parsed
        .into_iter()
        .map(|r| match r {
            Ok(_) => {
                let (file, m) = computed_files
                    .next()
                    .expect("one metrics entry per parsed file");
                FileResult::Ok {
                    path: file.path.clone(),
                    language: file.language.clone(),
                    metrics: (&m.metrics).into(),
                    functions: file
                        .functions
                        .iter()
                        .zip(&m.functions)
                        .map(|(f, fm)| FunctionResult {
                            name: f.name.clone(),
                            start_line: f.range.first_line(),
                            end_line: f.range.last_line(),
                            metrics: fm.into(),
                        })
                        .collect(),
                }
            }
            Err((path, language, error)) => FileResult::Error {
                path,
                language: language.map(str::to_string),
                error,
            },
        })
        .collect();

    Ok(AnalysisResult {
        run: run_info(root, project, &program),
        project: MetricsOutput::from(&computed.project),
        files,
    })
}

/// (absolute path, path relative to `root`) of every analysis target, sorted by relative path.
fn io_error(path: &Path, e: &dyn std::fmt::Display) -> AnalysisError {
    AnalysisError::Io {
        path: path.display().to_string(),
        message: e.to_string(),
    }
}

fn targets(root: &Path) -> Result<Vec<(PathBuf, PathBuf)>, AnalysisError> {
    if root.is_file() {
        let name = PathBuf::from(root.file_name().expect("a file path has a file name"));
        return Ok(vec![(root.to_path_buf(), name)]);
    }
    let supported = |p: &Path| {
        p.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| adapters().any(|a| a.extensions().contains(&e)))
    };
    let mut targets = Vec::new();
    for entry in ignore::WalkBuilder::new(root).build() {
        let entry = entry.map_err(|e| io_error(root, &e))?;
        let path = entry.path();
        if entry.file_type().is_some_and(|t| t.is_file()) && supported(path) {
            let relative = path
                .strip_prefix(root)
                .expect("walked paths are under the root")
                .to_path_buf();
            targets.push((path.to_path_buf(), relative));
        }
    }
    targets.sort_by(|a, b| a.1.cmp(&b.1));
    Ok(targets)
}

type ParseFailure = (String, Option<&'static str>, String);

fn parse(
    adapter: &dyn LanguageAdapter,
    path: &Path,
    relative: &Path,
) -> Result<File, ParseFailure> {
    let name = relative.to_string_lossy().replace('\\', "/");
    let source = std::fs::read_to_string(path).map_err(|e| {
        let error = AnalysisError::Io {
            path: name.clone(),
            message: e.to_string(),
        };
        (name.clone(), Some(adapter.language()), error.to_string())
    })?;
    adapter
        .to_ir(&name, &source)
        .map_err(|e| (name.clone(), Some(adapter.language()), e.to_string()))
}

fn run_info(root: &Path, project: String, program: &Program) -> RunInfo {
    let dir = if root.is_dir() {
        root
    } else {
        root.parent().expect("a canonical file path has a parent")
    };
    let parsers: BTreeMap<String, String> = program
        .files
        .iter()
        .map(|f| {
            let adapter = adapters()
                .find(|a| a.language() == f.language)
                .expect("files come from an adapter");
            (f.language.clone(), adapter.parser_version())
        })
        .collect();
    RunInfo {
        project,
        repository: git(dir, &["remote", "get-url", "origin"]),
        commit: git(dir, &["rev-parse", "HEAD"]),
        tool_version: env!("CARGO_PKG_VERSION").to_string(),
        metric_definition_version: DEFINITION_VERSION.to_string(),
        parsers,
        timestamp: time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .expect("UTC timestamps are always formattable"),
    }
}

/// The analyzed directory's name, or the file's stem.
fn default_project_name(root: &Path) -> Result<String, AnalysisError> {
    let name = if root.is_file() {
        root.file_stem()
    } else {
        root.file_name()
    };
    name.map(|n| n.to_string_lossy().into_owned())
        .ok_or_else(|| AnalysisError::Io {
            path: root.display().to_string(),
            message: "cannot derive a project name from this path; pass a project name explicitly"
                .into(),
        })
}

/// Output of a git command run in `dir`, or `None` if git fails (not a repository, no remote,
/// git not installed). The value is never guessed (ADR-0010).
fn git(dir: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}
