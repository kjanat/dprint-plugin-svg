# quoteStyle

Quote style for attribute values

|             |                                          |
| ----------- | ---------------------------------------- |
| **Type**    | `"preserve"` \| `"double"` \| `"single"` |
| **Default** | `"preserve"`                             |

## Values

### Input

```svg
<rect id='box' class="hero" />
```

### `"preserve"`

Keep the original quote character.

```svg
<rect id='box' class="hero" />
```

### `"double"`

Normalize to double quotes.

```svg
<rect id="box" class="hero" />
```

### `"single"`

Normalize to single quotes.

```svg
<rect id='box' class='hero' />
```

## Config

```json
{
  "svg": {
    "quoteStyle": "preserve"
  }
}
```
