# Architecture Decision Records

srcmetrics の設計判断の記録。1 件 1 ファイル（`NNNN.md`）。新しい判断は [template.md](template.md) の形式で追加し、下の一覧にも行を足す。

ADR-0001〜0023 に出てくる「PLAN §x」は、廃止した初期要求仕様 PLAN.md の節を指す（ADR-0024）。原文は git 履歴（コミット 5812948 の `PLAN.md`）で参照できる。

| ADR | 決定 | Status |
|---|---|---|
| [0001](0001.md) | 実装言語 Rust と Cargo workspace 構成 | Accepted |
| [0002](0002.md) | Parser に tree-sitter を採用 | Accepted |
| [0003](0003.md) | Common IR の設計 | Accepted |
| [0004](0004.md) | Language Adapter は汎用変換器と言語別の宣言的 Mapping で構成する | Accepted |
| [0005](0005.md) | メトリクス値の表現とエラー分類 | Accepted |
| [0006](0006.md) | 依存クレートの方針 | Accepted |
| [0007](0007.md) | Phase 1 メトリクス（Size / Cyclomatic / Nesting）の計算方針 | Accepted |
| [0008](0008.md) | Halstead の Operator / Operand 分類 | Accepted |
| [0009](0009.md) | 重複検出の単位とアルゴリズム | Accepted |
| [0010](0010.md) | 解析結果の形式と実行メタデータ | Accepted |
| [0011](0011.md) | 制御構造の個数と Number of Paths の定義 | Accepted |
| [0012](0012.md) | 呼び出し関係（Dependency Metrics）は名前ベースで解決する | Accepted |
| [0013](0013.md) | ドキュメントの判定 | Accepted |
| [0014](0014.md) | Cognitive Complexity の IR 上の定義 | Accepted |
| [0015](0015.md) | 派生メトリクス（Maintainability Index と正規化値）は標準メトリクスから計算する | Accepted |
| [0016](0016.md) | CSV 出力 | Accepted |
| [0017](0017.md) | 対応言語の追加（Go, Java, JavaScript, Rust, C++） | Accepted |
| [0018](0018.md) | 統計分析（記述統計・言語別ベースライン・相関） | Accepted |
| [0019](0019.md) | 可読性モデルは利用者のラベルから学習する実験的コンポーネントとする | Accepted |
| [0020](0020.md) | 可視化は自己完結の HTML レポートにする | Accepted |
| [0021](0021.md) | API / UI は CLI クレートの最小 HTTP サーバとする | Accepted |
| [0022](0022.md) | 公開名を srcmetrics とし、MIT OR Apache-2.0 で公開する | Accepted |
| [0023](0023.md) | 利用者向けドキュメントを VitePress で GitHub Pages に公開し、開発者向け文書と分ける | Accepted |
| [0024](0024.md) | 初期要求仕様 PLAN.md を廃止し、設計原則を SPEC.md に移す | Accepted |
| [0025](0025.md) | メトリクス定義の日本語版をコードに持たせ、日本語の定義ページも生成する | Accepted |
