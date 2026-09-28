---
layout: home

hero:
  name: srcmetrics
  text: 言語に依存しないソースコードメトリクス
  tagline: 規模・複雑さ・ネスト・Halstead・重複・依存関係・ドキュメントを、C, C++, Go, Java, JavaScript, Python, Rust, TypeScript で同じ定義で計測します。
  actions:
    - theme: brand
      text: はじめる
      link: /ja/guide/getting-started
    - theme: alt
      text: メトリクス定義（英語）
      link: /metrics/
    - theme: alt
      text: GitHub
      link: https://github.com/LunaInsidious/srcmetrics

features:
  - title: 1 つの定義を全言語に
    details: tree-sitter で構文解析したコードを共通の中間表現に変換し、すべてのメトリクスをそこから計算します。同じアルゴリズムを 8 言語で書くと、Cyclomatic・Cognitive Complexity が同じ値になります。
  - title: スコアではなく生の特徴量
    details: 関数・ファイル・プロジェクトの各スコープで 53 種類のメトリクスを個別に出力し、統計分析に使えます。組み込みの「可読性スコア」はありません。
  - title: 正直な値
    details: 当てはまらない値は 0 ではなく null とし、理由を併記します。構文解析に失敗したファイルも黙って捨てずに報告します。
  - title: 再現できる
    details: 解析したコミット、パーサのバージョン、メトリクス定義のバージョンを結果に記録します。
  - title: CLI・ライブラリ・HTTP API
    details: JSON / CSV の出力、統計、自己完結の HTML レポート。Rust ライブラリとしても使えます。
  - title: 自前のラベルで学習
    details: 人間による可読性の評価があれば、それを使って実験的なリッジ回帰モデルを学習できます。
---
