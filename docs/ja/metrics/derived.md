<!-- Generated from crates/srcmetrics/src/metrics. Do not edit; run `UPDATE_DOCS=1 cargo test -p srcmetrics --test docs`. -->

# 派生メトリクス

## 関数あたりの Cyclomatic Complexity {#derived-cyclomatic-per-function}

`derived.cyclomatic_per_function` — complexity.cyclomatic / size.function_count.

| 項目 | 内容 |
|---|---|
| 定義 | complexity.cyclomatic / size.function_count. |
| スコープ | file, project |
| 入力 | complexity.cyclomatic, size.function_count |
| 計算方法 | 分母が 0 か、入力が得られない場合は not_applicable。 |
| 単位 | ratio |
| 言語依存性 | 部分的に言語に依存する |
| 制約 | 正規化の一種で、分母の選び方が比較の結果に影響する。 |
| 参考文献 | - |

## LOC あたりのトークン数 {#derived-tokens-per-loc}

`derived.tokens_per_loc` — size.token_count / size.loc.

| 項目 | 内容 |
|---|---|
| 定義 | size.token_count / size.loc. |
| スコープ | file, project |
| 入力 | size.token_count, size.loc |
| 計算方法 | 分母が 0 か、入力が得られない場合は not_applicable。 |
| 単位 | ratio |
| 言語依存性 | 部分的に言語に依存する |
| 制約 | 正規化の一種で、分母の選び方が比較の結果に影響する。 |
| 参考文献 | - |

## 関数あたりの文の数 {#derived-statements-per-function}

`derived.statements_per_function` — size.statement_count / size.function_count.

| 項目 | 内容 |
|---|---|
| 定義 | size.statement_count / size.function_count. |
| スコープ | file, project |
| 入力 | size.statement_count, size.function_count |
| 計算方法 | 分母が 0 か、入力が得られない場合は not_applicable。 |
| 単位 | ratio |
| 言語依存性 | 部分的に言語に依存する |
| 制約 | 正規化の一種で、分母の選び方が比較の結果に影響する。 |
| 参考文献 | - |

## SLOC あたりの重複トークン数 {#derived-duplicate-tokens-per-sloc}

`derived.duplicate_tokens_per_sloc` — duplication.duplicate_token_count / size.sloc.

| 項目 | 内容 |
|---|---|
| 定義 | duplication.duplicate_token_count / size.sloc. |
| スコープ | file, project |
| 入力 | duplication.duplicate_token_count, size.sloc |
| 計算方法 | 分母が 0 か、入力が得られない場合は not_applicable。 |
| 単位 | ratio |
| 言語依存性 | 部分的に言語に依存する |
| 制約 | 正規化の一種で、分母の選び方が比較の結果に影響する。 |
| 参考文献 | - |

