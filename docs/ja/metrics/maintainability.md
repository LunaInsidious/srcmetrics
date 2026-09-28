# 保守性

## Maintainability Index {#maintainability-index}

`maintainability.index` — 保守性の合成指標（上限のない元の式）。

| 項目 | 内容 |
|---|---|
| 定義 | MI = 171 - 5.2 * ln(V) - 0.23 * CC - 16.2 * ln(SLOC)。 |
| スコープ | function, file |
| 入力 | halstead.volume (V), complexity.cyclomatic (CC), size.sloc (SLOC) |
| 計算方法 | V > 0 かつ SLOC > 0 のときだけ計算する（それ以外は not_applicable）。0〜100 に換算しない。 |
| 単位 | index |
| 言語依存性 | 部分的に言語に依存する |
| 制約 | 入力のメトリクスの制約を受け継ぐ。係数は 1990 年代のコードで求められたもの。 |
| 参考文献 | Oman, P. & Hagemeister, J. (1992). Metrics for assessing a software system's maintainability. ICSM. |

