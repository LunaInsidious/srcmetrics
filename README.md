# srcmetrics

Language-independent source code metrics for C, C++, Go, Java, JavaScript, Python, Rust and TypeScript.

srcmetrics collects quantitative features related to readability and maintainability — size,
control-flow complexity, nesting, Halstead measures, duplication, call dependencies and
documentation — and reports each of them **separately**, so that they can be used for
statistical analysis or to build your own readability models.

- **One definition, every language.** Source code is parsed with [tree-sitter] and converted to a
  small common intermediate representation. Metrics are computed from that representation only;
  language differences live in declarative per-language mapping tables.
- **No single "readability score".** How metrics relate to human-perceived readability is not
  known in advance, so srcmetrics keeps the raw features. An optional, experimental model can be
  fitted on *your own* labels.
- **"Not computable" is never "0".** A value that does not apply (e.g. average function length of
  a file without functions) is `null`, with the reason recorded next to it.
- **Reproducible.** Results record the parser versions, the metric definition version and the git
  commit that was analyzed.

[tree-sitter]: https://tree-sitter.github.io/

## Installation

Command-line tool:

```sh
cargo install srcmetrics-cli   # installs the `srcmetrics` command
```

Library:

```sh
cargo add srcmetrics
```

Requires Rust 1.85 or later. The grammars are compiled from C, so a C compiler is needed at build time.
Analysis runs fully offline.

## Supported languages

| Language | Extensions |
|---|---|
| C | `.c`, `.h` |
| C++ | `.cpp`, `.cc`, `.cxx`, `.hpp`, `.hh`, `.hxx` |
| Go | `.go` |
| Java | `.java` |
| JavaScript | `.js`, `.mjs`, `.cjs`, `.jsx` |
| Python | `.py` |
| Rust | `.rs` |
| TypeScript | `.ts`, `.mts`, `.cts`, `.tsx` |

The same algorithm written in all eight languages yields identical cyclomatic complexity,
cognitive complexity and maximum nesting depth; this is part of the test suite.

## Metrics

53 metrics at function, file and project scope. Run `srcmetrics metrics` for the full definitions
(definition, calculation, unit, language applicability, limitations and references), or see
[docs/METRICS.md](docs/METRICS.md).

| Group | Metrics |
|---|---|
| Size | LOC, SLOC, comment / blank LOC, comment ratio, statements, tokens, functions, function length (avg / max) |
| Complexity | cyclomatic, cognitive, number of paths, branch / conditional / loop / return / jump counts |
| Nesting | maximum and average nesting depth |
| Halstead | n1, n2, N1, N2, vocabulary, length, volume, difficulty, effort, time, bugs |
| Function | parameter count (avg / max), expression count, call count |
| Duplication | duplicate blocks, duplicate tokens, duplication ratio, longest duplicate (normalized-token clones) |
| Dependency | fan-in, fan-out, call depth, import count |
| Documentation | documented functions, documentation ratio, documentation lines |
| Derived | maintainability index; cyclomatic per function, tokens per LOC, statements per function, duplicate tokens per SLOC |

## Command-line usage

```sh
srcmetrics analyze src -o result.json          # analyze a directory (respects .gitignore)
srcmetrics analyze src --format csv > metrics.csv
srcmetrics stats result.json --scope function  # descriptive statistics, per-language baselines, correlations
srcmetrics report result.json -o report.html   # self-contained HTML: tables, histograms, correlation heatmap
srcmetrics metrics --format markdown           # metric definitions
srcmetrics serve                               # HTTP API and web UI on http://127.0.0.1:8080
```

Files that fail to parse are kept in the result with `"status": "error"` and reported on stderr;
they never silently disappear.

### Output

```json
{
  "run": {
    "project": "readme",
    "repository": null,
    "commit": "3f2c…",
    "tool_version": "0.1.0",
    "metric_definition_version": "0.1.0",
    "parsers": { "python": "tree-sitter 0.27.0 / tree-sitter-python 0.25.0" },
    "timestamp": "2026-09-29T00:00:00Z"
  },
  "files": [
    {
      "status": "ok",
      "path": "greet.py",
      "language": "python",
      "metrics": { "size.loc": 4.0, "size.function_count": 1.0, "size.comment_ratio": 0.0 },
      "unavailable": {},
      "functions": [
        {
          "name": "greet",
          "start_line": 1,
          "end_line": 4,
          "metrics": {
            "complexity.cyclomatic": 2.0,
            "complexity.cognitive": 1.0,
            "nesting.max_depth": 1.0,
            "halstead.volume": 46.507,
            "maintainability.index": 128.116
          },
          "unavailable": {}
        }
      ]
    }
  ]
}
```

(Abridged: every metric of each scope is present. The full result also has a `project` section.)
A `null` value always has an entry in `unavailable`: `not_applicable`, `unsupported` or `error: …`.

### Experimental readability model

srcmetrics has no built-in weights. If you have human ratings, you can fit a ridge regression on
them and score other code:

```sh
# labels.csv: path,function,score   (leave function empty to label whole files)
srcmetrics model train --labels labels.csv -o model.json result.json
srcmetrics model predict --model model.json result.json
```

The model file records the features used (and those excluded, with the reason), the
coefficients, R² and 5-fold cross-validation RMSE.

### HTTP API

`srcmetrics serve [--bind 127.0.0.1] [--port 8080]` listens on localhost by default and has no
authentication.

| Endpoint | |
|---|---|
| `GET /` | Web UI: paste code and see its metrics |
| `GET /api/metrics` | Metric definitions |
| `POST /api/analyze` | `{"filename": "a.py", "source": "…"}` → analysis result (400 with `{"error": …}` for unsupported extensions or syntax errors) |

## Library usage

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

Main entry points:

| API | Purpose |
|---|---|
| `analyze::analyze(path, project)` | Analyze a file or directory into an `AnalysisResult` (serializable with serde) |
| `analyze::analyze_source(filename, source)` | Analyze source text in memory |
| `lang::adapter_for_path(path)?.to_ir(path, source)` | Parse to the common IR (`ir::File`) |
| `metrics::compute(&program)` / `metrics::definitions()` | Compute metrics from the IR / list definitions |
| `csv::to_csv`, `report::to_html` | CSV and HTML output |
| `stats::Report::of`, `model::train` | Statistics and the experimental model |

## Known limitations

- Files with syntax errors are not analyzed (partial results would be unreliable). tree-sitter
  cannot parse C/C++ code whose braces are split across preprocessor branches
  (e.g. `#ifdef __cplusplus` / `extern "C" {` / `#endif`), which is common in C headers.
- `.h` files are parsed as C.
- Dependency metrics resolve calls by name only (no type or import resolution); same-named
  functions are not distinguished.
- Code inside macro invocations (e.g. Rust `println!(…)`) is not analyzed.

Each metric's definition lists its own limitations.

## Documentation

Design documents are written in Japanese:

- [PLAN.md](PLAN.md) — requirements
- [docs/SPEC.md](docs/SPEC.md) — technical specification
- [docs/ADR.md](docs/ADR.md) — architecture decision records
- [docs/METRICS.md](docs/METRICS.md) — metric definitions (generated from the code)

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
