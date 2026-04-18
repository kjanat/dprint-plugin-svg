# Multi-line attribute values

When an attribute value contains embedded newlines in the source — most
commonly a `<path d="…">` broken across lines at logical path-command
boundaries, the style used by the W3 SVG reference samples — the
formatter preserves those line breaks and realigns each continuation
line under the column directly after the opening quote.

This is a property of the emitter, not a configurable option. It
applies under every `wrappedAttributeIndent` mode; the column at which
the continuation aligns changes with the wrap style.

## Example — `wrappedAttributeIndent: "align-to-tag-name"`

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

The continuation `L800,550 L1000,300"` aligns under the `M` of
`M200,300` — the first character of the value. Wrapped attributes
(`fill`, `stroke`, `stroke-width`) align under `d=`.

## Example — `wrappedAttributeIndent: "one-level"` (default)

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

The wrap column is shallower (one indent level past the tag), so the
continuation pad is correspondingly smaller — but still lines up under
the opening quote of the value.

## Notes

- The continuation pad mirrors the wrap-prefix indent style: if the
  wrap prefix uses tabs, the continuation uses tabs too (plus spaces
  for the `name=` + quote span). This keeps visual alignment stable
  regardless of the reader's tab width.
- When any attribute in a tag has a multi-line value, the layout
  switches to one attribute per wrapped line — chaining multiple
  attributes per line would break the per-attribute column invariant
  that continuation alignment depends on.
