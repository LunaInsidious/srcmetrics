# Halstead

## 演算子の種類数 {#halstead-unique-operators}

`halstead.unique_operators` — n1.

| 項目 | 内容 |
|---|---|
| 定義 | 演算子の種類の数。 |
| スコープ | function, file, project |
| 入力 | トークン（種類とテキスト） |
| 計算方法 | 演算子：keyword と operator のトークン、および開き括弧 ( [ {。被演算子：identifier と literal のトークン。カンマ・セミコロン・閉じ括弧・コメントは数えない。種類はトークンのテキストが同じものを 1 種類とする。 |
| 単位 | count |
| 言語依存性 | 部分的に言語に依存する |
| 制約 | キーワード（`int` などの型のキーワードを含む）は演算子。文字列リテラルは 1 つの被演算子。 |
| 参考文献 | Halstead, M. H. (1977). Elements of Software Science. Elsevier. |

## 被演算子の種類数 {#halstead-unique-operands}

`halstead.unique_operands` — n2.

| 項目 | 内容 |
|---|---|
| 定義 | 被演算子の種類の数。 |
| スコープ | function, file, project |
| 入力 | トークン（種類とテキスト） |
| 計算方法 | 演算子：keyword と operator のトークン、および開き括弧 ( [ {。被演算子：identifier と literal のトークン。カンマ・セミコロン・閉じ括弧・コメントは数えない。種類はトークンのテキストが同じものを 1 種類とする。 |
| 単位 | count |
| 言語依存性 | 部分的に言語に依存する |
| 制約 | キーワード（`int` などの型のキーワードを含む）は演算子。文字列リテラルは 1 つの被演算子。 |
| 参考文献 | Halstead, M. H. (1977). Elements of Software Science. Elsevier. |

## 演算子の総数 {#halstead-total-operators}

`halstead.total_operators` — N1.

| 項目 | 内容 |
|---|---|
| 定義 | 演算子の出現回数。 |
| スコープ | function, file, project |
| 入力 | トークン（種類とテキスト） |
| 計算方法 | 演算子：keyword と operator のトークン、および開き括弧 ( [ {。被演算子：identifier と literal のトークン。カンマ・セミコロン・閉じ括弧・コメントは数えない。種類はトークンのテキストが同じものを 1 種類とする。 |
| 単位 | count |
| 言語依存性 | 部分的に言語に依存する |
| 制約 | キーワード（`int` などの型のキーワードを含む）は演算子。文字列リテラルは 1 つの被演算子。 |
| 参考文献 | Halstead, M. H. (1977). Elements of Software Science. Elsevier. |

## 被演算子の総数 {#halstead-total-operands}

`halstead.total_operands` — N2.

| 項目 | 内容 |
|---|---|
| 定義 | 被演算子の出現回数。 |
| スコープ | function, file, project |
| 入力 | トークン（種類とテキスト） |
| 計算方法 | 演算子：keyword と operator のトークン、および開き括弧 ( [ {。被演算子：identifier と literal のトークン。カンマ・セミコロン・閉じ括弧・コメントは数えない。種類はトークンのテキストが同じものを 1 種類とする。 |
| 単位 | count |
| 言語依存性 | 部分的に言語に依存する |
| 制約 | キーワード（`int` などの型のキーワードを含む）は演算子。文字列リテラルは 1 つの被演算子。 |
| 参考文献 | Halstead, M. H. (1977). Elements of Software Science. Elsevier. |

## 語彙数 {#halstead-vocabulary}

`halstead.vocabulary` — n.

| 項目 | 内容 |
|---|---|
| 定義 | n = n1 + n2。 |
| スコープ | function, file, project |
| 入力 | トークン（種類とテキスト） |
| 計算方法 | 演算子：keyword と operator のトークン、および開き括弧 ( [ {。被演算子：identifier と literal のトークン。カンマ・セミコロン・閉じ括弧・コメントは数えない。種類はトークンのテキストが同じものを 1 種類とする。 |
| 単位 | count |
| 言語依存性 | 部分的に言語に依存する |
| 制約 | キーワード（`int` などの型のキーワードを含む）は演算子。文字列リテラルは 1 つの被演算子。 |
| 参考文献 | Halstead, M. H. (1977). Elements of Software Science. Elsevier. |

## プログラム長 {#halstead-length}

`halstead.length` — N.

| 項目 | 内容 |
|---|---|
| 定義 | N = N1 + N2。 |
| スコープ | function, file, project |
| 入力 | トークン（種類とテキスト） |
| 計算方法 | 演算子：keyword と operator のトークン、および開き括弧 ( [ {。被演算子：identifier と literal のトークン。カンマ・セミコロン・閉じ括弧・コメントは数えない。種類はトークンのテキストが同じものを 1 種類とする。 |
| 単位 | count |
| 言語依存性 | 部分的に言語に依存する |
| 制約 | キーワード（`int` などの型のキーワードを含む）は演算子。文字列リテラルは 1 つの被演算子。 |
| 参考文献 | Halstead, M. H. (1977). Elements of Software Science. Elsevier. |

## Volume {#halstead-volume}

`halstead.volume` — V.

| 項目 | 内容 |
|---|---|
| 定義 | V = N * log2(n)。n = 0 なら not_applicable。 |
| スコープ | function, file, project |
| 入力 | トークン（種類とテキスト） |
| 計算方法 | 演算子：keyword と operator のトークン、および開き括弧 ( [ {。被演算子：identifier と literal のトークン。カンマ・セミコロン・閉じ括弧・コメントは数えない。種類はトークンのテキストが同じものを 1 種類とする。 |
| 単位 | bits |
| 言語依存性 | 部分的に言語に依存する |
| 制約 | キーワード（`int` などの型のキーワードを含む）は演算子。文字列リテラルは 1 つの被演算子。 |
| 参考文献 | Halstead, M. H. (1977). Elements of Software Science. Elsevier. |

## Difficulty {#halstead-difficulty}

`halstead.difficulty` — D.

| 項目 | 内容 |
|---|---|
| 定義 | D = (n1 / 2) * (N2 / n2)。n2 = 0 なら not_applicable。 |
| スコープ | function, file, project |
| 入力 | トークン（種類とテキスト） |
| 計算方法 | 演算子：keyword と operator のトークン、および開き括弧 ( [ {。被演算子：identifier と literal のトークン。カンマ・セミコロン・閉じ括弧・コメントは数えない。種類はトークンのテキストが同じものを 1 種類とする。 |
| 単位 | ratio |
| 言語依存性 | 部分的に言語に依存する |
| 制約 | キーワード（`int` などの型のキーワードを含む）は演算子。文字列リテラルは 1 つの被演算子。 |
| 参考文献 | Halstead, M. H. (1977). Elements of Software Science. Elsevier. |

## Effort {#halstead-effort}

`halstead.effort` — E.

| 項目 | 内容 |
|---|---|
| 定義 | E = D * V。 |
| スコープ | function, file, project |
| 入力 | トークン（種類とテキスト） |
| 計算方法 | 演算子：keyword と operator のトークン、および開き括弧 ( [ {。被演算子：identifier と literal のトークン。カンマ・セミコロン・閉じ括弧・コメントは数えない。種類はトークンのテキストが同じものを 1 種類とする。 |
| 単位 | count |
| 言語依存性 | 部分的に言語に依存する |
| 制約 | キーワード（`int` などの型のキーワードを含む）は演算子。文字列リテラルは 1 つの被演算子。 |
| 参考文献 | Halstead, M. H. (1977). Elements of Software Science. Elsevier. |

## 推定プログラミング時間 {#halstead-time}

`halstead.time` — T.

| 項目 | 内容 |
|---|---|
| 定義 | T = E / 18。 |
| スコープ | function, file, project |
| 入力 | トークン（種類とテキスト） |
| 計算方法 | 演算子：keyword と operator のトークン、および開き括弧 ( [ {。被演算子：identifier と literal のトークン。カンマ・セミコロン・閉じ括弧・コメントは数えない。種類はトークンのテキストが同じものを 1 種類とする。 |
| 単位 | seconds |
| 言語依存性 | 部分的に言語に依存する |
| 制約 | キーワード（`int` などの型のキーワードを含む）は演算子。文字列リテラルは 1 つの被演算子。 |
| 参考文献 | Halstead, M. H. (1977). Elements of Software Science. Elsevier. |

## 推定バグ数 {#halstead-bugs}

`halstead.bugs` — B.

| 項目 | 内容 |
|---|---|
| 定義 | B = V / 3000。 |
| スコープ | function, file, project |
| 入力 | トークン（種類とテキスト） |
| 計算方法 | 演算子：keyword と operator のトークン、および開き括弧 ( [ {。被演算子：identifier と literal のトークン。カンマ・セミコロン・閉じ括弧・コメントは数えない。種類はトークンのテキストが同じものを 1 種類とする。 |
| 単位 | count |
| 言語依存性 | 部分的に言語に依存する |
| 制約 | キーワード（`int` などの型のキーワードを含む）は演算子。文字列リテラルは 1 つの被演算子。 |
| 参考文献 | Halstead, M. H. (1977). Elements of Software Science. Elsevier. |

