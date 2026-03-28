# lineWidth

Fallback line width for formatting decisions when `maxInlineTagWidth` is not set.

|             |                            |
| ----------- | -------------------------- |
| **Type**    | `number`                   |
| **Default** | `100` (from global config) |
| **Minimum** | `1`                        |

When `maxInlineTagWidth` is omitted, its value equals `lineWidth`. This controls when the formatter wraps attributes or children onto new lines.

## Example

### `lineWidth: 60`

Short line width forces earlier wrapping:

```svg
<!-- input -->
<svg><rect x="10" y="20" width="100" height="50" fill="#ff0000" stroke="#000" /></svg>
```

```svg
<!-- output -->
<svg>
  <rect
    x="10"
    y="20"
    width="100"
    height="50"
    fill="#ff0000"
    stroke="#000" />
</svg>
```

### `lineWidth: 200`

Wide line width keeps short tags inline:

```svg
<!-- input -->
<svg><rect x="10" y="20" width="100" height="50" fill="#ff0000" stroke="#000" /></svg>
```

```svg
<!-- output -->
<svg>
  <rect x="10" y="20" width="100" height="50" fill="#ff0000" stroke="#000" />
</svg>
```

## Config

```json
{
  "svg": {
    "lineWidth": 80
  }
}
```
