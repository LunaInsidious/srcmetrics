# ライブラリ

```sh
cargo add srcmetrics
```

## ソース文字列を解析する

```rust
use srcmetrics::analyze::analyze_source;
use srcmetrics::result::FileResult;

fn main() -> Result<(), srcmetrics::error::AnalysisError> {
    let result = analyze_source("example.py", "def f(x):\n    return x if x else 0\n")?;
    let FileResult::Ok { functions, .. } = &result.files[0] else { unreachable!() };
    assert_eq!(functions[0].metrics.metrics["complexity.cyclomatic"], Some(2.0));
    Ok(())
}
```

ファイル名の拡張子で言語が決まります。構文エラーは位置付きの `AnalysisError::Parse` として返ります。

## ディレクトリを解析する

```rust
let result = srcmetrics::analyze::analyze(std::path::Path::new("src"), None)?;
let json = serde_json::to_string_pretty(&result).unwrap();
```

`AnalysisResult` は `Serialize` と `Deserialize` を実装しており、その JSON は CLI の[出力形式](./output)と同じです。

## 主な API

| API | 用途 |
|---|---|
| `analyze::analyze(path, project)` | ファイル・ディレクトリを解析して `AnalysisResult` を返す |
| `analyze::analyze_source(filename, source)` | メモリ上のソース文字列を解析 |
| `lang::adapter_for_path(path)?.to_ir(path, source)` | 共通の中間表現（`ir::File`）に変換 |
| `metrics::compute(&program)` | 中間表現から全メトリクスを計算 |
| `metrics::definitions()` | メトリクス定義 |
| `csv::to_csv`, `report::to_html` | CSV・HTML 出力 |
| `stats::Report::of` | ファイル・関数の統計 |
| `model::train`, `model::predict_all` | 実験的モデル |

API の詳細は [docs.rs](https://docs.rs/srcmetrics) にあります。
