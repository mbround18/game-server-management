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
            result.extend(find_cargo_toml_files(path.to_str().unwrap())?);
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
        output_path.map_or_else(|| project_dir.join("variables.json"), |p| p.to_path_buf());
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
            let var_name = caps.get(1).unwrap().as_str().to_string();
            let entry = env_vars.entry(var_name.clone()).or_default();

            // groups => (0: entire match) (1: var_name) (field_idx, type_idx)
            if let Some((_, field_idx, type_idx)) = groups {
                // If we have an index for the field, fill it
                if let Some(f_idx) = field_idx {
                    if let Some(field_cap) = caps.get(f_idx) {
                        entry.field = Some(field_cap.as_str().to_string());
                    }
                }
                // If we have an index for the type, fill it
                if let Some(t_idx) = type_idx {
                    if let Some(var_type_cap) = caps.get(t_idx) {
                        entry.var_type = Some(var_type_cap.as_str().to_string());
                    }
                }
                // fetch_var default is group(2)
                if pattern.contains("fetch_var") && caps.get(2).is_some() {
                    entry.default = Some(caps.get(2).unwrap().as_str().to_string());
                }
                // env_parse! default is group(2)
                if pattern.contains("env_parse!") && caps.get(2).is_some() {
                    entry.default = Some(caps.get(2).unwrap().as_str().trim().to_string());
                }
            }

            // If "bool", we set the var_type
            if let Some(hardcoded_type) = extra_type {
                if hardcoded_type == "bool" {
                    entry.var_type = Some("bool".to_string());
                }
            }
        }
    }

    Ok(())
}
