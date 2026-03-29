# Embedded Content Formatting & Text Content Mode

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable the SVG formatter to delegate `<style>`, `<script>`, and `<foreignObject>` content to external formatters (CSS, JS, HTML) via a callback, and add a configurable text content whitespace mode.

**Architecture:** The `svg-format` crate gains a new public function `format_with_host()` that accepts an embedded-formatting closure alongside `FormatOptions`. The existing `format_with_options()` becomes a convenience wrapper calling `format_with_host` with a no-op. The dprint plugin wires its `format_with_host` callback into this closure. A new `TextContentMode` enum (collapse/maintain/prettify) controls how text nodes are whitespace-handled, independent of embedded formatting.

**Tech Stack:** Rust, tree-sitter, dprint-core 0.67

---

## File Map

### svg-format crate (`/home/kjanat/svg-language-server/crates/svg-format/`)

| File         | Action | Responsibility                                                                                                                                                                     |
| ------------ | ------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `src/lib.rs` | Modify | Add `TextContentMode`, `EmbeddedLanguage`, `EmbeddedContent` types; add `format_with_host()` function; modify `Formatter` methods to thread callback and respect text content mode |

### dprint-plugin-svg (`/home/kjanat/dprint-plugin-svg/`)

| File                                              | Action | Responsibility                                                                                                                                    |
| ------------------------------------------------- | ------ | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| `src/lib.rs`                                      | Modify | Add `TextContentModeConfig` enum, `formatEmbeddedContent` config; wire `format_with_host` into `svg_format::format_with_host()`; import new types |
| `src/schema.rs`                                   | Modify | Add `text_content` and `format_embedded_content` to schema struct                                                                                 |
| `tests/plugin_settings.rs`                        | Modify | Add tests for new config options and embedded formatting behavior                                                                                 |
| `tests/configs/embedded-disabled.dprint.json`     | Create | Fixture for `formatEmbeddedContent: false`                                                                                                        |
| `tests/configs/text-content-collapse.dprint.json` | Create | Fixture for `textContent: "collapse"`                                                                                                             |
| `tests/configs/text-content-maintain.dprint.json` | Create | Fixture for `textContent: "maintain"`                                                                                                             |

---

## Task 1: Add `TextContentMode` to svg-format

**Files:**

- Modify: `/home/kjanat/svg-language-server/crates/svg-format/src/lib.rs:1-86`

- [ ] **Step 1: Write failing test for `TextContentMode::Maintain`**

In the `#[cfg(test)] mod tests` block at the bottom of `lib.rs`, add:

```rust
#[test]
fn text_content_maintain_preserves_relative_indentation() {
    let input = "<svg><text>\n  hello\n    world\n</text></svg>";
    let options = FormatOptions {
        text_content: TextContentMode::Maintain,
        ..Default::default()
    };
    let expected = "<svg>\n\t<text>\n\t\t hello\n\t\t   world\n\t</text>\n</svg>";
    assert_eq!(format_with_options(input, options), expected);
}

#[test]
fn text_content_collapse_trims_and_joins() {
    let input = "<svg><text>\n  hello  \n    world  \n</text></svg>";
    let options = FormatOptions {
        text_content: TextContentMode::Collapse,
        ..Default::default()
    };
    // Collapse strips excess whitespace per line, removes blank lines
    let expected = "<svg>\n\t<text>\n\t\thello\n\t\tworld\n\t</text>\n</svg>";
    assert_eq!(format_with_options(input, options), expected);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd /home/kjanat/svg-language-server && cargo test -p svg-format text_content 2>&1 | head -20`
Expected: compilation error — `TextContentMode` doesn't exist yet.

- [ ] **Step 3: Add the `TextContentMode` enum and integrate into `FormatOptions`**

Add after the `WrappedAttributeIndent` enum (after line 47):

```rust
/// How the formatter handles whitespace in text nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextContentMode {
    /// Trim each line, remove blank lines, re-indent to SVG depth.
    Collapse,
    /// Preserve content structure; dedent then re-indent to SVG depth.
    #[default]
    Maintain,
    /// Re-indent each non-empty line to SVG depth after trimming.
    Prettify,
}
```

Add the field to `FormatOptions` (inside the struct):

