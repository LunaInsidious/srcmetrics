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
Node     { id: NodeId, kind: NodeKind, parent: Option<NodeId>, children: Vec<NodeId>, range: SourceRange, label: Option<String> }
Function { id: FunctionId, name: Option<String>, parameters: Vec<Parameter>, node: NodeId, body: Option<NodeId>, range, doc: Option<SourceRange> }
Parameter{ name: Option<String>, range }
Token    { kind: TokenKind, text, range }
SourceRange { start: Position, end: Position }
Position { line (1 始まり), column (0 始まり, バイト単位), offset (バイト) }
```

- Node は File 内の配列（arena）に前順（親が子より先）で格納し、`NodeId` = 添字で parent / children を参照する。
- `label` は種別ごとの付加情報（`call` の呼び出し先名、`logical` の演算子。ADR-0012）。
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
| 2026-09-29 | Accepted | Node に `label` を追加（ADR-0012）。arena が前順であることを明記 |

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
  literals:    &[ノード種名]                           // literal トークンとして扱う（文字列等）。部分木は走査しない
  interpolations: &[ノード種名]                        // literal 内に埋め込まれたコード（`${...}`, f-string の `{...}`）。通常のコードとして走査する
  identifiers: &[ノード種名]                           // identifier トークンとして扱う
  ignored_parameters: &[テキスト]                      // 例: C の f(void)
}
```

関数の名前・引数・本体は、以下の汎用規則で取り出す。

- 関数名：Mapping の `name_fields`（例：C は `declarator`、Python は `name`、TypeScript は `name`, `pattern`）を順に辿り、identifier に到達したらそれを名前とする。到達しなければ無名
- 引数名：関数名と同じ規則で辿り、到達しなければ、辿り終えたノードの子のうち「identifier であるか name_fields を持つ」最後のものを名前とする（型が先に来る書き方に対応：Python `x: int`, `*args`、Java `String... xs`）
- 引数：Mapping の `parameter_fields`（既定は `parameters`。Go は `receiver`, `parameters`、JS/TS は `parameters`, `parameter`）を順に、関数ノードおよび name_fields の連鎖上で探す。フィールドのノードが identifier ならそれが 1 つの引数（JS `x => x`、Java `x -> ...`）、それ以外なら名前付き子ノード（コメントを除く）が引数。どれもなければ引数 0 個
- 本体：フィールド `body`
- 1 つの引数宣言に `name` フィールドが複数ある場合（Go の `a, b int`）は、名前ごとに 1 つの引数とする
- else 節のノードがない grammar（Go, Java）：Mapping の `else_field`（`alternative`）にある子で、それ自体が `branch` でないものを `else` とする
- Mapping の追加項目：`callee_fields`（呼び出しノードの呼び出し先フィールド。その中の最後の identifier の葉を呼び出し先名とする。ADR-0012）、`logical_operators`（`binary` ノードの `operator` フィールドがこれに一致すれば `logical`）、`default_case_keyword`（`case` ノードの最初の葉トークンがこれなら default ラベルとして `other`）

Token は具象構文木の葉（comment / literal は部分木ごと）から作り、以下の汎用規則で分類する。

1. `comments` に載っている → comment
2. `literals` に載っている → literal。ただし `interpolations` の子があれば、その前後の文字列部分をそれぞれ 1 つの literal トークンとし、埋め込みコードは通常どおり IR ノードとトークンにする
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
| 2026-09-29 | Accepted | Phase 1 レビュー指摘：template string / f-string 内のコードが IR から消えていたため `interpolations` を追加。幅 0 の葉はトークンにしない |
| 2026-09-29 | Accepted | Go 追加：`else_field` と、複数名の引数宣言の規則を追加 |
| 2026-09-29 | Accepted | Java 追加：関数名と引数名の規則を分離、単一 identifier の引数、default 判定を「最初の葉」に変更 |
| 2026-09-29 | Accepted | Phase 3 レビュー指摘：Go のメソッドのレシーバ（明示的な受け手）が引数に数えられていなかったため `parameter_fields` を追加 |

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
| serde, serde_json | JSON 出力（§11 必須） | JSON のエスケープ・直列化を自前で書く理由がない。`float_roundtrip` 機能を有効にし、書き出した数値を読み戻したとき完全に同じ値になるようにする（結果・モデルの再現性、§13.1） |
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

