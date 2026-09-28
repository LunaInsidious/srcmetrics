# ADR: [タイトル]

- **Status:** Proposed
- **Date:** YYYY-MM-DD
- **Deciders:** [決定者 / チーム]
- **Tags:** [architecture, parser, IR, performance...]

## Context

### Problem

[何が問題なのか、なぜ意思決定が必要なのか]

### Background

[意思決定に関係する前提・制約・既存実装など]

### Requirements

- [要件1]
- [要件2]
- [要件3]

## Decision

[最終的に何を採用するか]

### Details

[決定内容の具体的な説明]

```text
[必要に応じて構成図・データ構造・処理フローなど]
```

### Rationale

[なぜこの決定にしたのか]

- [理由1]
- [理由2]
- [理由3]

## Alternatives Considered

### [Alternative 1]

[候補の概要]

**Pros**
- [メリット]

**Cons**
- [デメリット]

**Rejected because:** [採用しなかった理由]

### [Alternative 2]

[候補の概要]

**Pros**
- [メリット]

**Cons**
- [デメリット]

**Rejected because:** [採用しなかった理由]

## Consequences

### Positive

- [この決定によって得られるメリット]
- [改善される点]

### Negative

- [新たに発生するコスト・制約]
- [複雑になる点]

### Risks

- [想定されるリスク]
- [将来的に問題になる可能性]

## Compatibility / Migration

[既存実装との互換性や移行方法]

- [変更が必要な箇所]
- [互換性への影響]
- [移行手順]

## Implementation Notes

[実装時に重要な注意事項]

- [注意点1]
- [注意点2]

## Validation

[この決定が妥当であることをどう検証するか]

- [ ] [テスト・ベンチマーク]
- [ ] [実装検証]
- [ ] [互換性検証]

## References

- [関連Issue / PR]
- [関連仕様]
- [参考資料]

## Revision History

| Date | Status | Change |
|---|---|---|
| YYYY-MM-DD | Proposed | Initial proposal |
| | | |
---

# ADR 一覧

上記はテンプレート。以下に採番済み ADR を記録する。

---

# ADR-0001: 実装言語 Rust と Cargo workspace 構成

- **Status:** Accepted
- **Date:** 2026-09-29
- **Deciders:** lunaInsidious, Claude
- **Tags:** architecture, language

## Context

### Problem

PLAN.md は Parser の実装技術を固定していない（§5.1）。実装言語とパッケージ構成を決める必要がある。

### Requirements

- 大規模コードベースを解析可能（§13.3）
- オフライン実行（§13.2）
- Parser / IR / Metric Engine を独立してテスト可能（§13.4）
- Phase 4 の API/UI をコア解析エンジンから分離できること

## Decision

Rust で実装し、Cargo workspace を以下の 2 クレートで構成する。

```text
crates/codestat      コアライブラリ（IR, Language Adapter, Metric Engine, 出力, 統計, モデル, レポート）
crates/codestat-cli  CLI バイナリ（引数解析, HTTP サーバ）
```

### Rationale

- tree-sitter（ADR-0002）の本体が Rust/C であり、バインディングの追加コストがない
- ネイティブ実行で大規模コードベースに対応しやすい
- コアをライブラリとして分けることで、CLI 固有・非同期ランタイム依存（HTTP サーバ）をコアに持ち込まない

## Alternatives Considered

### Python + py-tree-sitter

**Pros** 実装量が少ない、統計ライブラリが豊富
**Cons** 大規模解析で遅い、配布時に環境依存が大きい
**Rejected because:** ユーザー指定が Rust であり、性能面でも Rust が有利

### 単一クレート

**Pros** 構成が単純
**Cons** HTTP サーバ用の非同期依存がコアライブラリに入る
**Rejected because:** Metric Engine の独立性（§13.4）を依存関係のレベルで保てない

## Consequences

### Positive
- コアは同期・ネットワーク非依存のまま保てる

### Negative
- ビルド時間が長い（tree-sitter grammar の C コンパイル）

