# blankLines

Control blank lines between sibling elements.

|             |                                                    |
| ----------- | -------------------------------------------------- |
| **Type**    | `"remove" \| "preserve" \| "truncate" \| "insert"` |
| **Default** | `"truncate"`                                       |

## Values

### `"remove"`

Strip all blank lines between siblings:

```svg
<!-- input -->
<svg>
  <text>total visits</text>

  <!--Legend-->
  <g transform="translate(46, 248)"></g>
</svg>
```

```svg
<!-- output — all gaps removed -->
<svg>
  <text>total visits</text>
  <!--Legend-->
  <g transform="translate(46, 248)"></g>
</svg>
```

### `"preserve"`

Keep blank lines from source verbatim (0, 1, or 3 — all kept):

```svg
<!-- input -->
<svg>
  <text>total visits</text>



  <!--Legend-->
</svg>
```

```svg
<!-- output — triple gap kept -->
<svg>
  <text>total visits</text>



  <!--Legend-->
</svg>
```

### `"truncate"` (default)

Collapse 2+ blank lines to exactly 1:

```svg
<!-- input -->
<svg>
  <text>total visits</text>



  <!--Legend-->
  <g></g>
</svg>
```

```svg
<!-- output — triple gap collapsed to single -->
<svg>
  <text>total visits</text>

  <!--Legend-->
  <g></g>
</svg>
```

### `"insert"`

Force exactly 1 blank line between every sibling element, even if the source had no gap:

```svg
<!-- input -->
<svg>
  <text>total visits</text>
  <!--Legend-->
  <g></g>
</svg>
```

```svg
<!-- output — gaps inserted everywhere -->
<svg>
  <text>total visits</text>

  <!--Legend-->

  <g></g>
</svg>
```

## Config

```json
{
  "svg": {
    "blankLines": "truncate"
  }
}
```
