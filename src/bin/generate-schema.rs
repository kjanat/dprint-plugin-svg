use std::env;
use std::fs;
use std::path::PathBuf;

use dprint_plugin_svg::schema::generate_schema_value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = env::args()
        .nth(1)
        .unwrap_or_else(|| "deployment/schema.json".to_string());
    let output_path = PathBuf::from(output);

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let value = generate_schema_value()?;

    fs::write(output_path, serde_json::to_string_pretty(&value)?)?;
    Ok(())
}