## Revision History

| Date | Status | Change |
|---|---|---|
| 2026-09-29 | Accepted | Initial |

---

# ADR-0002: Parser に tree-sitter を採用

- **Status:** Accepted
- **Date:** 2026-09-29
- **Deciders:** lunaInsidious, Claude
- **Tags:** parser

## Context

### Problem

8 言語（§4）の構文木を得る手段が必要。

### Requirements

- 複数言語に同一 API で対応
- 構文木から共通 IR を生成できる
- 同一入力に同一結果（§13.1）、オフライン（§13.2）

## Decision

`tree-sitter` クレートと各言語の公式 grammar クレート（`tree-sitter-c`, `-python`, `-typescript` など）を使う。

### Rationale

- 全対象言語の grammar が同一 API（`Node::kind()`, フィールド名）で提供される
- 具象構文木の全トークン（匿名ノード含む）を取れるため Token / Halstead 計算に使える
- grammar はクレートのバージョンで固定されるので再現性を確保できる（Parser Version として出力、§17）

## Alternatives Considered

### 言語ごとの専用パーサ（syn, rustpython-parser, swc 等）

**Pros** 言語ごとに精度が高い
**Cons** 言語ごとに API が全く異なり、Adapter の実装量が言語数に比例して大きくなる
**Rejected because:** 言語追加のコスト（§12.1）が大きい

### 自作の軽量トークナイザ（lizard 方式）

**Pros** 依存が少ない
**Cons** ネスト・関数境界の判定精度が低い、言語ごとの手書き規則が増える
**Rejected because:** 構造メトリクス（ネスト、関数）の精度を確保できない

## Consequences

### Negative
- grammar ごとにノード種名・構造が異なるため、Mapping（ADR-0004）で吸収する必要がある
- tree-sitter はエラー回復を行うため、ERROR ノードの扱いを決める必要がある（ADR-0005）

## Revision History

| Date | Status | Change |
|---|---|---|
| 2026-09-29 | Accepted | Initial |

---

# ADR-0003: Common IR の設計

- **Status:** Accepted
- **Date:** 2026-09-29
- **Deciders:** lunaInsidious, Claude
- **Tags:** IR

## Context

### Problem

PLAN §7 の IR 要件（Program / File / Function / Parameter / Node / Token / SourceRange）を具体的なデータ構造にする必要がある。
また、言語非依存にメトリクスを計算するには §7.2 の Node 種別だけでは足りないものがある。

## Decision

### Details

```text
Program  { files: Vec<File> }
File     { path, language, source, nodes: Vec<Node>, root: NodeId, tokens: Vec<Token>, functions: Vec<Function> }
Node     { id: NodeId, kind: NodeKind, parent: Option<NodeId>, children: Vec<NodeId>, range: SourceRange }
Function { id: FunctionId, name: Option<String>, parameters: Vec<Parameter>, node: NodeId, body: Option<NodeId>, range, doc: Option<SourceRange> }
Parameter{ name: Option<String>, range }
Token    { kind: TokenKind, text, range }
SourceRange { start: Position, end: Position }
Position { line (1 始まり), column (0 始まり, バイト単位), offset (バイト) }
```

- Node は File 内の配列（arena）に格納し、`NodeId` = 添字で parent / children を参照する。
- File は元のソーステキストを保持する（空行判定に使う）。
- NodeKind は §7.2 に以下を追加する（§7.2「必要に応じて追加可能」に基づく）。
  - `function`：関数の境界。関数単位メトリクスで入れ子関数を除外するために必要
  - `else`：else 節。else-if 連鎖をネストと区別するために必要
  - `logical`：短絡論理演算（&&, ||, and, or）。Cyclomatic の判定点
  - `conditional`：三項演算子。Cyclomatic の判定点
  - `catch`：例外捕捉節。Cyclomatic / ネストの判定点
  - `import`：import / include。Dependency Count に使う
