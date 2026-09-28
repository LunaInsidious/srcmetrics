# MEMO — 開発メモ・調査記録

このドキュメントは **決定には至っていない情報**や
**後で振り返りたい知見**を残すためのメモ置き場である。

- 正確さよりも「忘れないこと」を優先
- 未検証・仮説・違和感を書いてよい
- 決定事項になったら ADR に昇格させる

---

## YYYY-MM-DD: タイトル（自由）

### 背景
なぜこのメモを書いたか。
調査・実装中に気づいたことなど。

---

### 内容 / 観察結果
- 調査した事実
- heapsnapshot の構造メモ
- 試して分かった挙動
- DevTools と実データの差異

※ 未確認事項はその旨を明記すること。

---

### 気になる点 / TODO
- 今は対応しないが、将来問題になりそうな点
- 実装を進める上での懸念

---

### 備考
- 関連するコード位置
- 関連する Issue / ADR 番号

---

## 2026-09-29: Phase 1 Adapter 実装時の観察

### 背景
C / Python / TypeScript の Mapping を作成した際に気づいた、言語間の差異と既知の制約。

### 内容 / 観察結果
- ~~文字列リテラルは部分木ごと 1 トークンにしている。そのため f-string / template string 内の埋め込み式はトークンにならない。~~ → Phase 1 レビューで、埋め込み式が IR ノードごと消えて Complexity 等も過小評価されると判明。`interpolations` で解消（ADR-0004 改訂）。
- Python のメソッドの `self` / `cls` は引数として数える。Java 等の暗黙の `this` とは数え方が異なる（Parameter Count は Partially Language Dependent）。
- Python `match` の `case _:` はワイルドカードだが `case` として数える（C の `default` と異なり、構文上は通常の case と区別されないため）。
- Python の docstring は `expression_statement` なので Statement として数えられる。
- C の `for (int i = 0; ...)` の初期化子は `declaration` として Statement に数えられる。Python の `for` には相当する宣言がない。

### 気になる点 / TODO
- Documentation（Phase 3）で、Python docstring をドキュメントとして扱う方法を決める必要がある。
