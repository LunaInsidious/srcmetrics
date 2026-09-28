# ソースコード可読性メトリクス分析システム 要求仕様書

## 1. 文書概要

### 1.1 目的

本システムは、複数のプログラミング言語で記述されたソースコードから、可読性・理解容易性・保守性に関連する定量的なメトリクスを収集・算出することを目的とする。

特定のプログラミング言語に依存した評価を可能な限り避け、共通の抽象化を通じて、異なる言語のソースコードから同種の指標を取得できることを基本方針とする。

本システムは、単一の「可読性スコア」を直接算出することを主目的としない。複数のメトリクスを独立した特徴量として提供し、将来的な統計分析、相関分析、可読性モデルの構築等に利用できることを重視する。

---

## 2. 背景・目的

ソースコードの可読性は、コード量、構造的複雑性、制御構造、語彙量、重複、関数の大きさなど、複数の要因によって構成される。

既存の静的解析ツールには特定言語向けのものや、ルール違反の検出を主目的とするものが多い。一方、本システムでは、特定のコーディング規約への適合性ではなく、ソースコードそのものから取得できる定量的特徴量を広く収集することを目的とする。

そのため、以下を基本方針とする。

- 複数言語に対応する
- 言語固有の構文・機能への依存を可能な限り減らす
- メトリクスを個別の数値として提供する
- メトリクスの定義と計算方法を明確にする
- 新しい言語やメトリクスを追加しやすい構造とする
- 後から別の評価モデルを構築できるよう、生の特徴量を保持する

---

## 3. スコープ

### 3.1 対象

本システムでは、ソースコードを解析し、以下のような情報を定量化する。

- ソースコードの規模
- コメント・ドキュメント量
- 制御構造の複雑性
- ネストの深さ
- 関数の複雑性・規模
- 演算子・オペランドの構成
- コードの重複
- 関数呼び出し・依存関係
- その他、言語横断的に定義可能な可読性関連指標

### 3.2 対象外

初期バージョンでは、以下を主要な評価対象としない。

- 特定言語固有の高度な型システムの複雑性
- 特定言語固有のフレームワーク依存性
- 特定言語のコーディング規約違反
- セキュリティ脆弱性の検出
- バグの検出
- 実行時性能の評価
- 人間による主観的な可読性評価そのもの
- 単一の「正解となる可読性スコア」の算出

ただし、将来的に拡張できる構造とする。

---

## 4. 対応言語

初期対応候補として以下を想定する。

- C
- C++
- Go
- Python
- Java
- JavaScript
- TypeScript
- Rust

対応言語は将来的に追加可能とする。

特定言語への対応処理は、共通のメトリクス計算部分から分離する。

---

## 5. 基本アーキテクチャ

システムは概ね以下の構成とする。

```text
Source Code
    │
    ▼
Language Parser
    │
    ▼
Language Adapter
    │
    ▼
Common Intermediate Representation (IR)
    │
    ├── Size Metrics
    ├── Complexity Metrics
    ├── Nesting Metrics
    ├── Halstead Metrics
    ├── Function Metrics
    ├── Duplication Metrics
    ├── Dependency Metrics
    └── Documentation Metrics
    │
    ▼
Metric Results
    │
    ▼
JSON / CSV / API 等
```

### 5.1 Parser

各言語のソースコードを構文解析し、言語固有の構文木を生成する。

Parserの具体的な実装技術は固定しないが、複数言語に対応可能で、構文木から共通IRを生成できることを要件とする。

### 5.2 Language Adapter

各言語の構文木を共通IRへ変換する。

言語固有の構文・ASTノード名・構文上の差異は、可能な限りこの層に閉じ込める。

### 5.3 Common IR

メトリクス計算に必要な情報を、言語非依存の形式で表現する。

IRはプログラムの意味を完全に表現することを目的とせず、メトリクス算出に必要な最小限の共通情報を提供する。

### 5.4 Metric Engine

Common IRを入力として各種メトリクスを計算する。

メトリクス計算処理はLanguage Adapterから独立させる。

---

## 6. 言語非依存性

### 6.1 基本方針

メトリクスは以下の3段階に分類する。

| 分類 | 意味 |
|---|---|
| Language Independent | 複数言語で同一またはほぼ同一の定義を適用可能 |
| Partially Language Dependent | 共通概念として扱えるが、言語による差異が存在 |
| Language Specific | 特定言語の機能・概念を前提とする |

標準的な可読性特徴量には、原則としてLanguage IndependentおよびPartially Language Dependentの指標を使用する。

Language Specificな指標は、拡張機能として扱える構造を想定する。

### 6.2 言語固有情報の隔離

メトリクス計算処理が、

- TypeScript固有のASTノード名
- C++固有のテンプレート構文
- Rust固有のライフタイム
- Java固有の継承構造

などを直接参照することを避ける。

---

## 7. Common IR 要件

