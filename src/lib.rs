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

/// GitHub repository URL, taken verbatim from `package.repository` in `Cargo.toml`.
///
/// Used as the `help_url` in [`PluginInfo`].
pub(crate) const REPO_URL: &str = env!("CARGO_PKG_REPOSITORY");

/// `owner/name` portion of [`REPO_URL`] — e.g. `kjanat/dprint-plugin-svg`.
///
/// `plugins.dprint.dev` mirrors the GitHub path, so every URL this plugin
/// advertises interpolates this value. Derived at compile time by slicing off
/// the `https://github.com/` prefix. If `package.repository` is ever changed
/// to a non-GitHub host, update the prefix literal below.
pub(crate) const REPO_PATH: &str = {
    let (_, path) = REPO_URL.split_at("https://github.com/".len());
    path
};

/// Canonical URL for this build's published JSON Schema on `plugins.dprint.dev`.
///
/// Used as the runtime-advertised `config_schema_url` in [`PluginInfo`] and as
/// the `$id` baked into the generated `schema.json` artifact.
pub(crate) const SCHEMA_URL: &str = const_format::concatcp!(
    "https://plugins.dprint.dev/",
    REPO_PATH,
    "/v",
    env!("CARGO_PKG_VERSION"),
    "/schema.json",
);

/// Update-notification URL for `dprint config update` discovery.
///
/// Stable per repository — resolves server-side to the latest release's wasm.
pub(crate) const UPDATE_URL: &str =
    const_format::concatcp!("https://plugins.dprint.dev/", REPO_PATH, "/latest.json",);

/// # The [`SyncPluginHandler`] implementation for the SVG formatter.
///
/// Stateless — all configuration is resolved per-request from the
/// dprint config map.
#[derive(Default)]
pub struct SvgWasmPluginHandler;

/// # Attribute ordering strategy exposed in the plugin config.
///
/// Maps 1:1 to [`svg_format::AttributeSort`].
///
/// ```svg-input
/// <rect y="20" x="10" height="50" width="100" id="box" />
/// ```
/// ```svg-output none
/// <rect y="20" x="10" height="50" width="100" id="box" />
/// ```
/// ```svg-output canonical
/// <rect id="box" x="10" y="20" width="100" height="50" />
/// ```
/// ```svg-output alphabetical
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
/// ```svg-input
/// <linearGradient id="sky" x1="0%" y1="0%"></linearGradient>
/// ```
/// ```svg-output auto
/// <linearGradient id="sky" x1="0%" y1="0%"></linearGradient>
/// ```
/// ```svg-output single-line
/// <linearGradient id="sky" x1="0%" y1="0%"></linearGradient>
/// ```
/// ```svg-output multi-line
/// <linearGradient id="sky"
///                 x1="0%" y1="0%">
/// </linearGradient>
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
/// ```svg-input
/// <rect id='box' class="hero" />
/// ```
/// ```svg-output preserve
/// <rect id='box' class="hero" />
/// ```
/// ```svg-output double
/// <rect id="box" class="hero" />
/// ```
/// ```svg-output single
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
/// Maps 1:1 to [`svg_format::WrappedAttributeIndent`]. Only applies
/// when a tag wraps to multiple lines (see `attributeLayout`).
///
/// ```svg-input
/// <rect id="box" x="10" y="20" width="100" height="50" fill="red" />
/// ```
/// ```svg-output one-level
/// <rect
///   id="box"
///   x="10" y="20" width="100" height="50"
///   fill="red" />
/// ```
/// ```svg-output align-to-tag-name
/// <rect id="box"
///       x="10" y="20" width="100" height="50"
///       fill="red" />
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
/// ```svg-input
/// <text>  hello   world  </text>
/// ```
/// ```svg-output collapse
/// <text>
///   hello world
/// </text>
/// ```
/// ```svg-output maintain
/// <text>
///   hello   world
/// </text>
/// ```
/// ```svg-output prettify
/// <text>
///   hello   world
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
/// ```svg-input
/// <svg>
///   <rect />
///
///
///   <!--legend-->
///   <circle />
/// </svg>
/// ```
/// ```svg-output remove
/// <svg>
///   <rect />
///   <!--legend-->
///   <circle />
/// </svg>
/// ```
/// ```svg-output preserve
/// <svg>
///   <rect />
///
///
///   <!--legend-->
///   <circle />
/// </svg>
/// ```
/// ```svg-output truncate
/// <svg>
///   <rect />
///
///   <!--legend-->
///   <circle />
/// </svg>
/// ```
/// ```svg-output insert
/// <svg>
///   <rect />
///
///   <!--legend-->
///
///   <circle />
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
    /// Attribute ordering mode (see [`AttributeSortConfig`]).
    pub attribute_sort: AttributeSortConfig,
    /// Attribute wrapping mode (see [`AttributeLayoutConfig`]).
    pub attribute_layout: AttributeLayoutConfig,
    /// Max attributes emitted per line in multi-line mode.
    pub attributes_per_line: u32,
    /// Emit a space before `/>` in self-closing tags.
    pub space_before_self_close: bool,
    /// Quote style for attribute values (see [`QuoteStyleConfig`]).
    pub quote_style: QuoteStyleConfig,
    /// Indentation style for wrapped attributes (see
    /// [`WrappedAttributeIndentConfig`]).
    pub wrapped_attribute_indent: WrappedAttributeIndentConfig,
    /// How whitespace inside text nodes is handled (see
    /// [`TextContentModeConfig`]).
    pub text_content: TextContentModeConfig,
    /// How blank lines between sibling elements are handled (see
    /// [`BlankLinesConfig`]).
    pub blank_lines: BlankLinesConfig,
    /// Delegate `<style>`/`<script>`/`<foreignObject>` to host plugins.
    pub format_embedded_content: bool,
    /// Resolved document line width, used for embedded content width budget.
    pub line_width: u32,
    /// Line ending style (inherits from the top-level `newLineKind` in
    /// the same `dprint.json` by default).
    pub new_line_kind: NewLineKind,
}