```rust
/// How text-node whitespace is handled.
pub text_content: TextContentMode,
```

Add to the `Default` impl:

```rust
text_content: TextContentMode::Maintain,
```

- [ ] **Step 4: Modify `write_text_node` to respect `TextContentMode`**

Replace the current `write_text_node` method (lines 317-330) with:

```rust
fn write_text_node(&mut self, node: Node<'_>, depth: usize) {
    let text = self.node_text(node).to_string();
    if text.trim().is_empty() {
        return;
    }

    match self.options.text_content {
        TextContentMode::Collapse => {
            for line in text.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                self.write_line(depth, trimmed);
            }
        }
        TextContentMode::Maintain => {
            self.write_preserved_block_text(node, depth);
        }
        TextContentMode::Prettify => {
            for line in text.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                self.write_line(depth, trimmed);
            }
        }
    }
}
```

Note: `Collapse` and `Prettify` currently produce the same output for text nodes. The difference will matter when inline-element handling is added later. For now they behave identically — trim per line, skip blanks, re-indent.

- [ ] **Step 5: Run tests**

Run: `cd /home/kjanat/svg-language-server && cargo test -p svg-format 2>&1 | tail -5`
Expected: all tests pass, including the two new ones.

- [ ] **Step 6: Commit**

```bash
cd /home/kjanat/svg-language-server
git add crates/svg-format/src/lib.rs
git commit -m "feat(svg-format): add TextContentMode option"
```

---

## Task 2: Add embedded content callback to svg-format

**Files:**

- Modify: `/home/kjanat/svg-language-server/crates/svg-format/src/lib.rs`

- [ ] **Step 1: Write failing test for embedded formatting**

Add to the test module:

```rust
#[test]
fn format_with_host_delegates_style_content() {
    let input = "<svg><style>.a{fill:red}</style></svg>";
    let mut called_with_lang = None;
    let mut called_with_content = None;
    let result = format_with_host(input, FormatOptions::default(), &mut |req| {
        called_with_lang = Some(req.language);
        called_with_content = Some(req.content.to_string());
        Some(".a {\n  fill: red;\n}".to_string())
    });
    assert_eq!(called_with_lang, Some(EmbeddedLanguage::Css));
    assert_eq!(called_with_content.as_deref(), Some(".a{fill:red}"));
    // Re-indented CSS at depth 2 (inside <svg><style>)
    assert_eq!(
        result,
        "<svg>\n\t<style>\n\t\t.a {\n\t\t  fill: red;\n\t\t}\n\t</style>\n</svg>"
    );
}

#[test]
fn format_with_host_falls_back_when_callback_returns_none() {
    let input = "<svg><style>.a { fill: red; }</style></svg>";
    let result = format_with_host(input, FormatOptions::default(), &mut |_| None);
    // Should fall back to preserved block text behavior
    let fallback = format_with_options(input, FormatOptions::default());
    assert_eq!(result, fallback);
}

#[test]
fn format_with_host_delegates_script_content() {
    let input = "<svg><script>alert(1)</script></svg>";
    let mut called_lang = None;
    format_with_host(input, FormatOptions::default(), &mut |req| {
        called_lang = Some(req.language);
        None
    });
    assert_eq!(called_lang, Some(EmbeddedLanguage::JavaScript));
}

#[test]
fn format_with_host_delegates_foreign_object_content() {
    let input =
        "<svg><foreignObject width=\"200\" height=\"200\"><div>hello</div></foreignObject></svg>";
    let mut called_lang = None;
    let mut called_content = None;
    format_with_host(input, FormatOptions::default(), &mut |req| {
        called_lang = Some(req.language);
        called_content = Some(req.content.to_string());
        None
    });
    assert_eq!(called_lang, Some(EmbeddedLanguage::Html));
    assert!(called_content.unwrap().contains("<div>hello</div>"));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd /home/kjanat/svg-language-server && cargo test -p svg-format format_with_host 2>&1 | head -20`
Expected: compilation error — `format_with_host`, `EmbeddedLanguage`, `EmbeddedContent` don't exist.

- [ ] **Step 3: Add `EmbeddedLanguage` and `EmbeddedContent` types**

Add after `TextContentMode` (before `FormatOptions`):

