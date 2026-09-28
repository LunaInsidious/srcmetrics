# 実験的な可読性モデル

srcmetrics には**組み込みの重みがなく**、既定で可読性スコアを計算することもありません。各メトリクスと人間が感じる可読性との関係は、事前には分からないためです。人間による評価値があれば、それを使ってモデルを学習できます。

## ラベル

ヘッダが `path,function,score` の CSV ファイルです。

```text
path,function,score
src/parser.py,parse_header,3.5
src/parser.py,parse_body,2
src/util.py,,4
```

- `path` は解析結果に出てくるパスです。
- `function` は関数名です。ファイル全体を評価するなら空にします。1 つのモデルのラベルは、すべて同じ種類（ファイルか関数）にします。
- 各ラベルは、ちょうど 1 つのファイルまたは関数に対応する必要があります。存在しないパス、解析に失敗したファイル、同じファイル内で名前が重複する関数はエラーです。
- `score` は有限の数値で、すべてが同じ値ではいけません。

## 学習

```sh
srcmetrics analyze src -o result.json
srcmetrics model train --labels labels.csv -o model.json result.json
```

モデルは、標準化した特徴量によるリッジ回帰です。

- 特徴量は既定で、ラベルのスコープのすべてのメトリクスです。ラベル付きのどれかの単位で `null` のもの、値が一定のものは除外し、理由をモデルファイルに記録します。`--features id1,id2` で明示的に選ぶこともでき、その場合はすべて使える特徴量である必要があります。
- `--lambda` は正則化の強さです（既定 1.0）。
- モデルファイルには、特徴量、係数、学習データでの R²、5 分割交差検証の RMSE（ラベルが 5 件以上のとき）が記録されます。

## 予測

```sh
srcmetrics model predict --model model.json result.json
```

モデルのスコープに応じて、すべてのファイルまたは関数のスコアを出力します。特徴量が `null` の単位や、解析に失敗したファイルは `score: null` と理由になります。
