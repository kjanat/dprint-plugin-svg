# dprint-plugin-svg

dprint Wasm plugin for formatting SVG files.

## Build

```sh
rustup target add wasm32-unknown-unknown
just build-wasm
```

Artifact path:

`target/wasm32-unknown-unknown/release/dprint_plugin_svg.wasm`

Or print the absolute path:

```sh
just plugin-path
```

## Use in dprint

```json
{
	"plugins": ["./target/wasm32-unknown-unknown/release/dprint_plugin_svg.wasm"],
	"svg": {
		"attributeSort": "canonical",
		"attributeLayout": "auto",
		"attributesPerLine": 1,
		"wrappedAttributeIndent": "one-level"
	}
}
```

## Supported `svg` Config

- `lineWidth` (number)
- `maxInlineTagWidth` (number)
- `useTabs` (boolean)
- `indentWidth` (number)
- `newLineKind` (`"auto" | "lf" | "crlf" | "system"`)
- `attributeSort` (`"none" | "canonical" | "alphabetical"`)
- `attributeLayout` (`"auto" | "single-line" | "multi-line"`)
- `attributesPerLine` (number > 0)
- `spaceBeforeSelfClose` (boolean)
- `quoteStyle` (`"preserve" | "double" | "single"`)
- `wrappedAttributeIndent` (`"one-level" | "align-to-tag-name"`)

## Dependency Policy

This plugin depends on [`svg-format`](https://github.com/kjanat/svg-language-server/tree/master/crates/svg-format)
from [`svg-language-server`](https://github.com/kjanat/svg-language-server) via a pinned git `rev`
in [`Cargo.toml`](https://github.com/kjanat/dprint-plugin-svg/blob/master/Cargo.toml)
for reproducible builds.

When upgrading formatter behavior:

1. Update the pinned `rev`.
2. Run `just test`, `just lint`, and `just build-wasm`.
3. Cut a new tag release.