- TokenKind は `keyword, identifier, literal, operator, punctuation, comment` とする。
  - §7.5 の `operand` はトークンの種別ではなく Halstead 上の分類なので、identifier と literal から導出する（Phase 2 で ADR 化）
  - §7.5 の `whitespace` はトークン化しない。空行は File のソーステキストから判定する
- Function に `doc`（ドキュメントコメントの範囲）を追加する（§7.4「少なくとも」に基づく）。Documentation Metrics で使う。
- 無名関数（ラムダ等）は `name: None`。

### Rationale

- arena 形式は参照循環を作らずに parent を持てる。Rust で扱いやすく、走査も速い
- 追加した種別はどれも「複数言語に共通して存在する概念」で、特定言語の概念ではない（§6.2）
- IR は完全な AST を目指さない（§19-3）。tree-sitter の名前付きノードのうち、Mapping にないものは `other` にする

## Alternatives Considered

### Rc<RefCell<Node>> による木構造
**Cons** parent 参照に Weak が必要で冗長、借用エラーが実行時になる
**Rejected because:** arena の方が単純

### Mapping にないノードを IR に含めない（子を親に繋ぎ替える）
**Pros** ノード数が減る
**Cons** 構造が変わるため、ネスト計算の根拠が追いにくくなる
**Rejected because:** §7.2 に `other` があり、構造を保つ方が単純

## Consequences

### Negative
- NodeKind を増やすたびに全言語の Mapping を見直す必要がある

## Revision History

| Date | Status | Change |
|---|---|---|
| 2026-09-29 | Accepted | Initial |

---

# ADR-0004: Language Adapter は汎用変換器と言語別の宣言的 Mapping で構成する

- **Status:** Accepted
- **Date:** 2026-09-29
- **Deciders:** lunaInsidious, Claude
- **Tags:** adapter, parser

## Context

### Problem

言語ごとに Adapter を手書きすると、同じ走査ロジックが言語数分だけ重複する。
§12.1 は「言語追加時は Parser / Adapter / Mapping のみ変更」を求めている。

## Decision

`LanguageAdapter` trait（`language()`, `extensions()`, `to_ir(path, source)`）を定義する。実装は tree-sitter 用の汎用変換器 1 つとし、言語ごとの差分は静的な `Mapping` テーブルだけで表現する。

```text
Mapping {
  language, extensions, grammar,
  kinds:       &[(tree-sitter ノード種名, NodeKind)]   // 載っていない名前付きノードは other
  comments:    &[ノード種名]                           // comment トークンとして扱う
  literals:    &[ノード種名]                           // 丸ごと 1 つの literal トークンとして扱う（文字列等）
  identifiers: &[ノード種名]                           // identifier トークンとして扱う
  ignored_parameters: &[テキスト]                      // 例: C の f(void)
}
```

関数の名前・引数・本体は、以下の汎用規則で取り出す。

- 名前：Mapping の `name_fields`（例：C は `declarator`、Python は `name`、TypeScript は `name`, `pattern`）を順に辿り、identifier に到達したらそれを名前とする。辿り終えたノードが identifier でなければ、その最初の identifier 子ノードを名前とする（例：Python `x: int`, `*args`）
- 引数：フィールド `parameters` を関数ノードおよび name_fields の連鎖上で探し、その名前付き子ノード（コメントを除く）を引数とする。`parameters` がなく単数の `parameter` フィールドがある場合（例：JS `x => x`）はそれを唯一の引数とする。どちらもなければ引数 0 個
- 本体：フィールド `body`
- Mapping の追加項目：`logical_operators`（`binary` ノードの `operator` フィールドがこれに一致すれば `logical`）、`default_case_keyword`（`case` ノードの先頭トークンがこれなら default ラベルとして `other`）

Token は具象構文木の葉（comment / literal は部分木ごと）から作り、以下の汎用規則で分類する。

1. `comments` に載っている → comment
2. `literals` に載っている → literal
3. `identifiers` に載っている → identifier
4. テキストに英字または `_` を含む → keyword（`int`, `return`, C の `#include` 等）
5. 区切り記号（`, ; ( ) [ ] { }`）→ punctuation
6. それ以外 → operator

