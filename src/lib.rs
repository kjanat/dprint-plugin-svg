//! # dprint Wasm plugin for formatting SVG files.
//!
//! Bridges the [`svg_format`] crate (tree-sitter-based SVG formatter) to the
//! dprint plugin protocol. Configuration is parsed from the user's
//! `dprint.json` and mapped to [`svg_format::FormatOptions`].
//!
//! Embedded `<style>`, `<script>`, and `<foreignObject>` content is delegated
//! to other dprint plugins via the host callback when
//! [`Configuration::format_embedded_content`] is enabled (the default).

use anyhow::{Result, anyhow};
use dprint_core::configuration::{
    ConfigKeyMap, ConfigurationDiagnostic, GlobalConfiguration, NewLineKind,
    ParseConfigurationError, get_unknown_property_diagnostics, get_value, resolve_new_line_kind,
};
use dprint_core::plugins::{
    CheckConfigUpdatesMessage, ConfigChange, FileMatchingInfo, FormatResult, PluginInfo,
    PluginResolveConfigurationResult, SyncFormatRequest, SyncHostFormatRequest, SyncPluginHandler,
};
use serde::Serialize;
use svg_format::{
    AttributeLayout, AttributeSort, BlankLines, FormatOptions, QuoteStyle, TextContentMode,
    WrappedAttributeIndent,
};

#[cfg(feature = "schema")]
pub mod schema;

/// Canonical URL for this build's published JSON Schema on `plugins.dprint.dev`.
///
/// Used as the runtime-advertised `config_schema_url` in [`PluginInfo`] and as the
/// `$id` baked into the generated `schema.json` artifact. Keeping both in sync
/// from a single constant prevents the two from drifting apart.
pub(crate) const SCHEMA_URL: &str = concat!(
    "https://plugins.dprint.dev/kjanat/dprint-plugin-svg/v",
    env!("CARGO_PKG_VERSION"),
    "/schema.json",
);

/// # The [`SyncPluginHandler`] implementation for the SVG formatter.
///
/// Stateless — all configuration is resolved per-request from the
/// dprint config map.
#[derive(Default)]
pub struct SvgWasmPluginHandler;

const INVALID_CONFIG_ERROR_FRAGMENT: &str = "configuration was not valid";

fn is_embedded_host_config_error(err: &anyhow::Error) -> bool {
    err.chain().any(|cause| {
        cause
            .to_string()
            .to_ascii_lowercase()
            .contains(INVALID_CONFIG_ERROR_FRAGMENT)
    })
}

/// # Attribute ordering strategy exposed in the plugin config.
///
/// Maps 1:1 to [`svg_format::AttributeSort`].
///
/// ```svg
/// <!-- input -->
/// <rect y="20" x="10" height="50" width="100" id="box" />
///
/// <!-- canonical (default) -->
/// <rect id="box" x="10" y="20" width="100" height="50" />
///
/// <!-- alphabetical -->
/// <rect height="50" id="box" width="100" x="10" y="20" />
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "schema",
    schemars(title = "attributeSort", description = "Attribute ordering strategy.")
)]
#[serde(rename_all = "kebab-case")]
pub enum AttributeSortConfig {
    /// Keep original source order.
    None,
    /// SVG-aware canonical grouping (id, class, geometry, presentation, ...).
    Canonical,
    /// Strict alphabetical order.
    Alphabetical,
}
dprint_core::generate_str_to_from![
    AttributeSortConfig,
    [None, "none"],
    [Canonical, "canonical"],
    [Alphabetical, "alphabetical"]
];

