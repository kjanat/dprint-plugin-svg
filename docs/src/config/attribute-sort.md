# attributeSort

Attribute ordering strategy

|             |                                               |
| ----------- | --------------------------------------------- |
| **Type**    | `"none"` \| `"canonical"` \| `"alphabetical"` |
| **Default** | `"canonical"`                                 |

## Values

### Input

```svg
<rect y="20" x="10" height="50" width="100" id="box" />
```

### `"none"`

Keep original source order.

```svg
<rect y="20" x="10" height="50" width="100" id="box" />
```

### `"canonical"`

SVG-aware canonical grouping (id, class, geometry, presentation, ...).

```svg
<rect id="box" x="10" y="20" width="100" height="50" />
```

### `"alphabetical"`

Strict alphabetical order.

```svg
<rect height="50" id="box" width="100" x="10" y="20" />
```

## Config

```json
{
  "svg": {
    "attributeSort": "canonical"
  }
}
```
