# 複雑さ

## Cyclomatic Complexity {#complexity-cyclomatic}

`complexity.cyclomatic` — 線形独立な経路の数（McCabe）。

| 項目 | 内容 |
|---|---|
| 定義 | 1 + 関数内の判定点の数。 |
| スコープ | function, file, project |
| 入力 | ノードの種類：branch, loop, case, catch, logical, conditional |
| 計算方法 | 関数：1 + 判定ノードの数（入れ子関数を除く）。`else if` / `elif`、短絡演算子（&&, \|\|, and, or）、三項演算子、default 以外の case ラベルがそれぞれ 1 つの判定点。ファイル：関数の合計 + トップレベルのコードの判定点。プロジェクト：ファイルの合計。 |
| 単位 | count |
| 言語依存性 | 部分的に言語に依存する |
| 制約 | 何を判定点とするかは言語ごとの Mapping に従う（例：Python の内包表記の `for` / `if` は数える。Python の `case _:` と Rust の `_ =>` は case として数える）。 |
| 参考文献 | McCabe, T. J. (1976). A Complexity Measure. IEEE TSE SE-2(4). |

## 分岐の数 {#complexity-branch-count}

`complexity.branch_count` — 分岐ノード（if, else if, elif）と default 以外の case ラベルの数。

| 項目 | 内容 |
|---|---|
| 定義 | 分岐ノード（if, else if, elif）と default 以外の case ラベルの数。 |
| スコープ | function, file, project |
| 入力 | ノードの種類 |
| 計算方法 | 関数：入れ子関数を除く。ファイル：ファイル全体。プロジェクト：ファイルの合計。 |
| 単位 | count |
| 言語依存性 | 部分的に言語に依存する |
| 制約 | どの構文がどのノードの種類になるかは、言語ごとの Mapping に従う。 |
| 参考文献 | - |

## 三項演算子の数 {#complexity-conditional-count}

`complexity.conditional_count` — 条件式（三項演算子）の数。

| 項目 | 内容 |
|---|---|
| 定義 | 条件式（三項演算子）の数。 |
| スコープ | function, file, project |
| 入力 | ノードの種類 |
| 計算方法 | 関数：入れ子関数を除く。ファイル：ファイル全体。プロジェクト：ファイルの合計。 |
| 単位 | count |
| 言語依存性 | 部分的に言語に依存する |
| 制約 | どの構文がどのノードの種類になるかは、言語ごとの Mapping に従う。 |
| 参考文献 | - |

## ループの数 {#complexity-loop-count}

`complexity.loop_count` — ループの数。

| 項目 | 内容 |
|---|---|
| 定義 | ループの数。 |
| スコープ | function, file, project |
| 入力 | ノードの種類 |
| 計算方法 | 関数：入れ子関数を除く。ファイル：ファイル全体。プロジェクト：ファイルの合計。 |
| 単位 | count |
| 言語依存性 | 部分的に言語に依存する |
| 制約 | どの構文がどのノードの種類になるかは、言語ごとの Mapping に従う。 |
| 参考文献 | - |

## return の数 {#complexity-return-count}

`complexity.return_count` — return 文の数。

| 項目 | 内容 |
|---|---|
| 定義 | return 文の数。 |
| スコープ | function, file, project |
| 入力 | ノードの種類 |
| 計算方法 | 関数：入れ子関数を除く。ファイル：ファイル全体。プロジェクト：ファイルの合計。 |
| 単位 | count |
| 言語依存性 | 部分的に言語に依存する |
| 制約 | どの構文がどのノードの種類になるかは、言語ごとの Mapping に従う。 |
| 参考文献 | - |

## ジャンプの数 {#complexity-jump-count}

`complexity.jump_count` — ジャンプ（break, continue, goto, throw / raise）の数。

| 項目 | 内容 |
|---|---|
| 定義 | ジャンプ（break, continue, goto, throw / raise）の数。 |
| スコープ | function, file, project |
| 入力 | ノードの種類 |
| 計算方法 | 関数：入れ子関数を除く。ファイル：ファイル全体。プロジェクト：ファイルの合計。 |
| 単位 | count |
| 言語依存性 | 部分的に言語に依存する |
| 制約 | どの構文がどのノードの種類になるかは、言語ごとの Mapping に従う。 |
| 参考文献 | - |

## 経路数 {#complexity-path-count}

`complexity.path_count` — 関数を通る非循環な実行経路の数。

| 項目 | 内容 |
|---|---|
| 定義 | 各ループを 0 回または 1 回通るとしたときの、関数を通る経路の数。 |
| スコープ | function |
| 入力 | ノードの種類と木構造 |
| 計算方法 | 並んだ子は掛け算。if の連鎖は各分岐の和（最後の else がなければ +1）。ループと三項演算子は子の積 + 1。連続する case ラベルや catch 節は経路の和 + 1。入れ子関数は 1。 |
| 単位 | count |
| 言語依存性 | 部分的に言語に依存する |
| 制約 | Nejmeh の NPATH とは異なる：短絡演算子や早期の脱出（return, jump）は数に影響しない。 |
| 参考文献 | Nejmeh, B. A. (1988). NPATH: a measure of execution path complexity. CACM 31(2) (related, not identical). |

## Cognitive Complexity {#complexity-cognitive}

`complexity.cognitive` — 関数の制御の流れの理解しにくさ（SonarSource）。

| 項目 | 内容 |
|---|---|
| 定義 | 直線的な流れを断ち切る構造ごとの加算を、ネストで重み付けした合計。 |
| スコープ | function, file, project |
| 入力 | ノードの種類、親子関係、呼び出しと論理演算子のラベル |
| 計算方法 | if の連鎖の先頭、ループ、catch、三項演算子、連続する case ラベル（switch）：1 + ネストのレベル。else if / elif と else：1。同じ論理演算子の並び：1。自分と同じ名前の呼び出し（再帰）：1。ネストのレベルは分岐、ループ、case、catch、三項演算子で深くなる。ファイル：関数の合計。プロジェクト：ファイルの合計。 |
| 単位 | count |
| 言語依存性 | 部分的に言語に依存する |
| 制約 | 入れ子関数（ラムダ）は外側の関数に加算せず、別に計測する。ラベル付きの break / continue や goto は加算しない（IR のジャンプはラベルを持たない）。 |
| 参考文献 | Campbell, G. A. (2018). Cognitive Complexity: A new way of measuring understandability. SonarSource. |

