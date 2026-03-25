use std::env;
use std::fs;
use std::path::PathBuf;

use schemars::schema_for;
use serde_json::{Value, json};

use dprint_plugin_svg::schema::DprintSvgConfigSchema;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = env::args()
        .nth(1)
        .unwrap_or_else(|| "deployment/schema.json".to_string());
    let output_path = PathBuf::from(output);

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut value = serde_json::to_value(schema_for!(DprintSvgConfigSchema))?;
    inject_schema_metadata(&mut value);

    fs::write(output_path, serde_json::to_string_pretty(&value)?)?;
    Ok(())
}

fn inject_schema_metadata(value: &mut Value) {
    let obj = value
        .as_object_mut()
        .expect("schema_for!(...) should serialize to an object");
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
}
