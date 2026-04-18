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

## Multi-line attribute values

When an attribute value itself contains embedded newlines in the
source — most commonly a `<path d="…">` broken across lines at logical
path-command boundaries (the W3 SVG reference style) — the formatter
preserves those line breaks and realigns each continuation line under
the column directly after the opening quote. The column changes with
the wrap mode above but the invariant holds either way.

### Under `"align-to-tag-name"`

```svg
<!-- input -->
<svg xmlns="http://www.w3.org/2000/svg"><path d="M200,300 L400,50 L600,300
L800,550 L1000,300" fill="none" stroke="#888888" stroke-width="2"/></svg>
```

```svg
<!-- output -->
<svg xmlns="http://www.w3.org/2000/svg">
  <path d="M200,300 L400,50 L600,300
           L800,550 L1000,300"
        fill="none"
        stroke="#888888"
        stroke-width="2" />
</svg>
```

### Under `"one-level"` (default)

```svg
<!-- output -->
<svg xmlns="http://www.w3.org/2000/svg">
  <path
    d="M200,300 L400,50 L600,300
       L800,550 L1000,300"
    fill="none"
    stroke="#888888"
    stroke-width="2" />
</svg>
```

The continuation pad mirrors the wrap-prefix indent style: if the wrap
prefix uses tabs, the continuation uses tabs too (plus spaces for the
`name=` + quote span) so alignment stays stable regardless of the
reader's tab width. When any attribute in a tag has a multi-line
value, the layout switches to one attribute per wrapped line.
