# useTabs

Use tabs for indentation instead of spaces.

|             |                              |
| ----------- | ---------------------------- |
| **Type**    | `boolean`                    |
| **Default** | `false` (from global config) |

The plugin default matches the W3 SVG reference samples, which use
two-space indentation. Set `useTabs: true` (or the top-level dprint
`"useTabs": true`) when you prefer tabs repo-wide.

## Example

### `useTabs: false` (default)

Uses `indentWidth` spaces per level (default `2`):

```svg
<svg>
  <rect x="0" y="0" width="10" height="10" />
</svg>
```

### `useTabs: true`

```svg
<svg>
	<rect x="0" y="0" width="10" height="10" />
</svg>
```

## Config

```json
{
  "svg": {
    "useTabs": true
  }
}
```