## Implementation Notes

- C 系の `else if` の連鎖は IR 上で腕ごとに 1 段ずつ深くなる（Branch → Else → Branch …）。ネストのレベルは祖先をたどらず、arena の前順（親が先）を利用して 1 回の走査で全ノード分を計算する（Phase 2 レビュー指摘：祖先をたどる実装は数百の腕で O(N²) になる）

## Validation

- [x] 同じアルゴリズムの C / Python / TypeScript 版で Cyclomatic と Max Nesting が一致するテスト（`tests/engine.rs`）
- [x] 2 万本の腕を持つ `else if` の連鎖を線形時間で処理するテスト（`tests/engine.rs`）

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

---

# ADR-0011: 制御構造の個数と Number of Paths の定義

- **Status:** Accepted
- **Date:** 2026-09-29
- **Deciders:** lunaInsidious, Claude
- **Tags:** metrics, complexity, algorithm

## Context

### Problem

PLAN §8.2 の Branch / Conditional / Loop / Return / Jump Count と Number of Paths の定義を決める必要がある。
Nejmeh の NPATH は「条件式」と「then / else の本体」を区別して計算するが、IR（ADR-0003）はノードの役割（フィールド）を持たない。

## Decision

- **個数**：それぞれ NodeKind の個数とする（関数スコープは入れ子関数を除く。ファイルはファイル全体、プロジェクトは合計）
  - Branch Count = `branch` + `case`（if / else-if / elif と、default 以外の case ラベル）
  - Conditional Count = `conditional`（三項演算子）
  - Loop Count = `loop`、Return Count = `return`、Jump Count = `jump`（break, continue, goto, throw / raise）
- **Number of Paths**（`complexity.path_count`、関数スコープのみ）：ループを 0 回または 1 回通るとしたときの、非循環な実行経路の数。IR の構造だけから以下で計算する
  - 通常のノード：子ノードの経路数の積（子がなければ 1）
  - if 連鎖の先頭 `branch`：then 部分（`else` と継続 `branch` 以外の子の積）＋ else 部分（`else` 子の経路数、継続 `branch` 子の経路数、どちらもなければ 1）
  - `loop`：子の積 ＋ 1（通らない経路）
  - `conditional`（三項演算子）：子の積 ＋ 1（2 つの値のどちらかを通る）
  - 兄弟の `case` の連続：各 case の経路数の和 ＋ 1（どの case にも当たらない経路／default）
  - 兄弟の `catch` の連続：各 catch の経路数の和 ＋ 1（例外が起きない経路）
  - 短絡演算子・return / jump による経路の打ち切りは考慮しない
- Number of Paths は Nejmeh の NPATH とは別物として扱い、名前にも NPATH を使わない

### Rationale

- 役割情報を IR に追加せずに、「分岐は足し算、並びは掛け算」という経路数の本質を表せる
- NPATH を名乗ると既存ツールと同じ値を期待されるため、別名にして定義書で違いを明示する

## Alternatives Considered

### IR に子ノードの役割（condition / consequence / alternative）を追加して NPATH を正確に実装する
**Pros** 既存の NPATH と比較できる
**Cons** 全言語の Mapping にフィールド名の対応を追加する必要があり、IR が AST に近づく（§19-3）
**Rejected because:** 現時点では得られる利点がコストに見合わない。必要になったら ADR を追加する

### Number of Paths を実装しない
**Rejected because:** 簡単な構造規則で意味のある値を出せるため

## Consequences

### Negative
- 短絡演算子による経路は数えない。三項演算子の 2 つの値の経路数は区別しない（子の積 ＋ 1 で近似）

## Implementation Notes

- 経路数は arena の逆順（子が先）に 1 回走査して全ノード分を計算する。再帰しないので、深い `else if` の連鎖でもスタックを溢れさせない（Phase 2 レビュー指摘）

