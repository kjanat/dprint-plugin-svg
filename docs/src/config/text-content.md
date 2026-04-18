# textContent

How text-node whitespace is handled

|             |                                              |
| ----------- | -------------------------------------------- |
| **Type**    | `"collapse"` \| `"maintain"` \| `"prettify"` |
| **Default** | `"maintain"`                                 |

## Values

### Input

```svg
<text>  hello   world  </text>
```

### `"collapse"`

Collapse whitespace runs to single spaces.

```svg
<text>
  hello world
</text>
```

### `"maintain"`

Preserve relative indentation structure.

```svg
<text>
  hello   world
</text>
```

### `"prettify"`

Trim each line and re-indent to SVG depth.

```svg
<text>
  hello   world
</text>
```

## Config

```json
{
  "svg": {
    "textContent": "maintain"
  }
}
```
