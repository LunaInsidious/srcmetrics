# srcmetrics 技術仕様書（開発者向け）

本書は srcmetrics の目的・設計原則と、現在の実装の内部構成を記述する。
利用方法（CLI、出力形式、ライブラリ API、HTTP API）は利用者向けドキュメント（`docs/`、GitHub Pages）を、
各メトリクスの意味は `docs/metrics/` を、設計判断の根拠は [ADR](adr/README.md) を参照。

## 0. 目的とスコープ

複数のプログラミング言語で書かれたソースコードから、可読性・理解容易性・保守性に関わる定量的なメトリクスを収集する。
単一の「可読性スコア」を出すことは目的としない。各メトリクスを独立した特徴量として提供し、統計分析・相関分析・可読性モデルの構築に使えるようにする。

対象外（将来の拡張候補を含む）：特定言語固有の型システムの複雑さ、フレームワーク依存性、コーディング規約違反、セキュリティ脆弱性やバグの検出、実行時性能、人間による主観的評価そのもの。

## 1. 設計原則

変更・追加はこれらの原則に従う。原則に反する変更が必要なら、先に ADR で原則を改める。コード中の「design principle Pn」はこの番号を指す。

| # | 原則 |
|---|---|
| P1 | 言語固有の処理は Language Adapter（言語別 Mapping）に閉じ込める。Metric Engine は特定言語の構文・ノード名・機能を参照しない |
| P2 | Metric Engine は Common IR だけを参照する |
| P3 | IR は完全な AST の共通化を目指さず、メトリクスの計算に必要な最小限の情報だけを持つ |
| P4 | メトリクスは独立した Calculator として実装し、Calculator 同士は互いの結果に依存しない（派生メトリクスは標準メトリクスの値から別段階で計算する：ADR-0015） |
| P5 | 可読性を単一の数値に集約しない。重みを事前に決めたスコアを標準機能にしない（実験的モデルは利用者のラベルから学習する場合だけ：ADR-0019） |
| P6 | 各メトリクスの定義・計算方法・単位・言語依存性（language_independent / partially_language_dependent / language_specific）・制約を明示する |
| P7 | 計算不能と値 0 を区別する。計算できない値は null とし理由を併記する。解析エラーは種類を区別し、黙って部分的な結果を出さない |
| P8 | 生のメトリクスを保持する。正規化値などの派生メトリクスは標準メトリクスと区別する（`derived.` 接頭辞） |
| P9 | 言語の追加で Metric Engine を変更しない |
| P10 | 同一の入力・パーサ版・メトリクス定義版に対して同一の結果を返す。結果には project, repository, commit, パーサ版, 定義版, 時刻を記録する |
| P11 | 解析とメトリクス計算は外部ネットワークに接続せずに行う |

## 2. 構成

```text
crates/srcmetrics/          コアライブラリ（同期・ネットワーク非依存）
  src/ir.rs                 Common IR
  src/error.rs              解析エラー
  src/lang/                 Language Adapter（tree-sitter 汎用変換器 + 言語別 Mapping）
  src/metrics/              Metric Engine（Calculator 群、メトリクスの登録簿（ID とスコープ））
  src/analyze.rs            解析パイプライン（パス走査 → IR → メトリクス → 結果）
  src/result.rs             解析結果の型（JSON 形式）
  src/csv.rs                CSV 出力
  src/stats.rs              記述統計・言語別ベースライン・相関
  src/model.rs              実験的な可読性モデル（利用者のラベルから学習するリッジ回帰）
  src/report.rs             自己完結 HTML レポート
crates/srcmetrics-cli/      CLI（clap）と HTTP サーバ（axum。src/serve.rs, src/ui.html）
tests/fixtures/             言語別フィクスチャ
docs/                       利用者向けドキュメント（VitePress、GitHub Pages）。docs/metrics/ にメトリクスの説明
design/                     開発者向け文書（本書、ADR、MEMO）
```

依存方向は `lang → ir ← metrics` の一方向で、`metrics` は `lang` と tree-sitter を参照しない（`tests/engine.rs` で検査）。

## 3. 処理の流れ

```text
analyze(path)
  ├─ 対象の列挙：ディレクトリは .gitignore を尊重して走査し、対応拡張子のファイルだけ（パス順）
  ├─ source ──adapter_for_path(拡張子)──▶ LanguageAdapter::to_ir ──▶ ir::File（失敗は status=error として記録）
  ├─ Program{成功したファイル} ──metrics::compute──▶ ProgramMetrics{project, files[{metrics, functions[]}]}
  └─ AnalysisResult{run, project, files}
```

## 4. Common IR（ADR-0003）