## Revision History

| Date | Status | Change |
|---|---|---|
| 2026-09-29 | Accepted | Initial |

---

# ADR-0012: 呼び出し関係（Dependency Metrics）は名前ベースで解決する

- **Status:** Accepted
- **Date:** 2026-09-29
- **Deciders:** lunaInsidious, Claude
- **Tags:** metrics, dependency, IR

## Context

### Problem

PLAN §8.6 の Fan-in / Fan-out / Call Depth には「どの関数を呼んでいるか」が必要だが、IR の `call` ノードは呼び出し先を持たない。PLAN §8.6 は「完全な意味解析が困難な言語では、取得可能な範囲を明示する」としている。

## Decision

- IR の Node に `label: Option<String>` を追加する（ADR-0003 の拡張）。意味は種別ごとに決める
  - `call`：呼び出し先の名前。Mapping の `callee_field`（多くの grammar で `function`）が指すノードの中の、**最後の identifier トークン**（`f(x)` → `f`、`a.b.c()` → `c`）
  - `logical`：演算子のテキスト（`&&`, `or` 等。ADR-0014 で使う）
  - その他：`None`
- 呼び出しは**名前だけで**解決する。型・スコープ・import は見ない
- メトリクス（関数スコープ。入れ子関数の呼び出しは入れ子関数のもの）
  - Fan-out：その関数が呼ぶ異なる名前の数（プロジェクト外の関数も含む）
  - Fan-in：その関数の名前を呼んでいる、プロジェクト内の異なる関数の数。同名の関数は区別できないので同じ値になる
  - Call Depth：プロジェクト内の関数の「名前」を頂点とするグラフ（ある名前の頂点は、その名前の全関数が呼ぶ名前へ辺を持つ）で、最長の呼び出し連鎖の辺の数。再帰（循環）は強連結成分にまとめ、成分の中の辺は数えない。無名関数は頂点にならず、1 ＋ 呼んでいる名前の最大の深さ
  - 頂点を関数ではなく名前にするのは、同名の関数が多い場合（`get`, `run` 等）に「呼び出し × 同名関数」の数の辺ができて計算量が 2 乗になるのを避けるため（Phase 3 レビュー指摘）。名前で解決する以上、同名の関数は区別できないので、意味も変わらない
- Dependency Count（ファイル / プロジェクト）：`import` ノードの数
- Call Count は Function Metrics の `function.call_count` を使い、重複定義しない

### Rationale

- 名前ベースなら全言語で同じ規則で計算でき、Mapping の追加は `callee_field` だけで済む
- 強連結成分でまとめると、循環があっても Call Depth が一意に決まり、計算量も O(関数 + 呼び出し)

## Alternatives Considered

### 言語ごとの意味解析（型解決・import 解決）
**Rejected because:** 言語ごとに大きな実装が必要で、§6 の言語非依存方針に反する

### Call Depth を DFS で「訪問済みは 0」として計算
**Cons** 循環があると、たどる順番によって値が変わる
**Rejected because:** 再現性（§13.1）はあるが、値の意味が説明できない

## Consequences

### Negative
- 同名のメソッド（例：多数のクラスの `run`）は区別されず、Fan-in が過大になる
- 関数ポインタ・コールバック経由の呼び出しは、呼び出し先の名前が関数名と一致しない限り追えない

## Revision History

| Date | Status | Change |
|---|---|---|
| 2026-09-29 | Accepted | Initial |
| 2026-09-29 | Accepted | Call Depth のグラフの頂点を関数名に変更（Phase 3 レビュー指摘：同名関数が多いと辺が 2 乗に増える） |

---

# ADR-0013: ドキュメントの判定

- **Status:** Accepted
- **Date:** 2026-09-29
- **Deciders:** lunaInsidious, Claude
- **Tags:** metrics, documentation, adapter

## Context

### Problem

PLAN §8.7 は言語固有のドキュメント構文を共通の Documentation 概念に変換することを求めている。

## Decision

Adapter が `Function.doc`（ドキュメントの範囲）を設定する。

