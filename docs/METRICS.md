# メトリクス定義書

このファイルは `crates/codestat/src/metrics` の定義から生成される。直接編集しないこと。
再生成: `UPDATE_DOCS=1 cargo test -p codestat --test docs`

Metric Definition Version: `0.1.0`

## `size.loc` — LOC

Physical lines of code.

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

## `size.sloc` — SLOC

Source lines of code.

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

## `size.comment_loc` — Comment LOC

Lines containing only comments.

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

## `size.blank_loc` — Blank LOC

Blank lines.

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

## `size.comment_ratio` — Comment Ratio

Share of comment lines.

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

## `size.statement_count` — Statement Count

Number of statements.

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

## `size.token_count` — Token Count

Number of non-comment tokens.

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

## `size.function_count` — Function Count

Number of functions.

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

## `size.function_length` — Function Length

Lines spanned by a function.

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

## `size.avg_function_length` — Average Function Length

Mean Function Length.

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

## `size.max_function_length` — Maximum Function Length

Longest Function Length.

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

## `complexity.cyclomatic` — Cyclomatic Complexity

Number of linearly independent paths (McCabe).

| Item | Value |
|---|---|
| Definition | 1 + number of decision points in a function. |
| Scope | function, file, project |
| Input | Node kinds: branch, loop, case, catch, logical, conditional |
| Calculation | Function: 1 + decision nodes, excluding nested functions. Each `else if` / `elif`, each short-circuit operator (&&, \|\|, and, or), each ternary and each non-default case label is one decision. File: sum over its functions + decisions in top-level code. Project: sum over files. |
| Unit | count |
| Language Applicability | partially_language_dependent |
| Limitations | Which constructs are decisions follows each language Mapping (e.g. Python comprehension `for`/`if` clauses count; Python `case _` counts as a case). |
| Reference | McCabe, T. J. (1976). A Complexity Measure. IEEE TSE SE-2(4). |

## `complexity.branch_count` — Branch Count

Number of branch nodes (if, else if, elif) plus non-default case labels.

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

## `complexity.conditional_count` — Conditional Count

Number of conditional (ternary) expressions.

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

## `complexity.loop_count` — Loop Count

Number of loops.

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

## `complexity.return_count` — Return Count

Number of return statements.

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

## `complexity.jump_count` — Jump Count

Number of jumps: break, continue, goto and throw / raise.

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

## `complexity.path_count` — Number of Paths

Acyclic execution paths through a function.

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

## `complexity.cognitive` — Cognitive Complexity

How hard a function's control flow is to understand (SonarSource).

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

## `nesting.max_depth` — Maximum Nesting Depth

Deepest nesting of control structures.

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

## `nesting.avg_depth` — Average Nesting Depth

Mean nesting level of statements.

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

## `halstead.unique_operators` — Unique Operators

n1.

