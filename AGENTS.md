# PROJECT KNOWLEDGE BASE

**Generated:** 2026-03-28

## OVERVIEW

Rust dprint WASM plugin for SVG formatting. Core logic in `src/lib.rs`, schema generation behind `schema` feature, integration-heavy tests in `tests/`.

## STRUCTURE

```txt
./
├── src/                 # plugin handler + config parsing + format bridge
│   └── bin/             # schema generator CLI
├── tests/               # integration tests + config fixtures
│   └── configs/         # dprint json fixtures used by tests
├── deployment/          # committed schema artifact
├── .github/             # CI/release + private dependency bootstrap
├── Cargo.toml           # crate metadata, features, deps
└── justfile             # canonical dev commands
```

## WHERE TO LOOK

| Task                           | Location                                                    | Notes                                                      |
| ------------------------------ | ----------------------------------------------------------- | ---------------------------------------------------------- |
| Plugin entry + dprint contract | `src/lib.rs`                                                | `SyncPluginHandler<Configuration>` impl                    |
| Config schema model/generation | `src/schema.rs`                                             | enabled by `schema` feature                                |
| Regenerate deployment schema   | `src/bin/generate-schema.rs`                                | run via `just schema`                                      |
| Config behavior tests          | `tests/plugin_settings.rs`                                  | fixture-driven; asserts formatting + diagnostics           |
| Schema drift/shape tests       | `tests/schema.rs`                                           | fails if `deployment/schema.json` stale                    |
| CI/release behavior            | `.github/workflows/ci.yml`, `.github/workflows/release.yml` | CI runs fmt/lint/test/build; release uploads wasm + schema |

## CONVENTIONS

- Canonical local workflow: `just fmt`, `just lint`, `just test`, `just build-wasm`, `just schema`.
- CI/release enforce `cargo test --all-targets --all-features`; schema tests always active in CI.
- `svg-format` dependency is a local sibling path (`../svg-language-server/...`), not crates.io.
- Formatting stack is dprint-led; Rust formatting invoked through dprint exec plugin.
- Formatter must be idempotent: second format pass returns no change.

## ANTI-PATTERNS (THIS PROJECT)

- Do not hand-edit `deployment/schema.json`; regenerate via schema binary.
- Do not bypass fixture-based config tests when adding/changing config keys.
- Do not assume range formatting supported; plugin intentionally no-ops ranged requests.
- Do not introduce non-UTF-8 tolerant behavior changes without test updates.

## UNIQUE STYLES

- Strongly typed string enums for config values (`generate_str_to_from!` patterns).
- Config diagnostics preferred over hard failure for invalid user config values.
- Tests validate both output text and operational semantics (idempotence, diagnostics, UTF-8 failure).

## COMMANDS

```bash
just fmt
just lint
just test
just build-wasm
just schema
```

## NOTES

- README dependency policy text currently references pinned git rev; manifest currently uses local path dep.
- CI private-dependency setup clones sibling repo into parent workspace; local env must mirror that layout.