- 汎用規則：関数の直前にあるコメントトークンの連続（間に空行を挟まない）で、最後のコメントが関数の開始行の前の行か同じ行で終わるもの。ただし、同じ行のコードの後ろにある末尾コメントは含めない
- Mapping の `decorators`：関数とドキュメントの間に書かれるノード型（Python の `decorator`、Rust の `attribute_item`、TS/JS の `decorator`）。これらを飛ばして、その上のコメントを探す
  - `/** */`, `///`, `//`, `#` を区別しない（どのコメント構文も「関数の説明」として書かれうるため）
- Mapping の `docstring: Option<ノード型>`：指定がある言語（Python の `string`）では、関数本体の最初の文がそのリテラルだけの式文なら、それをドキュメントとする。直前コメントより docstring を優先する
- メトリクス
  - Documentation Count（ファイル / プロジェクト）：ドキュメントのある関数の数
  - Documentation Ratio（ファイル / プロジェクト）：Documentation Count / 関数の数（関数がなければ not_applicable）
  - Documentation LOC（関数）：ドキュメントの行数（なければ 0）
  - Comment LOC / Comment Ratio は Size Metrics の `size.comment_loc` / `size.comment_ratio` を使い、重複定義しない

### Rationale

- 直前コメントという規則は C / Go / Java / JS / TS / Rust に共通して使える
- Python の慣習的なドキュメントは docstring なので、それだけを Mapping のフラグで扱う

## Alternatives Considered

### ドキュメント専用の構文（`/**`, `///`）だけを数える
**Cons** Go や C のように通常のコメントで関数を説明する慣習の言語で 0 になる
**Rejected because:** 言語間で比較できない

## Consequences

### Negative
- 行末コメントを使う言語で、関数の直前の行にある「前の文の行末コメント」は正しく除外されるが、コメントだけの行が前の関数の説明として書かれていても、次の関数のドキュメントと判定されうる

## Revision History

| Date | Status | Change |
|---|---|---|
| 2026-09-29 | Accepted | Initial |

---

# ADR-0014: Cognitive Complexity の IR 上の定義

- **Status:** Accepted
- **Date:** 2026-09-29
- **Deciders:** lunaInsidious, Claude
- **Tags:** metrics, complexity

## Context

### Problem

SonarSource の Cognitive Complexity を IR（NodeKind）だけで計算する規則を決める必要がある。

## Decision

関数ごと（入れ子関数は含めない）に以下を足す。`level` は関数内で外側にある「ネストを作る構造」の数（`branch` の連鎖の先頭、`loop`、`case`、`catch`、`conditional`）。

| 構造 | 加算 |
|---|---|
| if 連鎖の先頭 `branch` | 1 + level |
| 継続 `branch`（else if / elif） | 1 |
| `else`（中身が継続 `branch` でないもの） | 1 |
| `loop`, `catch`, `conditional` | 1 + level |
| 連続する兄弟 `case` の並び（= 1 つの switch） | 1 + level（並びごとに 1 回） |
| `logical` | 1（親が同じ演算子（label）の `logical` なら 0。`a && b && c` は 1、`a && b \|\| c` は 2） |
| 自分と同じ名前を呼ぶ `call`（再帰） | 1 |

- ファイル：関数の合計。プロジェクト：ファイルの合計

### Rationale

- 原典の「構造による加算」「ネストによる加算」「else 等の平坦な加算」を、既存の NodeKind と if 連鎖の規則（ADR-0007）で表現できる

## Alternatives Considered

### 原典どおり、入れ子関数（ラムダ）の複雑さを外側の関数に含める
**Cons** 他のメトリクス（Cyclomatic 等）は入れ子関数を別に数えており、一貫しない
**Rejected because:** 本システム内の一貫性を優先。定義書に差分を明記する

## Consequences

### Negative
- ラベル付き break / continue、goto の加算は、IR が `jump` のラベルを持たないため行わない

## Revision History

| Date | Status | Change |
|---|---|---|
| 2026-09-29 | Accepted | Initial |

---

