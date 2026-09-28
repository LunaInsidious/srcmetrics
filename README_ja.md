# srcmetrics

[English](README.md) | 日本語

C, C++, Go, Java, JavaScript, Python, Rust, TypeScript のソースコードから、言語に依存しない定義でメトリクスを取得するツール・ライブラリです。

srcmetrics は、可読性・保守性に関わる定量的な特徴量（規模、制御構造の複雑さ、ネスト、Halstead 尺度、重複、呼び出し関係、ドキュメント）を収集し、それぞれを**個別の値として**出力します。統計分析や、独自の可読性モデルの構築に使うことを想定しています。

- **1 つの定義を全言語に。** ソースコードを [tree-sitter] で構文解析し、言語に依存しない小さな中間表現（IR）に変換します。メトリクスはこの中間表現だけから計算します。言語ごとの違いは、言語別の宣言的な対応表に閉じ込めています。
- **単一の「可読性スコア」は出さない。** 各メトリクスと人間が感じる可読性との関係は事前には分からないため、生の特徴量をそのまま残します。必要なら、*利用者自身のラベル*から実験的なモデルを学習できます。
- **「計算できない」を「0」にしない。** 当てはまらない値（例：関数のないファイルの平均関数長）は `null` とし、その理由を併記します。
- **再現できる。** 結果には、パーサのバージョン、メトリクス定義のバージョン、解析した git コミットを記録します。

[tree-sitter]: https://tree-sitter.github.io/

## インストール

コマンドラインツール：

```sh
cargo install srcmetrics-cli   # `srcmetrics` コマンドが入ります
```

ライブラリ：

```sh
cargo add srcmetrics
```

Rust 1.85 以上が必要です。文法定義は C からコンパイルされるため、ビルド時に C コンパイラが必要です。解析はすべてオフラインで動きます。

## 対応言語

| 言語 | 拡張子 |
|---|---|
| C | `.c`, `.h` |
| C++ | `.cpp`, `.cc`, `.cxx`, `.hpp`, `.hh`, `.hxx` |
| Go | `.go` |
| Java | `.java` |
| JavaScript | `.js`, `.mjs`, `.cjs`, `.jsx` |
| Python | `.py` |
| Rust | `.rs` |
| TypeScript | `.ts`, `.mts`, `.cts`, `.tsx` |

同じアルゴリズムを 8 言語で書いたコードが、Cyclomatic Complexity・Cognitive Complexity・最大ネスト深さで同じ値になることを、テストで確認しています。

## メトリクス

関数・ファイル・プロジェクトの各スコープで 53 種類。すべての定義（意味、計算方法、単位、言語依存性、制約、参考文献）は `srcmetrics metrics` で出力できます。[docs/METRICS.md](docs/METRICS.md) にもあります。

| 分類 | メトリクス |
|---|---|
| 規模 | LOC, SLOC, コメント行・空行, コメント率, 文の数, トークン数, 関数の数, 関数の長さ（平均・最大） |
| 複雑さ | Cyclomatic, Cognitive, 経路数, 分岐・三項演算子・ループ・return・ジャンプの数 |
| ネスト | 最大・平均のネスト深さ |
| Halstead | n1, n2, N1, N2, 語彙数, 長さ, Volume, Difficulty, Effort, Time, Bugs |
| 関数 | 引数の数（平均・最大）, 式の数, 呼び出しの数 |
| 重複 | 重複ブロック数, 重複トークン数, 重複率, 最長の重複（正規化トークンによるクローン検出） |
| 依存関係 | Fan-in, Fan-out, 呼び出しの深さ, import の数 |
| ドキュメント | ドキュメントのある関数の数, その割合, ドキュメントの行数 |
| 派生 | Maintainability Index; 関数あたり Cyclomatic, LOC あたりトークン数, 関数あたり文の数, SLOC あたり重複トークン数 |

## コマンドラインでの使い方

```sh
srcmetrics analyze src -o result.json          # ディレクトリを解析（.gitignore を尊重）
srcmetrics analyze src --format csv > metrics.csv
srcmetrics stats result.json --scope function  # 記述統計・言語別ベースライン・相関
srcmetrics report result.json -o report.html   # 自己完結の HTML（表・ヒストグラム・相関ヒートマップ）
srcmetrics metrics --format markdown           # メトリクス定義
srcmetrics serve                               # HTTP API と Web UI（http://127.0.0.1:8080）
```

構文解析に失敗したファイルは、結果に `"status": "error"` として残り、標準エラーにも報告されます。黙って消えることはありません。

### 出力

