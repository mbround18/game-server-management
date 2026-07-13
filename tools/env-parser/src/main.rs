use clap::Parser;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[clap(about = "Scans Rust projects for environment variables")]
struct Cli {
    /// Directory to scan for Cargo.toml files
    directory: String,

    /// Optional output file path
    #[clap(long)]
    output: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct EnvVarInfo {
    field: Option<String>,
    var_type: Option<String>,
    default: Option<String>,
    description: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    let cargo_files = find_cargo_toml_files(&args.directory)?;
    println!("Found {} Cargo.toml files", cargo_files.len());

    for cargo_file in cargo_files {
        process_cargo_toml(&cargo_file, args.output.as_deref())?;
    }

    println!("Processing complete!");
    Ok(())
}

fn find_cargo_toml_files(dir: &str) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut result = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            result.extend(find_cargo_toml_files(path.to_string_lossy().as_ref())?);
        } else if path.file_name().is_some_and(|f| f == "Cargo.toml") {
            result.push(path);
        }
    }
    Ok(result)
}

fn process_cargo_toml(cargo_path: &Path, output_path: Option<&Path>) -> Result<(), Box<dyn Error>> {
    println!("Processing: {}", cargo_path.display());
    let project_dir = cargo_path.parent().ok_or("Missing parent directory")?;
    let rust_files = find_rust_files(project_dir)?;
    let mut env_vars: HashMap<String, EnvVarInfo> = HashMap::new();

    for file in rust_files {
        extract_env_vars_from_file(&file, &mut env_vars)?;
    }

    let out_path =
        output_path.map_or_else(|| project_dir.join("variables.json"), std::path::Path::to_path_buf);
    let json = serde_json::to_string_pretty(&env_vars)?;
    fs::write(&out_path, json)?;
    println!(
        "Wrote {} variables to {}",
        env_vars.len(),
        out_path.display()
    );
    Ok(())
}

fn find_rust_files(dir: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut rust_files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            rust_files.extend(find_rust_files(&path)?);
        } else if path.extension().is_some_and(|e| e == "rs") {
            rust_files.push(path);
        }
    }
    Ok(rust_files)
}

