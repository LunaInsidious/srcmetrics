<!-- Generated from crates/srcmetrics/src/metrics. Do not edit; run `UPDATE_DOCS=1 cargo test -p srcmetrics --test docs`. -->

# Derived

## Cyclomatic Complexity per Function {#derived-cyclomatic-per-function}

`derived.cyclomatic_per_function` — complexity.cyclomatic / size.function_count.

| Item | Value |
|---|---|
| Definition | complexity.cyclomatic / size.function_count. |
| Scope | file, project |
| Input | complexity.cyclomatic, size.function_count |
| Calculation | not_applicable when the denominator is 0 or an input is unavailable. |
| Unit | ratio |
| Language Applicability | partially_language_dependent |
| Limitations | A normalization: its choice of denominator affects comparisons. |
| Reference | - |

## Tokens per LOC {#derived-tokens-per-loc}

`derived.tokens_per_loc` — size.token_count / size.loc.

| Item | Value |
|---|---|
| Definition | size.token_count / size.loc. |
| Scope | file, project |
| Input | size.token_count, size.loc |
| Calculation | not_applicable when the denominator is 0 or an input is unavailable. |
| Unit | ratio |
| Language Applicability | partially_language_dependent |
| Limitations | A normalization: its choice of denominator affects comparisons. |
| Reference | - |

## Statements per Function {#derived-statements-per-function}

`derived.statements_per_function` — size.statement_count / size.function_count.

| Item | Value |
|---|---|
| Definition | size.statement_count / size.function_count. |
| Scope | file, project |
| Input | size.statement_count, size.function_count |
| Calculation | not_applicable when the denominator is 0 or an input is unavailable. |
| Unit | ratio |
| Language Applicability | partially_language_dependent |
| Limitations | A normalization: its choice of denominator affects comparisons. |
| Reference | - |

## Duplicate Tokens per SLOC {#derived-duplicate-tokens-per-sloc}

`derived.duplicate_tokens_per_sloc` — duplication.duplicate_token_count / size.sloc.

| Item | Value |
|---|---|
| Definition | duplication.duplicate_token_count / size.sloc. |
| Scope | file, project |
| Input | duplication.duplicate_token_count, size.sloc |
| Calculation | not_applicable when the denominator is 0 or an input is unavailable. |
| Unit | ratio |
| Language Applicability | partially_language_dependent |
| Limitations | A normalization: its choice of denominator affects comparisons. |
| Reference | - |

