# Maintainability

## Maintainability Index {#maintainability-index}

`maintainability.index` — Composite maintainability estimate (original, unbounded formula).

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

