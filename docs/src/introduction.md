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

## Global config inheritance

Several options fall back to dprint global config when omitted:

| Plugin option | Falls back to                    |
| ------------- | -------------------------------- |
| `lineWidth`   | `lineWidth` (default `100`)      |
| `useTabs`     | `useTabs` (default `false`)      |
| `indentWidth` | `indentWidth` (default `2`)      |
| `newLineKind` | `newLineKind` (default `"auto"`) |

All other options use plugin-specific defaults documented in each page.

## Configuration reference

Browse the sidebar for per-option documentation with before/after examples.
