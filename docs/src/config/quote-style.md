# quoteStyle

Quote style for attribute values.

|             |                                      |
| ----------- | ------------------------------------ |
| **Type**    | `"preserve" \| "double" \| "single"` |
| **Default** | `"preserve"`                         |

## Values

### `"preserve"` (default)

Keep whatever quote style the input uses:

```svg
<!-- input with mixed quotes -->
<svg>
  <rect x="10" y='20' width="100" height='50' />
</svg>
```

```svg
<!-- output — quotes unchanged -->
<svg>
  <rect x="10" y='20' width="100" height='50' />
</svg>
```

### `"double"`

Normalize all attribute values to double quotes:

```svg
<!-- input -->
<svg>
  <rect x='10' y='20' width='100' height='50' />
</svg>
```

```svg
<!-- output -->
<svg>
  <rect x="10" y="20" width="100" height="50" />
</svg>
```

### `"single"`

Normalize all attribute values to single quotes:

```svg
<!-- input -->
<svg>
  <rect x="10" y="20" width="100" height="50" />
</svg>
```

```svg
<!-- output -->
<svg>
  <rect x='10' y='20' width='100' height='50' />
</svg>
```

## Config

```json
{
  "svg": {
    "quoteStyle": "double"
  }
}
```