```rust
/// The language of embedded content found within an SVG element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddedLanguage {
    /// CSS inside `<style>`.
    Css,
    /// JavaScript inside `<script>`.
    JavaScript,
    /// HTML/XHTML inside `<foreignObject>`.
    Html,
}

/// A request to format embedded content within an SVG document.
pub struct EmbeddedContent<'a> {
    /// The language of the embedded content.
    pub language: EmbeddedLanguage,
    /// The raw content text (common indent removed).
    pub content: &'a str,
    /// The nesting depth in the SVG tree where this content lives.
    pub indent_depth: usize,
}
```

- [ ] **Step 4: Add `format_with_host` public function**

Add after `format_with_options` (after line 111):

```rust
/// Format an SVG source string, delegating embedded content to a callback.
///
/// The callback receives [`EmbeddedContent`] for `<style>`, `<script>`, and
/// `<foreignObject>` blocks. Return `Some(formatted)` to use the formatted
/// result, or `None` to fall back to the default text-preservation behavior.
pub fn format_with_host(
    source: &str,
    options: FormatOptions,
    format_embedded: &mut dyn FnMut(EmbeddedContent<'_>) -> Option<String>,
) -> String {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_svg::LANGUAGE.into())
        .expect("SVG grammar");

    let Some(tree) = parser.parse(source.as_bytes(), None) else {
        return source.to_owned();
    };

    if tree.root_node().has_error() {
        return source.to_owned();
    }

    let mut formatter = Formatter::new(source.as_bytes(), options);
    formatter.format_node(tree.root_node(), 0, format_embedded);
    formatter.finish(source)
}
```

Update `format_with_options` to delegate:

```rust
pub fn format_with_options(source: &str, options: FormatOptions) -> String {
    format_with_host(source, options, &mut |_| None)
}
```

- [ ] **Step 5: Thread `format_embedded` through `Formatter` methods**

Update every recursive method to accept the callback parameter. The methods that need it: `format_node`, `format_children`, `format_element_like`.

Update `format_node` signature and body:

```rust
fn format_node(
    &mut self,
    node: Node<'_>,
    depth: usize,
    fmt: &mut dyn FnMut(EmbeddedContent<'_>) -> Option<String>,
) {
    match node.kind() {
        "source_file" => self.format_children(node, depth, fmt),
        "svg_root_element" | "element" => self.format_element_like(node, depth, fmt),
        "start_tag" => self.write_tag_node(node, depth, false),
        "self_closing_tag" => self.write_tag_node(node, depth, true),
        "end_tag" => {
            let text = self.node_text(node).trim().to_string();
            self.write_line(depth, &text);
        }
        "style_text_double" | "style_text_single" | "script_text_double" | "script_text_single" => {
            self.write_preserved_block_text(node, depth);
        }
        "text" | "raw_text" => {
            self.write_text_node(node, depth);
        }
        "comment"
        | "cdata_section"
        | "doctype"
        | "processing_instruction"
        | "xml_declaration"
        | "entity_reference"
        | "erroneous_end_tag" => {
            let text = self.node_text(node).trim().to_string();
            self.write_line(depth, &text);
        }
        _ => self.format_children(node, depth, fmt),
    }
}
```

Update `format_children`:

```rust
fn format_children(
    &mut self,
    node: Node<'_>,
    depth: usize,
    fmt: &mut dyn FnMut(EmbeddedContent<'_>) -> Option<String>,
) {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        self.format_node(child, depth, fmt);
    }
}
```

Update `format_element_like` — this is the critical method. It needs to:

1. Detect `<foreignObject>` by checking the tag name of the start_tag child
2. For style/script text nodes, try the embedded callback before falling back
3. Extract raw content between start/end tags for foreignObject

