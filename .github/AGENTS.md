# GITHUB KNOWLEDGE BASE

**Generated:** 2026-03-29

## OVERVIEW

`.github/` owns CI verification, tagged release publishing, and GitHub Pages deploy semantics.

## STRUCTURE

```txt
.github/
└── workflows/
    ├── ci.yml       # PR + master verification
    ├── release.yml  # tag-driven GitHub release
    └── pages.yml    # mdBook -> GitHub Pages
```

## WHERE TO LOOK

| Task                              | Location                                      | Notes                                                     |
| --------------------------------- | --------------------------------------------- | --------------------------------------------------------- |
| CI triggers + required checks     | `.github/workflows/ci.yml`                    | PRs + pushes to `master`; build, format check, lint, test |
| Release trigger + uploaded assets | `.github/workflows/release.yml`               | tag pushes `v*`; uploads wasm + schema                    |
| Pages trigger + deploy target     | `.github/workflows/pages.yml`                 | pushes to `master` or manual dispatch                     |
| Canonical command names           | `justfile`                                    | workflows should mirror local commands                    |
| Release schema artifact           | `deployment/schema.json`                      | must exist and be current                                 |
| Wasm release artifact path        | `target/wasm32-unknown-unknown/wasm-release/` | build output renamed to `plugin.wasm` in release job      |

## CONVENTIONS

- Keep workflow steps reproducible locally with `just build-wasm`, `dprint check`, `just lint`, `just test`, `just schema`, `just book`.
- CI installs `just`, `dprint`, `tombi`, stable Rust, and the wasm target; local fixes should assume the same toolchain.
- Release flow is tag-only: build wasm, run tests, rename artifact, regenerate schema, publish release.
- Pages flow is `mdbook build docs`, then upload `docs/book` to the `github-pages` environment.
- Branch and tag triggers define publish semantics; current defaults are `master` and `v*`.

## ANTI-PATTERNS

- Do not change workflow command names without matching `justfile` updates.
- Do not move or rename release assets without updating upload paths and the artifact normalization step.
- Do not skip `just schema` for release-relevant schema/config changes.
- Do not move Pages output away from `docs/book` without updating `pages.yml` and `docs/book.toml` together.
- Do not add workflow-only behavior that cannot be reproduced locally with the canonical commands.