/// # Attribute wrapping mode exposed in the plugin config.
///
/// Maps 1:1 to [`svg_format::AttributeLayout`].
///
/// ```svg
/// <!-- auto: wraps when inline exceeds maxInlineTagWidth -->
/// <linearGradient
///     id="sky"
///     x1="0%"
///     y1="0%">
/// </linearGradient>
///
/// <!-- single-line: always one line -->
/// <linearGradient id="sky" x1="0%" y1="0%"></linearGradient>
///
/// <!-- multi-line: always wrap -->
/// <rect
///     id="box"
///     x="10" />
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "schema",
    schemars(title = "attributeLayout", description = "Attribute wrapping mode.")
)]
#[serde(rename_all = "kebab-case")]
pub enum AttributeLayoutConfig {
    /// Wrap only when inline width exceeds `maxInlineTagWidth`.
    Auto,
    /// Always keep all attributes on one line.
    SingleLine,
    /// Always wrap attributes onto separate lines.
    MultiLine,
}
dprint_core::generate_str_to_from![
    AttributeLayoutConfig,
    [Auto, "auto"],
    [SingleLine, "single-line"],
    [MultiLine, "multi-line"]
];

/// # Quoting strategy for attribute values.
///
/// Maps 1:1 to [`svg_format::QuoteStyle`].
///
/// ```svg
/// <!-- preserve: keeps original -->
/// <rect id='box' class="hero" />
///
/// <!-- double -->
/// <rect id="box" class="hero" />
///
/// <!-- single -->
/// <rect id='box' class='hero' />
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "schema",
    schemars(
        title = "quoteStyle",
        description = "Quoting strategy for attribute values."
    )
)]
#[serde(rename_all = "kebab-case")]
pub enum QuoteStyleConfig {
    /// Keep the original quote character.
    Preserve,
    /// Normalize to double quotes.
    Double,
    /// Normalize to single quotes.
    Single,
}
dprint_core::generate_str_to_from![
    QuoteStyleConfig,
    [Preserve, "preserve"],
    [Double, "double"],
    [Single, "single"]
];

/// # Indentation strategy for wrapped attributes.
///
/// Maps 1:1 to [`svg_format::WrappedAttributeIndent`].
///
/// ```svg
/// <!-- one-level (default) -->
/// <linearGradient
///     id="sky"
///     x1="0%">
/// </linearGradient>
///
/// <!-- align-to-tag-name -->
/// <linearGradient
///                 id="sky"
///                 x1="0%">
/// </linearGradient>
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "schema",
    schemars(
        title = "wrappedAttributeIndent",
        description = "Indentation strategy for wrapped attributes."
    )
)]
#[serde(rename_all = "kebab-case")]
pub enum WrappedAttributeIndentConfig {
    /// Indent one level deeper than the tag.
    OneLevel,
    /// Align to the column after `<tagName `.
    AlignToTagName,
}
dprint_core::generate_str_to_from![
    WrappedAttributeIndentConfig,
    [OneLevel, "one-level"],
    [AlignToTagName, "align-to-tag-name"]
];

/// # Line ending style.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "schema",
    schemars(title = "newLineKind", description = "Line ending style.")
)]
#[serde(rename_all = "kebab-case")]
pub enum NewLineKindConfig {
    /// Detect from the source file.
    Auto,
    /// Unix-style `\n`.
    Lf,
    /// Windows-style `\r\n`.
    Crlf,
}
dprint_core::generate_str_to_from![
    NewLineKindConfig,
    [Auto, "auto"],
    [Lf, "lf"],
    [Crlf, "crlf"]
];

/// # How the formatter handles whitespace in text nodes.
///
/// Maps 1:1 to [`svg_format::TextContentMode`].
///
/// ```svg
/// <!-- input -->
/// <text>  hello   world  </text>
///
/// <!-- collapse -->
/// <text>
///     hello world
/// </text>
///
/// <!-- maintain (default): preserves relative indentation -->
/// <text>
///     hello   world
/// </text>
///
/// <!-- prettify: trims each line -->
/// <text>
///     hello   world
/// </text>
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "schema",
    schemars(
        title = "textContent",
        description = "How text-node whitespace is handled."
    )
)]
#[serde(rename_all = "kebab-case")]
pub enum TextContentModeConfig {
    /// Collapse whitespace runs to single spaces.
    Collapse,
    /// Preserve relative indentation structure.
    Maintain,
    /// Trim each line and re-indent to SVG depth.
    Prettify,
}
dprint_core::generate_str_to_from![
    TextContentModeConfig,
    [Collapse, "collapse"],
    [Maintain, "maintain"],
    [Prettify, "prettify"]
];