```rust
fn format_element_like(
    &mut self,
    node: Node<'_>,
    depth: usize,
    fmt: &mut dyn FnMut(EmbeddedContent<'_>) -> Option<String>,
) {
    let mut cursor = node.walk();
    let children: Vec<Node<'_>> = node.named_children(&mut cursor).collect();
    if children.is_empty() {
        return;
    }

    // Self-closing form: <rect .../>
    if children.len() == 1 && children[0].kind() == "self_closing_tag" {
        self.format_node(children[0], depth, fmt);
        return;
    }

    // Check if this is a foreignObject element
    let is_foreign_object = children
        .iter()
        .find(|c| c.kind() == "start_tag")
        .map(|tag| {
            let text = self.node_text(*tag).trim();
            text.strip_prefix('<')
                .and_then(|s| s.split_whitespace().next())
                .map_or(false, |name| name == "foreignObject")
        })
        .unwrap_or(false);

    if is_foreign_object {
        if let Some(formatted) = self.try_format_foreign_object(node, depth, fmt) {
            // Write start tag
            if let Some(start) = children.iter().find(|c| c.kind() == "start_tag") {
                self.format_node(*start, depth, fmt);
            }
            self.write_indented_block(&formatted, depth + 1);
            // Write end tag
            if let Some(end) = children.iter().find(|c| c.kind() == "end_tag") {
                let text = self.node_text(*end).trim().to_string();
                self.write_line(depth, &text);
            }
            return;
        }
    }

    for child in children {
        match child.kind() {
            "start_tag" | "end_tag" => self.format_node(child, depth, fmt),
            "style_text_double" | "style_text_single" => {
                if self.node_text(child).trim().is_empty() {
                    continue;
                }
                let lang = EmbeddedLanguage::Css;
                if !self.try_format_embedded_text(child, lang, depth + 1, fmt) {
                    self.write_preserved_block_text(child, depth + 1);
                }
            }
            "script_text_double" | "script_text_single" => {
                if self.node_text(child).trim().is_empty() {
                    continue;
                }
                let lang = EmbeddedLanguage::JavaScript;
                if !self.try_format_embedded_text(child, lang, depth + 1, fmt) {
                    self.write_preserved_block_text(child, depth + 1);
                }
            }
            "text" | "raw_text" => {
                if !self.node_text(child).trim().is_empty() {
                    self.write_text_node(child, depth + 1);
                }
            }
            _ => self.format_node(child, depth + 1, fmt),
        }
    }
}
```

- [ ] **Step 6: Add helper methods for embedded formatting**

Add these methods to the `Formatter` impl block:

```rust
/// Try to format embedded text (style/script content) via the callback.
/// Returns `true` if the callback produced a result, `false` to fall back.
fn try_format_embedded_text(
    &mut self,
    node: Node<'_>,
    language: EmbeddedLanguage,
    depth: usize,
    fmt: &mut dyn FnMut(EmbeddedContent<'_>) -> Option<String>,
) -> bool {
    let raw = self.node_text(node).to_string();
    let content = dedent_block(&raw);
    if content.is_empty() {
        return false;
    }
    let req = EmbeddedContent {
        language,
        content: &content,
        indent_depth: depth,
    };
    if let Some(formatted) = fmt(req) {
        self.write_indented_block(&formatted, depth);
        true
    } else {
        false
    }
}

/// Try to format foreignObject inner content via the callback.
/// Returns `Some(formatted)` if successful.
fn try_format_foreign_object(
    &mut self,
    node: Node<'_>,
    _depth: usize,
    fmt: &mut dyn FnMut(EmbeddedContent<'_>) -> Option<String>,
) -> Option<String> {
    let mut cursor = node.walk();
    let children: Vec<Node<'_>> = node.named_children(&mut cursor).collect();
    let start_tag = children.iter().find(|c| c.kind() == "start_tag")?;
    let end_tag = children.iter().find(|c| c.kind() == "end_tag")?;

    let content_start = start_tag.end_byte();
    let content_end = end_tag.start_byte();
    if content_start >= content_end {
        return None;
    }

    let raw = std::str::from_utf8(&self.source[content_start..content_end]).ok()?;
    let content = dedent_block(raw);
    if content.is_empty() {
        return None;
    }

    let req = EmbeddedContent {
        language: EmbeddedLanguage::Html,
        content: &content,
        indent_depth: _depth + 1,
    };
    fmt(req)
}

/// Write pre-formatted text, re-indented to the given depth.
fn write_indented_block(&mut self, text: &str, depth: usize) {
    for line in text.lines() {
        if line.trim().is_empty() {
            self.out.push('\n');
        } else {
            self.write_line(depth, line.trim_start());
        }
    }
}
```

