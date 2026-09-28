# HTTP API と Web UI

```sh
srcmetrics serve [--bind 127.0.0.1] [--port 8080]
```

既定では `127.0.0.1` だけで待ち受け、**認証はありません**。ローカルでの利用を想定しています。`--port 0` で空いているポートを使い、アドレスは標準エラーに出ます。

| エンドポイント | |
|---|---|
| `GET /` | Web UI：コードを貼り付け、ファイル名を付けてメトリクスを表示 |
| `POST /api/analyze` | 1 ファイルを解析 |

## `POST /api/analyze`

```sh
curl -X POST http://127.0.0.1:8080/api/analyze \
  -H 'Content-Type: application/json' \
  -d '{"filename": "a.py", "source": "def f(x):\n    return x\n"}'
```

`filename` の拡張子で言語が決まります。応答は、1 ファイルを含む[出力形式](./output)の解析結果です。`repository` と `commit` は `null` になります。

未対応の拡張子と構文エラーは、ステータス 400 と `{"error": "…"}` を返します。
