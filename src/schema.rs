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
/// config or built-in defaults.
#[derive(Clone, Debug, Default, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DprintSvgConfigSchema {
    /// Whether the configuration is not allowed to be overridden or extended.
    pub locked: Option<bool>,

    /// Fallback line width for formatting decisions when maxInlineTagWidth is
    /// not provided.
    #[schemars(range(min = 1))]
    pub line_width: Option<u32>,

    /// Maximum inline tag width before wrapping attributes or children.
    #[schemars(range(min = 1))]
    pub max_inline_tag_width: Option<u32>,

    /// Use tabs for indentation instead of spaces.
    pub use_tabs: Option<bool>,

    /// Indent width when useTabs is false.
    #[schemars(range(min = 1))]
    pub indent_width: Option<u8>,

    /// The newline kind to write.
    pub new_line_kind: Option<NewLineKindConfig>,

    /// Attribute ordering strategy.
    pub attribute_sort: Option<AttributeSortConfig>,

    /// Attribute line-breaking strategy.
    pub attribute_layout: Option<AttributeLayoutConfig>,

    /// Maximum number of attributes per line in multi-line mode.
    #[schemars(range(min = 1))]
    pub attributes_per_line: Option<u32>,

    /// Whether to include a space before '/>' in self-closing tags.
    pub space_before_self_close: Option<bool>,

    /// Quote style for attribute values.
    pub quote_style: Option<QuoteStyleConfig>,

    /// Indent style for wrapped attributes.
    pub wrapped_attribute_indent: Option<WrappedAttributeIndentConfig>,

    /// How text-node whitespace is handled.
    pub text_content: Option<TextContentModeConfig>,

    /// How blank lines between sibling elements are handled.
    pub blank_lines: Option<BlankLinesConfig>,

    /// Whether to delegate embedded content (CSS, JS, HTML) to host plugins.
    pub format_embedded_content: Option<bool>,
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
    obj.insert(
        "$id".to_string(),
        json!(format!(
            "https://plugins.dprint.dev/kjanat/dprint-plugin-svg/{}/schema.json",
            env!("CARGO_PKG_VERSION")
        )),
    );

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