Add a free function for dedenting:

```rust
/// Remove common leading whitespace from a block of text.
fn dedent_block(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let first_non_empty = lines.iter().position(|l| !l.trim().is_empty());
    let last_non_empty = lines.iter().rposition(|l| !l.trim().is_empty());
    let (Some(start), Some(end)) = (first_non_empty, last_non_empty) else {
        return String::new();
    };

    let block = &lines[start..=end];
    let min_indent = block
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.chars().take_while(|c| c.is_whitespace()).count())
        .min()
        .unwrap_or(0);

    block
        .iter()
        .map(|l| {
            if l.trim().is_empty() {
                ""
            } else {
                &l[l.chars()
                    .take(min_indent)
                    .map(|c| c.len_utf8())
                    .sum::<usize>()..]
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
```

- [ ] **Step 7: Run all tests**

Run: `cd /home/kjanat/svg-language-server && cargo test -p svg-format 2>&1 | tail -10`
Expected: all tests pass, including the 4 new embedded formatting tests.

- [ ] **Step 8: Commit**

```bash
cd /home/kjanat/svg-language-server
git add crates/svg-format/src/lib.rs
git commit -m "feat(svg-format): add format_with_host for embedded content delegation"
```

---

## Task 3: Wire embedded formatting in the dprint plugin

**Files:**

- Modify: `/home/kjanat/dprint-plugin-svg/src/lib.rs`
- Create: `/home/kjanat/dprint-plugin-svg/tests/configs/text-content-maintain.dprint.json`
- Create: `/home/kjanat/dprint-plugin-svg/tests/configs/text-content-collapse.dprint.json`
- Create: `/home/kjanat/dprint-plugin-svg/tests/configs/embedded-disabled.dprint.json`

- [ ] **Step 1: Add new config enums and fields**

