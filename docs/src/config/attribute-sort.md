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

Sort attributes into SVG-idiomatic groups, applied in this order:

1. `id`
2. `class`
3. geometry / dimensional / presentation attributes (`x`, `y`, `width`,
   `height`, `viewBox`, `d`, `fill`, `stroke`, ...)
4. other attributes (alphabetical within the group)
5. `xmlns` and `xmlns:*` namespace declarations
6. `version`

The trailing position of `xmlns*` and `version` matches the W3 SVG
reference samples, where the namespace and profile declarations close
out the root tag rather than leading it.

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

Root `<svg>` tag with namespace + version:

```svg
<!-- input -->
<svg version="1.1" xmlns="http://www.w3.org/2000/svg" width="100" height="50">
```

```svg
<!-- output -->
<svg width="100" height="50" xmlns="http://www.w3.org/2000/svg" version="1.1">
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
