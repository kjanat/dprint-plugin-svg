# SRC KNOWLEDGE BASE

**Generated:** 2026-03-28

## OVERVIEW

`src/` owns plugin runtime behavior: config parsing, dprint handler contract, formatter invocation, and optional schema model.

## STRUCTURE

```txt
src/
├── lib.rs                   # plugin entrypoint + config resolution + format bridge
├── schema.rs                # schema structs + schema value generation (feature: schema)
└── bin/generate-schema.rs   # CLI to emit schema JSON to path arg
```

## WHERE TO LOOK

| Task                       | Location                     | Notes                                                                  |
| -------------------------- | ---------------------------- | ---------------------------------------------------------------------- |
| Add/change config key      | `src/lib.rs`                 | edit `Configuration`, `resolve_config`, conversion to `FormatOptions`  |
| Add enum config option     | `src/lib.rs`                 | update enum + `generate_str_to_from!` + map fn                         |
| Change formatting behavior | `src/lib.rs`                 | `format()` builds `FormatOptions`, newline normalization, no-op checks |
| Change schema shape/docs   | `src/schema.rs`              | schema mirrors `Configuration` + defaults/metadata                     |
| Regenerate schema artifact | `src/bin/generate-schema.rs` | write JSON consumed by `deployment/schema.json`                        |

## CONVENTIONS

- Keep parse boundary diagnostic-first: invalid user values should emit diagnostics, not panic.
- Preserve early return for unsupported requests (`range`) and cancelled tokens.
- Keep UTF-8 decoding explicit and error message includes file path.
- Keep stable config key names/casing (`camelCase` keys in plugin config).
- Embedded host delegation should gracefully fall back to original content for host configuration errors, but still surface invalid UTF-8 and non-config host failures.

## ANTI-PATTERNS

- Do not bypass mapping helpers (`map_attribute_sort`, etc.) when wiring config to `svg-format`.
- Do not add implicit behavior in `format()` that breaks idempotence.
- Do not couple schema generation to runtime-only paths; schema must stay feature-gated.