In `src/lib.rs`, add the `TextContentModeConfig` enum after `NewLineKindConfig`:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum TextContentModeConfig {
    Collapse,
    Maintain,
    Prettify,
}
dprint_core::generate_str_to_from![
    TextContentModeConfig,
    [Collapse, "collapse"],
    [Maintain, "maintain"],
    [Prettify, "prettify"]
];
```

Add two fields to `Configuration`:

```rust
pub struct Configuration {
    // ... existing fields ...
    pub text_content: TextContentModeConfig,
    pub format_embedded_content: bool,
}
```

- [ ] **Step 2: Add config parsing in `resolve_config`**

Add after the `new_line_kind` parsing block (before `attributes_per_line` validation):

```rust
let text_content = get_value(
    &mut config,
    "textContent",
    TextContentModeConfig::Maintain,
    &mut diagnostics,
);
let format_embedded_content = get_value(
    &mut config,
    "formatEmbeddedContent",
    true,
    &mut diagnostics,
);
```

Add the new fields to the `Configuration` struct initialization:

```rust
config: Configuration {
    // ... existing fields ...
    text_content,
    format_embedded_content,
},
```

- [ ] **Step 3: Add mapping function and update imports**

Update the svg_format import to include new types:

```rust
use svg_format::{
    AttributeLayout, AttributeSort, EmbeddedContent, FormatOptions, QuoteStyle, TextContentMode,
    WrappedAttributeIndent, format_with_host, format_with_options,
};
```

Add the mapping function:

```rust
fn map_text_content(value: TextContentModeConfig) -> TextContentMode {
    match value {
        TextContentModeConfig::Collapse => TextContentMode::Collapse,
        TextContentModeConfig::Maintain => TextContentMode::Maintain,
        TextContentModeConfig::Prettify => TextContentMode::Prettify,
    }
}
```

- [ ] **Step 4: Update the `format` method to use `format_with_host`**

Replace the format method body (lines 241-282):

```rust
fn format(
    &mut self,
    request: SyncFormatRequest<Configuration>,
    mut format_with_host_callback: impl FnMut(SyncHostFormatRequest) -> FormatResult,
) -> FormatResult {
    if request.range.is_some() || request.token.is_cancelled() {
        return Ok(None);
    }

    let source = std::str::from_utf8(&request.file_bytes).map_err(|err| {
        anyhow!(
            "Could not decode '{}' as UTF-8: {err}",
            request.file_path.display()
        )
    })?;

    let options = FormatOptions {
        indent_width: request.config.indent_width as usize,
        insert_spaces: !request.config.use_tabs,
        max_inline_tag_width: request.config.max_inline_tag_width as usize,
        attribute_sort: map_attribute_sort(request.config.attribute_sort),
        attribute_layout: map_attribute_layout(request.config.attribute_layout),
        attributes_per_line: request.config.attributes_per_line as usize,
        space_before_self_close: request.config.space_before_self_close,
        quote_style: map_quote_style(request.config.quote_style),
        wrapped_attribute_indent: map_wrapped_attribute_indent(
            request.config.wrapped_attribute_indent,
        ),
        text_content: map_text_content(request.config.text_content),
    };

    let line_width = request.config.max_inline_tag_width;
    let indent_width = request.config.indent_width as u32;
    let do_embedded = request.config.format_embedded_content;

    let mut formatted = format_with_host(source, options, &mut |embedded| {
        if !do_embedded {
            return None;
        }
        let ext = match embedded.language {
            svg_format::EmbeddedLanguage::Css => "css",
            svg_format::EmbeddedLanguage::JavaScript => "js",
            svg_format::EmbeddedLanguage::Html => "html",
        };
        let path = std::path::PathBuf::from(format!("file.{ext}"));
        let adjusted_width = line_width.saturating_sub(embedded.indent_depth as u32 * indent_width);
        let mut overrides = ConfigKeyMap::new();
        overrides.insert(
            "lineWidth".into(),
            dprint_core::configuration::ConfigKeyValue::Number(adjusted_width as i32),
        );
        match format_with_host_callback(SyncHostFormatRequest {
            file_path: &path,
            file_bytes: embedded.content.as_bytes(),
            range: None,
            override_config: &overrides,
        }) {
            Ok(Some(bytes)) => String::from_utf8(bytes).ok(),
            _ => None,
        }
    });

    let newline = resolve_new_line_kind(source, request.config.new_line_kind);
    if newline != "\n" {
        formatted = formatted.replace('\n', newline);
    }

    if formatted.as_bytes() == request.file_bytes.as_slice() {
        Ok(None)
    } else {
        Ok(Some(formatted.into_bytes()))
    }
}
```

- [ ] **Step 5: Create test fixture files**

`tests/configs/text-content-maintain.dprint.json`:

```json
{
  "svg": {
    "textContent": "maintain"
  }
}
```

`tests/configs/text-content-collapse.dprint.json`:

```json
{
  "svg": {
    "textContent": "collapse"
  }
}
```

`tests/configs/embedded-disabled.dprint.json`:

```json
{
  "svg": {
    "formatEmbeddedContent": false
  }
}
```

- [ ] **Step 6: Add plugin-level tests**

Add to `tests/plugin_settings.rs`:

```rust
#[test]
fn resolve_config_text_content_maintain() {
    let result = resolve_configuration("text-content-maintain.dprint.json");
    assert!(result.diagnostics.is_empty());
    assert_eq!(result.config.text_content, TextContentModeConfig::Maintain);
}

#[test]
fn resolve_config_text_content_collapse() {
    let result = resolve_configuration("text-content-collapse.dprint.json");
    assert!(result.diagnostics.is_empty());
    assert_eq!(result.config.text_content, TextContentModeConfig::Collapse);
}

#[test]
fn resolve_config_embedded_disabled() {
    let result = resolve_configuration("embedded-disabled.dprint.json");
    assert!(result.diagnostics.is_empty());
    assert!(!result.config.format_embedded_content);
}

