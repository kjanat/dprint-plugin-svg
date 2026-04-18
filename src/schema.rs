//! JSON Schema generation types for the dprint SVG plugin configuration.
//!
//! Gated behind the `schema` feature. The schema struct wraps config enums
//! from [`crate`] (which conditionally derive `JsonSchema`) so there is a
//! single source of truth for enum variants.

use schemars::{JsonSchema, Schema, generate::SchemaSettings};
use serde::Serialize;
use serde_json::{Map, Value, json};

use crate::{
    AttributeLayoutConfig, AttributeSortConfig, BlankLinesConfig, NewLineKindConfig,
    QuoteStyleConfig, TextContentModeConfig, WrappedAttributeIndentConfig,
};

/// # Top-level configuration schema for the dprint SVG plugin.
///
/// All fields are optional — omitted values fall back to dprint global
/// config or built-in defaults. Only plugin-owned options emit a JSON
/// Schema `"default"`; options inherited from globals (`lineWidth`,
/// `useTabs`, `indentWidth`, `newLineKind`) describe the fallback instead.
#[derive(Clone, Debug, Default, Serialize, JsonSchema)]
#[schemars(
    title = "dprint SVG plugin configuration",
    description = "All fields are optional. Options not set here inherit from the dprint global configuration."
)]
#[serde(rename_all = "camelCase")]
pub struct DprintSvgConfigSchema {
    /// Whether the configuration is not allowed to be overridden or extended.
    pub locked: Option<bool>,

    /// Fallback line width for formatting decisions when maxInlineTagWidth is
    /// not provided. Inherited from dprint global `lineWidth` when unset.
    #[schemars(
        range(min = 1),
        description = "Fallback line width. Inherited from dprint global lineWidth when unset. https://dprint-svg.kjanat.com/config/line-width.html"
    )]
    pub line_width: Option<u32>,

    /// Maximum inline tag width before wrapping attributes or children.
    #[schemars(
        range(min = 1),
        description = "Maximum inline tag width before wrapping attributes. https://dprint-svg.kjanat.com/config/max-inline-tag-width.html"
    )]
    pub max_inline_tag_width: Option<u32>,

    /// Use tabs for indentation instead of spaces. Inherited from dprint
    /// global `useTabs` when unset.
    #[schemars(
        description = "Use tabs for indentation. Inherited from dprint global useTabs when unset. https://dprint-svg.kjanat.com/config/use-tabs.html"
    )]
    pub use_tabs: Option<bool>,

    /// Indent width when useTabs is false. Inherited from dprint global
    /// `indentWidth` when unset.
    #[schemars(
        range(min = 1),
        description = "Indent width when useTabs is false. Inherited from dprint global indentWidth when unset. https://dprint-svg.kjanat.com/config/indent-width.html"
    )]
    pub indent_width: Option<u8>,

    /// The newline kind to write. Inherited from dprint global `newLineKind`
    /// when unset.
    #[schemars(
        description = "Line ending style. Inherited from dprint global newLineKind when unset. https://dprint-svg.kjanat.com/config/new-line-kind.html"
    )]
    pub new_line_kind: Option<NewLineKindConfig>,

    /// Attribute ordering strategy.
    #[serde(default = "defaults::attribute_sort")]
    #[schemars(
        description = "Attribute ordering strategy. https://dprint-svg.kjanat.com/config/attribute-sort.html"
    )]
    pub attribute_sort: Option<AttributeSortConfig>,

    /// Attribute line-breaking strategy.
    #[serde(default = "defaults::attribute_layout")]
    #[schemars(
        description = "Attribute wrapping mode. https://dprint-svg.kjanat.com/config/attribute-layout.html"
    )]
    pub attribute_layout: Option<AttributeLayoutConfig>,

    /// Maximum number of attributes per line in multi-line mode.
    #[schemars(
        range(min = 1),
        description = "Maximum attributes per line in multi-line mode. https://dprint-svg.kjanat.com/config/attributes-per-line.html"
    )]
    #[serde(default = "defaults::attributes_per_line")]
    pub attributes_per_line: Option<u32>,

    /// Whether to include a space before '/>' in self-closing tags.
    #[serde(default = "defaults::space_before_self_close")]
    #[schemars(
        description = "Include a space before /> in self-closing tags. https://dprint-svg.kjanat.com/config/space-before-self-close.html"
    )]
    pub space_before_self_close: Option<bool>,

    /// Quote style for attribute values.
    #[serde(default = "defaults::quote_style")]
    #[schemars(
        description = "Quote style for attribute values. https://dprint-svg.kjanat.com/config/quote-style.html"
    )]
    pub quote_style: Option<QuoteStyleConfig>,

    /// Indent style for wrapped attributes.
    #[serde(default = "defaults::wrapped_attribute_indent")]
    #[schemars(
        description = "Indentation strategy for wrapped attributes. https://dprint-svg.kjanat.com/config/wrapped-attribute-indent.html"
    )]
    pub wrapped_attribute_indent: Option<WrappedAttributeIndentConfig>,

    /// How text-node whitespace is handled.
    #[serde(default = "defaults::text_content")]
    #[schemars(
        description = "How text-node whitespace is handled. https://dprint-svg.kjanat.com/config/text-content.html"
    )]
    pub text_content: Option<TextContentModeConfig>,

    /// How blank lines between sibling elements are handled.
    #[serde(default = "defaults::blank_lines")]
    #[schemars(
        description = "How blank lines between sibling elements are handled. https://dprint-svg.kjanat.com/config/blank-lines.html"
    )]
    pub blank_lines: Option<BlankLinesConfig>,

    /// Whether to delegate embedded content (CSS, JS, HTML) to host plugins.
    #[serde(default = "defaults::format_embedded_content")]
    #[schemars(
        description = "Delegate embedded content (CSS, JS, HTML) to host plugins. https://dprint-svg.kjanat.com/config/format-embedded-content.html"
    )]
    pub format_embedded_content: Option<bool>,
}