### 7.1 基本構成

Common IRでは、少なくとも以下の概念を表現できるものとする。

```text
Program
File
Function
Parameter
Node
Token
SourceRange
```

### 7.2 Node

Nodeはソースコード中の構造要素を表現する。

基本的なNode種別として以下を想定する。

```text
block
statement
expression
declaration

branch
loop
case
return
jump

call
assignment
binary
unary

identifier
literal
other
```

必要に応じて追加可能とする。

### 7.3 Node属性

各Nodeは少なくとも以下を持つ。

```text
id
kind
parent
children
source range
```

### 7.4 Function

Functionは以下を持つ。

```text
id
name
parameters
body
source range
```

メトリクス値そのものはFunctionに保持せず、Metric Engineから算出する。

### 7.5 Token

Tokenは語彙・演算子関連のメトリクスに利用する。

基本的な分類として以下を想定する。

```text
operator
operand
keyword
identifier
literal
punctuation
comment
whitespace
```

### 7.6 Source Range

ソースコード上の位置を表現する。

```text
start
end
```

各Positionは少なくとも以下を持つ。

```text
line
column
offset
```

Source Rangeは、LOC、関数長、文長、重複検出等の計算に利用する。

---

## 8. メトリクス要件

メトリクスは単一のスコアに統合せず、原則として個別の値として提供する。

### 8.1 Size Metrics

初期候補：

- LOC
- SLOC
- Comment LOC
- Blank LOC
- Comment Ratio
- Statement Count
- Token Count
- Function Count
- Function Length
- Average Function Length
- Maximum Function Length

### 8.2 Complexity Metrics

初期候補：

- Cyclomatic Complexity
- Branch Count
- Conditional Count
- Loop Count
- Return Count
- Jump Count
- Number of Paths
- Maximum Nesting Depth
- Average Nesting Depth

### 8.3 Halstead Metrics

初期候補：

- Unique Operators
- Unique Operands
- Total Operators
- Total Operands
- Vocabulary
- Program Length
- Volume
- Difficulty
- Effort
- Estimated Program Time
- Estimated Bugs

Halstead系メトリクスについては、OperatorおよびOperandの分類方法を仕様として明確に定義する。

### 8.4 Function Metrics

初期候補：

- Parameter Count
- Function Length
- Statement Count
- Expression Count
- Complexity
- Maximum Nesting Depth
- Call Count
- Return Count

### 8.5 Duplication Metrics

初期候補：

- Duplicate Block Count
- Duplicate Token Count
- Duplication Ratio
- Maximum Duplicate Length

重複判定の単位は、行・トークン・正規化トークン等を比較検討し、定義を固定する。

### 8.6 Dependency Metrics

初期候補：

- Call Count
- Fan-in
- Fan-out
- Dependency Count
- Call Depth

依存関係の完全な意味解析が困難な言語では、取得可能な範囲を明示する。

### 8.7 Documentation Metrics

初期候補：

- Comment LOC
- Comment Ratio
- Documentation Count
- Documentation Ratio

言語固有のドキュメント構文については、共通のComment/Documentation概念へ可能な範囲で変換する。

---

## 9. メトリクス定義

各メトリクスについて、以下の仕様情報を管理する。

```text
Metric ID
Name
Description
Definition
Scope
Input
Calculation
Unit
Language Applicability
Limitations
Reference
```

例えば、

```yaml
id: complexity.cyclomatic
name: Cyclomatic Complexity
scope: function
unit: count
requires:
  - control_flow
```

のように、計算方法を明示可能とする。

同一名称のメトリクスであっても、言語によって計算方法が異なる場合は、その差異を明示する。

---

## 10. 可読性スコア

初期バージョンでは、複数のメトリクスを任意の重みで統合した単一の可読性スコアを標準機能として提供しない。

理由：

- 各メトリクスと人間の可読性との関係を事前に決定できない
- メトリクス間に相関が存在する可能性がある
- 言語・プロジェクト・開発者によって適切な重みが異なる可能性がある
- 将来的な統計分析のため、生の特徴量を保持する方が有用

将来的には、実験的なスコアリングモデルを別コンポーネントとして追加できるようにする。

---

## 11. 出力要件

解析結果は機械的に利用可能な形式で出力できるものとする。

最低限、JSON形式をサポートする。

例：

```json
{
  "language": "typescript",
  "file": "example.ts",
  "metrics": {
    "loc": 120,
    "sloc": 92,
    "comment_ratio": 0.18,
    "function_count": 8,
    "avg_function_length": 11.5,
    "cyclomatic": 14,
    "max_nesting": 4,
    "halstead_volume": 1250.4,
    "avg_parameter_count": 2.1,
    "duplication_ratio": 0.03
  }
}
```

ファイル単位だけでなく、可能な範囲で以下のスコープをサポートする。

```text
Token
Statement
Function
Class/Module 等
File
Project
```