#[test]
fn format_embedded_content_disabled_preserves_style() {
    let result = resolve_configuration("embedded-disabled.dprint.json");
    assert!(result.diagnostics.is_empty());

    let input = "<svg><style>.a{fill:red}</style></svg>";
    let output = format_with_config(&result.config, input).expect("should format");
    // With embedded disabled, style content is preserved (not sent to host)
    assert!(output.contains(".a{fill:red}"));
}
```

Update the import at the top of the test file to include:

```rust
use dprint_plugin_svg::{Configuration, SvgWasmPluginHandler, TextContentModeConfig};
```

- [ ] **Step 7: Run all tests**

Run: `cd /home/kjanat/dprint-plugin-svg && cargo test 2>&1 | tail -10`
Expected: all tests pass.

- [ ] **Step 8: Commit**

```bash
cd /home/kjanat/dprint-plugin-svg
git add src/lib.rs tests/plugin_settings.rs tests/configs/
git commit -m "feat: wire embedded content formatting and text content mode"
```

---

## Task 4: Update the JSON schema

**Files:**

- Modify: `/home/kjanat/dprint-plugin-svg/src/schema.rs`

- [ ] **Step 1: Add new fields to `DprintSvgConfigSchema`**

Add these fields to the struct:

```rust
    /// How text-node whitespace is handled.
    pub text_content: Option<TextContentModeConfig>,

    /// Whether to delegate embedded content (CSS, JS, HTML) to host plugins.
    pub format_embedded_content: Option<bool>,
```

Update the import to include `TextContentModeConfig`:

```rust
use crate::{
    AttributeLayoutConfig, AttributeSortConfig, NewLineKindConfig, QuoteStyleConfig,
    TextContentModeConfig, WrappedAttributeIndentConfig,
};
```

- [ ] **Step 2: Regenerate the schema**

Run: `cd /home/kjanat/dprint-plugin-svg && cargo run --features schema --bin generate-schema`
Expected: `deployment/schema.json` is updated with the new fields.

- [ ] **Step 3: Run schema tests to verify no drift**

Run: `cd /home/kjanat/dprint-plugin-svg && cargo test -p dprint-plugin-svg schema 2>&1 | tail -10`
Expected: all schema tests pass.

- [ ] **Step 4: Commit**

```bash
cd /home/kjanat/dprint-plugin-svg
git add src/schema.rs deployment/schema.json
git commit -m "feat: add textContent and formatEmbeddedContent to schema"
```

---

## Task 5: Update svg-format dependency rev and verify end-to-end

**Files:**

- Modify: `/home/kjanat/dprint-plugin-svg/Cargo.toml`

- [ ] **Step 1: Get the latest svg-format commit hash**

Run: `cd /home/kjanat/svg-language-server && git rev-parse HEAD`
Record the hash.

- [ ] **Step 2: Update Cargo.toml rev**

Update the `svg-format` dependency's `rev` field in `/home/kjanat/dprint-plugin-svg/Cargo.toml` to the new commit hash.

- [ ] **Step 3: Update lockfile**

Run: `cd /home/kjanat/dprint-plugin-svg && cargo update -p svg-format`

- [ ] **Step 4: Run full test suite**

Run: `cd /home/kjanat/dprint-plugin-svg && cargo test 2>&1 | tail -15`
Expected: all tests pass.

- [ ] **Step 5: Add idempotency test with embedded content**

Add to `tests/plugin_settings.rs` in the `formatting_is_idempotent` test, extend the `inputs` slice:

```rust
(
    "text-content-maintain.dprint.json",
    "<svg><text>\n  hello\n    world\n</text></svg>",
),
(
    "text-content-collapse.dprint.json",
    "<svg><text>\n  hello\n    world\n</text></svg>",
),
```

- [ ] **Step 6: Run tests again**

Run: `cd /home/kjanat/dprint-plugin-svg && cargo test 2>&1 | tail -10`
Expected: all tests pass including idempotency.

- [ ] **Step 7: Commit**

```bash
cd /home/kjanat/dprint-plugin-svg
git add Cargo.toml Cargo.lock tests/plugin_settings.rs
git commit -m "build: update svg-format rev; add idempotency tests for text modes"
```

---

## Unresolved Questions

- `write_indented_block` strips all leading whitespace with `trim_start()` before re-indenting — correct for CSS/JS (which use their own indentation), but could be wrong for HTML content that has significant leading whitespace. Worth reconsidering after real-world testing?
- The `Collapse` and `Prettify` text content modes currently produce identical output. Should `Collapse` join short multi-line text into a single line (e.g., `<text>hello world</text>` kept inline)?
- Should `lineWidth` adjustment for embedded content use tab width (from global config) instead of indent width when `useTabs` is true?
- Should `<foreignObject>` content extraction handle the case where tree-sitter didn't parse the inner HTML correctly (malformed XHTML)?
