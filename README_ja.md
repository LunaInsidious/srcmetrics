# srcmetrics

[English](https://github.com/LunaInsidious/srcmetrics/blob/main/README.md) | 日本語

C, C++, Go, Java, JavaScript, Python, Rust, TypeScript のソースコードを、言語に依存しない定義で計測するツール・ライブラリです。

規模、制御構造の複雑さ、ネスト、Halstead 尺度、重複、呼び出し関係、ドキュメントを**全言語で同じ定義**で計測し、各メトリクスを**個別に**出力します。単一の「可読性スコア」ではなく、統計分析に使える生の特徴量を提供します。計算できない値は 0 ではなく、理由付きの `null` になります。

**ドキュメント：https://lunainsidious.github.io/srcmetrics/ja/**

## インストール

```sh
cargo install srcmetrics-cli   # `srcmetrics` コマンド
cargo add srcmetrics           # ライブラリ
```

## クイックスタート

```sh
srcmetrics analyze src -o result.json          # プロジェクト・ファイル・関数ごとのメトリクス（JSON。--format csv も可）
srcmetrics report result.json -o report.html   # 自己完結の HTML レポート
```

```rust
let result = srcmetrics::analyze::analyze_source("example.py", "def f(x):\n    return x if x else 0\n")?;
```

## リンク

- [はじめに](https://lunainsidious.github.io/srcmetrics/ja/guide/getting-started)
- [メトリクス定義（英語）](https://lunainsidious.github.io/srcmetrics/metrics/)
- [対応言語と制約](https://lunainsidious.github.io/srcmetrics/ja/guide/languages)
- [API ドキュメント（docs.rs）](https://docs.rs/srcmetrics)
- 開発者向け文書：[design/](https://github.com/LunaInsidious/srcmetrics/tree/main/design)

## ライセンス

[Apache License, Version 2.0](https://github.com/LunaInsidious/srcmetrics/blob/main/LICENSE-APACHE) と [MIT license](https://github.com/LunaInsidious/srcmetrics/blob/main/LICENSE-MIT) のいずれかを選択できます。

明示的に別段の意思表示をしない限り、あなたが本プロジェクトへの取り込みを意図して提出した貢献は、Apache-2.0 ライセンスの定義に従い、追加の条件なしに上記のデュアルライセンスで提供されるものとします。
