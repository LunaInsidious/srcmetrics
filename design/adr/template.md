# ADR テンプレート

新しい ADR は `NNNN.md`（4 桁の連番）としてこのディレクトリに追加する。以下の形式に従う。

## ADR-NNNN: [タイトル]

- **Status:** Proposed
- **Date:** YYYY-MM-DD
- **Deciders:** [決定者 / チーム]
- **Tags:** [architecture, parser, IR, performance...]

### Context

#### Problem

[何が問題なのか、なぜ意思決定が必要なのか]

#### Background

[意思決定に関係する前提・制約・既存実装など]

#### Requirements

- [要件1]
- [要件2]
- [要件3]

### Decision

[最終的に何を採用するか]

#### Details

[決定内容の具体的な説明]

```text
[必要に応じて構成図・データ構造・処理フローなど]
```

#### Rationale

[なぜこの決定にしたのか]

- [理由1]
- [理由2]
- [理由3]

### Alternatives Considered

#### [Alternative 1]

[候補の概要]

**Pros**
- [メリット]

**Cons**
- [デメリット]

**Rejected because:** [採用しなかった理由]

#### [Alternative 2]

[候補の概要]

**Pros**
- [メリット]

**Cons**
- [デメリット]

**Rejected because:** [採用しなかった理由]

### Consequences

#### Positive

- [この決定によって得られるメリット]
- [改善される点]

#### Negative

- [新たに発生するコスト・制約]
- [複雑になる点]

#### Risks

- [想定されるリスク]
- [将来的に問題になる可能性]

### Compatibility / Migration

[既存実装との互換性や移行方法]

- [変更が必要な箇所]
- [互換性への影響]
- [移行手順]

### Implementation Notes

[実装時に重要な注意事項]

- [注意点1]
- [注意点2]

### Validation

[この決定が妥当であることをどう検証するか]

- [ ] [テスト・ベンチマーク]
- [ ] [実装検証]
- [ ] [互換性検証]

### References

- [関連Issue / PR]
- [関連仕様]
- [参考資料]

### Revision History

| Date | Status | Change |
|---|---|---|
| YYYY-MM-DD | Proposed | Initial proposal |
| | | |