/// # How blank lines between sibling elements are handled.
///
/// Maps 1:1 to [`svg_format::BlankLines`].
///
/// ```svg
/// <!-- input -->
/// <svg>
///     <rect />
///
///
///     <!--legend-->
///     <circle />
/// </svg>
///
/// <!-- remove: all gaps stripped -->
/// <svg>
///     <rect />
///     <!--legend-->
///     <circle />
/// </svg>
///
/// <!-- truncate (default): 2+ collapsed to 1 -->
/// <svg>
///     <rect />
///
///     <!--legend-->
///     <circle />
/// </svg>
///
/// <!-- insert: force gap between every sibling -->
/// <svg>
///     <rect />
///
///     <!--legend-->
///
///     <circle />
/// </svg>
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "schema",
    schemars(
        title = "blankLines",
        description = "How blank lines between sibling elements are handled."
    )
)]
#[serde(rename_all = "kebab-case")]
pub enum BlankLinesConfig {
    /// Strip all blank lines between siblings.
    Remove,
    /// Keep blank lines from source verbatim.
    Preserve,
    /// Collapse 2+ blank lines to exactly 1.
    Truncate,
    /// Force exactly 1 blank line between every sibling.
    Insert,
}
dprint_core::generate_str_to_from![
    BlankLinesConfig,
    [Remove, "remove"],
    [Preserve, "preserve"],
    [Truncate, "truncate"],
    [Insert, "insert"]
];

/// # Resolved plugin configuration.
///
/// Built from the user's `dprint.json` (or global defaults) during
/// [`SvgWasmPluginHandler::resolve_config`], then passed to every
/// [`SvgWasmPluginHandler::format`] call. Serialized back to the host
/// for config display.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    /// Threshold (columns) for switching to multi-line attributes.
    pub max_inline_tag_width: u32,
    /// Use tabs (`true`) or spaces (`false`) for indentation.
    pub use_tabs: bool,
    /// Spaces per indent level when `use_tabs` is false.
    pub indent_width: u8,
    pub attribute_sort: AttributeSortConfig,
    pub attribute_layout: AttributeLayoutConfig,
    /// Max attributes emitted per line in multi-line mode.
    pub attributes_per_line: u32,
    /// Emit a space before `/>` in self-closing tags.
    pub space_before_self_close: bool,
    pub quote_style: QuoteStyleConfig,
    pub wrapped_attribute_indent: WrappedAttributeIndentConfig,
    pub text_content: TextContentModeConfig,
    pub blank_lines: BlankLinesConfig,
    /// Delegate `<style>`/`<script>`/`<foreignObject>` to host plugins.
    pub format_embedded_content: bool,
    /// Resolved document line width, used for embedded content width budget.
    pub line_width: u32,
    pub new_line_kind: NewLineKind,
}

impl SyncPluginHandler<Configuration> for SvgWasmPluginHandler {
    fn plugin_info(&mut self) -> PluginInfo {
        PluginInfo {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            config_key: "svg".to_string(),
            help_url: "https://github.com/kjanat/dprint-plugin-svg".to_string(),
            config_schema_url: SCHEMA_URL.to_string(),
            update_url: Some(
                "https://plugins.dprint.dev/kjanat/dprint-plugin-svg/latest.json".to_string(),
            ),
        }
    }

    fn license_text(&mut self) -> String {
        include_str!("../LICENSE").to_string()
    }

