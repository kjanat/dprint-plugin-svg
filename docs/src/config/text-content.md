# textContent

How text-node whitespace is handled.

|             |                                          |
| ----------- | ---------------------------------------- |
| **Type**    | `"collapse" \| "maintain" \| "prettify"` |
| **Default** | `"maintain"`                             |

## Values

### `"collapse"`

Collapse runs of whitespace into single spaces, trim lines, skip blanks:

```svg
<!-- input -->
<svg>
  <text>  hello   world  </text>
</svg>
```

```svg
<!-- output -->
<svg>
  <text>
    hello world
  </text>
</svg>
```

### `"maintain"` (default)

Preserve content structure — dedent then re-indent to SVG depth:

```svg
<!-- input -->
<svg>
  <text>
    hello
      world
  </text>
</svg>
```

```svg
<!-- output — relative indentation preserved -->
<svg>
  <text>
    hello
      world
  </text>
</svg>
```

### `"prettify"`

Trim each line, remove blank lines, re-indent to SVG depth:

```svg
<!-- input -->
<svg>
  <text>
    hello
      world
  </text>
</svg>
```

```svg
<!-- output — all lines at same depth -->
<svg>
  <text>
    hello
    world
  </text>
</svg>
```

## Config

```json
{
  "svg": {
    "textContent": "maintain"
  }
}
```
