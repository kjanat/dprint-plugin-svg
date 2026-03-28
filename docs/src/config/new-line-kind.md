# newLineKind

The newline character(s) to use in formatted output.

|             |                               |
| ----------- | ----------------------------- |
| **Type**    | `"auto" \| "lf" \| "crlf"`    |
| **Default** | `"auto"` (from global config) |

## Values

### `"auto"` (default)

Detect the newline style from the input file and preserve it. Uses `\n` for new files.

### `"lf"`

Force Unix-style line endings (`\n`). Typical for Linux/macOS environments and most version-controlled SVG files.

### `"crlf"`

Force Windows-style line endings (`\r\n`).

## Config

```json
{
  "svg": {
    "newLineKind": "lf"
  }
}
```
