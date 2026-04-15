# TESTS KNOWLEDGE BASE

**Generated:** 2026-03-28

## OVERVIEW

`tests/` validates config resolution, formatter semantics, and schema drift. Style is fixture-driven integration testing.

## STRUCTURE

```txt
tests/
├── plugin_settings.rs  # config + formatting behavior tests
├── schema.rs           # schema shape + stale artifact checks (feature: schema)
├── configs/            # dprint json fixtures consumed by plugin_settings.rs
└── fixtures/           # currently empty placeholder
```

## WHERE TO LOOK

| Task                          | Location                                                   | Notes                                                                |
| ----------------------------- | ---------------------------------------------------------- | -------------------------------------------------------------------- |
| Add config behavior case      | `tests/configs/*.dprint.json` + `tests/plugin_settings.rs` | fixture file name should describe behavior                           |
| Validate diagnostics          | `tests/plugin_settings.rs`                                 | assert `diagnostics` content, not just success                       |
| Validate formatting semantics | `tests/plugin_settings.rs`                                 | assert exact output + idempotence pass                               |
| Validate schema contract      | `tests/schema.rs`                                          | property presence, enum values, key order                            |
| Guard committed schema        | `tests/schema.rs`                                          | `committed_schema_is_not_stale` compares generated vs committed JSON |

## CONVENTIONS

- Use integration tests (`tests/*.rs`), not unit tests in `src/`.
- Parse fixture JSON into `ConfigKeyMap` to match dprint boundary behavior.
- Keep negative-path coverage: unknown key diagnostics, invalid value fallback, invalid UTF-8.
- Keep range-format behavior explicit: ranged requests should return `None` (no change).
- For embedded host delegation, test both graceful fallback cases (missing/invalid host config preserves original content) and hard-failure cases (real host errors still propagate).

## ANTI-PATTERNS

- Do not assert only "no panic"; assert concrete output/diagnostic semantics.
- Do not edit `deployment/schema.json` without schema regeneration + stale-check test pass.
- Do not remove idempotence assertions when updating formatting behavior.
