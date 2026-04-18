# dprint-plugin-svg

A [dprint](https://dprint.dev) Wasm plugin for formatting SVG files.

## Install

```sh
dprint add kjanat/svg
```

## Quick start

Add plugin-level options under the `"svg"` key in your dprint config:

```json
{
  "svg": {
    "attributeSort": "canonical",
    "attributeLayout": "auto",
    "spaceBeforeSelfClose": true
  },
  "plugins": [
    "https://plugins.dprint.dev/kjanat/svg-{{LATEST_TAG}}.wasm"
  ]
}
```

## Top-level config inheritance

Several options fall back to the top-level keys in the same `dprint.json`
(not the user-profile "global" config — those are the per-file root
keys dprint passes to every plugin). When omitted from the `svg` section:

| Plugin option | Falls back to top-level          |
| ------------- | -------------------------------- |
| `lineWidth`   | `lineWidth` (default `100`)      |
| `useTabs`     | `useTabs` (default `false`)      |
| `indentWidth` | `indentWidth` (default `2`)      |
| `newLineKind` | `newLineKind` (default `"auto"`) |

## Plugin-owned defaults

Every other option resolves from
[`svg_format::FormatOptions::default()`](https://docs.rs/svg-format) via
the plugin's config-enum mappers. The table below is generated from the
live schema on every `mdbook build`; values shown are whatever the
current source ships.

{{#include ./_generated/defaults-table.md}}

## Configuration reference

Browse the sidebar for per-option documentation with before/after examples.