### Rationale

- 言語固有の知識はテーブル上のデータに集まり、コード上の分岐にならない（§6.2, §19-1）
- 言語追加 ＝ テーブル追加となり、変換器と Metric Engine は変わらない（§12.1）
- Mapping のテストはフィクスチャで言語ごとに書ける

## Alternatives Considered

### 言語ごとに Adapter を手書き
**Pros** 言語固有の例外を自由に書ける
**Cons** 走査・トークン化のコードが言語数分重複する
**Rejected because:** 重複と、言語ごとの振る舞いの差が生まれやすい

### tree-sitter のクエリ（.scm）で抽出
**Pros** 宣言的
**Cons** クエリ言語とその文法差の理解が別途必要。NodeKind の対応だけなら表で足りる
**Rejected because:** 表で表現できる範囲に対してオーバースペック

## Consequences

### Negative
- 汎用規則で表現できない言語の癖が見つかった場合、Mapping の項目追加が必要になる。追加するときは MEMO / ADR に記録する

## Revision History

| Date | Status | Change |
|---|---|---|
| 2026-09-29 | Accepted | Initial |
| 2026-09-29 | Accepted | 実装に合わせ、名前・引数の抽出規則、keyword 判定（「含む」）、logical / default case の Mapping 項目を明記 |

---

# ADR-0005: メトリクス値の表現とエラー分類

- **Status:** Accepted
- **Date:** 2026-09-29
- **Deciders:** lunaInsidious, Claude
- **Tags:** metrics, error

## Context

### Problem

§14, §15 は「計算不能」と「値 0」の区別、エラーの種類の区別を求めている。

## Decision

- メトリクス値：`MetricValue = Available(f64) | NotApplicable | Unsupported | Error(String)`
  - 例：関数が 0 個のファイルの「平均関数長」は `NotApplicable`
- 解析エラー：`AnalysisError`（§14）。現時点で実装する種別は `UnsupportedLanguage | Parse | IrConversion`
  - §14 の `Unsupported Syntax` / `Unsupported Language Feature` は、それを発生させる Adapter の処理が現れた時点で追加する（発生源のない列挙子は作らない）
  - Metric Not Applicable / Metric Calculation Error は `MetricValue` 側で表現する
  - 構文木に ERROR / MISSING ノードがある場合は、位置を添えて `Parse` エラーとする。回復結果から部分的なメトリクスは出さない
  - 対応していない拡張子は `UnsupportedLanguage` エラーとする（言語を推測しない）
- プロジェクト（ディレクトリ）解析では、失敗したファイルを `status: error` と理由付きで結果に記録し、他のファイルの解析は続ける
- JSON 出力では `metrics: {id: number | null}` と `unavailable: {id: "not_applicable" | "unsupported" | "error: ..."}` を併記する

### Rationale

- 列挙型にすると、0 と未計算の混同が型の上で起きない
- 部分的に壊れた構文木から出した値は再現性・妥当性を保証できないため、エラーとして明示する
- ファイル単位のエラーを記録して継続するのは「隠蔽」ではなく、結果の中で失敗が明示されるため許容する

## Alternatives Considered

### Option<f64>
**Cons** not_applicable / unsupported / error を区別できない（§15 違反）
**Rejected because:** 要件を満たさない

### ERROR ノードを含んでいても解析を続ける
**Pros** 多少壊れたコードでも値が出る
**Cons** 値の意味が保証できず、失敗が成功に見える
**Rejected because:** 不要なフォールバックになる

## Consequences

### Negative
- tree-sitter grammar が未対応の新しい構文を含むファイルは解析できない。その場合はエラーメッセージで位置と原因を示す

## Revision History

| Date | Status | Change |
|---|---|---|
| 2026-09-29 | Accepted | Initial |
| 2026-09-29 | Accepted | 実装済みのエラー種別を明記 |

