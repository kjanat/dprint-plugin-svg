# attributeLayout

Attribute wrapping mode

|             |                                               |
| ----------- | --------------------------------------------- |
| **Type**    | `"auto"` \| `"single-line"` \| `"multi-line"` |
| **Default** | `"auto"`                                      |

## Values

### Input

```svg
<linearGradient id="sky" x1="0%" y1="0%"></linearGradient>
```

### `"auto"`

Wrap only when inline width exceeds `maxInlineTagWidth`.

```svg
<linearGradient id="sky" x1="0%" y1="0%"></linearGradient>
```

### `"single-line"`

Always keep all attributes on one line.

```svg
<linearGradient id="sky" x1="0%" y1="0%"></linearGradient>
```

### `"multi-line"`

Always wrap attributes onto separate lines.

```svg
<linearGradient id="sky"
                x1="0%" y1="0%">
</linearGradient>
```

## Config

```json
{
  "svg": {
    "attributeLayout": "auto"
  }
}
```
