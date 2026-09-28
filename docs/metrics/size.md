# Size

## LOC {#size-loc}

`size.loc` — Physical lines of code.

| Item | Value |
|---|---|
| Definition | Number of lines in the file. |
| Scope | file, project |
| Input | File source text |
| Calculation | Count of lines; a trailing newline does not start a new line. Project: sum over files. |
| Unit | lines |
| Language Applicability | language_independent |
| Limitations | - |
| Reference | - |

## SLOC {#size-sloc}

`size.sloc` — Source lines of code.

| Item | Value |
|---|---|
| Definition | Lines occupied by at least one non-comment token. |
| Scope | function, file, project |
| Input | File source text, Token ranges |
| Calculation | A token spanning several lines (e.g. a multi-line string) occupies each of them. Function: lines of the function's range (including nested functions). Project: sum. |
| Unit | lines |
| Language Applicability | language_independent |
| Limitations | - |
| Reference | - |

## Comment LOC {#size-comment-loc}

`size.comment_loc` — Lines containing only comments.

| Item | Value |
|---|---|
| Definition | Lines occupied by a comment token and by no other token. |
| Scope | file, project |
| Input | File source text, Token ranges |
| Calculation | Lines with code and a trailing comment are SLOC, not Comment LOC. Project: sum. |
| Unit | lines |
| Language Applicability | language_independent |
| Limitations | Documentation strings that are string literals (e.g. Python docstrings) count as SLOC. |
| Reference | - |

## Blank LOC {#size-blank-loc}

`size.blank_loc` — Blank lines.

| Item | Value |
|---|---|
| Definition | Whitespace-only lines not occupied by any token. |
| Scope | file, project |
| Input | File source text, Token ranges |
| Calculation | Blank lines inside a multi-line comment or string are not blank. Project: sum. |
| Unit | lines |
| Language Applicability | language_independent |
| Limitations | - |
| Reference | - |

## Comment Ratio {#size-comment-ratio}

`size.comment_ratio` — Share of comment lines.

| Item | Value |
|---|---|
| Definition | Comment LOC / LOC. |
| Scope | file, project |
| Input | size.comment_loc, size.loc |
| Calculation | not_applicable when LOC is 0. Project: ratio of the sums. |
| Unit | ratio |
| Language Applicability | language_independent |
| Limitations | - |
| Reference | - |

## Statement Count {#size-statement-count}

`size.statement_count` — Number of statements.

| Item | Value |
|---|---|
| Definition | Nodes of kind statement, declaration, branch, loop, return or jump. |
| Scope | function, file, project |
| Input | Node kinds |
| Calculation | Function scope includes nested functions. Project: sum. |
| Unit | count |
| Language Applicability | partially_language_dependent |
| Limitations | Languages differ in what is a statement (e.g. C for-loop initializer declarations count; Python has no equivalent). |
| Reference | - |

## Token Count {#size-token-count}

`size.token_count` — Number of non-comment tokens.

| Item | Value |
|---|---|
| Definition | Tokens of every kind except comment. |
| Scope | function, file, project |
| Input | Tokens |
| Calculation | A string literal is one token. Function scope includes nested functions. Project: sum. |
| Unit | count |
| Language Applicability | partially_language_dependent |
| Limitations | Token granularity follows each grammar (e.g. C `#include` is one token). |
| Reference | - |

## Function Count {#size-function-count}

`size.function_count` — Number of functions.

| Item | Value |
|---|---|
| Definition | Functions in the IR, including methods, nested and anonymous functions. |
| Scope | file, project |
| Input | Functions |
| Calculation | Project: sum. |
| Unit | count |
| Language Applicability | language_independent |
| Limitations | - |
| Reference | - |

## Function Length {#size-function-length}

`size.function_length` — Lines spanned by a function.

| Item | Value |
|---|---|
| Definition | Last line - first line + 1 of the function's source range. |
| Scope | function |
| Input | Function source range |
| Calculation | Includes the signature, blank and comment lines, and nested functions. |
| Unit | lines |
| Language Applicability | language_independent |
| Limitations | - |
| Reference | - |

## Average Function Length {#size-avg-function-length}

`size.avg_function_length` — Mean Function Length.

| Item | Value |
|---|---|
| Definition | Mean of size.function_length over all functions. |
| Scope | file, project |
| Input | size.function_length |
| Calculation | not_applicable when there are no functions. |
| Unit | lines |
| Language Applicability | language_independent |
| Limitations | - |
| Reference | - |

## Maximum Function Length {#size-max-function-length}

`size.max_function_length` — Longest Function Length.

| Item | Value |
|---|---|
| Definition | Maximum of size.function_length over all functions. |
| Scope | file, project |
| Input | size.function_length |
| Calculation | not_applicable when there are no functions. |
| Unit | lines |
| Language Applicability | language_independent |
| Limitations | - |
| Reference | - |

