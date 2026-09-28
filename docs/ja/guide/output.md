# 出力形式

## JSON

```json
{
  "run": {
    "project": "readme",
    "repository": "git@github.com:you/project.git",
    "commit": "3f2c…",
    "tool_version": "0.1.0",
    "metric_definition_version": "0.1.0",
    "parsers": { "python": "tree-sitter 0.27.0 / tree-sitter-python 0.25.0" },
    "timestamp": "2026-09-29T00:00:00Z"
  },
  "project": { "metrics": { "size.loc": 4.0 }, "unavailable": {} },
  "files": [
    {
      "status": "ok",
      "path": "greet.py",
      "language": "python",
      "metrics": { "size.loc": 4.0, "size.comment_ratio": 0.0 },
      "unavailable": {},
      "functions": [
        {
          "name": "greet",
          "start_line": 1,
          "end_line": 4,
          "metrics": { "complexity.cyclomatic": 2.0, "nesting.max_depth": 1.0 },
          "unavailable": {}
        }
      ]
    },
    {
      "status": "error",
      "path": "broken.c",
      "language": "c",
      "error": "broken.c:1:12: parse error: …"
    }
  ]
}
```

（抜粋です。`metrics` には、そのスコープで定義されたすべてのメトリクスが入ります。）

### `run`

| フィールド | |
|---|---|
| `project` | `--project` の値、または解析したディレクトリ名 |
| `repository`, `commit` | 解析したパスの git の `origin` URL と `HEAD`。git 管理外なら `null` |
| `tool_version` | srcmetrics のバージョン |
| `metric_definition_version` | メトリクス定義のバージョン。計算方法が変わると変わる |
| `parsers` | 結果に含まれる言語ごとのパーサ・文法のバージョン |
| `timestamp` | RFC 3339、UTC |

### メトリクスの値と `null`

`metrics` は[メトリクス ID](/ja/metrics/) から数値または `null` への対応です。`null` には必ず `unavailable` に理由があります。

| 理由 | 意味 |
|---|---|
| `not_applicable` | 当てはまらない（例：関数のないファイルの平均関数長） |
| `unsupported` | この入力では提供していない |
| `error: …` | 計算できなかった。メッセージに理由 |

計算できない値を 0 として出すことはありません。

### ファイルと関数

- ファイルはパス順です。`status` は `ok` か `error` で、失敗したファイルはメトリクスの代わりに `error` を持ち、プロジェクトの集計には含まれません。
- 関数にはメソッド、入れ子の関数、ラムダも含まれます。無名関数の `name` は `null` です。行番号は 1 始まりで、両端を含みます。

## CSV

`srcmetrics analyze --format csv` は、スコープごとに 1 行を出力します。

```text
scope,path,language,function,start_line,end_line,status,error,<メトリクス ID…>
project,,,,,,,,…
file,src/a.py,python,,,,ok,,…
function,src/a.py,python,greet,1,4,ok,,…
file,src/broken.c,c,,,,error,"src/broken.c:1:12: parse error: …",…
```

メトリクスの列は `srcmetrics metrics` の順です。`null` は空のセルになり、理由は JSON 出力にだけあります。
