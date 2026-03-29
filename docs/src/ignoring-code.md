# Ignoring Code

Use HTML comments to prevent the formatter from touching specific elements,
ranges, or entire files.

Both `dprint-ignore` and `svg-format-ignore` prefixes are recognized.

## Ignore the next element

```svg
<!-- dprint-ignore -->
<rect y="20" x="10" height="50" width="100" />
```

The `<rect>` keeps its original attribute order and whitespace.

## Ignore a range

```svg
<!-- dprint-ignore-start -->
<rect y="20" x="10" />
<circle r="30" cx="1" cy="2" />
<!-- dprint-ignore-end -->
```

Everything between the start and end markers is preserved verbatim,
including blank lines and indentation.

## Ignore an entire file

Place this comment **anywhere** in the file:

```svg
<!-- dprint-ignore-file -->
```

The formatter returns the file unchanged.

## Prefix variants

Both prefixes work interchangeably:

| Directive                      | Alternative                        |
| ------------------------------ | ---------------------------------- |
| `<!-- dprint-ignore -->`       | `<!-- svg-format-ignore -->`       |
| `<!-- dprint-ignore-start -->` | `<!-- svg-format-ignore-start -->` |
| `<!-- dprint-ignore-end -->`   | `<!-- svg-format-ignore-end -->`   |
| `<!-- dprint-ignore-file -->`  | `<!-- svg-format-ignore-file -->`  |
