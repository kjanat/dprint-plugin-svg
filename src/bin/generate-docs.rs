//! Generate mdBook config pages from the JSON Schema + Rust rustdoc.
//!
//! Usage:
//!   `cargo run --features docs --bin generate-docs`
//!   `cargo run --features docs --bin generate-docs -- --check`
//!
//! Two inputs drive the output:
//!
//! 1. The JSON Schema of [`DprintSvgConfigSchema`]
//!    (via [`dprint_plugin_svg::schema::generate_schema_value`]) — supplies
//!    the option list, per-property description, type, default, and for
//!    enum properties the ordered list of string variants.
//! 2. The raw text of `src/lib.rs` — supplies the rustdoc on each
//!    `*Config` enum, which carries the paired ```svg-input``` and
//!    ```svg-output <variant>``` fenced examples that the generator
//!    interpolates into each page's `## Values` section.
//!
//! Hand-maintained pages (`introduction.md`, `ignoring-code.md`) are
//! untouched. Everything under `docs/src/config/`, the `_generated/`
//! fragments included by `introduction.md`, AND `SUMMARY.md` itself
//! are rewritten from scratch on every run.
//!
//! `SUMMARY.md` is generated (not `{{#include}}`-based) because
//! mdbook's `links` preprocessor expands `{{#include}}` on chapter
//! content only; in `SUMMARY.md` the helper remains literal text and
//! no config chapters get registered. A complete, statically-built
//! `SUMMARY.md` sidesteps that entirely.

use std::collections::{BTreeMap, HashMap};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use dprint_plugin_svg::schema::generate_schema_value;
use serde_json::Value;

const LIB_RS_SRC: &str = include_str!("../lib.rs");
const CONFIG_DIR: &str = "docs/src/config";
const GENERATED_DIR: &str = "docs/src/_generated";

/// One parsed property from the JSON Schema, normalized for rendering.
struct Property {
    name: String,
    description: String,
    type_label: String,
    /// Pretty-printed default for the documentation table (e.g.
    /// `` `"canonical"` `` with backticks around the JSON literal).
    default: Option<String>,
    /// Raw JSON representation of the default, for embedding into the
    /// example JSON config block (no backticks).
    default_json: Option<String>,
    enum_variants: Vec<EnumVariant>,
    /// Enum type name in Rust code (e.g. `AttributeSortConfig`) for
    /// cross-referencing rustdoc examples. `None` for scalar fields.
    enum_type: Option<String>,
}

struct EnumVariant {
    name: String,
    description: String,
}

/// One `svg-input` / `svg-output <variant>` block-set extracted from an
/// enum's rustdoc.
struct ExampleBlocks {
    input: Option<String>,
    outputs: HashMap<String, String>,
}

fn main() -> ExitCode {
    let check_mode = env::args().any(|a| a == "--check");
    let result = run(check_mode);
    match result {
        Ok(changed) => {
            if check_mode && changed > 0 {
                eprintln!(
                    "generate-docs --check: {changed} file(s) differ from committed state. \
                     Re-run `just book-docs` and commit the diff."
                );
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(e) => {
            eprintln!("generate-docs: {e}");
            ExitCode::from(2)
        }
    }
}

fn run(check_mode: bool) -> Result<usize, Box<dyn std::error::Error>> {
    let schema = generate_schema_value()?;
    let properties = parse_schema_properties(&schema)?;
    let examples = parse_example_blocks(LIB_RS_SRC);

    let mut outputs: BTreeMap<PathBuf, String> = BTreeMap::new();
    for property in &properties {
        let page = render_option_page(property, &examples);
        let file = PathBuf::from(CONFIG_DIR).join(page_filename(&property.name));
        outputs.insert(file, page);
    }
    outputs.insert(
        PathBuf::from(GENERATED_DIR).join("summary-config.md"),
        render_summary_fragment(&properties),
    );
    outputs.insert(
        PathBuf::from(GENERATED_DIR).join("defaults-table.md"),
        render_defaults_table(&properties),
    );
    outputs.insert(
        PathBuf::from("docs/src").join("SUMMARY.md"),
        render_summary(&properties),
    );
    outputs.insert(
        PathBuf::from(GENERATED_DIR).join("quickstart.md"),
        render_quickstart(),
    );

    let mut changed = 0usize;
    for (path, content) in outputs {
        if check_mode {
            let current = fs::read_to_string(&path).unwrap_or_default();
            if current != content {
                changed += 1;
                eprintln!("generate-docs: {} differs", path.display());
            }
        } else {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let current = fs::read_to_string(&path).unwrap_or_default();
            if current != content {
                fs::write(&path, &content)?;
                changed += 1;
            }
        }
    }
    Ok(changed)
}

fn page_filename(camel_case_name: &str) -> String {
    let mut kebab = String::with_capacity(camel_case_name.len() + 4);
    for (i, c) in camel_case_name.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                kebab.push('-');
            }
            kebab.push(c.to_ascii_lowercase());
        } else {
            kebab.push(c);
        }
    }
    kebab.push_str(".md");
    kebab
}

