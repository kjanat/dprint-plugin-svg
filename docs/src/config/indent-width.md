# indentWidth

Number of spaces per indentation level when `useTabs` is `false`.

|             |                          |
| ----------- | ------------------------ |
| **Type**    | `number`                 |
| **Default** | `2` (from global config) |
| **Range**   | `1`–`255`                |

Has no visible effect when `useTabs` is `true`.

## Example

### `indentWidth: 2`

```svg
<svg>
  <g>
    <rect x="0" y="0" width="10" height="10" />
  </g>
</svg>
```

### `indentWidth: 4`

```svg
<svg>
    <g>
        <rect x="0" y="0" width="10" height="10" />
    </g>
</svg>
```

## Config

```json
{
  "svg": {
    "useTabs": false,
    "indentWidth": 4
  }
}
```
