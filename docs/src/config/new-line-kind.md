# newLineKind

Line ending style. Inherited from dprint global newLineKind when unset

|             |                                       |
| ----------- | ------------------------------------- |
| **Type**    | `"auto"` \| `"lf"` \| `"crlf"`        |
| **Default** | *inherited from dprint global config* |

## Values

### `"auto"`

Detect from the source file.

### `"lf"`

Unix-style `\n`.

### `"crlf"`

Windows-style `\r\n`.

## Config

```json
{
  "svg": {
    "newLineKind": null
  }
}
```
