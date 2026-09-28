# はじめに

## インストール

コマンドラインツール：

```sh
cargo install srcmetrics-cli   # `srcmetrics` コマンドが入ります
```

Rust ライブラリ：

```sh
cargo add srcmetrics
```

Rust 1.85 以上が必要です。文法定義は C からコンパイルされるため、ビルド時に C コンパイラが必要です。解析はすべてオフラインで動きます。

## プロジェクトを解析する

```sh
srcmetrics analyze src -o result.json
```

`analyze` はディレクトリを `.gitignore` を尊重して走査し、[対応する拡張子](./languages)のファイルをすべて解析します。結果は、プロジェクト・各ファイル・各関数のメトリクスを含む 1 つの JSON になります。形式は[出力形式](./output)を参照してください。

構文解析に失敗したファイルは、結果に `"status": "error"` として残り、標準エラーにも一覧が出ます。

## 結果を見る

```sh
srcmetrics report result.json -o report.html        # 表・ヒストグラム・相関ヒートマップ
srcmetrics stats result.json --scope function       # 統計（JSON）
srcmetrics analyze src --format csv > metrics.csv   # プロジェクト・ファイル・関数ごとに 1 行
```

各メトリクスの意味は[メトリクス定義](/ja/metrics/)にあります。

## 次に読む

- [コマンドリファレンス](./cli)
- [ライブラリとして使う](./library)
- [対応言語と制約](./languages)
