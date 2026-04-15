# PROJECT KNOWLEDGE BASE

**Generated:** 2026-03-29 **Commit:** `7469009` **Branch:** `feat/embedded-formatting`

## OVERVIEW

Rust dprint WASM plugin for SVG formatting. Runtime stays concentrated in `src/lib.rs`; schema, docs, tests, and release artifacts are separate maintained contract surfaces.

## STRUCTURE

```txt
./
├── src/                 # runtime plugin code; local AGENTS
│   └── bin/             # schema generator CLI
├── tests/               # integration + schema checks; local AGENTS
│   └── configs/         # dprint fixture configs
├── docs/                # mdBook source + build boundary; local AGENTS
├── deployment/          # committed schema artifact
├── .github/             # CI, release, Pages workflows; local AGENTS
├── samples/             # SVG corpus for examples/manual checks
├── Cargo.toml           # manifest + pinned svg-format rev
└── justfile             # canonical local commands
```

## WHERE TO LOOK

| Task                          | Location                     | Notes                                                                    |
| ----------------------------- | ---------------------------- | ------------------------------------------------------------------------ |
| Plugin entry + config mapping | `src/lib.rs`                 | `SyncPluginHandler<Configuration>` impl, config parsing, host delegation |
| Schema model + defaults       | `src/schema.rs`              | enabled by `schema` feature; mirrors config contract                     |
| Regenerate committed schema   | `src/bin/generate-schema.rs` | `just schema` wraps generator + formatting                               |
| Config/format behavior tests  | `tests/plugin_settings.rs`   | fixture-driven; asserts output, diagnostics, idempotence                 |
| Schema drift + enum contract  | `tests/schema.rs`            | fails if `deployment/schema.json` is stale                               |
| Config reference docs         | `docs/src/config/*.md`       | source of truth for per-option docs                                      |
| Local command surface         | `justfile`                   | mirrors CI/release steps                                                 |
| Automation + publishing       | `.github/workflows/*.yml`    | CI, GitHub release, Pages deploy                                         |

## CONVENTIONS

- Canonical local workflow: `just fmt`, `just lint`, `just test`, `just build-wasm`, `just schema`, `just book`.
- `just fmt` depends on a built local wasm artifact because `.dprint.jsonc` points at `./target/wasm32-unknown-unknown/release/dprint_plugin_svg.wasm`.
- `svg-format` is pinned to a git `rev` in `Cargo.toml`; current reality is git dependency, not sibling path.
- CI/test path uses `cargo test --all-targets --all-features`; schema tests run in normal `just test`.
- Workflow triggers still use `master`.
- Formatter contract is idempotent: second format pass returns no change.

## ANTI-PATTERNS (THIS PROJECT)

- Do not hand-edit `deployment/schema.json`; regenerate via `just schema` or the schema bin.
- Do not change config surface without updating tests and schema/docs together.
- Do not assume range formatting is supported; ranged requests intentionally return no change.
- Do not weaken invalid UTF-8 failure behavior without explicit test updates.
- Do not trust README alone for config coverage; docs/schema/source are more complete.

## UNIQUE STYLES

- Config parsing is diagnostic-first: invalid or unknown user config should diagnose, not hard-fail.
- Strongly typed string enums use `generate_str_to_from!` patterns.
- Embedded CSS, JS, and HTML delegate through the dprint host when `formatEmbeddedContent` is enabled.
- Tests assert operational semantics, not only formatted text.

## COMMANDS

```bash
just fmt
just lint
just test
just build-wasm
just schema
just book
```

## NOTES

- `docs/book/` is mdBook output; Pages builds it from `docs/src/`.
- Release normalizes `dprint_plugin_svg.wasm` to `plugin.wasm` before upload.
- README config summary lags current docs/schema; prefer `docs/src/config/` and `src/schema.rs`.
