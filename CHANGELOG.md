# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- mdbook configuration reference with per-option before/after examples.
- GitHub Pages workflow for auto-deploying docs on push to master.
- `just book` recipe for local mdbook builds.
- SVG sample corpus covering diagnostics, hover info, path commands, and color/style edge cases.

### Changed

- Add docs site metadata for the custom domain and GitHub edit links.

## [0.1.0] - 2026-03-28

### Added

- Wasm dprint plugin for formatting SVG files.
- Configuration options: `lineWidth`, `maxInlineTagWidth`, `useTabs`, `indentWidth`, `attributeSort`, `attributeLayout`, `attributesPerLine`, `spaceBeforeSelfClose`, `quoteStyle`, `wrappedAttributeIndent`, `newLineKind`.
- Config schema generated from Rust types via schemars, published as release artifact alongside `plugin.wasm`.
- Hand-written deployment config schema for editor validation.
- CI pipeline with format check, clippy lint, tests, and wasm build.
- Tag-based release workflow publishing wasm and schema artifacts.
- Fixture-based integration tests for plugin configuration and formatting.
- Schema tests for validity, field coverage, enum variants, and staleness.
- Regression tests for idempotent formatting, unknown config keys, and invalid UTF-8.

### Fixed

- Config schema and runtime parsing use the same newline enum.
- Schema output is deterministic across regeneration.

[Unreleased]: https://github.com/kjanat/dprint-plugin-svg/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/kjanat/dprint-plugin-svg/releases/tag/v0.1.0
