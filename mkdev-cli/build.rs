use std::{env, fs, path::Path};

fn main() {
    let root_manifest_path = Path::new("..").join("Cargo.toml");
    let cargo_toml =
        fs::read_to_string(&root_manifest_path).expect("Failed to read parent Cargo.toml");

    let version = extract_string(&cargo_toml, "version");
    let authors = extract_array(&cargo_toml, "authors");
    let description = extract_string(&cargo_toml, "description");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("built_metadata.rs");

    fs::write(
        &dest_path,
        format!(
            r#"
                pub const VERSION: &str = "{version}";
                pub const AUTHORS: &str = "{authors}";
                pub const DESCRIPTION: &str = "{description}";
                pub const LONG_VERSION: &str = "{version} — {description}\n© 2025 {authors}. Licensed under MIT License — see LICENSE for details.";
            "#,
            version = version.escape_default(),
            authors = authors.escape_default(),
            description = description.escape_default(),
        ),
    )
    .expect("Failed to write built_metadata.rs");
}

fn extract_string(toml: &str, key: &str) -> String {
    toml.lines()
        .find(|line| line.trim_start().starts_with(&format!("{key} =")))
        .and_then(|line| {
            line.split_once('=')
                .map(|(_, v)| v.trim().trim_matches('"').to_string())
        })
        .unwrap_or_else(|| panic!("Missing `{key}` in root Cargo.toml"))
}

fn extract_array(toml: &str, key: &str) -> String {
    toml.lines()
        .find(|line| line.trim_start().starts_with(&format!("{key} =")))
        .map(|line| {
            line.split_once('=')
                .unwrap()
                .1
                .trim()
                .trim_start_matches('[')
                .trim_end_matches(']')
                .split(',')
                .map(|s| s.trim().trim_matches('"'))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_else(|| panic!("Missing `{key}` in root Cargo.toml"))
}
