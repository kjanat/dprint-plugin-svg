# spaceBeforeSelfClose

Whether to include a space before `/>` in self-closing tags.

|             |           |
| ----------- | --------- |
| **Type**    | `boolean` |
| **Default** | `true`    |

## Example

### `spaceBeforeSelfClose: true` (default)

```svg
<svg>
  <circle cx="50" cy="50" r="25" />
  <path d="M0 0L10 10" />
</svg>
```

### `spaceBeforeSelfClose: false`

```svg
<svg>
  <circle cx="50" cy="50" r="25"/>
  <path d="M0 0L10 10"/>
</svg>
```

## Config

```json
{
  "svg": {
    "spaceBeforeSelfClose": false
  }
}
```