ただし、Class等の言語依存性が高いスコープについては、共通IRで十分に抽象化できる場合に限る。

---

## 12. 拡張性

### 12.1 言語追加

新しい言語への対応では、原則として以下のみを追加・変更する。

```text
Parser
Language Adapter
Language-specific mapping
```

既存のMetric Engineを変更せずに、新しい言語を追加できることを目標とする。

### 12.2 メトリクス追加

新しいメトリクスはMetric Engineへ独立したCalculatorとして追加できる構造とする。

```text
Common IR
    │
    ├── SizeCalculator
    ├── ComplexityCalculator
    ├── HalsteadCalculator
    ├── DuplicationCalculator
    └── NewMetricCalculator
```

既存のParserおよび他のCalculatorへの影響を最小限にする。

---

## 13. 非機能要件

### 13.1 再現性

同一のソースコード、同一のParserバージョン、同一のメトリクス定義に対して、原則として同一の結果を返す。

### 13.2 オフライン実行

基本的なソースコード解析およびメトリクス計算は、外部ネットワークへの接続を必要とせず実行可能とする。

### 13.3 パフォーマンス

大規模なコードベースを解析可能な構成とする。

ただし、初期バージョンでは絶対的な処理時間を要求値として固定せず、実測値を収集して後から目標値を設定する。

### 13.4 保守性

Parser、IR、Metric Engineを分離し、各コンポーネントを独立してテスト可能とする。

### 13.5 テスト容易性

各メトリクスについて、既知の入力と期待値を用いた単体テストを作成可能とする。

---

## 14. エラー処理

以下の状況を区別して扱う。

```text
Parser Error
Unsupported Syntax
Unsupported Language Feature
IR Conversion Error
Metric Not Applicable
Metric Calculation Error
```

あるメトリクスを計算できない場合、可能であれば `null` または明示的な未適用状態として出力し、0と混同しない。

---

## 15. メトリクスの適用範囲

各メトリクスについて、適用可能な範囲を記録する。

例：

```text
available
not_applicable
unsupported
error
```

これにより、ある言語で値が存在しないことと、値が0であることを区別する。

---

## 16. 正規化

言語間比較を行う場合、絶対値だけでなく必要に応じて以下の正規化値を提供できる構造とする。

例：

```text
complexity / function
tokens / LOC
statements / function
comments / LOC
duplication / SLOC
```

ただし、正規化方法自体が分析結果に影響するため、標準メトリクスと派生メトリクスを区別する。

---

## 17. データ保存

解析結果は、後から統計分析できる形式で保存可能とする。

最低限、以下を識別できるものとする。

```text
Project
Repository
Commit / Version
File
Language
Parser Version
Metric Definition Version
Timestamp
```

これにより、異なるバージョンの解析結果を比較できるようにする。

---

## 18. 今後の拡張候補

初期スコープ外だが、以下を将来的な拡張候補とする。

- Cognitive Complexity
- Maintainability Index
- より高度な重複検出
- コールグラフ
- モジュール間依存関係
- Cohesion
- Coupling
- Code Churn
- Git履歴との統合
- 人間による可読性評価データとの統合
- 機械学習による可読性モデル
- プロジェクト・言語別の統計的ベースライン
- 可視化
- Web API
- Web UI

---

## 19. 設計上の原則

本システムでは以下を基本原則とする。

1. **言語固有の処理はLanguage Adapterに閉じ込める**
2. **Metric EngineはCommon IRだけを参照する**
3. **IRは完全なASTの共通化を目指さない**
4. **メトリクスは可能な限り独立したCalculatorとして実装する**
5. **可読性を単一の数値に早期に集約しない**
6. **メトリクスの定義・計算方法・適用範囲を明示する**
7. **計算不能と値0を区別する**
8. **生のメトリクスを保持し、後から再分析可能にする**
9. **言語追加時に既存Metric Engineを変更しないことを目標とする**
10. **同一入力に対する解析結果の再現性を確保する**

---

## 20. 初期実装の優先順位

### Phase 1

- Parser基盤
- C / TypeScript / Python 等の複数言語対応
- Common IR
- Source Range
- Token
- Function
- Size Metrics
- Cyclomatic Complexity
- Nesting Metrics

### Phase 2

- Halstead Metrics
- Function Metrics
- Duplication Metrics
- JSON出力
- メトリクス定義管理

### Phase 3

- Dependency Metrics
- Documentation Metrics
- Cognitive Complexity
- Maintainability Index
- より多くの対応言語

### Phase 4

- 統計分析
- 可読性モデル
- 可視化
- API / UI

---

## 21. 成果物

初期バージョンでは以下を成果物とする。

- ソースコード解析エンジン
- Common IR
- Language Adapter
- Metric Engine
- メトリクス定義
- 対応言語Parser
- 単体テスト
- メトリクス計算結果のJSON出力
- 技術仕様書
- メトリクス定義書