# ADR-0015: 派生メトリクス（Maintainability Index と正規化値）は標準メトリクスから計算する

- **Status:** Accepted
- **Date:** 2026-09-29
- **Deciders:** lunaInsidious, Claude
- **Tags:** metrics, derived

## Context

### Problem

Maintainability Index（MI）と §16 の正規化値は、他のメトリクス（Halstead Volume, Cyclomatic, SLOC 等）の組み合わせで決まる。各 Calculator は互いに独立させる方針（§12.2）なので、これらをどこで計算するかを決める必要がある。

## Decision

Metric Engine を 2 段階にする。

1. 標準メトリクス：各 Calculator が IR から計算する（従来どおり）
2. 派生メトリクス：標準メトリクスの値（`Metrics` の表）だけを入力とする関数で、各スコープ（関数・ファイル・プロジェクト）の表に追加する

- 派生メトリクスは IR を見ない。入力のどれかが available でなければ not_applicable（入力が error なら error）
- MI：`maintainability.index = 171 − 5.2·ln(V) − 0.23·CC − 16.2·ln(SLOC)`（V = halstead.volume, CC = complexity.cyclomatic, SLOC = size.sloc）。V と SLOC が正のときだけ計算する。関数・ファイルのスコープ
  - このため `size.sloc` を関数スコープにも追加する
- 正規化値（§16）は ID を `derived.` で始め、標準メトリクスと区別する
  - `derived.cyclomatic_per_function` = complexity.cyclomatic / size.function_count
  - `derived.tokens_per_loc` = size.token_count / size.loc
  - `derived.statements_per_function` = size.statement_count / size.function_count
  - `derived.duplicate_tokens_per_sloc` = duplication.duplicate_token_count / size.sloc
  - スコープはファイルとプロジェクト

### Rationale

- 計算式が標準メトリクスの値の関数として明示され、定義書の Input 欄と一致する
- Calculator 間の依存（実行順序）を作らずに済む

## Alternatives Considered

### MI の Calculator が Halstead / Cyclomatic / SLOC を IR から再計算する
**Cons** 同じ計算が 2 か所で走り、定義が食い違う危険がある
**Rejected because:** 値の出どころを 1 つにする方が安全

## Revision History

| Date | Status | Change |
|---|---|---|
| 2026-09-29 | Accepted | Initial |

---

# ADR-0016: CSV 出力

- **Status:** Accepted
- **Date:** 2026-09-29
- **Deciders:** lunaInsidious, Claude
- **Tags:** output

## Context

### Problem

PLAN §5 の出力に CSV がある。統計ツール（表計算、R、pandas）に直接読ませたい。

## Decision

- 1 行 = 1 つのスコープの値（project / file / function）。列は `scope, path, language, function, start_line, end_line, status, error` と、全メトリクス ID（定義順）
- 値が null のセルは空。理由は JSON 出力の `unavailable` を参照する（CSV には書かない）
- 解析に失敗したファイルは `status=error` と `error` 列だけを埋めた行
- CSV のエスケープ（`,` `"` 改行を含むセルを `"` で囲み、`"` を `""` にする）は自前で書く。csv クレートは使わない（書き出しのみで、必要な処理が 10 行程度のため。ADR-0006）

## Alternatives Considered

### スコープごとに別ファイル
**Cons** 1 コマンドで複数ファイルを出すと扱いが面倒
**Rejected because:** scope 列で絞り込めば足りる

## Revision History

| Date | Status | Change |
|---|---|---|
| 2026-09-29 | Accepted | Initial |

---

# ADR-0017: 対応言語の追加（Go, Java, JavaScript, Rust, C++）

- **Status:** Accepted
- **Date:** 2026-09-29
- **Deciders:** lunaInsidious, Claude
- **Tags:** parser, languages

## Context

### Problem

PLAN §4 の残りの言語に対応する。

## Decision

各言語の公式 tree-sitter grammar クレート（tree-sitter-go, -java, -javascript, -rust, -cpp）を追加し、ADR-0004 の Mapping だけで対応する。Metric Engine は変更しない。

