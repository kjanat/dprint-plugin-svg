# attributeSort

Attribute ordering strategy.

|             |                                           |
| ----------- | ----------------------------------------- |
| **Type**    | `"none" \| "canonical" \| "alphabetical"` |
| **Default** | `"canonical"`                             |

## Values

### `"none"`

Preserve the original attribute order:

```svg
<!-- input -->
<svg>
  <rect y="20" x="10" height="50" width="100" fill="#ff0000" id="box" />
</svg>
```

```svg
<!-- output — order unchanged -->
<svg>
  <rect y="20" x="10" height="50" width="100" fill="#ff0000" id="box" />
</svg>
```

### `"canonical"` (default)

Sort attributes in SVG-idiomatic order (`id`, `class`, positional, dimensional, presentation, etc.):

```svg
<!-- input -->
<svg>
  <rect y="20" x="10" height="50" width="100" fill="#ff0000" id="box" />
</svg>
```

```svg
<!-- output -->
<svg>
  <rect id="box" x="10" y="20" width="100" height="50" fill="#ff0000" />
</svg>
```

### `"alphabetical"`

Sort attributes in strict alphabetical order:

```svg
<!-- input -->
<svg>
  <rect y="20" x="10" height="50" width="100" fill="#ff0000" id="box" />
</svg>
```

```svg
<!-- output -->
<svg>
  <rect fill="#ff0000" height="50" id="box" width="100" x="10" y="20" />
</svg>
```

## Config

```json
{
  "svg": {
    "attributeSort": "canonical"
  }
}
```
