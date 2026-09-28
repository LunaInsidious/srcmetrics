<!-- Generated from crates/srcmetrics/src/metrics. Do not edit; run `UPDATE_DOCS=1 cargo test -p srcmetrics --test docs`. -->

# メトリクス定義

各メトリクスは、下の表のスコープで、ID ごとに個別に出力されます。計算できない値は 0 ではなく `null` になり、理由が `unavailable` に入ります。

メトリクス定義のバージョン：`0.1.0`（すべての解析結果に記録されます）。

| メトリクス | 名前 | スコープ | 単位 |
|---|---|---|---|
| [`size.loc`](./size#size-loc) | LOC | file, project | lines |
| [`size.sloc`](./size#size-sloc) | SLOC | function, file, project | lines |
| [`size.comment_loc`](./size#size-comment-loc) | コメント行数 | file, project | lines |
| [`size.blank_loc`](./size#size-blank-loc) | 空行数 | file, project | lines |
| [`size.comment_ratio`](./size#size-comment-ratio) | コメント率 | file, project | ratio |
| [`size.statement_count`](./size#size-statement-count) | 文の数 | function, file, project | count |
| [`size.token_count`](./size#size-token-count) | トークン数 | function, file, project | count |
| [`size.function_count`](./size#size-function-count) | 関数の数 | file, project | count |
| [`size.function_length`](./size#size-function-length) | 関数の長さ | function | lines |
| [`size.avg_function_length`](./size#size-avg-function-length) | 平均関数長 | file, project | lines |
| [`size.max_function_length`](./size#size-max-function-length) | 最大関数長 | file, project | lines |
| [`complexity.cyclomatic`](./complexity#complexity-cyclomatic) | Cyclomatic Complexity | function, file, project | count |
| [`complexity.branch_count`](./complexity#complexity-branch-count) | 分岐の数 | function, file, project | count |
| [`complexity.conditional_count`](./complexity#complexity-conditional-count) | 三項演算子の数 | function, file, project | count |
| [`complexity.loop_count`](./complexity#complexity-loop-count) | ループの数 | function, file, project | count |
| [`complexity.return_count`](./complexity#complexity-return-count) | return の数 | function, file, project | count |
| [`complexity.jump_count`](./complexity#complexity-jump-count) | ジャンプの数 | function, file, project | count |
| [`complexity.path_count`](./complexity#complexity-path-count) | 経路数 | function | count |
| [`complexity.cognitive`](./complexity#complexity-cognitive) | Cognitive Complexity | function, file, project | count |
| [`nesting.max_depth`](./nesting#nesting-max-depth) | 最大ネスト深さ | function, file, project | levels |
| [`nesting.avg_depth`](./nesting#nesting-avg-depth) | 平均ネスト深さ | function, file, project | levels |
| [`halstead.unique_operators`](./halstead#halstead-unique-operators) | 演算子の種類数 | function, file, project | count |
| [`halstead.unique_operands`](./halstead#halstead-unique-operands) | 被演算子の種類数 | function, file, project | count |
| [`halstead.total_operators`](./halstead#halstead-total-operators) | 演算子の総数 | function, file, project | count |
| [`halstead.total_operands`](./halstead#halstead-total-operands) | 被演算子の総数 | function, file, project | count |
| [`halstead.vocabulary`](./halstead#halstead-vocabulary) | 語彙数 | function, file, project | count |
| [`halstead.length`](./halstead#halstead-length) | プログラム長 | function, file, project | count |
| [`halstead.volume`](./halstead#halstead-volume) | Volume | function, file, project | bits |
| [`halstead.difficulty`](./halstead#halstead-difficulty) | Difficulty | function, file, project | ratio |
| [`halstead.effort`](./halstead#halstead-effort) | Effort | function, file, project | count |
| [`halstead.time`](./halstead#halstead-time) | 推定プログラミング時間 | function, file, project | seconds |
| [`halstead.bugs`](./halstead#halstead-bugs) | 推定バグ数 | function, file, project | count |
| [`function.parameter_count`](./function#function-parameter-count) | 引数の数 | function | count |
| [`function.avg_parameter_count`](./function#function-avg-parameter-count) | 平均引数数 | file, project | count |
| [`function.max_parameter_count`](./function#function-max-parameter-count) | 最大引数数 | file, project | count |
| [`function.expression_count`](./function#function-expression-count) | 式の数 | function, file, project | count |
| [`function.call_count`](./function#function-call-count) | 呼び出しの数 | function, file, project | count |
| [`duplication.duplicate_block_count`](./duplication#duplication-duplicate-block-count) | 重複ブロック数 | file, project | count |
| [`duplication.duplicate_token_count`](./duplication#duplication-duplicate-token-count) | 重複トークン数 | file, project | count |
| [`duplication.duplication_ratio`](./duplication#duplication-duplication-ratio) | 重複率 | file, project | ratio |
| [`duplication.max_duplicate_length`](./duplication#duplication-max-duplicate-length) | 最長の重複 | file, project | tokens |
| [`dependency.fan_out`](./dependency#dependency-fan-out) | Fan-out | function | count |
| [`dependency.fan_in`](./dependency#dependency-fan-in) | Fan-in | function | count |
| [`dependency.call_depth`](./dependency#dependency-call-depth) | 呼び出しの深さ | function | calls |
| [`dependency.dependency_count`](./dependency#dependency-dependency-count) | 依存の数 | file, project | count |
| [`documentation.doc_loc`](./documentation#documentation-doc-loc) | ドキュメント行数 | function | lines |
| [`documentation.documented_function_count`](./documentation#documentation-documented-function-count) | ドキュメントのある関数の数 | file, project | count |
| [`documentation.documentation_ratio`](./documentation#documentation-documentation-ratio) | ドキュメント率 | file, project | ratio |
| [`maintainability.index`](./maintainability#maintainability-index) | Maintainability Index | function, file | index |
| [`derived.cyclomatic_per_function`](./derived#derived-cyclomatic-per-function) | 関数あたりの Cyclomatic Complexity | file, project | ratio |
| [`derived.tokens_per_loc`](./derived#derived-tokens-per-loc) | LOC あたりのトークン数 | file, project | ratio |
| [`derived.statements_per_function`](./derived#derived-statements-per-function) | 関数あたりの文の数 | file, project | ratio |
| [`derived.duplicate_tokens_per_sloc`](./derived#derived-duplicate-tokens-per-sloc) | SLOC あたりの重複トークン数 | file, project | ratio |