- 各言語について `tests/fixtures/equivalence/classify.<ext>` を追加し、既存の等価テスト（Cyclomatic / Max Nesting が全言語で一致）に含める
- 汎用規則で表せない言語の癖が出た場合は、Mapping 項目の追加として ADR-0004 を改訂する

## Consequences

### Negative
- C++ はテンプレート・マクロにより C と同様に解析できないファイルがある（MEMO の C の事例と同じ）

## Revision History

| Date | Status | Change |
|---|---|---|
| 2026-09-29 | Accepted | Initial |

---

# ADR-0018: 統計分析（記述統計・言語別ベースライン・相関）

- **Status:** Accepted
- **Date:** 2026-09-29
- **Deciders:** lunaInsidious, Claude
- **Tags:** phase4, statistics

## Context

### Problem

PLAN §20 Phase 4 の「統計分析」と §18 の「プロジェクト・言語別の統計的ベースライン」を実現する。入力は解析結果（ADR-0010 の JSON）とする。

## Decision

- 入力：1 つ以上の解析結果 JSON。分析単位（unit）はファイルまたは関数（`--scope`）
- メトリクスごとの記述統計：n（値のある単位の数）、missing（null の数）、mean、標準偏差（不偏、n − 1）、min、Q1、median、Q3、max
  - 分位点は線形補間（R の type 7 と同じ）
  - n = 0 の統計値は null。n = 1 の標準偏差は null
- 言語別ベースライン：上記を言語ごとにも計算する
- 相関行列：全メトリクスの組について Pearson と Spearman（順位は同順位を平均順位にする）
  - 両方の値がある単位だけを使う（pairwise complete）。使った単位数も出力する
  - 単位数が 3 未満、またはどちらかの分散が 0 なら null
- 出力：JSON
- 数値計算ライブラリは使わず自前で実装する（平均・分位点・順位・相関は数十行で、依存を増やす理由がない。ADR-0006）

### Rationale

- 生の特徴量を保持したまま（§19-8）、後段の分析に必要な要約を再現可能に出す
- pairwise complete は、メトリクスごとに not_applicable の出方が違う（例：関数のないファイル）ため

## Alternatives Considered

### 欠損のある単位を全メトリクスから除く（listwise）
**Cons** 関数のないファイルが 1 つでもあると、関数系メトリクスのせいで他の相関からもその単位が消える
**Rejected because:** データを無駄に捨てる

## Revision History

| Date | Status | Change |
|---|---|---|
| 2026-09-29 | Accepted | Initial |

---

# ADR-0019: 可読性モデルは利用者のラベルから学習する実験的コンポーネントとする

- **Status:** Accepted
- **Date:** 2026-09-29
- **Deciders:** lunaInsidious, Claude
- **Tags:** phase4, model

## Context

### Problem

PLAN §20 Phase 4 の「可読性モデル」。一方 PLAN §10 は、任意の重みによる単一スコアを標準機能にしないとしている。人間による可読性評価データは本リポジトリにない。

## Decision

- **既定の重み・既定のスコアは持たない**。モデルは、利用者が与えたラベル（人間の評価値など）から学習する場合にだけ作られる
- 学習：リッジ回帰（特徴量は z 標準化、切片は正則化しない）。正規方程式をガウスの消去法（部分ピボット）で解く
  - 正則化の強さ `lambda` は引数で指定する（既定値 1.0。値はモデルファイルに記録）
  - 特徴量：指定したスコープ（file / function）のメトリクス。`--features` で指定しなければ、そのスコープの全メトリクスから「ラベル付きの単位のどれかで null のもの」「分散が 0 のもの」を除いたもの。除いた特徴量とその理由はモデルファイルに記録する
- 評価：学習データでの R²、k 分割交差検証（k = 5、単位の並び順で決定的に分割）の RMSE
- 出力：モデル JSON（特徴量、平均、標準偏差、係数、切片、lambda、評価値、除外した特徴量、実験的であることの注記）
- 予測：モデル JSON と解析結果から、各単位のスコアを出す。特徴量が null の単位の予測値は null（理由付き）
- ラベルの形式：CSV（`path,function,score`。`function` が空ならファイル単位）。関数は `path` と `function`（名前）で照合し、同じファイルに同名の関数が複数あればエラー
- CSV の読み込みには `csv` クレートを使う（引用符・改行を含むセルを正しく読むのは自前の書き出し（ADR-0016）より複雑で、誤読すると学習結果が静かに壊れるため）

