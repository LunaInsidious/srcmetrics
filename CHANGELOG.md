# Changelog

All notable changes to this project are documented in this file. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project follows
[Semantic Versioning](https://semver.org/) (before 1.0, a minor version may contain breaking changes).

## [0.2.0] - Unreleased

Metric explanations now live only in the documentation; the code keeps the metric ids and the
scopes they are reported at.

### Breaking changes

- Library: `metrics::MetricDefinition`, `metrics::Applicability`, `metrics::definitions()`,
  `metrics::to_markdown()`, `metrics::reference_pages()` and `Calculator::definitions()` are
  removed. Use `metrics::MetricSpec` (`id`, `scopes`), `metrics::specs()` and
  `Calculator::specs()`.
- CLI: the `srcmetrics metrics` command is removed. What each metric means is documented at
  <https://lunainsidious.github.io/srcmetrics/metrics/>.
- HTTP API: `GET /api/metrics` is removed.

### Added

- Japanese metric reference: <https://lunainsidious.github.io/srcmetrics/ja/metrics/>.

### Unchanged

- Metric ids, scopes, values and the analysis result format (JSON / CSV). The metric definition
  version stays `0.1.0`.

## [0.1.0] - 2026-09-28

Initial release.

[0.2.0]: https://github.com/LunaInsidious/srcmetrics/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/LunaInsidious/srcmetrics/releases/tag/v0.1.0
