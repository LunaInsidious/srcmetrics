---
layout: home

hero:
  name: srcmetrics
  text: Language-independent source code metrics
  tagline: Size, complexity, nesting, Halstead, duplication, dependencies and documentation — one definition for C, C++, Go, Java, JavaScript, Python, Rust and TypeScript.
  actions:
    - theme: brand
      text: Get started
      link: /guide/getting-started
    - theme: alt
      text: Metric reference
      link: /metrics/
    - theme: alt
      text: GitHub
      link: https://github.com/LunaInsidious/srcmetrics

features:
  - title: One definition, every language
    details: Code is parsed with tree-sitter into a small common representation, and every metric is computed from it. The same algorithm in eight languages gets the same cyclomatic and cognitive complexity.
  - title: Raw features, not a score
    details: 53 metrics at function, file and project scope, reported separately for statistical analysis. There is no built-in "readability score".
  - title: Honest values
    details: A value that does not apply is null with its reason, never 0. Files that fail to parse are reported, never dropped.
  - title: Reproducible
    details: Every result records the commit, parser versions and metric definition version it was computed with.
  - title: CLI, library and HTTP API
    details: Export JSON or CSV, get statistics and self-contained HTML reports, or embed the Rust library.
  - title: Bring your own labels
    details: If you have human readability ratings, fit an experimental ridge-regression model on them.
---
