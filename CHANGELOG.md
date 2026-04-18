# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.8] - 2026-04-18

### Fixed

- Minified SVGs with chained single-coordinate path commands (e.g. `v.007-.009`
  or `h1-2`) and chained curveto arguments (C/S/Q/T with trailing coordinate
  groups) now actually format instead of silently round-tripping unchanged.
  Under the previous pinned `tree-sitter-svg`, the grammar rejected these
  compact forms as parse errors, and `svg_format` fell back to returning the
  source verbatim — the plugin reported success with an unformatted file.
  Upstream grammar fixes (tree-sitter-svg `dddee79`, `4f178c0`, `6f50d36`)
  extend h/v and C/S/Q/T repeats via a shared `_number_continuation`
  external scanner token, matching the SVG 2 path spec's implicit-repetition
  rule for every lineto/curveto family. Real-world regression: Firefox-style
  compact SVGs (`samples/firefox.svg`) now format into multi-line indented
  output as the README advertises.
- CRLF sources whose tree-sitter parse fails no longer bloat on each format pass. When
  `svg_format` falls back to returning source bytes verbatim (parse error, ignore-file
  directive, etc.) the bytes may still contain `\r`. The subsequent
  `formatted.replace('\n', newline)` under auto-detected CRLF would turn each `\r\n` into
  `\r\r\n`, adding one byte per line ending per pass and never reaching a stable fixed
  point — dprint bailed with "Formatting not stable. Bailed after 5 tries." The plugin
  now normalizes any stray CRs out of `formatted` before re-applying the target newline,
  so the transformation stays idempotent regardless of what `svg_format` hands back.
- Plugin-reported `config_schema_url` and generated schema `$id` now include the `v` prefix
  (e.g. `/v0.2.7/schema.json`) so they match the release tag path served by
  `plugins.dprint.dev`. The previous no-prefix URL returned 404, breaking editor schema
  validation and `dprint config update` discovery. Both the runtime-advertised URL in
  `src/lib.rs` and the baked-in `$id` emitted by `src/schema.rs` were affected; fixing only
  one would have let the next release regenerate the broken URL into the schema artifact.
- Wasm build on Clang 16+ no longer fails with `incompatible pointer types` errors in
  `tree-sitter-language-0.1.7/wasm/src/stdlib.c`. Added `-Wno-error=incompatible-pointer-types`
  to the wasm CFLAGS as a targeted workaround for the upstream typedef issue.

### Added

- Cancellation is now honored mid-format, not just before formatting starts. Each embedded
  `<style>`/`<script>`/`<foreignObject>` host-format delegation checks
  `request.token.is_cancelled()` before issuing, and the main format path re-checks after
  `svg_format::format_with_host` returns. A cancelled request yields `Ok(None)` (no change)
  instead of a partially-formatted result.

### Changed

- Published wasm binary is now built under a dedicated `[profile.wasm-release]` with
  `opt-level = "z"`, fat LTO, `codegen-units = 1`, `panic = "abort"`, and symbol strip.
  The resulting binary is ~20% smaller (893 KB → 710 KB locally), which reduces cold-load
  time in editors and the dprint CLI. Release compile time increases (~12s → ~85s) but
  is paid only at tag-push time. The stock `[profile.release]` is left unchanged for any
  other consumers. Build path moved to `target/wasm32-unknown-unknown/wasm-release/`;
  `just build-wasm`, `.dprint.jsonc`, and the release workflow were updated in lockstep.
- Pinned Rust toolchain to `nightly` with the `wasm32-unknown-unknown` target and
  `rustfmt`, `clippy` components via `rust-toolchain.toml` so contributors get a consistent
  build environment.
- `justfile` recipes are now grouped under `check`, `build`, and `docs` for a cleaner
  `just --list` output; `set unstable := true` enables the recipe-group feature.

## [0.2.7] - 2026-04-15

### Fixed

- Embedded host formatter configuration errors now fall back to preserving the original
  `<script>`, `<style>`, or `<foreignObject>` content instead of failing the whole SVG format request.
- `/bump` command used `git push --atomic` which doesn't push tags; changed to `git push --follow-tags`.

## [0.2.6] - 2026-03-30

### Fixed

- `blankLines` option now applies inside embedded `<script>`/`<style>` blocks, not just between sibling elements.
  Double blank lines in host-formatted CSS/JS are collapsed per the configured policy.
- Leading/trailing blank lines in host formatter output are stripped so no blank line leaks between tags and embedded content.

### Added

- Sample SVG with embedded script (`validity-vs-reliability.svg`).
- OpenCode `/bump` command for automated version bump workflow.

## [0.2.5] - 2026-03-30

### Fixed

- Embedded `<script>`/`<style>` formatting failed on XML-encoded content (e.g. `&lt;` in `for (i < n)`).
  Entity references are now decoded before delegating to the host formatter and re-encoded on return.

## [0.2.4] - 2026-03-30

### Fixed

- WASM stack overflow crash ("out of bounds memory access") when formatting large or complex SVG files
  (e.g. Inkscape-generated documents with hundreds of nodes). Increased WASM stack size from 1 MB to 10 MB.

## [0.2.3] - 2026-03-29

### Fixed

- Update the pinned `svg-format` dependency to include the latest inline entity-reference repair behavior for text content.

## [0.2.2] - 2026-03-29

### Fixed

- Publish the hosted schema URL and plugin update URL from wasm plugin metadata so `dprint config update` can discover
  new `dprint-plugin-svg` releases.

## [0.2.1] - 2026-03-29

### Fixed

- Release workflow ran schema gen after renaming the wasm artifact, breaking the build.
- Docs site now auto-injects the latest tag into plugin URLs.

## [0.2.0] - 2026-03-29

### Added

- `textContent` option: control text-node whitespace handling (`collapse`, `maintain`, `prettify`; default: `maintain`).
- `formatEmbeddedContent` option: delegate `<style>` (CSS), `<script>` (JS), and `<foreignObject>` (HTML) to other dprint
  plugins via the host callback (default: `true`).
- `blankLines` option: control blank lines between sibling elements
  (`remove`, `preserve`, `truncate`, `insert`; default: `truncate`).
- Config doc pages for `textContent`, `blankLines`, and `formatEmbeddedContent`.
- Ignore directives: `<!-- dprint-ignore -->`, `<!-- dprint-ignore-start/end -->`, `<!-- dprint-ignore-file -->`
  (also works with `svg-format-` prefix).
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
- Configuration options: `lineWidth`, `maxInlineTagWidth`, `useTabs`, `indentWidth`, `attributeSort`, `attributeLayout`,
  `attributesPerLine`, `spaceBeforeSelfClose`, `quoteStyle`, `wrappedAttributeIndent`, `newLineKind`.
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

[Unreleased]: https://github.com/kjanat/dprint-plugin-svg/compare/v0.2.8...HEAD
[0.2.8]: https://github.com/kjanat/dprint-plugin-svg/compare/v0.2.7...v0.2.8
[0.2.7]: https://github.com/kjanat/dprint-plugin-svg/compare/v0.2.6...v0.2.7
[0.2.6]: https://github.com/kjanat/dprint-plugin-svg/compare/v0.2.5...v0.2.6
[0.2.5]: https://github.com/kjanat/dprint-plugin-svg/compare/v0.2.4...v0.2.5
[0.2.4]: https://github.com/kjanat/dprint-plugin-svg/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/kjanat/dprint-plugin-svg/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/kjanat/dprint-plugin-svg/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/kjanat/dprint-plugin-svg/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/kjanat/dprint-plugin-svg/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/kjanat/dprint-plugin-svg/releases/tag/v0.1.0
