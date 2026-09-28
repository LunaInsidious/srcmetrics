<!-- Generated from crates/srcmetrics/src/metrics. Do not edit; run `UPDATE_DOCS=1 cargo test -p srcmetrics --test docs`. -->

# 関数

## 引数の数 {#function-parameter-count}

`function.parameter_count` — 宣言された引数の数。

| 項目 | 内容 |
|---|---|
| 定義 | IR 上の関数の引数。 |
| スコープ | function |
| 入力 | 関数の引数 |
| 計算方法 | 可変長引数（例：`*args`）は 1 つと数える。Python の `*` や `/`、C の `(void)` のような区切りは引数ではない。 |
| 単位 | count |
| 言語依存性 | 部分的に言語に依存する |
| 制約 | 明示的な受け手（Python の `self`）は数え、暗黙の受け手（`this`）は数えない。 |
| 参考文献 | - |

## 平均引数数 {#function-avg-parameter-count}

`function.avg_parameter_count` — 関数の引数の数の平均。

| 項目 | 内容 |
|---|---|
| 定義 | function.parameter_count の平均。 |
| スコープ | file, project |
| 入力 | 関数の引数 |
| 計算方法 | 関数がなければ not_applicable。 |
| 単位 | count |
| 言語依存性 | 部分的に言語に依存する |
| 制約 | function.parameter_count を参照。 |
| 参考文献 | - |

## 最大引数数 {#function-max-parameter-count}

`function.max_parameter_count` — 関数の引数の数の最大値。

| 項目 | 内容 |
|---|---|
| 定義 | function.parameter_count の最大値。 |
| スコープ | file, project |
| 入力 | 関数の引数 |
| 計算方法 | 関数がなければ not_applicable。 |
| 単位 | count |
| 言語依存性 | 部分的に言語に依存する |
| 制約 | function.parameter_count を参照。 |
| 参考文献 | - |

## 式の数 {#function-expression-count}

`function.expression_count` — 部分式を含む式の数。

| 項目 | 内容 |
|---|---|
| 定義 | expression, call, assignment, binary, logical, conditional, unary の種類のノード。 |
| スコープ | function, file, project |
| 入力 | ノードの種類 |
| 計算方法 | 関数：入れ子関数を除く。ファイル：ファイル全体。プロジェクト：合計。識別子とリテラルは単独では式に数えない。 |
| 単位 | count |
| 言語依存性 | 部分的に言語に依存する |
| 制約 | 言語の Mapping に載っていない式の形（例：ラムダの本体は入れ子関数。対応付けのない式は `other`）は数えない。 |
| 参考文献 | - |

## 呼び出しの数 {#function-call-count}

`function.call_count` — 呼び出し箇所の数。

| 項目 | 内容 |
|---|---|
| 定義 | call の種類のノード（関数呼び出しとコンストラクタ呼び出し）。 |
| スコープ | function, file, project |
| 入力 | ノードの種類 |
| 計算方法 | 関数：入れ子関数を除く。ファイル：ファイル全体。プロジェクト：合計。 |
| 単位 | count |
| 言語依存性 | 言語に依存しない |
| 制約 | - |
| 参考文献 | - |

