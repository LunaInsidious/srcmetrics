<!-- Generated from crates/srcmetrics/src/metrics. Do not edit; run `UPDATE_DOCS=1 cargo test -p srcmetrics --test docs`. -->

# Nesting

## Maximum Nesting Depth {#nesting-max-depth}

`nesting.max_depth` — Deepest nesting of control structures.

| Item | Value |
|---|---|
| Definition | Maximum over control structures (branch, loop, case, catch) of 1 + the number of control structures enclosing it. |
| Scope | function, file, project |
| Input | Node kinds and parent links |
| Calculation | `else if` / `elif` continue their if-chain and do not add a level. Nesting restarts at function boundaries. 0 when there is no control structure. File / Project: maximum. |
| Unit | levels |
| Language Applicability | language_independent |
| Limitations | A branch directly inside a branch without a block (C `if (a) if (b) x;`) is treated as an if-chain continuation. |
| Reference | - |

## Average Nesting Depth {#nesting-avg-depth}

`nesting.avg_depth` — Mean nesting level of statements.

| Item | Value |
|---|---|
| Definition | Mean over statements of the number of control structures enclosing the statement. |
| Scope | function, file, project |
| Input | Node kinds and parent links |
| Calculation | Statements as in size.statement_count. not_applicable when there are no statements. File / Project: mean over all their statements. |
| Unit | levels |
| Language Applicability | language_independent |
| Limitations | Inherits the statement differences of size.statement_count. |
| Reference | - |