fn parse_schema_properties(schema: &Value) -> Result<Vec<Property>, String> {
    let properties_obj = schema
        .get("properties")
        .and_then(Value::as_object)
        .ok_or("schema is missing top-level `properties`")?;
    let definitions = schema.get("definitions").and_then(Value::as_object);
    let mut out = Vec::new();
    for (name, value) in properties_obj {
        if name == "locked" {
            continue;
        }
        out.push(extract_property(name, value, definitions)?);
    }
    Ok(out)
}

fn extract_property(
    name: &str,
    value: &Value,
    definitions: Option<&serde_json::Map<String, Value>>,
) -> Result<Property, String> {
    let description = value
        .get("description")
        .and_then(Value::as_str)
        .map(strip_trailing_url)
        .unwrap_or_default();
    let default_raw = value.get("default");
    let default = default_raw.map(format_default_value);
    let default_json = default_raw.map(format_default_json);

    let (type_label, enum_type, enum_variants) = if let Some(ref_target) = enum_ref_target(value) {
        let definitions = definitions.ok_or_else(|| {
            format!("{name}: property references $ref but schema has no definitions")
        })?;
        let definition = definitions
            .get(ref_target)
            .ok_or_else(|| format!("{name}: $ref target {ref_target} missing from definitions"))?;
        let variants = collect_enum_variants(definition)?;
        let type_label = render_enum_type(&variants);
        (type_label, Some(ref_target.to_string()), variants)
    } else {
        (render_scalar_type(value), None, Vec::new())
    };

    Ok(Property {
        name: name.to_string(),
        description,
        type_label,
        default,
        default_json,
        enum_variants,
        enum_type,
    })
}

fn format_default_json(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "null".to_string())
}

/// Schema property descriptions end with a ` https://…` link back to
/// their own page on the hosted book (useful in IDE tooltips, noise in
/// the book body). Strip the trailing URL — with optional preceding
/// period or whitespace — so the generated page doesn't self-reference.
fn strip_trailing_url(desc: &str) -> String {
    let trimmed = desc.trim_end();
    if let Some(pos) = trimmed.rfind("https://") {
        let before = trimmed[..pos].trim_end_matches(|c: char| c.is_whitespace() || c == '.');
        before.trim().to_string()
    } else {
        trimmed.trim().to_string()
    }
}

fn enum_ref_target(value: &Value) -> Option<&str> {
    // Schemars nullable enums render as `anyOf: [$ref, type: null]`.
    let candidates = value.get("anyOf").and_then(Value::as_array)?;
    for c in candidates {
        if let Some(r) = c.get("$ref").and_then(Value::as_str)
            && let Some(name) = r.strip_prefix("#/definitions/")
        {
            return Some(name);
        }
    }
    None
}

fn collect_enum_variants(definition: &Value) -> Result<Vec<EnumVariant>, String> {
    let one_of = definition
        .get("oneOf")
        .and_then(Value::as_array)
        .ok_or("enum definition missing `oneOf`")?;
    let mut variants = Vec::with_capacity(one_of.len());
    for entry in one_of {
        let name = entry
            .get("const")
            .and_then(Value::as_str)
            .ok_or("variant missing `const`")?
            .to_string();
        let description = entry
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        variants.push(EnumVariant { name, description });
    }
    Ok(variants)
}

fn render_enum_type(variants: &[EnumVariant]) -> String {
    variants
        .iter()
        .map(|v| format!("`\"{}\"`", v.name))
        .collect::<Vec<_>>()
        .join(" \\| ")
}

fn render_scalar_type(value: &Value) -> String {
    // Schema emits `type: ["boolean", "null"]` for nullable scalars —
    // strip the `null` variant when rendering user-facing types.
    if let Some(arr) = value.get("type").and_then(Value::as_array) {
        let mut names: Vec<&str> = arr
            .iter()
            .filter_map(Value::as_str)
            .filter(|s| *s != "null")
            .collect();
        names.sort_unstable();
        if !names.is_empty() {
            return format!("`{}`", names.join(" \\| "));
        }
    }
    if let Some(s) = value.get("type").and_then(Value::as_str) {
        return format!("`{s}`");
    }
    "`string`".to_string()
}

fn format_default_value(value: &Value) -> String {
    match value {
        Value::Null => "`null`".to_string(),
        Value::Bool(b) => format!("`{b}`"),
        Value::Number(n) => format!("`{n}`"),
        Value::String(s) => format!("`\"{s}\"`"),
        other => format!("`{other}`"),
    }
}

