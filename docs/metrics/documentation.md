# Documentation

## Documentation LOC {#documentation-doc-loc}

`documentation.doc_loc` — Lines of a function's documentation.

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

## Documentation Count {#documentation-documented-function-count}

`documentation.documented_function_count` — Number of documented functions.

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

## Documentation Ratio {#documentation-documentation-ratio}

`documentation.documentation_ratio` — Share of documented functions.

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

