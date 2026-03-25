//! JSON Schema generation types for the dprint SVG plugin configuration.
//!
//! Gated behind the `schema` feature. The schema struct wraps config enums
//! from [`crate`] (which conditionally derive `JsonSchema`) so there is a
//! single source of truth for enum variants.

use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    AttributeLayoutConfig, AttributeSortConfig, QuoteStyleConfig, WrappedAttributeIndentConfig,
};

/// Newline style. Kept here because [`dprint_core::configuration::RawNewLineKind`]
/// is a foreign type that cannot derive `JsonSchema`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum NewLineKindSchema {
    Auto,
    Lf,
    Crlf,
    System,
}

/// Top-level configuration schema for the dprint SVG plugin.
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
    pub new_line_kind: Option<NewLineKindSchema>,

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
}
