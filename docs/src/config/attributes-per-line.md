# attributesPerLine

Maximum number of attributes per line when attributes are wrapped (multi-line mode).

|             |          |
| ----------- | -------- |
| **Type**    | `number` |
| **Default** | `1`      |
| **Minimum** | `1`      |

Only applies when attributes are broken onto multiple lines (via `attributeLayout` or `maxInlineTagWidth`). Values below `1` are clamped to `1`.

## Example

### `attributesPerLine: 1` (default)

```svg
<svg>
  <rect
    id="box"
    x="10"
    y="20"
    width="100"
    height="50"
    fill="#ff0000" />
</svg>
```

### `attributesPerLine: 2`

```svg
<svg>
  <rect
    id="box" x="10"
    y="20" width="100"
    height="50" fill="#ff0000" />
</svg>
```

### `attributesPerLine: 3`

```svg
<svg>
  <rect
    id="box" x="10" y="20"
    width="100" height="50" fill="#ff0000" />
</svg>
```

## Config

```json
{
  "svg": {
    "attributesPerLine": 2
  }
}
```