/// Default value functions for plugin-owned schema fields.
///
/// Each default sources from `svg_format::FormatOptions::default()`
/// through the `unmap_*` helpers in [`crate`], so schema defaults are
/// guaranteed to track the upstream library. Options not modeled by
/// the library (`lineWidth`, `newLineKind`, `formatEmbeddedContent`,
/// `useTabs`, `indentWidth`) are inherited from dprint global config
/// at resolve time — their schema entries carry no `default`.
mod defaults {
    use super::*;
    use crate::{
        unmap_attribute_layout, unmap_attribute_sort, unmap_blank_lines, unmap_quote_style,
        unmap_text_content, unmap_wrapped_attribute_indent,
    };

    fn svg() -> svg_format::FormatOptions {
        svg_format::FormatOptions::default()
    }

    pub fn attribute_sort() -> Option<AttributeSortConfig> {
        Some(unmap_attribute_sort(svg().attribute_sort))
    }
    pub fn attribute_layout() -> Option<AttributeLayoutConfig> {
        Some(unmap_attribute_layout(svg().attribute_layout))
    }
    pub fn attributes_per_line() -> Option<u32> {
        Some(u32::try_from(svg().attributes_per_line).unwrap_or(1))
    }
    pub fn space_before_self_close() -> Option<bool> {
        Some(svg().space_before_self_close)
    }
    pub fn quote_style() -> Option<QuoteStyleConfig> {
        Some(unmap_quote_style(svg().quote_style))
    }
    pub fn wrapped_attribute_indent() -> Option<WrappedAttributeIndentConfig> {
        Some(unmap_wrapped_attribute_indent(
            svg().wrapped_attribute_indent,
        ))
    }
    pub fn text_content() -> Option<TextContentModeConfig> {
        Some(unmap_text_content(svg().text_content))
    }
    pub fn blank_lines() -> Option<BlankLinesConfig> {
        Some(unmap_blank_lines(svg().blank_lines))
    }
    pub fn format_embedded_content() -> Option<bool> {
        Some(true)
    }
}

/// Generate the raw JSON Schema for [`DprintSvgConfigSchema`] using draft-07.
pub fn generate_root_schema() -> Schema {
    SchemaSettings::draft07()
        .into_generator()
        .into_root_schema_for::<DprintSvgConfigSchema>()
}

/// Generate the schema as a [`serde_json::Value`], finalized with `$schema`,
/// `$id`, and stable key ordering.
pub fn generate_schema_value() -> Result<Value, serde_json::Error> {
    let mut value = serde_json::to_value(generate_root_schema())?;
    finalize_schema_value(&mut value);
    Ok(value)
}

/// Inject `$schema` / `$id` metadata and sort top-level keys for stable output.
pub fn finalize_schema_value(value: &mut Value) {
    let obj = value
        .as_object_mut()
        .expect("schema_for!(...) should serialize to an object");

    if let Some(properties) = obj.get_mut("properties").and_then(Value::as_object_mut) {
        reorder_root_keys(properties, &[]);
    }

    if let Some(definitions) = obj.get_mut("definitions").and_then(Value::as_object_mut) {
        reorder_root_keys(definitions, &[]);
    }

    obj.insert(
        "$schema".to_string(),
        json!("http://json-schema.org/draft-07/schema#"),
    );
    obj.insert("$id".to_string(), json!(crate::SCHEMA_URL));

    reorder_root_keys(
        obj,
        &[
            "$schema",
            "$id",
            "title",
            "description",
            "type",
            "properties",
            "definitions",
        ],
    );
}

/// Sort keys in a JSON object: `priority_keys` first (in order), then
/// remaining keys alphabetically. Produces deterministic output.
fn reorder_root_keys(obj: &mut Map<String, Value>, priority_keys: &[&str]) {
    let mut existing = std::mem::take(obj);

    for key in priority_keys {
        if let Some(value) = existing.remove(*key) {
            obj.insert((*key).to_string(), value);
        }
    }

    let mut remaining_keys: Vec<_> = existing.keys().cloned().collect();
    remaining_keys.sort();

    for key in remaining_keys {
        if let Some(value) = existing.remove(&key) {
            obj.insert(key, value);
        }
    }
}
