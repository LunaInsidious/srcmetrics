# Experimental readability model

srcmetrics has **no built-in weights** and does not compute a readability score by default: how
each metric relates to human-perceived readability is not known in advance. If you have human
ratings, you can fit a model on them.

## Labels

A CSV file with the header `path,function,score`:

```text
path,function,score
src/parser.py,parse_header,3.5
src/parser.py,parse_body,2
src/util.py,,4
```

- `path` is the path as it appears in the analysis result.
- `function` is the function name; leave it empty to rate a whole file. All labels of one model
  must be of the same kind.
- Each label must match exactly one file or function. Unknown paths, files that failed to parse and
  function names that are not unique in their file are errors.
- `score` must be a finite number, and not all scores may be equal.

## Train

```sh
srcmetrics analyze src -o result.json
srcmetrics model train --labels labels.csv -o model.json result.json
```

The model is a ridge regression on standardized features:

- Features default to every metric of the labels' scope. Metrics that are `null` for some labelled
  unit, or constant over them, are excluded and listed with the reason in the model file.
  `--features id1,id2` selects features explicitly; each must then be usable.
- `--lambda` sets the regularization strength (default 1.0).
- The model file records the features, coefficients, R² on the training data and the RMSE of
  5-fold cross-validation (with at least 5 labels).

## Predict

```sh
srcmetrics model predict --model model.json result.json
```

Prints a score for every file or function (matching the model's scope). A unit with a `null`
feature, or a file that failed to parse, gets `score: null` and the reason.
