# Getting started

## Install

The command-line tool:

```sh
cargo install srcmetrics-cli   # installs the `srcmetrics` command
```

The Rust library:

```sh
cargo add srcmetrics
```

Rust 1.85 or later is required. The grammars are compiled from C, so a C compiler is needed at build
time. Analysis runs fully offline.

## Analyze a project

```sh
srcmetrics analyze src -o result.json
```

`analyze` walks the directory (respecting `.gitignore`), analyzes every file with a
[supported extension](./languages), and writes one JSON document with metrics for the project,
each file and each function. See [Output format](./output) for its structure.

Files that fail to parse are kept in the result with `"status": "error"` and listed on stderr.

## Look at the results

```sh
srcmetrics report result.json -o report.html        # tables, histograms, correlation heatmap
srcmetrics stats result.json --scope function       # statistics as JSON
srcmetrics analyze src --format csv > metrics.csv   # one row per project / file / function
```

The meaning of every metric is in the [metric reference](/metrics/).

## Next steps

- [Command-line reference](./cli)
- [Use it as a library](./library)
- [Supported languages and limitations](./languages)