impl SyncPluginHandler<Configuration> for SvgWasmPluginHandler {
    fn plugin_info(&mut self) -> PluginInfo {
        PluginInfo {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            config_key: "svg".to_string(),
            help_url: REPO_URL.to_string(),
            config_schema_url: SCHEMA_URL.to_string(),
            update_url: Some(UPDATE_URL.to_string()),
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

        // Invariant: every fallback below mirrors
        // `svg_format::FormatOptions::default()`. The plugin wraps the
        // library — its defaults must never drift. Options not modeled
        // by the library (`lineWidth`, `newLineKind`,
        // `formatEmbeddedContent`) keep plugin-specific fallbacks.
        let svg_defaults = svg_format::FormatOptions::default();

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
            global_config
                .use_tabs
                .unwrap_or(!svg_defaults.insert_spaces),
            &mut diagnostics,
        );
        let indent_width = get_value(
            &mut config,
            "indentWidth",
            global_config
                .indent_width
                .unwrap_or_else(|| u8::try_from(svg_defaults.indent_width).unwrap_or(2)),
            &mut diagnostics,
        );
        let attribute_sort = get_value(
            &mut config,
            "attributeSort",
            unmap_attribute_sort(svg_defaults.attribute_sort),
            &mut diagnostics,
        );
        let attribute_layout = get_value(
            &mut config,
            "attributeLayout",
            unmap_attribute_layout(svg_defaults.attribute_layout),
            &mut diagnostics,
        );
        let mut attributes_per_line = get_value(
            &mut config,
            "attributesPerLine",
            u32::try_from(svg_defaults.attributes_per_line).unwrap_or(1),
            &mut diagnostics,
        );
        let space_before_self_close = get_value(
            &mut config,
            "spaceBeforeSelfClose",
            svg_defaults.space_before_self_close,
            &mut diagnostics,
        );
        let quote_style = get_value(
            &mut config,
            "quoteStyle",
            unmap_quote_style(svg_defaults.quote_style),
            &mut diagnostics,
        );
        let wrapped_attribute_indent = get_value(
            &mut config,
            "wrappedAttributeIndent",
            unmap_wrapped_attribute_indent(svg_defaults.wrapped_attribute_indent),
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
            unmap_text_content(svg_defaults.text_content),
            &mut diagnostics,
        );
        let blank_lines = get_value(
            &mut config,
            "blankLines",
            unmap_blank_lines(svg_defaults.blank_lines),
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

        let mut formatted = svg_format::format_with_host(source, options, &mut |embedded| {
            if !do_embedded || request.token.is_cancelled() {
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
            // Embedded-content formatting is opportunistic: any host failure
            // (misconfigured host plugin, parse error inside the embedded
            // block, non-UTF-8 bytes back, …) preserves the original block
            // and lets the rest of the SVG — and other files in the run —
            // format anyway. Failing the whole run because one <style> or
            // <script> body is malformed is too punishing, and the host's
            // line/col refers to the embedded buffer (not the file), so
            // surfacing it doesn't help locate the problem either. See
            // issue #5.
            match host_format(SyncHostFormatRequest {
                file_path: &path,
                file_bytes: embedded.content.as_bytes(),
                range: None,
                override_config: &overrides,
            }) {
                Ok(Some(bytes)) => String::from_utf8(bytes).ok(),
                Ok(None) | Err(_) => None,
            }
        });

        if request.token.is_cancelled() {
            return Ok(None);
        }

        // Defensive: strip any stray CRs from `formatted` before applying the
        // target newline. svg_format is expected to produce pure LF, but if it
        // ever passes through source CRs (e.g. on the parse-error fallback)
        // a blanket replace('\n', "\r\n") would double them into "\r\r\n",
        // growing the file by one byte per CRLF on every format pass.
        if formatted.contains('\r') {
            formatted = formatted.replace("\r\n", "\n").replace('\r', "\n");
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

// ── Config enum ⇄ svg_format enum mappers ──────────────────────────
//
// Each pair converts between the plugin-facing Config enum (serialized
// in `.dprint.json`) and the upstream `svg_format` enum (consumed by
// the formatter). The `unmap_*` direction is used in `resolve_config`
// to source defaults from `svg_format::FormatOptions::default()`, so
// the plugin can never drift from upstream defaults.

fn map_attribute_sort(value: AttributeSortConfig) -> AttributeSort {
    match value {
        AttributeSortConfig::None => AttributeSort::None,
        AttributeSortConfig::Canonical => AttributeSort::Canonical,
        AttributeSortConfig::Alphabetical => AttributeSort::Alphabetical,
    }
}

pub(crate) fn unmap_attribute_sort(value: AttributeSort) -> AttributeSortConfig {
    match value {
        AttributeSort::None => AttributeSortConfig::None,
        AttributeSort::Canonical => AttributeSortConfig::Canonical,
        AttributeSort::Alphabetical => AttributeSortConfig::Alphabetical,
    }
}

fn map_attribute_layout(value: AttributeLayoutConfig) -> AttributeLayout {
    match value {
        AttributeLayoutConfig::Auto => AttributeLayout::Auto,
        AttributeLayoutConfig::SingleLine => AttributeLayout::SingleLine,
        AttributeLayoutConfig::MultiLine => AttributeLayout::MultiLine,
    }
}

pub(crate) fn unmap_attribute_layout(value: AttributeLayout) -> AttributeLayoutConfig {
    match value {
        AttributeLayout::Auto => AttributeLayoutConfig::Auto,
        AttributeLayout::SingleLine => AttributeLayoutConfig::SingleLine,
        AttributeLayout::MultiLine => AttributeLayoutConfig::MultiLine,
    }
}

fn map_quote_style(value: QuoteStyleConfig) -> QuoteStyle {
    match value {
        QuoteStyleConfig::Preserve => QuoteStyle::Preserve,
        QuoteStyleConfig::Double => QuoteStyle::Double,
        QuoteStyleConfig::Single => QuoteStyle::Single,
    }
}

pub(crate) fn unmap_quote_style(value: QuoteStyle) -> QuoteStyleConfig {
    match value {
        QuoteStyle::Preserve => QuoteStyleConfig::Preserve,
        QuoteStyle::Double => QuoteStyleConfig::Double,
        QuoteStyle::Single => QuoteStyleConfig::Single,
    }
}

fn map_wrapped_attribute_indent(value: WrappedAttributeIndentConfig) -> WrappedAttributeIndent {
    match value {
        WrappedAttributeIndentConfig::OneLevel => WrappedAttributeIndent::OneLevel,
        WrappedAttributeIndentConfig::AlignToTagName => WrappedAttributeIndent::AlignToTagName,
    }
}

pub(crate) fn unmap_wrapped_attribute_indent(
    value: WrappedAttributeIndent,
) -> WrappedAttributeIndentConfig {
    match value {
        WrappedAttributeIndent::OneLevel => WrappedAttributeIndentConfig::OneLevel,
        WrappedAttributeIndent::AlignToTagName => WrappedAttributeIndentConfig::AlignToTagName,
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

pub(crate) fn unmap_blank_lines(value: BlankLines) -> BlankLinesConfig {
    match value {
        BlankLines::Remove => BlankLinesConfig::Remove,
        BlankLines::Preserve => BlankLinesConfig::Preserve,
        BlankLines::Truncate => BlankLinesConfig::Truncate,
        BlankLines::Insert => BlankLinesConfig::Insert,
    }
}

fn map_text_content(value: TextContentModeConfig) -> TextContentMode {
    match value {
        TextContentModeConfig::Collapse => TextContentMode::Collapse,
        TextContentModeConfig::Maintain => TextContentMode::Maintain,
        TextContentModeConfig::Prettify => TextContentMode::Prettify,
    }
}

pub(crate) fn unmap_text_content(value: TextContentMode) -> TextContentModeConfig {
    match value {
        TextContentMode::Collapse => TextContentModeConfig::Collapse,
        TextContentMode::Maintain => TextContentModeConfig::Maintain,
        TextContentMode::Prettify => TextContentModeConfig::Prettify,
    }
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
dprint_core::generate_plugin_code!(SvgWasmPluginHandler, SvgWasmPluginHandler, Configuration);