| 型 | 内容 |
|---|---|
| `Program` | `files` |
| `File` | `path, language, source, nodes(arena), root, tokens, functions` |
| `Node` | `id, kind, parent, children, range, label`（label は call の呼び出し先名、logical の演算子） |
| `Function` | `id, name(無名は None), parameters, node, body, range, doc` |
| `Parameter` | `name, range` |
| `Token` | `kind, text, range` |
| `SourceRange` | `start, end`（半開区間）。`Position` は `line`(1 始まり), `column`(0 始まり, バイト), `offset`(バイト) |

- `NodeKind`: block, statement, expression, declaration, branch, else, loop, case, catch, return, jump, call, assignment, binary, logical, conditional, unary, identifier, literal, import, function, other
- `TokenKind`: keyword, identifier, literal, operator, punctuation, comment
- 補助: `File::function_nodes`（入れ子関数を除く関数内ノード）、`top_level_nodes`、`ancestors`、`tokens_in`（範囲内トークンのスライス）

## 5. Language Adapter（ADR-0004）

- `LanguageAdapter { language(), extensions(), to_ir(path, source) }`。実装は `TreeSitterAdapter` のみで、言語ごとの差分は `Mapping` テーブルに置く。
- 変換は具象構文木を 1 回だけ反復的に走査する（深い式でもスタックを溢れさせない）。名前付きノードは IR ノード、葉（comment / literal に指定した型は部分木ごと）はトークンになる。
- 構文木に ERROR / MISSING ノードがあれば `AnalysisError::Parse`（行・列付き）を返す。

### 対応言語

| language | 拡張子 | grammar |
|---|---|---|
| c | .c, .h | tree-sitter-c |
| cpp | .cpp, .cc, .cxx, .hpp, .hh, .hxx | tree-sitter-cpp |
| go | .go | tree-sitter-go |
| java | .java | tree-sitter-java |
| javascript | .js, .mjs, .cjs, .jsx | tree-sitter-javascript |
| python | .py | tree-sitter-python |
| rust | .rs | tree-sitter-rust |
| typescript | .ts, .mts, .cts | tree-sitter-typescript |
| tsx | .tsx | tree-sitter-typescript (TSX) |

### Mapping の項目（ADR-0004, 0012, 0013）

| 項目 | 内容 |
|---|---|
| kinds | ノード型 → NodeKind（ないものは other） |
| callee_fields | 呼び出しノードの呼び出し先フィールド（label に呼び出し先名を入れる） |
| logical_operators | binary を logical にする演算子 |
| default_case_keyword | default ラベルを示すキーワード（case から除外） |
| else_field | else 節ノードがない grammar での else 部分のフィールド |
| comments / literals / interpolations / identifiers | トークン分類。interpolations は literal 内の埋め込みコード |
| name_fields | 名前を探すフィールドの順序 |
| parameter_fields | 引数を持つフィールド（Go は receiver, parameters） |
| decorators | ドキュメントと関数の間に書かれるノード型 |
| docstring | 関数本体の先頭でドキュメントとなるリテラルのノード型 |
| ignored_parameters | 引数とみなさないテキスト（C の void 等） |

### 言語の追加手順

1. grammar クレートを `crates/srcmetrics/Cargo.toml` に追加（ADR に記録）
2. `src/lang/<lang>.rs` に `Mapping` を書き、`src/lang/mod.rs` の `ADAPTERS` に登録
3. `tests/fixtures/equivalence/classify.<ext>` を追加し、`tests/engine.rs` の `EQUIVALENCE_LANGUAGES` に加える（Cyclomatic / Cognitive / Max Nesting が全言語で一致すること）

Metric Engine は変更しない。

## 6. Metric Engine

- `Calculator { definitions(), compute(&Program) -> ProgramMetrics }`。各 Calculator は独立しており、互いの結果を参照しない。
- `metrics::compute` が全 Calculator を実行し、スコープごとの `Metrics`（id → `MetricValue`、id 順）を併合する。
- `MetricValue = Available(f64) | NotApplicable | Unsupported | Error(String)`（ADR-0005）。
- 各 Calculator は、出力するメトリクスの ID とスコープを `SPECS`（`MetricSpec`）に登録する。出力がこれと一致することを `tests/engine.rs` で検査する。メトリクスの意味・計算方法・制約は利用者向けドキュメント（英語 `docs/metrics/`、日本語 `docs/ja/metrics/`）に手で書き、コードには持たない（ADR-0026）。全 ID が両言語のページに載っていることを `tests/docs.rs` で検査する。
- 定義や計算方法を変えたら `DEFINITION_VERSION` を上げる。初回リリースまでの開発中は `0.1.0` のまま（公開済みの解析結果がないため）。

### 実装済み Calculator

