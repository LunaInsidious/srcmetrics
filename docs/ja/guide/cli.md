# コマンドリファレンス

```text
srcmetrics <COMMAND>

  analyze   ファイル・ディレクトリを解析
  stats     記述統計・言語別ベースライン・相関
  report    自己完結の HTML レポート
  model     実験的な可読性モデル（train / predict）
  serve     HTTP API と Web UI
  metrics   メトリクス定義
```

エラー時は終了コード 1 と、次に何をすべきかを示すメッセージを出します。

## analyze

```sh
srcmetrics analyze <PATH> [--project NAME] [-o FILE] [--format json|csv]
```

- `PATH` はファイルかディレクトリです。ディレクトリは `.gitignore` を尊重して走査し、[対応する拡張子](./languages)のファイルだけを解析します。未対応の拡張子のファイルを直接指定するとエラーです。
- `--project` は結果に記録するプロジェクト名です（既定はディレクトリ名）。
- `--format csv` はプロジェクト・ファイル・関数ごとに 1 行を出力します。[出力形式](./output)を参照してください。
- 構文解析に失敗したファイルは結果に `"status": "error"` として残り、件数とエラーが標準エラーに出ます。終了コードは 0 のままです。

## stats

```sh
srcmetrics stats <RESULT.json>... [--scope file|function]
```

1 つ以上の結果のファイルまたは関数について、次を計算します。

- メトリクスごと：`n`（値のある単位の数）、`missing`、`mean`、`sd`（不偏）、`min`、`q1`、`median`、`q3`、`max`。分位点は線形補間（R の type 7）。
- 言語ごとの同じ統計（`by_language`）。言語別のベースラインとして使えます。
- すべてのメトリクスの組の Pearson と Spearman の相関。両方の値がある単位だけを使い、使った単位数 `n` も出します。3 単位未満や分散 0 の場合は `null` です。

## report

```sh
srcmetrics report <RESULT.json> [-o FILE]
```

実行メタデータ、プロジェクトのメトリクス、言語別の中央値、全メトリクスのヒストグラム、Spearman の相関ヒートマップを 1 つの HTML ファイルに書き出します。外部リソースを読み込まないので、オフラインで開け、1 ファイルで共有できます。

## model

```sh
srcmetrics model train --labels LABELS.csv [--features IDS] [--lambda L] [-o MODEL.json] <RESULT.json>...
srcmetrics model predict --model MODEL.json <RESULT.json>...
```

[実験的な可読性モデル](./model)を参照してください。

## serve

```sh
srcmetrics serve [--bind 127.0.0.1] [--port 8080]
```

[HTTP API](./http-api) を参照してください。

## metrics

```sh
srcmetrics metrics [--format json|markdown]
```

すべてのメトリクス定義（id、名前、説明、定義、スコープ、入力、計算方法、単位、言語依存性、制約、参考文献）を出力します。同じ内容が[メトリクス定義](/ja/metrics/)にあります。
