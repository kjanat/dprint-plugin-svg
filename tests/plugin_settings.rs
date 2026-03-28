use std::fs;
use std::path::{Path, PathBuf};

use dprint_core::configuration::{
    ConfigKeyMap, ConfigKeyValue, GlobalConfiguration, resolve_global_config,
};
use dprint_core::plugins::{
    FormatConfigId, NullCancellationToken, SyncFormatRequest, SyncPluginHandler,
};
use dprint_plugin_svg::{Configuration, SvgWasmPluginHandler};
use serde_json::Value;

fn config_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("configs")
        .join(file_name)
}

fn json_to_config_value(value: &Value) -> ConfigKeyValue {
    match value {
        Value::String(text) => ConfigKeyValue::String(text.clone()),
        Value::Bool(value) => ConfigKeyValue::Bool(*value),
        Value::Number(number) => {
            let int_value = number
                .as_i64()
                .expect("only integer numbers are supported in fixture configs");
            ConfigKeyValue::Number(int_value as i32)
        }
        Value::Array(values) => {
            ConfigKeyValue::Array(values.iter().map(json_to_config_value).collect())
        }
        Value::Object(map) => {
            let mut config_map = ConfigKeyMap::new();
            for (key, value) in map {
                config_map.insert(key.clone(), json_to_config_value(value));
            }
            ConfigKeyValue::Object(config_map)
        }
        Value::Null => ConfigKeyValue::Null,
    }
}

fn load_dprint_fixture(file_name: &str) -> (GlobalConfiguration, ConfigKeyMap) {
    let text = fs::read_to_string(config_path(file_name)).expect("fixture should exist");
    let root: Value = serde_json::from_str(&text).expect("fixture should be valid JSON");
    let object = root
        .as_object()
        .expect("fixture root should be a JSON object");

    let mut global_config_map = ConfigKeyMap::new();
    let mut plugin_config_map = ConfigKeyMap::new();

    for (key, value) in object {
        if key == "svg" {
            let plugin_obj = value
                .as_object()
                .expect("svg config should be a JSON object");
            for (plugin_key, plugin_value) in plugin_obj {
                plugin_config_map.insert(plugin_key.clone(), json_to_config_value(plugin_value));
            }
        } else {
            global_config_map.insert(key.clone(), json_to_config_value(value));
        }
    }

    let global = resolve_global_config(&mut global_config_map).config;
    (global, plugin_config_map)
}

fn resolve_configuration(
    file_name: &str,
) -> dprint_core::plugins::PluginResolveConfigurationResult<Configuration> {
    let (global, plugin) = load_dprint_fixture(file_name);
    let mut handler = SvgWasmPluginHandler;
    handler.resolve_config(plugin, &global)
}

fn format_with_config(config: &Configuration, input: &str) -> Option<String> {
    let mut handler = SvgWasmPluginHandler;
    let token = NullCancellationToken;
    let request = SyncFormatRequest {
        file_path: Path::new("test.svg"),
        file_bytes: input.as_bytes().to_vec(),
        config_id: FormatConfigId::from_raw(1),
        config,
        range: None,
        token: &token,
    };

    handler
        .format(request, |_req| Ok(None))
        .expect("format should succeed")
        .map(|bytes| String::from_utf8(bytes).expect("formatted text should be valid UTF-8"))
}

#[test]
fn resolve_config_uses_global_defaults() {
    let result = resolve_configuration("defaults-global.dprint.json");

    assert!(result.diagnostics.is_empty());
    assert_eq!(result.config.max_inline_tag_width, 88);
    assert!(!result.config.use_tabs);
    assert_eq!(result.config.indent_width, 4);
}

#[test]
fn format_respects_spaces_indent_and_self_close_spacing() {
    let result = resolve_configuration("spaces-no-self-close.dprint.json");
    assert!(result.diagnostics.is_empty());

    let input = "<svg><rect id='x'/></svg>";
    let output = format_with_config(&result.config, input).expect("should produce formatted text");
    let expected = "<svg>\n    <rect id='x'/>\n</svg>";
    assert_eq!(output, expected);
}

#[test]
fn format_respects_attribute_sort_and_quote_style() {
    let result = resolve_configuration("alphabetical-double-quotes.dprint.json");
    assert!(result.diagnostics.is_empty());

    let input = "<svg><rect y='2' x='1' id='x' class='c'/></svg>";
    let output = format_with_config(&result.config, input).expect("should produce formatted text");
    let expected = "<svg>\n\t<rect class=\"c\" id=\"x\" x=\"1\" y=\"2\" />\n</svg>";
    assert_eq!(output, expected);
}