    fn resolve_config(
        &mut self,
        mut config: ConfigKeyMap,
        global_config: &GlobalConfiguration,
    ) -> PluginResolveConfigurationResult<Configuration> {
        let mut diagnostics = Vec::<ConfigurationDiagnostic>::new();

        let line_width = get_value(
            &mut config,
            "lineWidth",
            global_config.line_width.unwrap_or(100),
            &mut diagnostics,
        );
        let max_inline_tag_width = get_value(
            &mut config,
            "maxInlineTagWidth",
            line_width,
            &mut diagnostics,
        );
        let use_tabs = get_value(
            &mut config,
            "useTabs",
            global_config.use_tabs.unwrap_or(true),
            &mut diagnostics,
        );
        let indent_width = get_value(
            &mut config,
            "indentWidth",
            global_config.indent_width.unwrap_or(2),
            &mut diagnostics,
        );
        let attribute_sort = get_value(
            &mut config,
            "attributeSort",
            AttributeSortConfig::Canonical,
            &mut diagnostics,
        );
        let attribute_layout = get_value(
            &mut config,
            "attributeLayout",
            AttributeLayoutConfig::Auto,
            &mut diagnostics,
        );
        let mut attributes_per_line =
            get_value(&mut config, "attributesPerLine", 1_u32, &mut diagnostics);
        let space_before_self_close =
            get_value(&mut config, "spaceBeforeSelfClose", true, &mut diagnostics);
        let quote_style = get_value(
            &mut config,
            "quoteStyle",
            QuoteStyleConfig::Preserve,
            &mut diagnostics,
        );
        let wrapped_attribute_indent = get_value(
            &mut config,
            "wrappedAttributeIndent",
            WrappedAttributeIndentConfig::OneLevel,
            &mut diagnostics,
        );
        let global_new_line_default = match global_config.new_line_kind {
            Some(NewLineKind::Auto) => NewLineKindConfig::Auto,
            Some(NewLineKind::LineFeed) => NewLineKindConfig::Lf,
            Some(NewLineKind::CarriageReturnLineFeed) => NewLineKindConfig::Crlf,
            None => NewLineKindConfig::Auto,
        };
        let new_line_kind_config = get_value(
            &mut config,
            "newLineKind",
            global_new_line_default,
            &mut diagnostics,
        );
        let new_line_kind = match new_line_kind_config {
            NewLineKindConfig::Auto => NewLineKind::Auto,
            NewLineKindConfig::Lf => NewLineKind::LineFeed,
            NewLineKindConfig::Crlf => NewLineKind::CarriageReturnLineFeed,
        };

        let text_content = get_value(
            &mut config,
            "textContent",
            TextContentModeConfig::Maintain,
            &mut diagnostics,
        );
        let blank_lines = get_value(
            &mut config,
            "blankLines",
            BlankLinesConfig::Truncate,
            &mut diagnostics,
        );
        let format_embedded_content =
            get_value(&mut config, "formatEmbeddedContent", true, &mut diagnostics);

        if attributes_per_line == 0 {
            diagnostics.push(ConfigurationDiagnostic {
                property_name: "attributesPerLine".to_string(),
                message: "Expected a value greater than 0.".to_string(),
            });
            attributes_per_line = 1;
        }

        diagnostics.extend(get_unknown_property_diagnostics(config));

        PluginResolveConfigurationResult {
            file_matching: FileMatchingInfo {
                file_extensions: vec!["svg".to_string()],
                file_names: Vec::new(),
            },
            diagnostics,
            config: Configuration {
                max_inline_tag_width,
                use_tabs,
                indent_width,
                attribute_sort,
                attribute_layout,
                attributes_per_line,
                space_before_self_close,
                quote_style,
                wrapped_attribute_indent,
                text_content,
                blank_lines,
                format_embedded_content,
                line_width,
                new_line_kind,
            },
        }
    }

    fn check_config_updates(
        &self,
        _message: CheckConfigUpdatesMessage,
    ) -> Result<Vec<ConfigChange>> {
        Ok(Vec::new())
    }

    fn format(
        &mut self,
        request: SyncFormatRequest<Configuration>,
        mut host_format: impl FnMut(SyncHostFormatRequest) -> FormatResult,
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
            blank_lines: map_blank_lines(request.config.blank_lines),
            ignore_prefixes: vec!["svg-format".into(), "dprint".into()],
        };

        let line_width = request.config.line_width;
        let indent_width = request.config.indent_width as u32;
        let do_embedded = request.config.format_embedded_content;
        let mut host_err: Option<anyhow::Error> = None;

