# Supported languages and limitations

| Language | Extensions | Grammar |
|---|---|---|
| C | `.c`, `.h` | tree-sitter-c |
| C++ | `.cpp`, `.cc`, `.cxx`, `.hpp`, `.hh`, `.hxx` | tree-sitter-cpp |
| Go | `.go` | tree-sitter-go |
| Java | `.java` | tree-sitter-java |
| JavaScript | `.js`, `.mjs`, `.cjs`, `.jsx` | tree-sitter-javascript |
| Python | `.py` | tree-sitter-python |
| Rust | `.rs` | tree-sitter-rust |
| TypeScript | `.ts`, `.mts`, `.cts`, `.tsx` | tree-sitter-typescript |

The exact grammar versions are recorded in every result (`run.parsers`).

## How languages are compared

Each language is converted to the same small intermediate representation (branches, loops, cases,
calls, tokens, …), and every metric is computed from that representation only. The same algorithm
written in all eight languages yields identical cyclomatic complexity, cognitive complexity and
maximum nesting depth; this is part of the test suite.

Some metrics still depend on how a language is written (for example, Python has no closing
braces, so its functions are shorter in lines). Each metric's
[language applicability](/metrics/) says whether it is language independent or partially
language dependent, and its limitations list the known differences.

## Known limitations

- **Files with syntax errors are not analyzed**, because partial results would be unreliable. They
  are reported with `status: "error"`.
- tree-sitter cannot parse C/C++ code whose braces are split across preprocessor branches, such as
  `#ifdef __cplusplus` / `extern "C" {` / `#endif`, which is common in C headers.
- `.h` files are parsed as C.
- Dependency metrics resolve calls by name only (no types, scopes or imports); functions with the
  same name are not distinguished.
- Code inside macro invocations (e.g. Rust `println!(…)`) is
  not analyzed.
