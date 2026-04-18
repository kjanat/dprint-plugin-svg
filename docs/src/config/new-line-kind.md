# newLineKind

Line ending style. Inherited from the top-level `newLineKind` in dprint.json when unset

|             |                                                             |
| ----------- | ----------------------------------------------------------- |
| **Type**    | `"auto"` \| `"lf"` \| `"crlf"`                              |
| **Default** | *inherits from the top-level key in the same `dprint.json`* |

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
