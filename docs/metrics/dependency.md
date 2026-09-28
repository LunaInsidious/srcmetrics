# Dependency

## Fan-out {#dependency-fan-out}

`dependency.fan_out` — Number of distinct functions a function calls.

| Item | Value |
|---|---|
| Definition | Distinct callee names of the call nodes in the function. |
| Scope | function |
| Input | Call nodes and their callee labels |
| Calculation | Excludes calls made by nested functions. Includes callees defined outside the project. Calls without a callee name (e.g. `f()()`) are not counted. |
| Unit | count |
| Language Applicability | partially_language_dependent |
| Limitations | Calls are resolved by callee name only (no types, scopes or imports). |
| Reference | Henry, S. & Kafura, D. (1981). Software Structure Metrics Based on Information Flow. IEEE TSE SE-7(5). |

## Fan-in {#dependency-fan-in}

`dependency.fan_in` — Number of distinct project functions that call a function.

| Item | Value |
|---|---|
| Definition | Distinct functions in the project having a call whose callee name is this function's name. |
| Scope | function |
| Input | Call nodes and their callee labels, function names |
| Calculation | Functions with the same name share the value. Anonymous functions have 0. |
| Unit | count |
| Language Applicability | partially_language_dependent |
| Limitations | Name-based: same-named methods of different classes are not distinguished, which overestimates fan-in. |
| Reference | Henry, S. & Kafura, D. (1981). Software Structure Metrics Based on Information Flow. IEEE TSE SE-7(5). |

## Call Depth {#dependency-call-depth}

`dependency.call_depth` — Longest chain of calls through project functions.

| Item | Value |
|---|---|
| Definition | Longest path, in edges, from the function's name in the project call graph of function names, with strongly connected components (recursion) collapsed. |
| Scope | function |
| Input | Call nodes and their callee labels, function names |
| Calculation | Nodes are the names of project functions; a name calls the union of what its functions call. Edges inside a strongly connected component are not counted. An anonymous function has 1 + the deepest name it calls. 0 when no project function is called. |
| Unit | calls |
| Language Applicability | partially_language_dependent |
| Limitations | Name-based: same-named functions share one value. |
| Reference | - |

## Dependency Count {#dependency-dependency-count}

`dependency.dependency_count` — Number of import / include declarations.

| Item | Value |
|---|---|
| Definition | Nodes of kind import. |
| Scope | file, project |
| Input | Import nodes |
| Calculation | Each imported item that the grammar represents as a separate declaration counts once (e.g. each Go import spec). Project: sum. |
| Unit | count |
| Language Applicability | partially_language_dependent |
| Limitations | Import granularity differs between languages (Python `from a import b, c` is one import). |
| Reference | - |

