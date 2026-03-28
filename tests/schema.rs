#![cfg(feature = "schema")]

use std::fs;
use std::path::PathBuf;

use schemars::schema_for;
use serde_json::Value;

use dprint_plugin_svg::schema::{DprintSvgConfigSchema, generate_schema_value};

fn deployment_schema_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("deployment/schema.json")
}

#[test]
fn schema_serializes_to_valid_json() {
    let schema = schema_for!(DprintSvgConfigSchema);
    let json = serde_json::to_string_pretty(&schema).expect("schema should serialize to JSON");
    let _: Value = serde_json::from_str(&json).expect("serialized schema should be valid JSON");
}

#[test]
fn schema_has_expected_properties() {
    let schema = schema_for!(DprintSvgConfigSchema);
    let value = serde_json::to_value(&schema).expect("schema should convert to Value");
    let properties = value["properties"]
        .as_object()
        .expect("schema should have a properties object");

    let expected_fields = [
        "locked",
        "lineWidth",
        "maxInlineTagWidth",
        "useTabs",
        "indentWidth",
        "newLineKind",
        "attributeSort",
        "attributeLayout",
        "attributesPerLine",
        "spaceBeforeSelfClose",
        "quoteStyle",
        "wrappedAttributeIndent",
    ];

    for field in &expected_fields {
        assert!(
            properties.contains_key(*field),
            "schema missing expected property: {field}"
        );
    }
}

#[test]
fn generated_schema_uses_stable_top_level_key_order() {
    let value = generate_schema_value().expect("schema should generate");
    let keys: Vec<&str> = value
        .as_object()
        .expect("schema should be an object")
        .keys()
        .map(String::as_str)
        .collect();

    assert_eq!(
        keys,
        vec![
            "$schema",
            "$id",
            "title",
            "description",
            "type",
            "properties",
            "definitions",
        ]
    );
}

#[test]
fn schema_enum_variants_match_expected_values() {
    let schema = schema_for!(DprintSvgConfigSchema);
    let value = serde_json::to_value(&schema).expect("schema should convert to Value");
    let definitions = value["definitions"]
        .as_object()
        .expect("schema should have definitions");

    let expected: &[(&str, &[&str])] = &[
        (
            "AttributeSortConfig",
            &["none", "canonical", "alphabetical"],
        ),
        (
            "AttributeLayoutConfig",
            &["auto", "single-line", "multi-line"],
        ),
        ("QuoteStyleConfig", &["preserve", "double", "single"]),
        (
            "WrappedAttributeIndentConfig",
            &["one-level", "align-to-tag-name"],
        ),
        ("NewLineKindConfig", &["auto", "lf", "crlf"]),
    ];

    for (name, variants) in expected {
        let def = definitions
            .get(*name)
            .unwrap_or_else(|| panic!("missing definition: {name}"));
        let enum_values: Vec<&str> = def["enum"]
            .as_array()
            .unwrap_or_else(|| panic!("{name} should have an enum array"))
            .iter()
            .map(|v| v.as_str().expect("enum variant should be a string"))
            .collect();
        assert_eq!(enum_values, *variants, "enum variants mismatch for {name}");
    }
}

#[test]
fn attributes_per_line_minimum_is_one() {
    let schema = schema_for!(DprintSvgConfigSchema);
    let value = serde_json::to_value(&schema).expect("schema should convert to Value");
    let apl = &value["properties"]["attributesPerLine"];

    // schemars emits minimum as a number; ensure it's >= 1
    let minimum = apl["minimum"]
        .as_f64()
        .expect("attributesPerLine should have a minimum");
    assert!(
        minimum >= 1.0,
        "attributesPerLine minimum should be >= 1, got {minimum}"
    );
}

#[test]
fn committed_schema_is_not_stale() {
    let path = deployment_schema_path();
    let committed: Value = serde_json::from_str(
        &fs::read_to_string(&path).expect("deployment/schema.json should exist"),
    )
    .expect("committed schema should be valid JSON");

    let generated = generate_schema_value().expect("schema should generate");

    assert_eq!(
        committed, generated,
        "deployment/schema.json is stale — run `cargo run --features schema --bin generate-schema -- deployment/schema.json` to regenerate"
    );
}
