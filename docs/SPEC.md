# codestat 技術仕様書

要求仕様は [PLAN.md](../PLAN.md)、設計判断の根拠は [ADR.md](ADR.md)、各メトリクスの定義は [METRICS.md](METRICS.md)（コードから生成）を参照。
本書は「現在の実装が何をどう行うか」を記述する。

## 1. 構成

```text
crates/codestat/          コアライブラリ（同期・ネットワーク非依存）
  src/ir.rs               Common IR
  src/error.rs            解析エラー
  src/lang/               Language Adapter（tree-sitter 汎用変換器 + 言語別 Mapping）
  src/metrics/            Metric Engine（Calculator 群、定義レジストリ）
  src/analyze.rs          解析パイプライン（パス走査 → IR → メトリクス → 結果）
  src/result.rs           解析結果の型（JSON 形式）
crates/codestat-cli/      CLI（clap）
tests/fixtures/           言語別フィクスチャ
```

依存方向は `lang → ir ← metrics` の一方向で、`metrics` は `lang` と tree-sitter を参照しない（`tests/engine.rs` で検査）。

## 2. 処理の流れ

```text
analyze(path)
  ├─ 対象の列挙：ディレクトリは .gitignore を尊重して走査し、対応拡張子のファイルだけ（パス順）
  ├─ source ──adapter_for_path(拡張子)──▶ LanguageAdapter::to_ir ──▶ ir::File（失敗は status=error として記録）
  ├─ Program{成功したファイル} ──metrics::compute──▶ ProgramMetrics{project, files[{metrics, functions[]}]}
  └─ AnalysisResult{run, project, files}
```

## 3. Common IR（ADR-0003）

| 型 | 内容 |
|---|---|
| `Program` | `files` |
| `File` | `path, language, source, nodes(arena), root, tokens, functions` |
| `Node` | `id, kind, parent, children, range` |
| `Function` | `id, name(無名は None), parameters, node, body, range, doc` |
| `Parameter` | `name, range` |
| `Token` | `kind, text, range` |
| `SourceRange` | `start, end`（半開区間）。`Position` は `line`(1 始まり), `column`(0 始まり, バイト), `offset`(バイト) |

- `NodeKind`: block, statement, expression, declaration, branch, else, loop, case, catch, return, jump, call, assignment, binary, logical, conditional, unary, identifier, literal, import, function, other
- `TokenKind`: keyword, identifier, literal, operator, punctuation, comment
- 補助: `File::function_nodes`（入れ子関数を除く関数内ノード）、`top_level_nodes`、`ancestors`、`tokens_in`（範囲内トークンのスライス）

## 4. Language Adapter（ADR-0004）

- `LanguageAdapter { language(), extensions(), to_ir(path, source) }`。実装は `TreeSitterAdapter` のみで、言語ごとの差分は `Mapping` テーブルに置く。
- 変換は具象構文木を 1 回だけ反復的に走査する（深い式でもスタックを溢れさせない）。名前付きノードは IR ノード、葉（comment / literal に指定した型は部分木ごと）はトークンになる。
- 構文木に ERROR / MISSING ノードがあれば `AnalysisError::Parse`（行・列付き）を返す。

### 対応言語

| language | 拡張子 | grammar |
|---|---|---|
| c | .c, .h | tree-sitter-c |
| python | .py | tree-sitter-python |
| typescript | .ts, .mts, .cts | tree-sitter-typescript |
| tsx | .tsx | tree-sitter-typescript (TSX) |

### 言語の追加手順

1. grammar クレートを `crates/codestat/Cargo.toml` に追加（ADR に記録）
2. `src/lang/<lang>.rs` に `Mapping` を書き、`src/lang/mod.rs` の `ADAPTERS` に登録
3. `tests/fixtures/equivalence/classify.<ext>` を追加し、`tests/engine.rs` の等価テストに加える

Metric Engine は変更しない。

## 5. Metric Engine

- `Calculator { definitions(), compute(&Program) -> ProgramMetrics }`。各 Calculator は独立しており、互いの結果を参照しない。
- `metrics::compute` が全 Calculator を実行し、スコープごとの `Metrics`（id → `MetricValue`、id 順）を併合する。
- `MetricValue = Available(f64) | NotApplicable | Unsupported | Error(String)`（ADR-0005）。
- 定義は各 Calculator の `DEFINITIONS` に PLAN §9 の全項目で記述し、`docs/METRICS.md` はそこから生成する（`tests/docs.rs` で同期を検査）。
- 定義や計算方法を変えたら `DEFINITION_VERSION` を上げる。

### 実装済み Calculator

| Calculator | メトリクス |
|---|---|
| SizeCalculator | size.loc, sloc, comment_loc, blank_loc, comment_ratio, statement_count, token_count, function_count, function_length, avg_function_length, max_function_length |
| ComplexityCalculator | complexity.cyclomatic, branch_count, conditional_count, loop_count, return_count, jump_count, path_count |
| NestingCalculator | nesting.max_depth, nesting.avg_depth |
| HalsteadCalculator | halstead.unique_operators, unique_operands, total_operators, total_operands, vocabulary, length, volume, difficulty, effort, time, bugs |
| FunctionCalculator | function.parameter_count, avg_parameter_count, max_parameter_count, expression_count, call_count |
| DuplicationCalculator | duplication.duplicate_block_count, duplicate_token_count, duplication_ratio, max_duplicate_length |

### メトリクスの追加手順

1. `src/metrics/<name>.rs` に Calculator と `DEFINITIONS` を書き、`calculators()` に登録
2. 手組み IR（`ir::builder`）で単体テストを書く
3. `UPDATE_DOCS=1 cargo test -p codestat --test docs` で METRICS.md を再生成

## 6. 解析結果（ADR-0010）

- `run`：project, repository, commit（git 管理外なら null）, tool_version, metric_definition_version, parsers（言語 → パーサ版）, timestamp（RFC 3339, UTC）
- `project` / 各ファイル / 各関数：`metrics`（ID → 数値 or null）と `unavailable`（ID → `not_applicable` / `unsupported` / `error: ...`）
- 各ファイル：`status` が `ok`（path, language, metrics, functions）または `error`（path, language, error）
- 各関数：name（無名は null）, start_line, end_line, metrics
- 有限でない数値（オーバーフロー等）は null と `error: value ... is not finite` にする
- パーサ版は Mapping の `grammar_crate` と `TREE_SITTER_VERSION` から作り、`Cargo.lock` と一致することをテストで検査する

## 7. CLI

```text
codestat analyze <PATH> [--project NAME] [-o FILE]   解析結果を JSON で出力
codestat metrics [--format json|markdown]            メトリクス定義を出力（markdown は docs/METRICS.md と同一）
```

- 解析できなかったファイルがあれば、結果に含めたうえで stderr に件数と理由を出す（終了コードは 0）
- パスが存在しない・未対応拡張子のファイルを直接指定した等は、終了コード 1 と対処方法付きのメッセージ

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
| `tests/docs.rs` | METRICS.md の同期 |
| `tests/analyze.rs` | ディレクトリ解析、エラーファイルの記録、run メタデータ、JSON 往復 |
| `crates/codestat-cli/tests/cli.rs` | CLI の出力と終了コード |
