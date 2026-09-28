# Complexity

## Cyclomatic Complexity {#complexity-cyclomatic}

`complexity.cyclomatic` — Number of linearly independent paths (McCabe).

| Item | Value |
|---|---|
| Definition | 1 + number of decision points in a function. |
| Scope | function, file, project |
| Input | Node kinds: branch, loop, case, catch, logical, conditional |
| Calculation | Function: 1 + decision nodes, excluding nested functions. Each `else if` / `elif`, each short-circuit operator (&&, \|\|, and, or), each ternary and each non-default case label is one decision. File: sum over its functions + decisions in top-level code. Project: sum over files. |
| Unit | count |
| Language Applicability | partially_language_dependent |
| Limitations | Which constructs are decisions follows each language Mapping (e.g. Python comprehension `for`/`if` clauses count; Python `case _:` and Rust `_ =>` count as cases). |
| Reference | McCabe, T. J. (1976). A Complexity Measure. IEEE TSE SE-2(4). |

## Branch Count {#complexity-branch-count}

`complexity.branch_count` — Number of branch nodes (if, else if, elif) plus non-default case labels.

| Item | Value |
|---|---|
| Definition | Number of branch nodes (if, else if, elif) plus non-default case labels. |
| Scope | function, file, project |
| Input | Node kinds |
| Calculation | Function: excluding nested functions. File: the whole file. Project: sum over files. |
| Unit | count |
| Language Applicability | partially_language_dependent |
| Limitations | Which constructs map to each node kind follows each language Mapping. |
| Reference | - |

## Conditional Count {#complexity-conditional-count}

`complexity.conditional_count` — Number of conditional (ternary) expressions.

| Item | Value |
|---|---|
| Definition | Number of conditional (ternary) expressions. |
| Scope | function, file, project |
| Input | Node kinds |
| Calculation | Function: excluding nested functions. File: the whole file. Project: sum over files. |
| Unit | count |
| Language Applicability | partially_language_dependent |
| Limitations | Which constructs map to each node kind follows each language Mapping. |
| Reference | - |

## Loop Count {#complexity-loop-count}

`complexity.loop_count` — Number of loops.

| Item | Value |
|---|---|
| Definition | Number of loops. |
| Scope | function, file, project |
| Input | Node kinds |
| Calculation | Function: excluding nested functions. File: the whole file. Project: sum over files. |
| Unit | count |
| Language Applicability | partially_language_dependent |
| Limitations | Which constructs map to each node kind follows each language Mapping. |
| Reference | - |

## Return Count {#complexity-return-count}

`complexity.return_count` — Number of return statements.

| Item | Value |
|---|---|
| Definition | Number of return statements. |
| Scope | function, file, project |
| Input | Node kinds |
| Calculation | Function: excluding nested functions. File: the whole file. Project: sum over files. |
| Unit | count |
| Language Applicability | partially_language_dependent |
| Limitations | Which constructs map to each node kind follows each language Mapping. |
| Reference | - |

## Jump Count {#complexity-jump-count}

`complexity.jump_count` — Number of jumps: break, continue, goto and throw / raise.

| Item | Value |
|---|---|
| Definition | Number of jumps: break, continue, goto and throw / raise. |
| Scope | function, file, project |
| Input | Node kinds |
| Calculation | Function: excluding nested functions. File: the whole file. Project: sum over files. |
| Unit | count |
| Language Applicability | partially_language_dependent |
| Limitations | Which constructs map to each node kind follows each language Mapping. |
| Reference | - |

## Number of Paths {#complexity-path-count}

`complexity.path_count` — Acyclic execution paths through a function.

| Item | Value |
|---|---|
| Definition | Number of paths through the function when each loop runs zero times or once. |
| Scope | function |
| Input | Node kinds and tree structure |
| Calculation | Children in sequence multiply. An if-chain is the sum of its arms, +1 without a final else. A loop or ternary is its children's product + 1. Consecutive case labels or catch clauses are the sum of their paths + 1. Nested functions count as 1. |
| Unit | count |
| Language Applicability | partially_language_dependent |
| Limitations | Not Nejmeh's NPATH: short-circuit operators and early exits (return, jump) do not change the count. |
| Reference | Nejmeh, B. A. (1988). NPATH: a measure of execution path complexity. CACM 31(2) (related, not identical). |

## Cognitive Complexity {#complexity-cognitive}

`complexity.cognitive` — How hard a function's control flow is to understand (SonarSource).

| Item | Value |
|---|---|
| Definition | Sum of increments for breaks in linear flow, weighted by nesting. |
| Scope | function, file, project |
| Input | Node kinds, parent links, call and logical labels |
| Calculation | if-chain head, loop, catch, ternary, and each run of case labels (a switch): 1 + nesting level. else if / elif and else: 1. Each sequence of like logical operators: 1. A call to the function's own name (recursion): 1. Nesting levels are opened by branches, loops, cases, catches and ternaries. File: sum over functions. Project: sum over files. |
| Unit | count |
| Language Applicability | partially_language_dependent |
| Limitations | Nested functions (lambdas) are measured separately instead of adding to the enclosing function. Labelled break / continue and goto add nothing (jumps have no labels in the IR). |
| Reference | Campbell, G. A. (2018). Cognitive Complexity: A new way of measuring understandability. SonarSource. |