fn extract_env_vars_from_file(
    file_path: &Path,
    env_vars: &mut HashMap<String, EnvVarInfo>,
) -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string(file_path)?;

    // Each pattern is now in a raw string literal (r#"..."#),
    // which avoids having to escape backslashes multiple times.
    // We also allow multiline with (?s) and an optional trailing comma with (?:,)?
    let patterns = vec![
        (
            r#"(?s)std::env::var\("([A-Z0-9_]+)"\)(?:,)?"#,
            Some((1, None, None)),
            None,
        ),
        (
            r#"(?s)env::var\("([A-Z0-9_]+)"\)(?:,)?"#,
            Some((1, None, None)),
            None,
        ),
        (
            r#"(?s)env::var\("([A-Z0-9_]+)"\)\.unwrap_or_else\(\|[_a-zA-Z]*\|\s*settings\.([a-zA-Z0-9_]+)\.clone\(\)\)(?:,)?"#,
            Some((1, Some(2), None)),
            None,
        ),
        (
            // More flexible approach with optional whitespace after 'env_parse!' and optional trailing comma.
            r#"(?s)env_parse!\s*\(\s*\"([A-Z0-9_]+)\"\s*,\s*(.*?)\s*,\s*([a-zA-Z0-9_:<>]+)\s*\)(?:,)?"#,
            Some((1, None, Some(3))),
            None,
        ),
        (
            r#"(?s)std::env::var\("([A-Z0-9_]+)"\)\.unwrap_or_else\(\|[_a-zA-Z]*\|\s*"([^"]+)"\.to_string\(\)\)(?:,)?"#,
            Some((1, None, None)),
            None,
        ),
        (
            r#"(?s)fetch_var\("([A-Z0-9_]+)"(?:,\s*"([^"]*)")?\)(?:,)?"#,
            Some((1, None, None)),
            Some("fetch_var"),
        ),
        (
            r#"(?s)is_env_var_truthy\("([A-Z0-9_]+)"\)(?:,)?"#,
            Some((1, None, None)),
            Some("bool"),
        ),
    ];

    for (pattern, groups, extra_type) in patterns {
        let regex = Regex::new(pattern)?;
        for caps in regex.captures_iter(&content) {
            let Some(var_name_cap) = caps.get(1) else {
                continue;
            };
            let var_name = var_name_cap.as_str().to_owned();
            let entry = env_vars.entry(var_name.clone()).or_default();

            // groups => (0: entire match) (1: var_name) (field_idx, type_idx)
            if let Some((_, field_idx, type_idx)) = groups {
                // If we have an index for the field, fill it
                if let Some(f_idx) = field_idx
                    && let Some(field_cap) = caps.get(f_idx)
                {
                    entry.field = Some(field_cap.as_str().to_owned());
                }
                // If we have an index for the type, fill it
                if let Some(t_idx) = type_idx
                    && let Some(var_type_cap) = caps.get(t_idx)
                {
                    entry.var_type = Some(var_type_cap.as_str().to_owned());
                }
                // fetch_var default is group(2)
                if pattern.contains("fetch_var")
                    && let Some(default_cap) = caps.get(2)
                {
                    entry.default = Some(default_cap.as_str().to_owned());
                }
                // env_parse! default is group(2)
                if pattern.contains("env_parse!")
                    && let Some(default_cap) = caps.get(2)
                {
                    entry.default = Some(default_cap.as_str().trim().to_owned());
                }
            }

            // If "bool", we set the var_type
            if let Some(hardcoded_type) = extra_type
                && hardcoded_type == "bool"
            {
                entry.var_type = Some("bool".to_owned());
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write_sample_source(path: &Path) {
        fs::write(
            path,
            r#"
pub fn example() {
    let _ = std::env::var("STD_ENV");
    let _ = env::var("SHORT_ENV");
    let _ = env::var("FIELD_ENV").unwrap_or_else(|_| settings.field.clone());
    let _ = env_parse!("PARSED_ENV", "42", u32);
    let _ = std::env::var("STRING_ENV").unwrap_or_else(|_| "fallback".to_string());
    let _ = fetch_var("FETCH_ENV", "fallback");
    let _ = is_env_var_truthy("TRUTHY_ENV");
}
"#,
        )
        .unwrap();
    }

    #[test]
    fn find_cargo_toml_files_recurses_into_nested_projects() {
        let temp_dir = tempdir().unwrap();
        let root = temp_dir.path();
        fs::write(root.join("Cargo.toml"), "[package]\nname = \"root\"\n").unwrap();

        let nested = root.join("nested");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("Cargo.toml"), "[package]\nname = \"nested\"\n").unwrap();

        let files = find_cargo_toml_files(root.to_str().unwrap()).unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn extract_env_vars_from_file_captures_known_patterns() {
        let temp_dir = tempdir().unwrap();
        let src = temp_dir.path().join("src.rs");
        write_sample_source(&src);

        let mut env_vars = HashMap::new();
        extract_env_vars_from_file(&src, &mut env_vars).unwrap();

        let std_env = env_vars.get("STD_ENV").unwrap();
        assert_eq!(std_env.description, "");
        let short_env = env_vars.get("SHORT_ENV").unwrap();
        assert_eq!(short_env.description, "");

        let field_env = env_vars.get("FIELD_ENV").unwrap();
        assert_eq!(field_env.field.as_deref(), Some("field"));

        let parsed_env = env_vars.get("PARSED_ENV").unwrap();
        assert_eq!(parsed_env.var_type.as_deref(), Some("u32"));
        assert_eq!(parsed_env.default.as_deref(), Some("\"42\""));

        let string_env = env_vars.get("STRING_ENV").unwrap();
        assert!(string_env.default.is_none());

        let fetch_env = env_vars.get("FETCH_ENV").unwrap();
        assert_eq!(fetch_env.default.as_deref(), Some("fallback"));

        let truthy_env = env_vars.get("TRUTHY_ENV").unwrap();
        assert_eq!(truthy_env.var_type.as_deref(), Some("bool"));
    }

    #[test]
    fn process_cargo_toml_writes_variables_json() {
        let temp_dir = tempdir().unwrap();
        let project_dir = temp_dir.path().join("project");
        let src_dir = project_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(
            project_dir.join("Cargo.toml"),
            "[package]\nname = \"sample\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        write_sample_source(&src_dir.join("lib.rs"));

        let cargo_path = project_dir.join("Cargo.toml");
        process_cargo_toml(&cargo_path, None).unwrap();

        let variables = fs::read_to_string(project_dir.join("variables.json")).unwrap();
        assert!(variables.contains("\"STD_ENV\""));
        assert!(variables.contains("\"FIELD_ENV\""));
        assert!(variables.contains("\"PARSED_ENV\""));
    }
}
