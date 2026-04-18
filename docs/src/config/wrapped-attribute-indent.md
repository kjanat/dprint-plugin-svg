# wrappedAttributeIndent

Indent style for wrapped attributes relative to the tag.

|             |                                      |
| ----------- | ------------------------------------ |
| **Type**    | `"one-level" \| "align-to-tag-name"` |
| **Default** | `"one-level"`                        |

Only visible when attributes are broken onto multiple lines.

## Values

### `"one-level"` (default)

Put the tag name alone on the first line; indent wrapped attributes by
one indentation level from the tag's opening `<`:

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

Keep the first attribute inline with `<tag`; subsequent attributes wrap
aligned under it. The wrap column equals `indent + "<" + tag_name + " "`,
so multi-line attribute values (e.g. a `d="..."` broken across source
lines) also align cleanly under the opening quote:

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