### Rationale

- §10 の「事前に重みを決められない」という理由をそのまま尊重しつつ、§18 の「人間による可読性評価データとの統合」「機械学習による可読性モデル」の入口を用意する
- リッジ回帰は係数が解釈でき、メトリクス間の強い相関（多重共線性）にも安定する

## Alternatives Considered

### 文献の重みを使った既定スコア
**Rejected because:** §10 に反する

### 勾配ブースティング等の非線形モデル
**Cons** 依存と実装量が大きく、係数の解釈ができない
**Rejected because:** 実験の入口としては線形モデルで足りる

## Revision History

| Date | Status | Change |
|---|---|---|
| 2026-09-29 | Accepted | Initial |

---

# ADR-0020: 可視化は自己完結の HTML レポートにする

- **Status:** Accepted
- **Date:** 2026-09-29
- **Deciders:** lunaInsidious, Claude
- **Tags:** phase4, visualization

## Context

### Problem

PLAN §20 Phase 4 の「可視化」。オフライン実行（§13.2）を守る必要がある。

## Decision

- 解析結果 JSON から、1 つの HTML ファイルを生成する（CSS・SVG をインライン。外部の JavaScript・CSS・フォントを読み込まない）
- 内容：実行メタデータ、プロジェクトのメトリクス表、メトリクスごとのヒストグラム（SVG）、相関ヒートマップ（Spearman、SVG）、言語別の中央値表
- 統計値は ADR-0018 の実装を使う
- グラフ描画ライブラリは使わず、SVG を文字列として組み立てる

### Rationale

- ファイル 1 つなので、オフラインで開け、共有も簡単
- 必要な図は棒と矩形だけなので、ライブラリは不要

## Alternatives Considered

### Chart.js 等を CDN から読み込む
**Rejected because:** オフラインで表示できない

## Revision History

| Date | Status | Change |
|---|---|---|
| 2026-09-29 | Accepted | Initial |

---

# ADR-0021: API / UI は CLI クレートの最小 HTTP サーバとする

- **Status:** Accepted
- **Date:** 2026-09-29
- **Deciders:** lunaInsidious, Claude
- **Tags:** phase4, api, ui, dependencies

## Context

### Problem

PLAN §20 Phase 4 の「API / UI」。範囲が広がりやすいので最小限に絞る。

## Decision

- `codestat serve [--bind 127.0.0.1] [--port 8080]`（既定はローカルのみで待ち受け）
- エンドポイント
  - `POST /api/analyze`：`{"filename": "a.py", "source": "..."}` → 1 ファイルの解析結果（ADR-0010 と同じ形式）。言語は filename の拡張子で決める
  - `GET /api/metrics`：メトリクス定義
  - `GET /`：ソースを貼り付けて解析結果を表示する 1 ページの UI（HTML は CLI バイナリに埋め込む）
- エラー：未対応の拡張子・解析エラーは 400 と `{"error": "..."}`
- コアに「ディスクを読まずにソース文字列を解析する」関数を追加し、ディレクトリ解析と同じ計算経路を使う
- 依存：`axum` と `tokio` を CLI クレートにだけ追加する（ADR-0001：コアを非同期ランタイムから切り離す）

### Rationale

- axum は tokio 上の定番で、ルーティング・JSON の処理が少ないコードで書ける
- 認証やプロジェクト管理は扱わない（ローカルでの利用を想定）

## Alternatives Considered

### std の TcpListener で HTTP を自前実装
**Cons** HTTP の解析・エラー処理を自前で持つことになり、不具合の温床になる
**Rejected because:** 依存を 1 つ増やす方が安全

## Revision History

| Date | Status | Change |
|---|---|---|
| 2026-09-29 | Accepted | Initial |
