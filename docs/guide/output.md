# Output format

## JSON

```json
{
  "run": {
    "project": "readme",
    "repository": "git@github.com:you/project.git",
    "commit": "3f2c…",
    "tool_version": "0.1.0",
    "metric_definition_version": "0.1.0",
    "parsers": { "python": "tree-sitter 0.27.0 / tree-sitter-python 0.25.0" },
    "timestamp": "2026-09-29T00:00:00Z"
  },
  "project": { "metrics": { "size.loc": 4.0 }, "unavailable": {} },
  "files": [
    {
      "status": "ok",
      "path": "greet.py",
      "language": "python",
      "metrics": { "size.loc": 4.0, "size.comment_ratio": 0.0 },
      "unavailable": {},
      "functions": [
        {
          "name": "greet",
          "start_line": 1,
          "end_line": 4,
          "metrics": { "complexity.cyclomatic": 2.0, "nesting.max_depth": 1.0 },
          "unavailable": {}
        }
      ]
    },
    {
      "status": "error",
      "path": "broken.c",
      "language": "c",
      "error": "broken.c:1:12: parse error: …"
    }
  ]
}
```

(Abridged: `metrics` contains every metric defined for that scope.)

### `run`

| Field | |
|---|---|
| `project` | `--project`, or the name of the analyzed directory |
| `repository`, `commit` | The git `origin` URL and `HEAD` of the analyzed path; `null` outside a git repository |
| `tool_version` | srcmetrics version |
| `metric_definition_version` | Version of the metric definitions; changes when a calculation changes |
| `parsers` | Parser and grammar version per language present in the result |
| `timestamp` | RFC 3339, UTC |

### Metric values and `null`

`metrics` maps [metric ids](/metrics/) to numbers or `null`. Every `null` has an entry in
`unavailable` saying why:

| Reason | Meaning |
|---|---|
| `not_applicable` | The metric does not apply, e.g. the average function length of a file without functions |
| `unsupported` | The metric is not available for this input |
| `error: …` | The value could not be computed; the message says why |

A value that cannot be computed is never reported as 0.

### Files and functions

- Files are sorted by path. `status` is `ok` or `error`; failed files have `error` instead of
  metrics and are not included in project totals.
- Functions include methods, nested functions and lambdas. `name` is `null` for anonymous
  functions. Lines are 1-based and inclusive.

## CSV

`srcmetrics analyze --format csv` writes one row per scope:

```text
scope,path,language,function,start_line,end_line,status,error,<metric ids…>
project,,,,,,,,…
file,src/a.py,python,,,,ok,,…
function,src/a.py,python,greet,1,4,ok,,…
file,src/broken.c,c,,,,error,"src/broken.c:1:12: parse error: …",…
```

Metric columns follow the order of `srcmetrics metrics`. `null` values are empty cells; their
reasons are only in the JSON output.