fn parse_example_blocks(source: &str) -> HashMap<String, ExampleBlocks> {
    let mut out: HashMap<String, ExampleBlocks> = HashMap::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        // Enum declarations follow a `pub enum <Name>Config {` line, so
        // scan backwards to find the preceding /// doc-comment block
        // and extract tagged fenced blocks from it.
        if let Some(enum_name) = enum_declaration_name(line)
            && let Some(doc_start) = find_doc_block_start(&lines, i)
        {
            let doc_slice = &lines[doc_start..i];
            let blocks = extract_example_blocks_from_doc(doc_slice);
            out.insert(enum_name, blocks);
        }
        i += 1;
    }
    out
}

fn enum_declaration_name(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix("pub enum ")?;
    let name_end = rest.find(|c: char| !c.is_ascii_alphanumeric() && c != '_')?;
    let name = &rest[..name_end];
    if name.ends_with("Config") {
        Some(name.to_string())
    } else {
        None
    }
}

fn find_doc_block_start(lines: &[&str], enum_line: usize) -> Option<usize> {
    // Walk backward past attribute blocks until a `///` line is hit,
    // then back-up across contiguous `///` lines to the block's top.
    // Multi-line `#[cfg_attr(...)]` attributes have continuation lines
    // (e.g. `    schemars(...)`, `)]`) that don't start with `#[`, so
    // we track bracket depth to span them as a single unit instead of
    // bailing on the first wayward line.
    //
    // Walking bottom-up:
    //   - each `]` adds +1 to depth (we're about to enter the block)
    //   - each `[` adds -1 to depth (we've exited the block)
    //   - while depth > 0 we're *inside* a multi-line attr and every
    //     non-`#[`/non-`///` line is just a continuation of it.
    let mut cursor = enum_line;
    let mut depth: i32 = 0;
    while cursor > 0 {
        cursor -= 1;
        let raw = lines[cursor];
        let trimmed = raw.trim_start();

        if depth > 0 {
            depth += inverse_bracket_delta(raw);
            continue;
        }

        if trimmed.starts_with("#[") || trimmed.is_empty() {
            // Single-line attribute OR blank gap — keep walking.
            continue;
        }

        if trimmed.starts_with("///") {
            let mut top = cursor;
            while top > 0 {
                let prev = lines[top - 1].trim_start();
                if prev.starts_with("///") {
                    top -= 1;
                } else {
                    break;
                }
            }
            return Some(top);
        }

        // Non-attribute, non-doc line. If it contains any closing
        // bracket we're likely at the tail of a multi-line attr
        // (e.g. `)]`). Enter depth-tracking mode and keep walking.
        let delta = inverse_bracket_delta(raw);
        if delta > 0 {
            depth += delta;
            continue;
        }

        return None;
    }
    None
}

/// Net `]` vs `[` count on a single line (reversed from the normal
/// direction because we walk the file bottom-up).
fn inverse_bracket_delta(line: &str) -> i32 {
    line.chars()
        .map(|c| match c {
            ']' => 1,
            '[' => -1,
            _ => 0,
        })
        .sum()
}

fn extract_example_blocks_from_doc(doc_lines: &[&str]) -> ExampleBlocks {
    let mut input: Option<String> = None;
    let mut outputs: HashMap<String, String> = HashMap::new();
    let mut current_tag: Option<String> = None;
    let mut current_variant: Option<String> = None;
    let mut current_body: Vec<String> = Vec::new();

    for raw in doc_lines {
        let stripped = strip_doc_prefix(raw);
        if let Some(fence_info) = stripped.strip_prefix("```") {
            if current_tag.is_some() {
                // closing fence — commit the block.
                let body = current_body.join("\n");
                match (current_tag.take(), current_variant.take()) {
                    (Some(tag), _) if tag == "svg-input" => {
                        input = Some(body);
                    }
                    (Some(tag), Some(variant)) if tag == "svg-output" => {
                        outputs.insert(variant, body);
                    }
                    _ => {}
                }
                current_body.clear();
            } else {
                let info = fence_info.trim();
                let mut parts = info.splitn(2, char::is_whitespace);
                let tag = parts.next().unwrap_or("").to_string();
                let variant = parts.next().map(|s| s.trim().to_string());
                if tag == "svg-input" || tag == "svg-output" {
                    current_tag = Some(tag);
                    current_variant = variant;
                }
            }
        } else if current_tag.is_some() {
            current_body.push(stripped.to_string());
        }
    }

    ExampleBlocks { input, outputs }
}

fn strip_doc_prefix(line: &str) -> &str {
    let trimmed = line.trim_start();
    if let Some(rest) = trimmed.strip_prefix("/// ") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("///") {
        rest
    } else {
        trimmed
    }
}

