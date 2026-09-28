# Metrics

Every metric is reported separately under its id, at the scopes listed below. A value that cannot be computed is `null`, with the reason in `unavailable`; it is never 0.

When a calculation changes, the metric definition version changes; every analysis result records the version it was computed with (`run.metric_definition_version`).

| Metric | Name | Scopes | Unit |
|---|---|---|---|
| [`size.loc`](./size#size-loc) | LOC | file, project | lines |
| [`size.sloc`](./size#size-sloc) | SLOC | function, file, project | lines |
| [`size.comment_loc`](./size#size-comment-loc) | Comment LOC | file, project | lines |
| [`size.blank_loc`](./size#size-blank-loc) | Blank LOC | file, project | lines |
| [`size.comment_ratio`](./size#size-comment-ratio) | Comment Ratio | file, project | ratio |
| [`size.statement_count`](./size#size-statement-count) | Statement Count | function, file, project | count |
| [`size.token_count`](./size#size-token-count) | Token Count | function, file, project | count |
| [`size.function_count`](./size#size-function-count) | Function Count | file, project | count |
| [`size.function_length`](./size#size-function-length) | Function Length | function | lines |
| [`size.avg_function_length`](./size#size-avg-function-length) | Average Function Length | file, project | lines |
| [`size.max_function_length`](./size#size-max-function-length) | Maximum Function Length | file, project | lines |
| [`complexity.cyclomatic`](./complexity#complexity-cyclomatic) | Cyclomatic Complexity | function, file, project | count |
| [`complexity.branch_count`](./complexity#complexity-branch-count) | Branch Count | function, file, project | count |
| [`complexity.conditional_count`](./complexity#complexity-conditional-count) | Conditional Count | function, file, project | count |
| [`complexity.loop_count`](./complexity#complexity-loop-count) | Loop Count | function, file, project | count |
| [`complexity.return_count`](./complexity#complexity-return-count) | Return Count | function, file, project | count |
| [`complexity.jump_count`](./complexity#complexity-jump-count) | Jump Count | function, file, project | count |
| [`complexity.path_count`](./complexity#complexity-path-count) | Number of Paths | function | count |
| [`complexity.cognitive`](./complexity#complexity-cognitive) | Cognitive Complexity | function, file, project | count |
| [`nesting.max_depth`](./nesting#nesting-max-depth) | Maximum Nesting Depth | function, file, project | levels |
| [`nesting.avg_depth`](./nesting#nesting-avg-depth) | Average Nesting Depth | function, file, project | levels |
| [`halstead.unique_operators`](./halstead#halstead-unique-operators) | Unique Operators | function, file, project | count |
| [`halstead.unique_operands`](./halstead#halstead-unique-operands) | Unique Operands | function, file, project | count |
| [`halstead.total_operators`](./halstead#halstead-total-operators) | Total Operators | function, file, project | count |
| [`halstead.total_operands`](./halstead#halstead-total-operands) | Total Operands | function, file, project | count |
| [`halstead.vocabulary`](./halstead#halstead-vocabulary) | Vocabulary | function, file, project | count |
| [`halstead.length`](./halstead#halstead-length) | Program Length | function, file, project | count |
| [`halstead.volume`](./halstead#halstead-volume) | Volume | function, file, project | bits |
| [`halstead.difficulty`](./halstead#halstead-difficulty) | Difficulty | function, file, project | ratio |
| [`halstead.effort`](./halstead#halstead-effort) | Effort | function, file, project | count |
| [`halstead.time`](./halstead#halstead-time) | Estimated Program Time | function, file, project | seconds |
| [`halstead.bugs`](./halstead#halstead-bugs) | Estimated Bugs | function, file, project | count |
| [`function.parameter_count`](./function#function-parameter-count) | Parameter Count | function | count |
| [`function.avg_parameter_count`](./function#function-avg-parameter-count) | Average Parameter Count | file, project | count |
| [`function.max_parameter_count`](./function#function-max-parameter-count) | Maximum Parameter Count | file, project | count |
| [`function.expression_count`](./function#function-expression-count) | Expression Count | function, file, project | count |
| [`function.call_count`](./function#function-call-count) | Call Count | function, file, project | count |
| [`duplication.duplicate_block_count`](./duplication#duplication-duplicate-block-count) | Duplicate Block Count | file, project | count |
| [`duplication.duplicate_token_count`](./duplication#duplication-duplicate-token-count) | Duplicate Token Count | file, project | count |
| [`duplication.duplication_ratio`](./duplication#duplication-duplication-ratio) | Duplication Ratio | file, project | ratio |
| [`duplication.max_duplicate_length`](./duplication#duplication-max-duplicate-length) | Maximum Duplicate Length | file, project | tokens |
| [`dependency.fan_out`](./dependency#dependency-fan-out) | Fan-out | function | count |
| [`dependency.fan_in`](./dependency#dependency-fan-in) | Fan-in | function | count |
| [`dependency.call_depth`](./dependency#dependency-call-depth) | Call Depth | function | calls |
| [`dependency.dependency_count`](./dependency#dependency-dependency-count) | Dependency Count | file, project | count |
| [`documentation.doc_loc`](./documentation#documentation-doc-loc) | Documentation LOC | function | lines |
| [`documentation.documented_function_count`](./documentation#documentation-documented-function-count) | Documentation Count | file, project | count |
| [`documentation.documentation_ratio`](./documentation#documentation-documentation-ratio) | Documentation Ratio | file, project | ratio |
| [`maintainability.index`](./maintainability#maintainability-index) | Maintainability Index | function, file | index |
| [`derived.cyclomatic_per_function`](./derived#derived-cyclomatic-per-function) | Cyclomatic Complexity per Function | file, project | ratio |
| [`derived.tokens_per_loc`](./derived#derived-tokens-per-loc) | Tokens per LOC | file, project | ratio |
| [`derived.statements_per_function`](./derived#derived-statements-per-function) | Statements per Function | file, project | ratio |
| [`derived.duplicate_tokens_per_sloc`](./derived#derived-duplicate-tokens-per-sloc) | Duplicate Tokens per SLOC | file, project | ratio |