| Calculator | メトリクス |
|---|---|
| SizeCalculator | size.loc, sloc, comment_loc, blank_loc, comment_ratio, statement_count, token_count, function_count, function_length, avg_function_length, max_function_length |
| ComplexityCalculator | complexity.cyclomatic, branch_count, conditional_count, loop_count, return_count, jump_count, path_count |
| CognitiveCalculator | complexity.cognitive |
| NestingCalculator | nesting.max_depth, nesting.avg_depth |
| HalsteadCalculator | halstead.unique_operators, unique_operands, total_operators, total_operands, vocabulary, length, volume, difficulty, effort, time, bugs |
| FunctionCalculator | function.parameter_count, avg_parameter_count, max_parameter_count, expression_count, call_count |
| DuplicationCalculator | duplication.duplicate_block_count, duplicate_token_count, duplication_ratio, max_duplicate_length |
| DependencyCalculator | dependency.fan_in, fan_out, call_depth, dependency_count |
| DocumentationCalculator | documentation.doc_loc, documented_function_count, documentation_ratio |

### 派生メトリクス（ADR-0015）

全 Calculator の結果を併合した後、各スコープの表に `derived.rs` の派生メトリクスを追加する。派生メトリクスは IR を見ず、標準メトリクスの値だけから計算する。

| ID | 式 | スコープ |
|---|---|---|
| maintainability.index | 171 − 5.2·ln(V) − 0.23·CC − 16.2·ln(SLOC) | function, file |
| derived.cyclomatic_per_function | cyclomatic / function_count | file, project |
| derived.tokens_per_loc | token_count / loc | file, project |
| derived.statements_per_function | statement_count / function_count | file, project |
| derived.duplicate_tokens_per_sloc | duplicate_token_count / sloc | file, project |

### 計算量

深い構造（数千段の `else if` の連鎖、長い呼び出しの連鎖）でもスタックを溢れさせないよう、再帰は使わない。ネストのレベルと経路数は arena の前順（親が先）を使って 1 回の走査で計算し、Call Depth は反復版の Tarjan 法で強連結成分をまとめて計算する。

### メトリクスの追加手順

1. `src/metrics/<name>.rs` に Calculator と `SPECS` を書き、`calculators()` に登録
2. 手組み IR（`ir::builder`）で単体テストを書く
3. `docs/metrics/` と `docs/ja/metrics/` に説明を書く（各メトリクスの節は「`` `ID` — 説明 ``」の行で始める）

## 7. 出力・CLI・HTTP API の実装上の決まり

利用方法と形式は利用者向けドキュメント（`docs/guide/`）に書く。ここには実装上の決まりだけを置く。

- 結果の形式は ADR-0010、CSV は ADR-0016、統計は ADR-0018、モデルは ADR-0019、レポートは ADR-0020、HTTP API は ADR-0021
- 有限でない数値（オーバーフロー等）は null と `error: value ... is not finite` にする
- JSON は `float_roundtrip` で書き出し、読み戻したとき同じ値になる（P10）
- パーサ版は Mapping の `grammar_crate` と `TREE_SITTER_VERSION` から作り、`Cargo.lock` と一致することをテストで検査する
- CLI のエラーは終了コード 1 と、次に何をすべきかが分かるメッセージ。解析できなかったファイルは結果に含めたうえで stderr に件数と理由を出す（終了コード 0）

## 8. エラー

| 種別 | 発生条件 | 利用者が次にすること |
|---|---|---|
| `UnsupportedLanguage` | 拡張子に対応する Adapter がない | 対応拡張子一覧（メッセージに表示）を確認 |
| `Io` | パスが存在しない・読めない | パスと権限を確認 |
| `Parse` | 構文木に ERROR / MISSING がある | 指摘位置の構文を直すか、拡張子と言語が合っているか確認 |
| `IrConversion` | grammar の互換性など内部不整合 | 入力ファイルを添えて報告 |
| `MetricValue::Error` | メトリクス計算中の不整合（例：トークンに覆われない行） | 入力ファイルを添えて報告 |

## 9. テスト

| ファイル | 内容 |
|---|---|
| `src/**` の `#[cfg(test)]` | IR 補助関数、Calculator 単体（手組み IR） |
| `tests/adapter.rs` | 言語別の IR 変換 |
| `tests/engine.rs` | 定義と出力の整合、言語横断の等価性、Metric Engine の言語非依存性 |
| `tests/docs.rs` | 全メトリクス ID が `docs/metrics/`, `docs/ja/metrics/` に載っていること |
| `tests/analyze.rs` | ディレクトリ解析、エラーファイルの記録、run メタデータ、JSON 往復 |
| `tests/model.rs` | ラベルの照合、学習、予測 |
| `crates/srcmetrics-cli/tests/cli.rs` | CLI の出力と終了コード |
| `crates/srcmetrics-cli/tests/serve.rs` | HTTP API / UI（実際にサーバを起動） |

ドキュメントサイトは `npm run docs:build` でビルドし、リンク切れがあれば失敗する。