#[test]
fn format_respects_multiline_layout_and_wrapped_alignment() {
    let result = resolve_configuration("multiline-align.dprint.json");
    assert!(result.diagnostics.is_empty());

    let input = "<svg><linearGradient id=\"sky\" x1=\"0%\" y1=\"0%\"></linearGradient></svg>";
    let output = format_with_config(&result.config, input).expect("should produce formatted text");
    let aligned = format!("\t{}", " ".repeat("linearGradient".len() + 2));
    let expected = format!(
        "<svg>\n\t<linearGradient\n{aligned}id=\"sky\"\n{aligned}x1=\"0%\"\n{aligned}y1=\"0%\">\n\t</linearGradient>\n</svg>"
    );
    assert_eq!(output, expected);
}

#[test]
fn format_respects_new_line_kind_crlf() {
    let result = resolve_configuration("crlf-newline.dprint.json");
    assert!(result.diagnostics.is_empty());

    let input = "<svg><rect/></svg>";
    let output = format_with_config(&result.config, input).expect("should produce formatted text");
    let expected = "<svg>\r\n\t<rect />\r\n</svg>";
    assert_eq!(output, expected);
}

#[test]
fn resolve_config_validates_attributes_per_line() {
    let result = resolve_configuration("attrs-per-line-invalid.dprint.json");

    assert_eq!(result.config.attributes_per_line, 1);
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.property_name == "attributesPerLine")
    );
}

#[test]
fn range_format_request_returns_no_change() {
    let result = resolve_configuration("range-request.dprint.json");
    assert!(result.diagnostics.is_empty());
    let config = result.config;
    let token = NullCancellationToken;
    let mut handler = SvgWasmPluginHandler;

    let format_result = handler
        .format(
            SyncFormatRequest {
                file_path: Path::new("test.svg"),
                file_bytes: b"<svg><rect/></svg>".to_vec(),
                config_id: FormatConfigId::from_raw(1),
                config: &config,
                range: Some(0..4),
                token: &token,
            },
            |_req| Ok(None),
        )
        .expect("format should succeed");

    assert!(format_result.is_none());
}

#[test]
fn global_new_line_kind_is_used_when_svg_setting_is_missing() {
    let result = resolve_configuration("global-crlf-only.dprint.json");
    assert!(result.diagnostics.is_empty());

    let input = "<svg><rect/></svg>";
    let output = format_with_config(&result.config, input).expect("should produce formatted text");
    assert!(output.contains("\r\n"));
}

#[test]
fn long_path_data_is_preserved() {
    let result = resolve_configuration("defaults-global.dprint.json");
    assert!(result.diagnostics.is_empty());

    let input = "<svg><path d=\"m19.6 66.5 19.7-11 .3-1-.3-.5h-1l-3.3-.2-11.2-.3\" style=\"fill:#d97959\"/></svg>";
    let output = format_with_config(&result.config, input).expect("should produce formatted text");

    let expected = "<svg>\n    <path d=\"m19.6 66.5 19.7-11 .3-1-.3-.5h-1l-3.3-.2-11.2-.3\" style=\"fill:#d97959\" />\n</svg>";
    assert_eq!(output, expected);
}

#[test]
fn formatting_is_idempotent() {
    let inputs: &[(&str, &str)] = &[
        (
            "defaults-global.dprint.json",
            "<svg><rect y='2' x='1' id='x' class='c'/></svg>",
        ),
        (
            "alphabetical-double-quotes.dprint.json",
            "<svg><rect y='2' x='1' id='x' class='c'/></svg>",
        ),
        ("crlf-newline.dprint.json", "<svg><rect/></svg>"),
        (
            "multiline-align.dprint.json",
            "<svg><linearGradient id=\"sky\" x1=\"0%\" y1=\"0%\"></linearGradient></svg>",
        ),
    ];

    for (fixture, input) in inputs {
        let result = resolve_configuration(fixture);
        assert!(result.diagnostics.is_empty(), "diagnostics in {fixture}");

        let first = format_with_config(&result.config, input)
            .unwrap_or_else(|| panic!("first format should change {fixture}"));
        let second = format_with_config(&result.config, &first);
        assert!(
            second.is_none(),
            "second format pass should produce no changes for {fixture}"
        );
    }
}

#[test]
fn unknown_config_key_produces_diagnostic() {
    let result = resolve_configuration("unknown-key.dprint.json");
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.property_name == "atributeSort"),
        "expected diagnostic for unknown key 'atributeSort'"
    );
}

#[test]
fn format_rejects_invalid_utf8() {
    let result = resolve_configuration("defaults-global.dprint.json");
    let mut handler = SvgWasmPluginHandler;
    let token = NullCancellationToken;
    let request = SyncFormatRequest {
        file_path: Path::new("bad.svg"),
        file_bytes: vec![0xFF, 0xFE],
        config_id: FormatConfigId::from_raw(1),
        config: &result.config,
        range: None,
        token: &token,
    };
    let err = handler.format(request, |_| Ok(None));
    assert!(err.is_err(), "invalid UTF-8 should return Err");
}
