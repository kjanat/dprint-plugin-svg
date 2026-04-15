# DOCS KNOWLEDGE BASE

**Generated:** 2026-03-29

## OVERVIEW

`docs/` owns mdBook source for user-facing docs and the generated site output boundary.

## STRUCTURE

```txt
docs/
├── book.toml      # mdBook config; build dir = book
├── src/           # authored markdown
│   ├── SUMMARY.md # nav + chapter order
│   ├── introduction.md
│   ├── ignoring-code.md
│   └── config/    # per-option reference pages
└── book/          # generated HTML output from mdBook
```

## WHERE TO LOOK

| Task                    | Location                      | Notes                                                 |
| ----------------------- | ----------------------------- | ----------------------------------------------------- |
| mdBook config           | `docs/book.toml`              | `build-dir = "book"`, theme, CNAME, edit URL template |
| Nav + chapter order     | `docs/src/SUMMARY.md`         | add pages here or they stay out of sidebar            |
| Intro + high-level docs | `docs/src/*.md`               | top-level authored pages                              |
| Config reference pages  | `docs/src/config/*.md`        | one page per setting                                  |
| Local docs build        | `justfile`                    | `just book` runs `mdbook build docs`                  |
| Pages deploy path       | `.github/workflows/pages.yml` | builds docs, uploads `docs/book`                      |

## CONVENTIONS

- Edit `docs/src/**`; treat `docs/book/**` as generated output.
- Keep `docs/src/SUMMARY.md` in sync with added, moved, or renamed pages.
- Keep general prose in `docs/src/`; keep option-specific docs in `docs/src/config/`.
- `book.toml` owns site metadata, CNAME, theme, and GitHub edit-link behavior.
- Pages publish flow is `mdbook build docs` -> `docs/book` -> upload in `.github/workflows/pages.yml`.
- Config docs should match `src/schema.rs` and behavior tested in `tests/`, not README shortcuts.

## ANTI-PATTERNS

- Do not edit `docs/book/**` by hand.
- Do not add pages without updating `docs/src/SUMMARY.md`.
- Do not change `book.toml` `build-dir` or Pages upload path independently; they must stay aligned.
- Do not document config behavior that diverges from `src/lib.rs`, `src/schema.rs`, or tests.
