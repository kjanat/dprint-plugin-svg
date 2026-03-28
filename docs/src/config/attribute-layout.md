# attributeLayout

Attribute line-breaking strategy.

|             |                                           |
| ----------- | ----------------------------------------- |
| **Type**    | `"auto" \| "single-line" \| "multi-line"` |
| **Default** | `"auto"`                                  |

## Values

### `"auto"` (default)

The formatter decides based on `maxInlineTagWidth`. Tags that fit within the threshold stay on one line; longer tags wrap.

```svg
<!-- short tag stays inline -->
<svg>
  <circle cx="50" cy="50" r="25" />
</svg>

<!-- long tag wraps -->
<svg>
  <rect
    id="bg"
    x="0"
    y="0"
    width="1920"
    height="1080"
    fill="#1a1a2e"
    stroke="#16213e"
    stroke-width="2" />
</svg>
```

### `"single-line"`

Force all attributes onto the same line as the tag name, regardless of width:

```svg
<svg>
  <rect id="bg" x="0" y="0" width="1920" height="1080" fill="#1a1a2e" stroke="#16213e" stroke-width="2" />
</svg>
```

### `"multi-line"`

Force every tag to wrap attributes, one per line (or per `attributesPerLine`):

```svg
<svg>
  <circle
    cx="50"
    cy="50"
    r="25" />
</svg>
```

## Config

```json
{
  "svg": {
    "attributeLayout": "auto"
  }
}
```
