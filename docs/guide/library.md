# Library

```sh
cargo add srcmetrics
```

## Analyze source text

```rust
use srcmetrics::analyze::analyze_source;
use srcmetrics::result::FileResult;

fn main() -> Result<(), srcmetrics::error::AnalysisError> {
    let result = analyze_source("example.py", "def f(x):\n    return x if x else 0\n")?;
    let FileResult::Ok { functions, .. } = &result.files[0] else { unreachable!() };
    assert_eq!(functions[0].metrics.metrics["complexity.cyclomatic"], Some(2.0));
    Ok(())
}
```

The file name's extension selects the language. A syntax error is returned as
`AnalysisError::Parse` with its position.

## Analyze a directory

```rust
let result = srcmetrics::analyze::analyze(std::path::Path::new("src"), None)?;
let json = serde_json::to_string_pretty(&result).unwrap();
```

`AnalysisResult` implements `Serialize` and `Deserialize`; its JSON is the
[output format](./output) of the CLI.

## Main API

| API | Purpose |
|---|---|
| `analyze::analyze(path, project)` | Analyze a file or directory into an `AnalysisResult` |
| `analyze::analyze_source(filename, source)` | Analyze source text in memory |
| `lang::adapter_for_path(path)?.to_ir(path, source)` | Parse to the common intermediate representation (`ir::File`) |
| `metrics::compute(&program)` | Compute every metric from the intermediate representation |
| `metrics::definitions()` | Metric definitions |
| `csv::to_csv`, `report::to_html` | CSV and HTML output |
| `stats::Report::of` | Statistics over files or functions |
| `model::train`, `model::predict_all` | The experimental model |

The full API documentation is on [docs.rs](https://docs.rs/srcmetrics).
