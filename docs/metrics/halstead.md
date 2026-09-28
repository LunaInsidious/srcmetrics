# Halstead

## Unique Operators {#halstead-unique-operators}

`halstead.unique_operators` — n1.

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

## Unique Operands {#halstead-unique-operands}

`halstead.unique_operands` — n2.

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

## Total Operators {#halstead-total-operators}

`halstead.total_operators` — N1.

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

## Total Operands {#halstead-total-operands}

`halstead.total_operands` — N2.

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

## Vocabulary {#halstead-vocabulary}

`halstead.vocabulary` — n.

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

## Program Length {#halstead-length}

`halstead.length` — N.

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

## Volume {#halstead-volume}

`halstead.volume` — V.

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

## Difficulty {#halstead-difficulty}

`halstead.difficulty` — D.

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

## Effort {#halstead-effort}

`halstead.effort` — E.

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

## Estimated Program Time {#halstead-time}

`halstead.time` — T.

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

## Estimated Bugs {#halstead-bugs}

`halstead.bugs` — B.

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