---

# ADR-0006: 依存クレートの方針

- **Status:** Accepted
- **Date:** 2026-09-29
- **Deciders:** lunaInsidious, Claude
- **Tags:** dependencies

## Context

### Problem

AGENTS.md は「便利そう」という理由での依存追加を禁止している。採用基準を決める必要がある。

## Decision

依存は「要件を満たすのに必要で、自前実装が不合理に大きいもの」に限る。以下を採用する。

| クレート | 用途 | 根拠 |
|---|---|---|
| tree-sitter, tree-sitter-<lang> | 構文解析 | ADR-0002 |
| serde, serde_json | JSON 出力（§11 必須） | JSON のエスケープ・直列化を自前で書く理由がない |
| clap（CLI クレートのみ） | サブコマンド・引数解析 | サブコマンドが 5 つ以上あり、手書きのヘルプ・エラー処理は大きい |
| ignore | ディレクトリ走査 | .gitignore を尊重しないと target/ や node_modules を解析してしまう（§13.3） |
| time | タイムスタンプの RFC 3339 形式（§17） | 暦計算を自前で書かない |

採用しないもの：

- thiserror / anyhow：エラー型は少数で、std の `Error` 実装で足りる
- 数値計算ライブラリ（Phase 4）：必要な統計・回帰は小規模で、自前実装が小さい（Phase 4 で改めて ADR 化）

Phase 2 以降で追加する依存（csv, axum, tokio 等）は、そのときに個別の ADR を書く。

## Revision History

| Date | Status | Change |
|---|---|---|
| 2026-09-29 | Accepted | Initial |

---

# ADR-0007: Phase 1 メトリクス（Size / Cyclomatic / Nesting）の計算方針

- **Status:** Accepted
- **Date:** 2026-09-29
- **Deciders:** lunaInsidious, Claude
- **Tags:** metrics, algorithm

## Context

### Problem

PLAN §8.1, §8.2 はメトリクス名を挙げるだけで、行の分類・判定点・ネストの数え方を定めていない。言語をまたいで同じ値になる定義が必要。

## Decision

### Details

- **行の分類**：コメント以外のトークンがある行は Code、コメントトークンだけの行は Comment、どのトークンもない空白だけの行は Blank とする。複数行にわたるトークンは、またがるすべての行を占める。
  - 空白でないのにどのトークンにも覆われない行は、IR とソースの不整合なので `Error` を返す（推測で分類しない）
- **Size の関数スコープ**はテキスト上の量なので、入れ子関数を含む。**Complexity / Nesting の関数スコープ**は入れ子関数を除く（入れ子関数は独立した Function として計算する）。
- **Cyclomatic**：関数 = 1 + 判定点の数。判定点は `branch, loop, case, catch, logical, conditional` の各ノード。ファイル = 関数の合計 + トップレベルの判定点。プロジェクト = ファイルの合計。
- **if 連鎖の継続**：親が `else` または `branch` である `branch`（`else if`, `elif`）を「継続」とする。
  - Cyclomatic では継続も判定点として 1 と数える
  - Nesting では継続は新しいレベルを作らず、連鎖の先頭と同じレベルとする
- **Nesting**：ネストを作るのは `branch`（継続を除く）, `loop`, `case`, `catch`。Max Depth は制御構造ごとの「1 + 外側の制御構造の数」の最大値。Avg Depth は文ごとの「外側の制御構造の数」の平均値。関数境界でネストはリセットする。

### Rationale

- C / TS の `else if` は「else 節の中の if」、Python の `elif` は「if の子ノード」と構文木の形が違う。親の種別だけで判定する規則にすれば、Mapping を変えずに両方を同じように扱える
- 判定点を NodeKind の集合で定義すると、言語追加時に Metric Engine を変更しなくてよい（§12.1）

## Alternatives Considered

### Mapping 側で `else if` 専用の NodeKind を作る
**Cons** C / TS では、else 節の中の if を文脈なしの表では区別できない（Mapping に文脈判定のロジックが必要になる）
**Rejected because:** 汎用変換器が複雑になる

