# useTabs

Use tabs for indentation instead of spaces.

|             |                             |
| ----------- | --------------------------- |
| **Type**    | `boolean`                   |
| **Default** | `true` (from global config) |

## Example

### `useTabs: true`

```svg
<svg>
	<rect x="0" y="0" width="10" height="10" />
</svg>
```

### `useTabs: false`

Uses `indentWidth` spaces per level (default 2):

```svg
<svg>
  <rect x="0" y="0" width="10" height="10" />
</svg>
```

## Config

```json
{
  "svg": {
    "useTabs": false
  }
}
```
