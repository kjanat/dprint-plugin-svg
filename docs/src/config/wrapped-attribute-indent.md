# wrappedAttributeIndent

Indent style for wrapped attributes relative to the tag.

|             |                                      |
| ----------- | ------------------------------------ |
| **Type**    | `"one-level" \| "align-to-tag-name"` |
| **Default** | `"one-level"`                        |

Only visible when attributes are broken onto multiple lines.

## Values

### `"one-level"` (default)

Indent wrapped attributes by one indentation level from the tag's opening `<`:

```svg
<svg>
  <rect
    id="box"
    x="10"
    y="20"
    width="100"
    height="50" />
</svg>
```

### `"align-to-tag-name"`

Align wrapped attributes to the character after the tag name:

```svg
<svg>
  <rect id="box"
        x="10"
        y="20"
        width="100"
        height="50" />
</svg>
```

## Config

```json
{
  "svg": {
    "wrappedAttributeIndent": "align-to-tag-name"
  }
}
```
