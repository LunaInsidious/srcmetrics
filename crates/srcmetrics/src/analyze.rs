//! Analysis pipeline: path -> IR -> metrics -> `AnalysisResult` (ADR-0010).

use crate::error::AnalysisError;
use crate::ir::{File, Program};
use crate::lang::{LanguageAdapter, adapter_for_path, adapters};
use crate::metrics::{self, DEFINITION_VERSION, FileMetrics};
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
    let mut program = Program::default();
    let mut failures = Vec::new();
    for (path, relative) in targets(root)? {
        let adapter = adapter_for_path(&relative.to_string_lossy())?;
        match parse(adapter, &path, &relative) {
            Ok(file) => program.files.push(file),
            Err(failure) => failures.push(failure),
        }
    }
    let dir = if root.is_dir() {
        root
    } else {
        root.parent().expect("a canonical file path has a parent")
    };
    let run = run_info(
        project,
        git(dir, &["remote", "get-url", "origin"]),
        git(dir, &["rev-parse", "HEAD"]),
        &program,
    );
    Ok(assemble(program, failures, run))
}

/// Analyzes source text given with its file name (whose extension selects the language),
/// without reading the file system. A parse failure is an error, since it is the only file.
pub fn analyze_source(filename: &str, source: &str) -> Result<AnalysisResult, AnalysisError> {
    let file = adapter_for_path(filename)?.to_ir(filename, source)?;
    let stem = Path::new(filename)
        .file_stem()
        .expect("a file name with a supported extension has a stem");
    let project = stem.to_string_lossy().into_owned();
    let program = Program { files: vec![file] };
    let run = run_info(project, None, None, &program);
    Ok(assemble(program, vec![], run))
}

/// Computes the metrics of `program` and builds the result; files are sorted by path.
fn assemble(program: Program, failures: Vec<FileResult>, run: RunInfo) -> AnalysisResult {
    let computed = metrics::compute(&program);
    let mut files: Vec<FileResult> = program
        .files
        .iter()
        .zip(&computed.files)
        .map(|(f, m)| file_result(f, m))
        .collect();
    files.extend(failures);
    files.sort_by(|a, b| a.path().cmp(b.path()));
    AnalysisResult {
        run,
        project: MetricsOutput::from(&computed.project),
        files,
    }
}

fn file_result(file: &File, metrics: &FileMetrics) -> FileResult {
    let functions = file
        .functions
        .iter()
        .zip(&metrics.functions)
        .map(|(f, m)| FunctionResult {
            name: f.name.clone(),
            start_line: f.range.first_line(),
            end_line: f.range.last_line(),
            metrics: m.into(),
        })
        .collect();
    FileResult::Ok {
        path: file.path.clone(),
        language: file.language.clone(),
        metrics: (&metrics.metrics).into(),
        functions,
    }
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

/// Reads and converts one file; a failure becomes the file's error result.
fn parse(adapter: &dyn LanguageAdapter, path: &Path, relative: &Path) -> Result<File, FileResult> {
    let name = relative.to_string_lossy().replace('\\', "/");
    let failure = |error: AnalysisError| FileResult::Error {
        path: name.clone(),
        language: Some(adapter.language().to_string()),
        error: error.to_string(),
    };
    let source = std::fs::read_to_string(path).map_err(|e| {
        failure(AnalysisError::Io {
            path: name.clone(),
            message: e.to_string(),
        })
    })?;
    adapter.to_ir(&name, &source).map_err(failure)
}

fn run_info(
    project: String,
    repository: Option<String>,
    commit: Option<String>,
    program: &Program,
) -> RunInfo {
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
        repository,
        commit,
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