        let mut formatted = svg_format::format_with_host(source, options, &mut |embedded| {
            if !do_embedded {
                return None;
            }
            let ext = match embedded.language {
                svg_format::EmbeddedLanguage::Css => "css",
                svg_format::EmbeddedLanguage::JavaScript => "js",
                svg_format::EmbeddedLanguage::Html => "html",
            };
            let path = request.file_path.with_extension(ext);
            let adjusted_width = line_width
                .saturating_sub(embedded.indent_depth as u32 * indent_width)
                .max(1);
            let mut overrides = ConfigKeyMap::new();
            overrides.insert(
                "lineWidth".into(),
                dprint_core::configuration::ConfigKeyValue::Number(adjusted_width as i32),
            );
            overrides.insert(
                "newLineKind".into(),
                dprint_core::configuration::ConfigKeyValue::String("lf".into()),
            );
            match host_format(SyncHostFormatRequest {
                file_path: &path,
                file_bytes: embedded.content.as_bytes(),
                range: None,
                override_config: &overrides,
            }) {
                Ok(Some(bytes)) => match String::from_utf8(bytes) {
                    Ok(s) => Some(s),
                    Err(e) => {
                        host_err.get_or_insert_with(|| {
                            anyhow!(
                                "embedded {ext} in '{}' produced invalid UTF-8: {e}",
                                request.file_path.display()
                            )
                        });
                        None
                    }
                },
                Ok(None) => None,
                Err(err) => {
                    if is_embedded_host_config_error(&err) {
                        return None;
                    }
                    host_err.get_or_insert_with(|| {
                        anyhow!(
                            "failed to format embedded {ext} in '{}': {err}",
                            request.file_path.display()
                        )
                    });
                    None
                }
            }
        });

        if let Some(e) = host_err {
            return Err(e);
        }

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
}

// ── Config enum → svg_format enum mappers ───────────────────────────

fn map_attribute_sort(value: AttributeSortConfig) -> AttributeSort {
    match value {
        AttributeSortConfig::None => AttributeSort::None,
        AttributeSortConfig::Canonical => AttributeSort::Canonical,
        AttributeSortConfig::Alphabetical => AttributeSort::Alphabetical,
    }
}

fn map_attribute_layout(value: AttributeLayoutConfig) -> AttributeLayout {
    match value {
        AttributeLayoutConfig::Auto => AttributeLayout::Auto,
        AttributeLayoutConfig::SingleLine => AttributeLayout::SingleLine,
        AttributeLayoutConfig::MultiLine => AttributeLayout::MultiLine,
    }
}

fn map_quote_style(value: QuoteStyleConfig) -> QuoteStyle {
    match value {
        QuoteStyleConfig::Preserve => QuoteStyle::Preserve,
        QuoteStyleConfig::Double => QuoteStyle::Double,
        QuoteStyleConfig::Single => QuoteStyle::Single,
    }
}

fn map_wrapped_attribute_indent(value: WrappedAttributeIndentConfig) -> WrappedAttributeIndent {
    match value {
        WrappedAttributeIndentConfig::OneLevel => WrappedAttributeIndent::OneLevel,
        WrappedAttributeIndentConfig::AlignToTagName => WrappedAttributeIndent::AlignToTagName,
    }
}

fn map_blank_lines(value: BlankLinesConfig) -> BlankLines {
    match value {
        BlankLinesConfig::Remove => BlankLines::Remove,
        BlankLinesConfig::Preserve => BlankLines::Preserve,
        BlankLinesConfig::Truncate => BlankLines::Truncate,
        BlankLinesConfig::Insert => BlankLines::Insert,
    }
}

fn map_text_content(value: TextContentModeConfig) -> TextContentMode {
    match value {
        TextContentModeConfig::Collapse => TextContentMode::Collapse,
        TextContentModeConfig::Maintain => TextContentMode::Maintain,
        TextContentModeConfig::Prettify => TextContentMode::Prettify,
    }
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
dprint_core::generate_plugin_code!(SvgWasmPluginHandler, SvgWasmPluginHandler, Configuration);