fn render_option_page(property: &Property, examples: &HashMap<String, ExampleBlocks>) -> String {
    let mut out = String::new();
    out.push_str(&format!("# {}\n\n", property.name));
    let (lead, _) = split_description(&property.description);
    if !lead.is_empty() {
        out.push_str(lead);
        out.push_str("\n\n");
    }

    out.push_str("|             |        |\n");
    out.push_str("| ----------- | ------ |\n");
    out.push_str(&format!("| **Type**    | {} |\n", property.type_label));
    if let Some(default) = &property.default {
        out.push_str(&format!("| **Default** | {default} |\n"));
    } else {
        out.push_str("| **Default** | *inherits from the top-level key in the same `dprint.json`* |\n");
    }
    out.push('\n');

    if !property.enum_variants.is_empty() {
        out.push_str("## Values\n\n");
        let blocks = property
            .enum_type
            .as_ref()
            .and_then(|name| examples.get(name));
        let input_block = blocks.and_then(|b| b.input.as_deref());
        if let Some(input) = input_block {
            out.push_str("### Input\n\n```svg\n");
            out.push_str(input);
            if !input.ends_with('\n') {
                out.push('\n');
            }
            out.push_str("```\n\n");
        }
        for variant in &property.enum_variants {
            out.push_str(&format!("### `\"{}\"`\n\n", variant.name));
            if !variant.description.is_empty() {
                out.push_str(variant.description.trim());
                out.push_str("\n\n");
            }
            if let Some(output) = blocks.and_then(|b| b.outputs.get(&variant.name)) {
                out.push_str("```svg\n");
                out.push_str(output);
                if !output.ends_with('\n') {
                    out.push('\n');
                }
                out.push_str("```\n\n");
            }
        }
    }

    out.push_str("## Config\n\n");
    out.push_str("```json\n{\n  \"svg\": {\n    \"");
    out.push_str(&property.name);
    out.push_str("\": ");
    out.push_str(property.default_json.as_deref().unwrap_or("null"));
    out.push_str("\n  }\n}\n```\n");
    out
}

fn split_description(desc: &str) -> (&str, &str) {
    // The schema field description is a single line, so there's nothing
    // structured to split — return it as the lead.
    (desc.trim(), "")
}

fn render_summary_fragment(properties: &[Property]) -> String {
    let mut out = String::new();
    for property in properties {
        out.push_str(&format!(
            "- [{}](./config/{})\n",
            property.name,
            page_filename(&property.name)
        ));
    }
    out
}

/// Render the Quick-start JSON config snippet with the plugin version
/// baked in. Sourced from `env!("CARGO_PKG_VERSION")` at generator
/// compile time, which means the snippet always matches whatever
/// `Cargo.toml` declared at the commit being built — no CI-side
/// `{{LATEST_TAG}}` sed hack, no `git describe` tag-timing race.
fn render_quickstart() -> String {
    format!(
        "```json\n\
         {{\n\
         \x20 \"svg\": {{\n\
         \x20   \"attributeSort\": \"canonical\",\n\
         \x20   \"attributeLayout\": \"auto\",\n\
         \x20   \"spaceBeforeSelfClose\": true\n\
         \x20 }},\n\
         \x20 \"plugins\": [\n\
         \x20   \"https://plugins.dprint.dev/kjanat/svg-v{version}.wasm\"\n\
         \x20 ]\n\
         }}\n\
         ```\n",
        version = env!("CARGO_PKG_VERSION"),
    )
}

/// Render the full `SUMMARY.md` — the static Introduction/Ignoring Code
/// preamble plus one sidebar entry per config property. mdbook does NOT
/// expand `{{#include}}` inside SUMMARY.md, so the sidebar must be
/// materialised here instead of deferred to the preprocessor.
fn render_summary(properties: &[Property]) -> String {
    let mut out = String::new();
    out.push_str("# Summary\n\n");
    out.push_str("[Introduction](./introduction.md)\n");
    out.push_str("[Ignoring Code](./ignoring-code.md)\n\n");
    out.push_str("## Configuration\n\n");
    out.push_str(&render_summary_fragment(properties));
    out
}

fn render_defaults_table(properties: &[Property]) -> String {
    let mut out = String::new();
    out.push_str("| Option | Type | Default |\n");
    out.push_str("| ------ | ---- | ------- |\n");
    for property in properties {
        let default = property.default.as_deref().unwrap_or("*top-level*");
        out.push_str(&format!(
            "| [{name}](./config/{file}) | {ty} | {default} |\n",
            name = property.name,
            file = page_filename(&property.name),
            ty = property.type_label,
        ));
    }
    out
}

// Silence `unused` when building without the schema feature — the
// generator is a `schema`-feature binary in the first place, so the
// code below only lives when `#[cfg(feature = "schema")]` is active.
#[allow(dead_code)]
fn _prove_dir_constants_are_used() {
    let _ = Path::new(CONFIG_DIR);
    let _ = Path::new(GENERATED_DIR);
}
