//! # Proton Management
//!
//! This module provides functionality for finding, downloading, and configuring Proton,
//! Valve's compatibility tool for running Windows games on Linux. It is a key component
//! for enabling Windows-based game servers to run in a Linux environment.
//!
//! The module can automatically locate installed Proton versions, download specific
//! versions from GitHub, and set up the necessary environment for a game server to
//! use Proton.
use flate2::read::GzDecoder;
use glob::glob;
use reqwest;
use std::env;
use std::fs::{File, create_dir_all};
use std::io;
use std::path::Path;
use std::process::Command;
use tar::Archive;
use tempfile::tempdir;
use thiserror::Error;
use tracing::{debug, info};
use which::which;

mod releases;
mod types;

pub use releases::{
    ReleaseError, fetch_latest_release, fetch_specific_release, list_available_releases,
};
pub use types::{ProtonRelease, ProtonVersion, VersionError, parse_version};

/// Represents errors that can occur during Proton-related operations.
#[derive(Error, Debug)]
pub enum ProtonError {
    /// An I/O error occurred, such as failing to create a directory or file.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// A network error occurred, such as failing to download a Proton release.
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// Proton could not be found, either locally or through download.
    #[error("Failed to find proton: {0}")]
    NotFound(String),

    /// An error occurred during path conversion.
    #[error("Path conversion error")]
    PathConversion,

    /// An error related to environment variables.
    #[error("Environment error: {0}")]
    EnvError(String),
}

/// Configuration for a Proton instance.
///
/// This struct holds the necessary information to configure and run a game server with Proton.
pub struct ProtonConfig {
    /// The path to the Proton executable.
    pub path: String,
    /// The path to the Wine prefix to be used by Proton.
    pub prefix: Option<String>,
    /// The version of Proton.
    pub version: String,
    /// The Steam App ID of the game being run.
    pub app_id: String,
    /// A list of environment variables to be set for the Proton environment.
    pub env_vars: Vec<(String, String)>,
}

impl ProtonConfig {
    /// Creates a `Command` to run a game executable with Proton.
    pub fn create_command(&self, game_exe: &str) -> Command {
        let mut cmd = Command::new(&self.path);
        cmd.arg("runinprefix").arg(game_exe);

        // Set environment variables
        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }

        // Set STEAM_COMPAT_CLIENT_INSTALL_PATH if prefix is specified
        if let Some(prefix) = &self.prefix {
            cmd.env("STEAM_COMPAT_CLIENT_INSTALL_PATH", prefix);
        }

        cmd
    }
}

/// Finds an installed Proton version based on a version pattern.
///
/// This function searches for a Proton installation in common Steam and custom directories.
/// It can search for a specific version or the first one it finds. If a version is not
/// found locally, it will attempt to download it if a version string is provided.
///
/// # Arguments
///
/// * `version`: An optional version string. If `Some`, it will look for a matching version.
///   If `None`, it will return the first Proton installation it finds.
pub fn find_proton(version: Option<&str>) -> Result<ProtonConfig, ProtonError> {
    let home = env::var("HOME").unwrap_or_else(|_| "/home/steam".to_string());
    let proton_dir = env::var("PROTON_DIR").unwrap_or_else(|_| format!("{home}/proton"));

    // Try glob search in common compatibility tools directories first
    let glob_patterns = [
        "/home/steam/.steam/root/compatibilitytools.d/*Proton*/proton".to_string(),
        "/home/steam/.steam/steam/compatibilitytools.d/*Proton*/proton".to_string(),
        format!(
            "{}/.local/share/Steam/compatibilitytools.d/*Proton*/proton",
            home
        ),
        format!("{}/.steam/root/compatibilitytools.d/*Proton*/proton", home),
        format!("{}/.steam/steam/compatibilitytools.d/*Proton*/proton", home),
        format!("{}/.steam/compatibilitytools.d/*Proton*/proton", home),
        format!("{}/GE-Proton*/proton", proton_dir),
        format!("{}/*Proton*/proton", proton_dir),
    ];

    // If version is specified, try to find a specific version first
    if let Some(v) = version {
        debug!("Searching for specific Proton version: {}", v);

        for pattern in &glob_patterns {
            let version_pattern = pattern.replace("*Proton*", &format!("*{v}*"));
            debug!("Trying pattern: {}", version_pattern);

            if let Ok(paths) = glob(&version_pattern) {
                for path in paths.flatten() {
                    if path.is_file() {
                        debug!("Found specific Proton version at: {:?}", path);
                        return create_proton_config(&path, v);
                    }
                }
            }
        }
    }

    // If specific version wasn't found or not specified, try generic patterns
    debug!("Searching for any Proton version using glob patterns");
    for pattern in &glob_patterns {
        debug!("Trying pattern: {}", pattern);
        if let Ok(paths) = glob(pattern) {
            for path in paths.flatten() {
                if path.is_file() {
                    debug!("Found Proton at: {:?}", path);
                    // Extract version from path
                    let version = path
                        .parent()
                        .and_then(|p| p.file_name())
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");

                    return create_proton_config(&path, version);
                }
            }
        }
    }

    // If glob search failed, try specific paths
    let fallback_paths = [
        "/usr/bin/proton".to_string(),
        format!("{}/.local/share/Steam/steamapps/common/Proton/proton", home),
        "/usr/local/bin/proton".to_string(),
        format!("{}/Proton/proton", home),
        format!("{}/proton/proton", home),
        format!("{}/proton", proton_dir),
    ];

    debug!("Glob search failed, trying specific paths");
    for path in &fallback_paths {
        debug!("Checking path: {}", path);
        if Path::new(path).exists() {
            debug!("Found Proton at: {}", path);
            return create_proton_config(path, "system");
        } else if let Ok(resolved_path) = which(path) {
            debug!("Found Proton at: {:?}", resolved_path);
            return create_proton_config(&resolved_path, "system");
        }
    }

    // If version was specified and not found, try to download it
    if let Some(v) = version {
        // Don't try to download if it looks like a path
        if !v.contains('/') {
            debug!("Attempting to download Proton version: {}", v);
            return download_proton(v);
        } else {
            debug!("Version '{}' looks like a path but wasn't found", v);
        }
    }

    Err(ProtonError::NotFound(
        "No Proton installation found.".to_string(),
    ))
}

