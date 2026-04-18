# blankLines

How blank lines between sibling elements are handled

|             |                                                          |
| ----------- | -------------------------------------------------------- |
| **Type**    | `"remove"` \| `"preserve"` \| `"truncate"` \| `"insert"` |
| **Default** | `"truncate"`                                             |

## Values

### Input

```svg
<svg>
  <rect />


  <!--legend-->
  <circle />
</svg>
```

### `"remove"`

Strip all blank lines between siblings.

```svg
<svg>
  <rect />
  <!--legend-->
  <circle />
</svg>
```

### `"preserve"`

Keep blank lines from source verbatim.

```svg
<svg>
  <rect />


  <!--legend-->
  <circle />
</svg>
```

### `"truncate"`

Collapse 2+ blank lines to exactly 1.

```svg
<svg>
  <rect />

  <!--legend-->
  <circle />
</svg>
```

### `"insert"`

Force exactly 1 blank line between every sibling.

```svg
<svg>
  <rect />

  <!--legend-->

  <circle />
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
