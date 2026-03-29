# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.2] - 2026-03-29

### Fixed

- Publish the hosted schema URL and plugin update URL from wasm plugin metadata so `dprint config update` can discover new `dprint-plugin-svg` releases.

## [0.2.1] - 2026-03-29

### Fixed

- Release workflow ran schema gen after renaming the wasm artifact, breaking the build.
- Docs site now auto-injects the latest tag into plugin URLs.

## [0.2.0] - 2026-03-29

### Added

- `textContent` option: control text-node whitespace handling (`collapse`, `maintain`, `prettify`; default: `maintain`).
- `formatEmbeddedContent` option: delegate `<style>` (CSS), `<script>` (JS), and `<foreignObject>` (HTML) to other dprint plugins via the host callback (default: `true`).
- `blankLines` option: control blank lines between sibling elements (`remove`, `preserve`, `truncate`, `insert`; default: `truncate`).
- Config doc pages for `textContent`, `blankLines`, and `formatEmbeddedContent`.
- Ignore directives: `<!-- dprint-ignore -->`, `<!-- dprint-ignore-start/end -->`, `<!-- dprint-ignore-file -->` (also works with `svg-format-` prefix).
- Rustdoc with before/after SVG examples on all config enums and public API.
- Default values emitted into JSON Schema for editor autocompletion.
- mdbook configuration reference with per-option before/after examples.
- GitHub Pages workflow for auto-deploying docs on push to master.
- `just book` recipe for local mdbook builds.
- SVG sample corpus covering diagnostics, hover info, path commands, and color/style edge cases.

### Fixed

- Embedded content width budget uses resolved `lineWidth` instead of `maxInlineTagWidth`.
- Host formatter errors are propagated instead of silently swallowed.
- Force LF line endings on embedded content to prevent CRLF doubling.

### Changed

- Default text-node handling changed from trim-and-reindent to preserve-relative-indentation (`maintain` mode).
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

[Unreleased]: https://github.com/kjanat/dprint-plugin-svg/compare/v0.2.2...HEAD
[0.2.2]: https://github.com/kjanat/dprint-plugin-svg/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/kjanat/dprint-plugin-svg/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/kjanat/dprint-plugin-svg/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/kjanat/dprint-plugin-svg/releases/tag/v0.1.0
