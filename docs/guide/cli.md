# Command-line reference

```text
srcmetrics <COMMAND>

  analyze   Analyze a file or directory
  stats     Descriptive statistics, per-language baselines and correlations
  report    Self-contained HTML report
  model     Experimental readability model (train / predict)
  serve     HTTP API and web UI
```

Errors exit with status 1 and a message saying what to do next.

## analyze

```sh
srcmetrics analyze <PATH> [--project NAME] [-o FILE] [--format json|csv]
```

- `PATH` is a file or a directory. Directories are walked respecting `.gitignore`; only files with
  [supported extensions](./languages) are analyzed. A file given explicitly with an unsupported
  extension is an error.
- `--project` sets the project name recorded in the result (default: the directory name).
- `--format csv` writes one row per project, file and function. See [Output format](./output).
- Files that fail to parse are kept in the result with `"status": "error"`, and their count and
  errors are printed on stderr. The exit status is still 0.

## stats

```sh
srcmetrics stats <RESULT.json>... [--scope file|function]
```

Computes, over the files or functions of one or more results:

- per metric: `n` (units with a value), `missing`, `mean`, `sd` (sample), `min`, `q1`, `median`,
  `q3`, `max`. Quantiles use linear interpolation (R type 7).
- the same per language (`by_language`), as baselines.
- Pearson and Spearman correlations for every pair of metrics, using the units where both values
  are present (`n` is reported). Fewer than 3 units or zero variance gives `null`.

## report

```sh
srcmetrics report <RESULT.json> [-o FILE]
```

Writes a single HTML file with the run metadata, project metrics, per-language medians,
histograms of every metric and a Spearman correlation heatmap. It loads no external resources, so
it works offline and can be shared as one file.

## model

```sh
srcmetrics model train --labels LABELS.csv [--features IDS] [--lambda L] [-o MODEL.json] <RESULT.json>...
srcmetrics model predict --model MODEL.json <RESULT.json>...
```

See [Experimental readability model](./model).

## serve

```sh
srcmetrics serve [--bind 127.0.0.1] [--port 8080]
```

See [HTTP API](./http-api).

