# maxInlineTagWidth

Maximum width of a tag (including attributes) before the formatter wraps attributes or children onto new lines.

|             |                      |
| ----------- | -------------------- |
| **Type**    | `number`             |
| **Default** | value of `lineWidth` |
| **Minimum** | `1`                  |

This is the primary knob for controlling when tags break. When a tag's total width exceeds this threshold, attributes wrap according to `attributeLayout` and `attributesPerLine`.

## Example

### `maxInlineTagWidth: 40`

```svg
<!-- input -->
<svg><circle cx="50" cy="50" r="25" fill="red" /></svg>
```

```svg
<!-- output -->
<svg>
  <circle
    cx="50"
    cy="50"
    r="25"
    fill="red" />
</svg>
```

### `maxInlineTagWidth: 120`

```svg
<!-- input -->
<svg><circle cx="50" cy="50" r="25" fill="red" /></svg>
```

```svg
<!-- output -->
<svg>
  <circle cx="50" cy="50" r="25" fill="red" />
</svg>
```

## Config

```json
{
  "svg": {
    "maxInlineTagWidth": 80
  }
}
```
