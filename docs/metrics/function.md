<!-- Generated from crates/srcmetrics/src/metrics. Do not edit; run `UPDATE_DOCS=1 cargo test -p srcmetrics --test docs`. -->

# Function

## Parameter Count {#function-parameter-count}

`function.parameter_count` — Number of declared parameters.

| Item | Value |
|---|---|
| Definition | Parameters of the function in the IR. |
| Scope | function |
| Input | Function parameters |
| Calculation | Variadic parameters (e.g. `*args`) count as one. Separators such as Python `*` and `/` and C `(void)` are not parameters. |
| Unit | count |
| Language Applicability | partially_language_dependent |
| Limitations | Explicit receivers (Python `self`) count; implicit ones (`this`) do not. |
| Reference | - |

## Average Parameter Count {#function-avg-parameter-count}

`function.avg_parameter_count` — Mean Parameter Count over functions.

| Item | Value |
|---|---|
| Definition | Mean of function.parameter_count. |
| Scope | file, project |
| Input | Function parameters |
| Calculation | not_applicable when there are no functions. |
| Unit | count |
| Language Applicability | partially_language_dependent |
| Limitations | See function.parameter_count. |
| Reference | - |

## Maximum Parameter Count {#function-max-parameter-count}

`function.max_parameter_count` — Largest Parameter Count over functions.

| Item | Value |
|---|---|
| Definition | Maximum of function.parameter_count. |
| Scope | file, project |
| Input | Function parameters |
| Calculation | not_applicable when there are no functions. |
| Unit | count |
| Language Applicability | partially_language_dependent |
| Limitations | See function.parameter_count. |
| Reference | - |

## Expression Count {#function-expression-count}

`function.expression_count` — Number of expressions, including sub-expressions.

| Item | Value |
|---|---|
| Definition | Nodes of kind expression, call, assignment, binary, logical, conditional or unary. |
| Scope | function, file, project |
| Input | Node kinds |
| Calculation | Function: excluding nested functions. File: the whole file. Project: sum. Identifiers and literals are not expressions on their own. |
| Unit | count |
| Language Applicability | partially_language_dependent |
| Limitations | Expression forms not listed in a language Mapping (e.g. lambdas' bodies are nested functions; unmapped expression types are `other`) are not counted. |
| Reference | - |

## Call Count {#function-call-count}

`function.call_count` — Number of call sites.

| Item | Value |
|---|---|
| Definition | Nodes of kind call (function calls and constructor calls). |
| Scope | function, file, project |
| Input | Node kinds |
| Calculation | Function: excluding nested functions. File: the whole file. Project: sum. |
| Unit | count |
| Language Applicability | language_independent |
| Limitations | - |
| Reference | - |