/// Creates a `ProtonConfig` from a given path and version string.
fn create_proton_config<P: AsRef<Path>>(
    path: P,
    version: &str,
) -> Result<ProtonConfig, ProtonError> {
    let path_str = path
        .as_ref()
        .to_str()
        .ok_or(ProtonError::PathConversion)?
        .to_string();

    // Create basic environment variables
    let env_vars = Vec::new();

    // Add parent directory as STEAM_COMPAT_DATA_PATH if it exists
    let _parent = path.as_ref().parent().ok_or_else(|| {
        ProtonError::NotFound("Could not find parent directory for proton".to_string())
    })?;

    Ok(ProtonConfig {
        path: path_str,
        prefix: None, // Will be set by caller if needed
        version: version.to_string(),
        app_id: "0".to_string(), // Will be set by caller if needed
        env_vars,
    })
}

/// Downloads and installs a specific version of Proton GE.
///
/// This function downloads a Proton GE release from GitHub, extracts it to the
/// appropriate directory, and returns a `ProtonConfig` for it.
///
/// # Arguments
///
/// * `version`: The version of Proton GE to download (e.g., "GE-Proton8-25").
pub fn download_proton(version: &str) -> Result<ProtonConfig, ProtonError> {
    // Define the download URL and target directory
    let download_url = format!(
        "https://github.com/GloriousEggroll/proton-ge-custom/releases/download/{0}/{0}.tar.gz",
        version
    );

    // Create the compatibility tools directory
    let home = env::var("HOME")
        .map_err(|_| ProtonError::EnvError("HOME environment variable not found".to_string()))?;
    let target_dir = format!("{}/.steam/steam/compatibilitytools.d", home);
    let proton_dir = format!("{}/{}", target_dir, version);

    debug!("Creating directory: {}", target_dir);
    create_dir_all(&target_dir)?;

    // Check if this version is already installed
    let proton_path = format!("{}/proton", proton_dir);
    if Path::new(&proton_path).exists() {
        debug!("Proton {} is already installed at {}", version, proton_path);
        return create_proton_config(&proton_path, version);
    }

    info!("Proton {} not found locally, downloading...", version);

    // Download the Proton package
    info!("Downloading Proton {} from {}", version, download_url);
    let temp_dir = tempdir()?;
    let tar_gz_path = temp_dir.path().join(format!("{}.tar.gz", version));

    let mut response = reqwest::blocking::get(&download_url)?;
    let mut file = File::create(&tar_gz_path)?;

    response.copy_to(&mut file)?;
    debug!("Downloaded Proton package to {:?}", tar_gz_path);

    // Extract the archive
    info!("Extracting Proton to {}", target_dir);
    let tar_gz = File::open(&tar_gz_path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(&target_dir)?;

    debug!("Proton extracted successfully");

    // Return the ProtonConfig
    create_proton_config(&proton_path, version)
}

/// Sets up the Proton prefix for a game.
///
/// This function ensures the Wine prefix directory exists and configures the `ProtonConfig`
/// with the necessary environment variables for that prefix.
///
/// # Arguments
///
/// * `config`: A mutable reference to the `ProtonConfig`.
/// * `prefix_path`: The path to the Wine prefix directory.
pub fn setup_prefix(config: &mut ProtonConfig, prefix_path: &str) -> Result<(), ProtonError> {
    // Ensure the prefix directory exists
    debug!("Setting up Proton prefix at {}", prefix_path);
    create_dir_all(prefix_path)?;

    // Update the config with the prefix path
    config.prefix = Some(prefix_path.to_string());

    // Add necessary environment variables
    config.env_vars.push((
        "STEAM_COMPAT_DATA_PATH".to_string(),
        prefix_path.to_string(),
    ));
    config.env_vars.push((
        "STEAM_COMPAT_CLIENT_INSTALL_PATH".to_string(),
        prefix_path.to_string(),
    ));

    // Create pfx directory if it doesn't exist
    let pfx_dir = format!("{}/pfx", prefix_path);
    if !Path::new(&pfx_dir).exists() {
        debug!("Creating Proton pfx directory at {}", pfx_dir);
        create_dir_all(&pfx_dir)?;
    }

    Ok(())
}

/// Initializes the Proton environment with necessary variables.
///
/// This function sets up a collection of environment variables that are required for
/// Proton to function correctly, including paths, Steam App ID, and other settings.
///
/// # Arguments
///
/// * `config`: A mutable reference to the `ProtonConfig` to be updated with environment variables.
pub fn init_proton_env(config: &mut ProtonConfig) -> Result<(), ProtonError> {
    // Setup basic Proton/Wine environment variables
    let prefix_path = config
        .prefix
        .clone()
        .unwrap_or_else(|| "/tmp/proton-prefix".to_string());
    let pfx_path = format!("{}/pfx", prefix_path);

    // Critical Proton variables
    config
        .env_vars
        .push(("STEAM_RUNTIME".to_string(), "1".to_string()));
    let app_id = env::var("STEAM_APP_ID").unwrap_or_else(|_| config.app_id.clone());
    config
        .env_vars
        .push(("SteamAppId".to_string(), app_id.clone()));
    config.env_vars.push(("SteamGameId".to_string(), app_id));
    config
        .env_vars
        .push(("WINEPREFIX".to_string(), pfx_path.clone()));

    // Setup Steam client paths
    let home = env::var("HOME").unwrap_or_else(|_| "/home/steam".to_string());
    let steam_root = format!("{}/.local/share/Steam", home);
    let steam_lib_paths = [
        format!("{}/linux64", steam_root),
        format!(
            "{}/ubuntu12_32/steam-runtime/amd64/usr/lib/x86_64-linux-gnu",
            steam_root
        ),
        format!(
            "{}/steamapps/common/SteamLinuxRuntime/usr/lib/pressure-vessel/overrides/lib/x86_64-linux-gnu",
            steam_root
        ),
        format!(
            "{}/steamapps/common/SteamLinuxRuntime_soldier/usr/lib/pressure-vessel/overrides/lib/x86_64-linux-gnu",
            steam_root
        ),
    ];

    // Find and configure steam libraries
    let mut library_paths = String::new();
    for steam_lib in &steam_lib_paths {
        if Path::new(steam_lib).exists() {
            debug!("Using Steam library path: {}", steam_lib);

            // Find steamclient.so
            let steamclient_path = format!("{}/steamclient.so", steam_lib);
            if Path::new(&steamclient_path).exists() {
                debug!("Found steamclient.so at: {}", steamclient_path);
                config
                    .env_vars
                    .push(("STEAM_CLIENT_LIBRARY_PATH".to_string(), steam_lib.clone()));
            }

            // Add to library path
            if library_paths.is_empty() {
                library_paths = steam_lib.clone();
            } else {
                library_paths = format!("{}:{}", library_paths, steam_lib);
            }
        }
    }

    // Add library paths to LD_LIBRARY_PATH
    if !library_paths.is_empty() {
        let current_lib_path = env::var("LD_LIBRARY_PATH").unwrap_or_default();
        let new_lib_path = if current_lib_path.is_empty() {
            library_paths
        } else {
            format!("{}:{}", library_paths, current_lib_path)
        };

        config
            .env_vars
            .push(("LD_LIBRARY_PATH".to_string(), new_lib_path));
    }

    // Configure optional Proton settings
    for (key, default) in [
        ("PROTON_LOG", "1"),                 // Enable logs to debug issues
        ("PROTON_DUMP_DEBUG_COMMANDS", "1"), // More verbose debugging
        ("PROTON_USE_WINED3D", "0"),
        ("PROTON_NO_ESYNC", "0"),
        ("PROTON_NO_FSYNC", "0"),
    ] {
        if let Ok(value) = env::var(key) {
            config.env_vars.push((key.to_string(), value));
        } else {
            config.env_vars.push((key.to_string(), default.to_string()));
        }
    }

    Ok(())
}