| Item | Value |
|---|---|
| Definition | Number of distinct operators. |
| Scope | function, file, project |
| Input | Tokens (kind and text) |
| Calculation | Operators: keyword and operator tokens and opening brackets ( [ {. Operands: identifier and literal tokens. Commas, semicolons, closing brackets and comments are not counted. Distinct = same token text. |
| Unit | count |
| Language Applicability | partially_language_dependent |
| Limitations | Keywords, including type keywords such as `int`, are operators. A string literal is one operand. |
| Reference | Halstead, M. H. (1977). Elements of Software Science. Elsevier. |

## `halstead.unique_operands` — Unique Operands

n2.

| Item | Value |
|---|---|
| Definition | Number of distinct operands. |
| Scope | function, file, project |
| Input | Tokens (kind and text) |
| Calculation | Operators: keyword and operator tokens and opening brackets ( [ {. Operands: identifier and literal tokens. Commas, semicolons, closing brackets and comments are not counted. Distinct = same token text. |
| Unit | count |
| Language Applicability | partially_language_dependent |
| Limitations | Keywords, including type keywords such as `int`, are operators. A string literal is one operand. |
| Reference | Halstead, M. H. (1977). Elements of Software Science. Elsevier. |

## `halstead.total_operators` — Total Operators

N1.

| Item | Value |
|---|---|
| Definition | Number of operator occurrences. |
| Scope | function, file, project |
| Input | Tokens (kind and text) |
| Calculation | Operators: keyword and operator tokens and opening brackets ( [ {. Operands: identifier and literal tokens. Commas, semicolons, closing brackets and comments are not counted. Distinct = same token text. |
| Unit | count |
| Language Applicability | partially_language_dependent |
| Limitations | Keywords, including type keywords such as `int`, are operators. A string literal is one operand. |
| Reference | Halstead, M. H. (1977). Elements of Software Science. Elsevier. |

## `halstead.total_operands` — Total Operands

N2.

| Item | Value |
|---|---|
| Definition | Number of operand occurrences. |
| Scope | function, file, project |
| Input | Tokens (kind and text) |
| Calculation | Operators: keyword and operator tokens and opening brackets ( [ {. Operands: identifier and literal tokens. Commas, semicolons, closing brackets and comments are not counted. Distinct = same token text. |
| Unit | count |
| Language Applicability | partially_language_dependent |
| Limitations | Keywords, including type keywords such as `int`, are operators. A string literal is one operand. |
| Reference | Halstead, M. H. (1977). Elements of Software Science. Elsevier. |

## `halstead.vocabulary` — Vocabulary

n.

| Item | Value |
|---|---|
| Definition | n = n1 + n2. |
| Scope | function, file, project |
| Input | Tokens (kind and text) |
| Calculation | Operators: keyword and operator tokens and opening brackets ( [ {. Operands: identifier and literal tokens. Commas, semicolons, closing brackets and comments are not counted. Distinct = same token text. |
| Unit | count |
| Language Applicability | partially_language_dependent |
| Limitations | Keywords, including type keywords such as `int`, are operators. A string literal is one operand. |
| Reference | Halstead, M. H. (1977). Elements of Software Science. Elsevier. |

## `halstead.length` — Program Length

N.

| Item | Value |
|---|---|
| Definition | N = N1 + N2. |
| Scope | function, file, project |
| Input | Tokens (kind and text) |
| Calculation | Operators: keyword and operator tokens and opening brackets ( [ {. Operands: identifier and literal tokens. Commas, semicolons, closing brackets and comments are not counted. Distinct = same token text. |
| Unit | count |
| Language Applicability | partially_language_dependent |
| Limitations | Keywords, including type keywords such as `int`, are operators. A string literal is one operand. |
| Reference | Halstead, M. H. (1977). Elements of Software Science. Elsevier. |

## `halstead.volume` — Volume

V.

| Item | Value |
|---|---|
| Definition | V = N * log2(n); not_applicable when n = 0. |
| Scope | function, file, project |
| Input | Tokens (kind and text) |
| Calculation | Operators: keyword and operator tokens and opening brackets ( [ {. Operands: identifier and literal tokens. Commas, semicolons, closing brackets and comments are not counted. Distinct = same token text. |
| Unit | bits |
| Language Applicability | partially_language_dependent |
| Limitations | Keywords, including type keywords such as `int`, are operators. A string literal is one operand. |
| Reference | Halstead, M. H. (1977). Elements of Software Science. Elsevier. |

## `halstead.difficulty` — Difficulty

D.

| Item | Value |
|---|---|
| Definition | D = (n1 / 2) * (N2 / n2); not_applicable when n2 = 0. |
| Scope | function, file, project |
| Input | Tokens (kind and text) |
| Calculation | Operators: keyword and operator tokens and opening brackets ( [ {. Operands: identifier and literal tokens. Commas, semicolons, closing brackets and comments are not counted. Distinct = same token text. |
| Unit | ratio |
| Language Applicability | partially_language_dependent |
| Limitations | Keywords, including type keywords such as `int`, are operators. A string literal is one operand. |
| Reference | Halstead, M. H. (1977). Elements of Software Science. Elsevier. |

## `halstead.effort` — Effort

E.

| Item | Value |
|---|---|
| Definition | E = D * V. |
| Scope | function, file, project |
| Input | Tokens (kind and text) |
| Calculation | Operators: keyword and operator tokens and opening brackets ( [ {. Operands: identifier and literal tokens. Commas, semicolons, closing brackets and comments are not counted. Distinct = same token text. |
| Unit | count |
| Language Applicability | partially_language_dependent |
| Limitations | Keywords, including type keywords such as `int`, are operators. A string literal is one operand. |
| Reference | Halstead, M. H. (1977). Elements of Software Science. Elsevier. |

## `halstead.time` — Estimated Program Time

T.

| Item | Value |
|---|---|
| Definition | T = E / 18. |
| Scope | function, file, project |
| Input | Tokens (kind and text) |
| Calculation | Operators: keyword and operator tokens and opening brackets ( [ {. Operands: identifier and literal tokens. Commas, semicolons, closing brackets and comments are not counted. Distinct = same token text. |
| Unit | seconds |
| Language Applicability | partially_language_dependent |
| Limitations | Keywords, including type keywords such as `int`, are operators. A string literal is one operand. |
| Reference | Halstead, M. H. (1977). Elements of Software Science. Elsevier. |

## `halstead.bugs` — Estimated Bugs

B.

| Item | Value |
|---|---|
| Definition | B = V / 3000. |
| Scope | function, file, project |
| Input | Tokens (kind and text) |
| Calculation | Operators: keyword and operator tokens and opening brackets ( [ {. Operands: identifier and literal tokens. Commas, semicolons, closing brackets and comments are not counted. Distinct = same token text. |
| Unit | count |
| Language Applicability | partially_language_dependent |
| Limitations | Keywords, including type keywords such as `int`, are operators. A string literal is one operand. |
| Reference | Halstead, M. H. (1977). Elements of Software Science. Elsevier. |

## `function.parameter_count` — Parameter Count

Number of declared parameters.

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

## `function.avg_parameter_count` — Average Parameter Count

Mean Parameter Count over functions.

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

## `function.max_parameter_count` — Maximum Parameter Count

Largest Parameter Count over functions.

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

## `function.expression_count` — Expression Count

Number of expressions, including sub-expressions.

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

## `function.call_count` — Call Count

Number of call sites.

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

## `duplication.duplicate_block_count` — Duplicate Block Count

Number of duplicated token runs.

| Item | Value |
|---|---|
| Definition | Number of duplicated token runs. |
| Scope | file, project |
| Input | Tokens (kind and text) |
| Calculation | Tokens are normalized (identifier -> $id, literal -> $lit, comments removed). Every window of 50 consecutive normalized tokens that occurs at least twice marks its tokens as duplicated; a maximal run of duplicated tokens is one block. File: clones within the file only. Project: clones across all files. |
| Unit | count |
| Language Applicability | language_independent |
| Limitations | Repetitive token sequences (e.g. long array literals) are reported as duplicates. |
| Reference | - |

## `duplication.duplicate_token_count` — Duplicate Token Count

Number of duplicated tokens.

| Item | Value |
|---|---|
| Definition | Number of duplicated tokens. |
| Scope | file, project |
| Input | Tokens (kind and text) |
| Calculation | Tokens are normalized (identifier -> $id, literal -> $lit, comments removed). Every window of 50 consecutive normalized tokens that occurs at least twice marks its tokens as duplicated; a maximal run of duplicated tokens is one block. File: clones within the file only. Project: clones across all files. |
| Unit | count |
| Language Applicability | language_independent |
| Limitations | Repetitive token sequences (e.g. long array literals) are reported as duplicates. |
| Reference | - |

## `duplication.duplication_ratio` — Duplication Ratio

Duplicate Token Count / non-comment tokens; not_applicable when there are no tokens.

| Item | Value |
|---|---|
| Definition | Duplicate Token Count / non-comment tokens; not_applicable when there are no tokens. |
| Scope | file, project |
| Input | Tokens (kind and text) |
| Calculation | Tokens are normalized (identifier -> $id, literal -> $lit, comments removed). Every window of 50 consecutive normalized tokens that occurs at least twice marks its tokens as duplicated; a maximal run of duplicated tokens is one block. File: clones within the file only. Project: clones across all files. |
| Unit | ratio |
| Language Applicability | language_independent |
| Limitations | Repetitive token sequences (e.g. long array literals) are reported as duplicates. |
| Reference | - |

## `duplication.max_duplicate_length` — Maximum Duplicate Length

Longest duplicated token run; 0 when there is none.

| Item | Value |
|---|---|
| Definition | Longest duplicated token run; 0 when there is none. |
| Scope | file, project |
| Input | Tokens (kind and text) |
| Calculation | Tokens are normalized (identifier -> $id, literal -> $lit, comments removed). Every window of 50 consecutive normalized tokens that occurs at least twice marks its tokens as duplicated; a maximal run of duplicated tokens is one block. File: clones within the file only. Project: clones across all files. |
| Unit | tokens |
| Language Applicability | language_independent |
| Limitations | Repetitive token sequences (e.g. long array literals) are reported as duplicates. |
| Reference | - |

## `dependency.fan_out` — Fan-out

Number of distinct functions a function calls.

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

## `dependency.fan_in` — Fan-in

Number of distinct project functions that call a function.

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

## `dependency.call_depth` — Call Depth

Longest chain of calls through project functions.

| Item | Value |
|---|---|
| Definition | Longest path, in edges, from the function in the project call graph with strongly connected components (recursion) collapsed. |
| Scope | function |
| Input | Call nodes and their callee labels, function names |
| Calculation | Edges go from a function to every project function named like a callee. Edges inside a strongly connected component are not counted. 0 when the function calls no project function. |
| Unit | calls |
| Language Applicability | partially_language_dependent |
| Limitations | Calls are resolved by callee name only (no types, scopes or imports). |
| Reference | - |

## `dependency.dependency_count` — Dependency Count

Number of import / include declarations.

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

## `documentation.doc_loc` — Documentation LOC

Lines of a function's documentation.

| Item | Value |
|---|---|
| Definition | A function is documented when a comment block directly precedes it (above its decorators / attributes, with no blank line in between and not a trailing comment), or, where the language has docstrings, when its body starts with one. |
| Scope | function |
| Input | Function documentation range |
| Calculation | Lines spanned by the documentation; 0 when the function is undocumented. |
| Unit | lines |
| Language Applicability | partially_language_dependent |
| Limitations | Any comment style counts (not only `/**` or `///`). |
| Reference | - |

## `documentation.documented_function_count` — Documentation Count

Number of documented functions.

| Item | Value |
|---|---|
| Definition | A function is documented when a comment block directly precedes it (above its decorators / attributes, with no blank line in between and not a trailing comment), or, where the language has docstrings, when its body starts with one. |
| Scope | file, project |
| Input | Function documentation range |
| Calculation | Functions with documentation. Project: sum. |
| Unit | count |
| Language Applicability | partially_language_dependent |
| Limitations | Any comment style counts (not only `/**` or `///`). |
| Reference | - |

## `documentation.documentation_ratio` — Documentation Ratio

Share of documented functions.

| Item | Value |
|---|---|
| Definition | Documentation Count / Function Count. |
| Scope | file, project |
| Input | Function documentation range |
| Calculation | not_applicable when there are no functions. |
| Unit | ratio |
| Language Applicability | partially_language_dependent |
| Limitations | Any comment style counts (not only `/**` or `///`). |
| Reference | - |

## `maintainability.index` — Maintainability Index

Composite maintainability estimate (original, unbounded formula).

| Item | Value |
|---|---|
| Definition | MI = 171 - 5.2 * ln(V) - 0.23 * CC - 16.2 * ln(SLOC). |
| Scope | function, file |
| Input | halstead.volume (V), complexity.cyclomatic (CC), size.sloc (SLOC) |
| Calculation | not_applicable unless V > 0 and SLOC > 0. Not rescaled to 0-100. |
| Unit | index |
| Language Applicability | partially_language_dependent |
| Limitations | Inherits the limitations of its inputs; the coefficients were fitted on 1990s code. |
| Reference | Oman, P. & Hagemeister, J. (1992). Metrics for assessing a software system's maintainability. ICSM. |

## `derived.cyclomatic_per_function` — Cyclomatic Complexity per Function

complexity.cyclomatic / size.function_count.

| Item | Value |
|---|---|
| Definition | complexity.cyclomatic / size.function_count. |
| Scope | file, project |
| Input | complexity.cyclomatic, size.function_count |
| Calculation | not_applicable when the denominator is 0 or an input is unavailable. |
| Unit | ratio |
| Language Applicability | partially_language_dependent |
| Limitations | A normalization: its choice of denominator affects comparisons (PLAN.md §16). |
| Reference | - |

## `derived.tokens_per_loc` — Tokens per LOC

size.token_count / size.loc.

| Item | Value |
|---|---|
| Definition | size.token_count / size.loc. |
| Scope | file, project |
| Input | size.token_count, size.loc |
| Calculation | not_applicable when the denominator is 0 or an input is unavailable. |
| Unit | ratio |
| Language Applicability | partially_language_dependent |
| Limitations | A normalization: its choice of denominator affects comparisons (PLAN.md §16). |
| Reference | - |

## `derived.statements_per_function` — Statements per Function

size.statement_count / size.function_count.

| Item | Value |
|---|---|
| Definition | size.statement_count / size.function_count. |
| Scope | file, project |
| Input | size.statement_count, size.function_count |
| Calculation | not_applicable when the denominator is 0 or an input is unavailable. |
| Unit | ratio |
| Language Applicability | partially_language_dependent |
| Limitations | A normalization: its choice of denominator affects comparisons (PLAN.md §16). |
| Reference | - |

## `derived.duplicate_tokens_per_sloc` — Duplicate Tokens per SLOC

duplication.duplicate_token_count / size.sloc.

| Item | Value |
|---|---|
| Definition | duplication.duplicate_token_count / size.sloc. |
| Scope | file, project |
| Input | duplication.duplicate_token_count, size.sloc |
| Calculation | not_applicable when the denominator is 0 or an input is unavailable. |
| Unit | ratio |
| Language Applicability | partially_language_dependent |
| Limitations | A normalization: its choice of denominator affects comparisons (PLAN.md §16). |
| Reference | - |

