# srcmetrics

English | [日本語](https://github.com/LunaInsidious/srcmetrics/blob/main/README_ja.md)

Language-independent source code metrics for C, C++, Go, Java, JavaScript, Python, Rust and TypeScript.

srcmetrics measures size, control-flow complexity, nesting, Halstead measures, duplication, call
dependencies and documentation with **one definition for every language**, and reports each metric
**separately** — as raw features for statistical analysis, not as a single "readability score".
A value that cannot be computed is `null` with its reason, never 0.

**Documentation: https://lunainsidious.github.io/srcmetrics/**

## Install

```sh
cargo install srcmetrics-cli   # the `srcmetrics` command
cargo add srcmetrics           # the library
```

## Quick start

```sh
srcmetrics analyze src -o result.json          # metrics per project, file and function (JSON; --format csv)
srcmetrics report result.json -o report.html   # self-contained HTML report
```

```rust
let result = srcmetrics::analyze::analyze_source("example.py", "def f(x):\n    return x if x else 0\n")?;
```

## Links

- [Getting started](https://lunainsidious.github.io/srcmetrics/guide/getting-started)
- [Metric reference](https://lunainsidious.github.io/srcmetrics/metrics/)
- [Supported languages and limitations](https://lunainsidious.github.io/srcmetrics/guide/languages)
- [API documentation (docs.rs)](https://docs.rs/srcmetrics)
- Developer documents (Japanese): [design/](https://github.com/LunaInsidious/srcmetrics/tree/main/design)

## License

Licensed under either of [Apache License, Version 2.0](https://github.com/LunaInsidious/srcmetrics/blob/main/LICENSE-APACHE)
or [MIT license](https://github.com/LunaInsidious/srcmetrics/blob/main/LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