### ネストを「ブロックの入れ子数」で数える
**Cons** 波括弧なしの本体（`for (...) x++;`）でネストを過小評価する。Python と C でブロックの付き方が異なる
**Rejected because:** 言語間で値が揃わない

## Consequences

### Negative
- C / TS で波括弧なしに if を直接入れ子にした場合（`if (a) if (b) x;`）は継続と誤認される（MetricDefinition の limitations に記載）

## Validation

- [x] 同じアルゴリズムの C / Python / TypeScript 版で Cyclomatic と Max Nesting が一致するテスト（`tests/engine.rs`）

## Revision History

| Date | Status | Change |
|---|---|---|
| 2026-09-29 | Accepted | Initial |

---

# ADR-0008: Halstead の Operator / Operand 分類

- **Status:** Accepted
- **Date:** 2026-09-29
- **Deciders:** lunaInsidious, Claude
- **Tags:** metrics, halstead

## Context

### Problem

PLAN §8.3 は「Operator / Operand の分類方法を仕様として明確に定義する」ことを求めている。Halstead の原典は言語ごとに分類が異なり、ツール間でも値が一致しない。

## Decision

IR の TokenKind だけで分類する。

| 分類 | TokenKind |
|---|---|
| Operator | `operator`, `keyword`, 開き括弧 `(` `[` `{`（punctuation） |
| Operand | `identifier`, `literal` |
| 数えない | `comment`, `,` `;` 閉じ括弧 `)` `]` `}`（punctuation） |

- 括弧は対で 1 つの operator とし、開き括弧だけを数える
- 区切り記号（`,` `;`）は数えない。Python のように文末記号がない言語との差を小さくするため
- 異なり数（n1, n2）はトークンテキストの完全一致で数える
- 派生値：n = n1 + n2、N = N1 + N2、V = N·log2(n)、D = (n1/2)·(N2/n2)、E = D·V、T = E/18（秒）、B = V/3000
- スコープ：関数（関数の範囲内のトークン。Size と同じくテキスト上の量なので入れ子関数を含む）、ファイル、プロジェクト（全ファイルのトークンを合わせて異なり数を数える）
- 分母が 0 の場合（n = 0 の V、n2 = 0 の D など）は `not_applicable`

### Rationale

- TokenKind は ADR-0004 の汎用規則で全言語同じように決まるため、Halstead も Mapping の変更なしで全言語に適用できる

## Alternatives Considered

### 言語ごとに operator 表を持つ
**Pros** 原典に近い値
**Cons** Mapping が肥大化し、言語間の定義差が大きくなる
**Rejected because:** 言語非依存性（§6）を優先

## Consequences

### Negative
- keyword を operator とするため、型名キーワード（`int` など）も operator になる（METRICS.md の limitations に記載）

## Revision History

| Date | Status | Change |
|---|---|---|
| 2026-09-29 | Accepted | Initial |

---

# ADR-0009: 重複検出の単位とアルゴリズム

- **Status:** Accepted
- **Date:** 2026-09-29
- **Deciders:** lunaInsidious, Claude
- **Tags:** metrics, duplication, algorithm

## Context

### Problem

PLAN §8.5 は「行・トークン・正規化トークン等を比較検討し、定義を固定する」ことを求めている。

## Decision

**正規化トークン列の固定長窓の一致**で検出する。

- 正規化：identifier → `$id`、literal → `$lit`、comment は除外、それ以外はテキストのまま（変数名・値だけが違う「Type-2 クローン」を検出する）
- 最小一致長：**50 トークン**（固定）
- 手順
  1. 各トークン位置から始まる 50 トークンの窓のハッシュを計算する
  2. 同じハッシュの窓を集め、実際にトークン列が等しいか確認する（ハッシュ衝突の排除）
  3. 2 回以上出現する窓に含まれるトークンを「重複トークン」とする
  4. 重複トークンが連続する最大区間を 1 つの「重複ブロック」とする
