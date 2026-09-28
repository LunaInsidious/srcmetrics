# 対応言語と制約

| 言語 | 拡張子 | 文法 |
|---|---|---|
| C | `.c`, `.h` | tree-sitter-c |
| C++ | `.cpp`, `.cc`, `.cxx`, `.hpp`, `.hh`, `.hxx` | tree-sitter-cpp |
| Go | `.go` | tree-sitter-go |
| Java | `.java` | tree-sitter-java |
| JavaScript | `.js`, `.mjs`, `.cjs`, `.jsx` | tree-sitter-javascript |
| Python | `.py` | tree-sitter-python |
| Rust | `.rs` | tree-sitter-rust |
| TypeScript | `.ts`, `.mts`, `.cts`, `.tsx` | tree-sitter-typescript |

文法の正確なバージョンは、すべての結果（`run.parsers`）に記録されます。

## 言語をどう揃えているか

各言語を同じ小さな中間表現（分岐、ループ、case、呼び出し、トークンなど）に変換し、すべてのメトリクスをその中間表現だけから計算します。同じアルゴリズムを 8 言語で書いたコードが、Cyclomatic Complexity・Cognitive Complexity・最大ネスト深さで同じ値になることをテストで確認しています。

それでも、書き方の違いに左右されるメトリクスはあります（例：Python には閉じ括弧がないので、関数の行数が短くなる）。各メトリクスの[言語依存性](/metrics/)に、言語に依存しないか部分的に依存するかと、既知の違いが書かれています。

## 既知の制約

- **構文エラーのあるファイルは解析しません**。部分的な結果は信頼できないためです。`status: "error"` として報告されます。
- tree-sitter は、波括弧がプリプロセッサの分岐をまたぐ C/C++ のコード（例：`#ifdef __cplusplus` / `extern "C" {` / `#endif`）を解析できません。C のヘッダでよく見られる書き方です。
- `.h` は C として解析します。
- 依存関係のメトリクスは、呼び出しを名前だけで解決します（型・スコープ・import は見ません）。同じ名前の関数は区別されません。
- マクロ呼び出しの中のコード（例：Rust の `println!(…)`）は解析されません。
