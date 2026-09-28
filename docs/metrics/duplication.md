<!-- Generated from crates/srcmetrics/src/metrics. Do not edit; run `UPDATE_DOCS=1 cargo test -p srcmetrics --test docs`. -->

# Duplication

## Duplicate Block Count {#duplication-duplicate-block-count}

`duplication.duplicate_block_count` — Number of duplicated token runs.

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

## Duplicate Token Count {#duplication-duplicate-token-count}

`duplication.duplicate_token_count` — Number of duplicated tokens.

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

## Duplication Ratio {#duplication-duplication-ratio}

`duplication.duplication_ratio` — Duplicate Token Count / non-comment tokens; not_applicable when there are no tokens.

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

## Maximum Duplicate Length {#duplication-max-duplicate-length}

`duplication.max_duplicate_length` — Longest duplicated token run; 0 when there is none.

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