- スコープ
  - ファイル：同じファイル内での重複だけを数える（他のファイルに依存しないので、ファイル単位の値が再現可能）
  - プロジェクト：全ファイルをまたいだ重複を数える

### Rationale

- 行単位は整形（改行位置）の違いに弱く、言語ごとの行の書き方にも依存する
- 正規化しないトークン列では、変数名を変えただけのコピーを検出できない
- 50 トークンは jscpd の既定値と同じ水準で、定型の短い並び（引数リスト等）を誤検出しにくい

## Alternatives Considered

### 行単位の比較
**Rejected because:** 整形・言語の差に弱い

### AST 部分木の比較
**Pros** 構文単位で正確
**Cons** IR は完全な AST ではない（§19-3）。計算量も大きい
**Rejected because:** IR の方針と合わない

### 最小一致長を設定可能にする
**Cons** 値が設定に依存し、結果の比較（§17）が難しくなる
**Rejected because:** 再現性を優先。必要になったら定義のバージョンを上げて変更する

## Consequences

### Negative
- 同じトークンの繰り返し（長い配列リテラル等）も重複として検出される

## Revision History

| Date | Status | Change |
|---|---|---|
| 2026-09-29 | Accepted | Initial |

---

# ADR-0010: 解析結果の形式と実行メタデータ

- **Status:** Accepted
- **Date:** 2026-09-29
- **Deciders:** lunaInsidious, Claude
- **Tags:** output, reproducibility

## Context

### Problem

PLAN §11 は JSON 出力、§17 は Project / Repository / Commit / File / Language / Parser Version / Metric Definition Version / Timestamp の識別を求めている。

## Decision

```json
{
  "run": {
    "project": "codestat", "repository": "git@...", "commit": "abc123",
    "tool_version": "0.1.0", "metric_definition_version": "0.1.0",
    "parsers": {"c": "tree-sitter 0.27.0 / tree-sitter-c 0.24.2"},
    "timestamp": "2026-09-29T00:00:00Z"
  },
  "project": {"metrics": {...}, "unavailable": {...}},
  "files": [{
    "path": "src/a.c", "language": "c", "status": "ok",
    "metrics": {"size.loc": 10, "size.comment_ratio": null},
    "unavailable": {"size.comment_ratio": "not_applicable"},
    "functions": [{"name": "f", "start_line": 1, "end_line": 5, "metrics": {...}, "unavailable": {...}}]
  }, {
    "path": "src/b.c", "language": "c", "status": "error", "error": "src/b.c:3:1: parse error: ..."
  }]
}
```

- メトリクス名は定義 ID（`size.loc` 等）をそのまま使う
- `metrics` の値は数値か `null`。`null` の理由は `unavailable` に `not_applicable` / `unsupported` / `error: <message>` で書く
- `repository` / `commit` は解析対象ディレクトリの git 情報。git 管理外なら `null`（推測しない）
- Parser Version は grammar クレートのバージョンを Mapping に定数で持ち、`Cargo.lock` と一致するかをテストで検査する
- 解析対象：指定されたディレクトリを `.gitignore` を尊重して走査し、対応拡張子のファイルだけを対象とする。明示的に指定したファイルが未対応拡張子ならエラー
- パース失敗ファイルは `status: "error"` で記録し、プロジェクト集計には含めない
- ファイル順はパスの辞書順（再現性、§13.1）

### Rationale

- ID をそのまま使うと、定義書（METRICS.md）と出力が 1 対 1 で対応する
- 値と理由を分けると、数値列だけを取り出して統計処理しやすい（Phase 4）

## Alternatives Considered

### 各値を `{"value": 1, "status": "available"}` にする
**Cons** 出力が冗長で、統計処理の前に展開が必要
**Rejected because:** 大半の値は available なので、例外だけを別に書く方が扱いやすい

## Revision History

| Date | Status | Change |
|---|---|---|
| 2026-09-29 | Accepted | Initial |
