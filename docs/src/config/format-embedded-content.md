# formatEmbeddedContent

Whether to delegate embedded content (`<style>`, `<script>`, `<foreignObject>`) to other dprint plugins for formatting.

|             |           |
| ----------- | --------- |
| **Type**    | `boolean` |
| **Default** | `true`    |

When enabled, the SVG plugin sends embedded content to the dprint host which routes it to the appropriate plugin based on file extension:

| Element           | Language   | Virtual path |
| ----------------- | ---------- | ------------ |
| `<style>`         | CSS        | `file.css`   |
| `<script>`        | JavaScript | `file.js`    |
| `<foreignObject>` | HTML       | `file.html`  |

If no plugin is installed for the language, the content falls back to the [textContent](./text-content.md) mode handling.

The `lineWidth` override sent to the host plugin is adjusted for nesting depth so embedded content respects the available column budget.

## Config

```json
{
  "svg": {
    "formatEmbeddedContent": true
  }
}
```

To disable:

```json
{
  "svg": {
    "formatEmbeddedContent": false
  }
}
```