```json
{
  "run": {
    "project": "readme",
    "repository": null,
    "commit": "3f2c…",
    "tool_version": "0.1.0",
    "metric_definition_version": "0.1.0",
    "parsers": { "python": "tree-sitter 0.27.0 / tree-sitter-python 0.25.0" },
    "timestamp": "2026-09-29T00:00:00Z"
  },
  "files": [
    {
      "status": "ok",
      "path": "greet.py",
      "language": "python",
      "metrics": { "size.loc": 4.0, "size.function_count": 1.0, "size.comment_ratio": 0.0 },
      "unavailable": {},
      "functions": [
        {
          "name": "greet",
          "start_line": 1,
          "end_line": 4,
          "metrics": {
            "complexity.cyclomatic": 2.0,
            "complexity.cognitive": 1.0,
            "nesting.max_depth": 1.0,
            "halstead.volume": 46.507,
            "maintainability.index": 128.116
          },
          "unavailable": {}
        }
      ]
    }
  ]
}
```

（抜粋です。実際には各スコープの全メトリクスが入り、`project` セクションもあります。）
`null` の値には必ず `unavailable` に理由（`not_applicable`、`unsupported`、`error: …`）があります。

### 実験的な可読性モデル

srcmetrics に組み込みの重みはありません。人間による評価値があれば、それを使ってリッジ回帰を学習し、他のコードを採点できます。

```sh
# labels.csv: path,function,score   （ファイル単位の評価なら function は空）
srcmetrics model train --labels labels.csv -o model.json result.json
srcmetrics model predict --model model.json result.json
```

モデルファイルには、使った特徴量（と、除外した特徴量とその理由）、係数、R²、5 分割交差検証の RMSE が記録されます。

### HTTP API

`srcmetrics serve [--bind 127.0.0.1] [--port 8080]` は、既定ではローカルホストだけで待ち受けます。認証はありません。

| エンドポイント | |
|---|---|
| `GET /` | Web UI：コードを貼り付けてメトリクスを表示 |
| `GET /api/metrics` | メトリクス定義 |
| `POST /api/analyze` | `{"filename": "a.py", "source": "…"}` → 解析結果（未対応の拡張子・構文エラーは 400 と `{"error": …}`） |

## ライブラリとしての使い方

```rust
use srcmetrics::analyze::analyze_source;
use srcmetrics::result::FileResult;

fn main() -> Result<(), srcmetrics::error::AnalysisError> {
    let result = analyze_source("example.py", "def f(x):\n    return x if x else 0\n")?;
    let FileResult::Ok { functions, .. } = &result.files[0] else { unreachable!() };
    assert_eq!(functions[0].metrics.metrics["complexity.cyclomatic"], Some(2.0));
    Ok(())
}
```

主な入口：

| API | 用途 |
|---|---|
| `analyze::analyze(path, project)` | ファイルまたはディレクトリを解析し `AnalysisResult`（serde で直列化可能）を返す |
| `analyze::analyze_source(filename, source)` | メモリ上のソース文字列を解析 |
| `lang::adapter_for_path(path)?.to_ir(path, source)` | 共通 IR（`ir::File`）に変換 |
| `metrics::compute(&program)` / `metrics::definitions()` | IR からメトリクスを計算 / 定義を一覧 |
| `csv::to_csv`, `report::to_html` | CSV・HTML 出力 |
| `stats::Report::of`, `model::train` | 統計と実験的モデル |

## 既知の制約

- 構文エラーのあるファイルは解析しません（部分的な結果は信頼できないため）。tree-sitter は、波括弧がプリプロセッサの分岐をまたぐ C/C++ コード（例：`#ifdef __cplusplus` / `extern "C" {` / `#endif`）を解析できません。これは C のヘッダでよく見られる書き方です。
- `.h` は C として解析します。
- 依存関係メトリクスは、呼び出しを名前だけで解決します（型や import は解決しません）。同じ名前の関数は区別されません。
- マクロ呼び出しの中のコード（例：Rust の `println!(…)`）は解析されません。

各メトリクスの定義にも、それぞれの制約が書かれています。

## ドキュメント

- [PLAN.md](PLAN.md) — 要求仕様
- [docs/SPEC.md](docs/SPEC.md) — 技術仕様書
- [docs/ADR.md](docs/ADR.md) — 設計判断の記録（ADR）
- [docs/METRICS.md](docs/METRICS.md) — メトリクス定義書（コードから生成）

## ライセンス

以下のいずれかを選択できます。

- Apache License, Version 2.0（[LICENSE-APACHE](LICENSE-APACHE)）
- MIT license（[LICENSE-MIT](LICENSE-MIT)）

明示的に別段の意思表示をしない限り、あなたが本プロジェクトへの取り込みを意図して提出した貢献は、Apache-2.0 ライセンスの定義に従い、追加の条件なしに上記のデュアルライセンスで提供されるものとします。
