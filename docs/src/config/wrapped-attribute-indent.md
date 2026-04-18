# wrappedAttributeIndent

Indentation strategy for wrapped attributes

|             |                                        |
| ----------- | -------------------------------------- |
| **Type**    | `"one-level"` \| `"align-to-tag-name"` |
| **Default** | `"align-to-tag-name"`                  |

## Values

### Input

```svg
<rect id="box" x="10" y="20" width="100" height="50" fill="red" />
```

### `"one-level"`

Indent one level deeper than the tag.

```svg
<rect
  id="box"
  x="10" y="20" width="100" height="50"
  fill="red" />
```

### `"align-to-tag-name"`

Align to the column after `<tagName`.

```svg
<rect id="box"
      x="10" y="20" width="100" height="50"
      fill="red" />
```

## Config

```json
{
  "svg": {
    "wrappedAttributeIndent": "align-to-tag-name"
  }
}
```
