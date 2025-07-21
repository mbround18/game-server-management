//! # Update Module
//!
//! This module provides functionality to check for and perform updates of the game server.
//!
//! It compares the build IDs from the current app manifest and the latest app info from SteamCMD.
//! If an update is available (i.e. the build IDs differ), the `update_server` function can be used
//! to update the installation via SteamCMD.
//!
//! ## Example
//!
//! ```rust,no_run
//! use std::path::Path;
//! use gsm_instance::update::{update_is_available, update_server};
//! use gsm_instance::errors::InstanceError;
//!
//! // Paths to the manifest and app info files
//! let manifest_path = Path::new("/home/steam/myserver/steamapps/appmanifest_123456.acf");
//! let appinfo_path = Path::new("/home/steam/Steam/appcache/appinfo.vdf");
//!
//! // Check if an update is available
//! let available = update_is_available(manifest_path, appinfo_path)?;
//! if available {
//!     // Run the update with any extra arguments (if needed)
//!     update_server(123456, Path::new("/home/steam/myserver"), false, &vec!["verbose".to_string()])?;
//! }
//! # Ok::<(), InstanceError>(())
//! ```

use crate::errors::InstanceError;
use crate::steamcmd::steamcmd_command;
use regex::Regex;
use std::fs;
use std::path::Path;
use tracing::{debug, info};

/// Struct holding build ID information.
#[derive(Debug, PartialEq, Eq)]
pub struct UpdateInfo {
    pub current_build_id: String,
    pub latest_build_id: String,
}

impl UpdateInfo {
    /// Creates a new UpdateInfo by reading the manifest and appinfo files.
    pub fn new(manifest_path: &Path, appinfo_path: &Path) -> Result<Self, InstanceError> {
        let manifest_data = fs::read_to_string(manifest_path)
            .map_err(|e| InstanceError::CommandExecutionError(e.to_string()))?;
        let current_build_id = extract_build_id_from_manifest(&manifest_data).to_string();

        let appinfo_data = fs::read_to_string(appinfo_path)
            .map_err(|e| InstanceError::CommandExecutionError(e.to_string()))?;
        let latest_build_id = extract_build_id_from_app_info(&appinfo_data).to_string();

        Ok(UpdateInfo {
            current_build_id,
            latest_build_id,
        })
    }

    /// Returns true if an update is available (build IDs differ).
    pub fn update_available(&self) -> bool {
        self.current_build_id != self.latest_build_id
    }
}

/// Extracts the build ID from the manifest file contents using regex.
///
/// Expected format: `"buildid"    "123456"`.
fn extract_build_id_from_manifest(manifest: &str) -> &str {
    let re = Regex::new(r#""buildid"\s+"(\d+)""#).unwrap();
    if let Some(caps) = re.captures(manifest) {
        caps.get(1).map_or("", |m| m.as_str())
    } else {
        panic!("Failed to extract buildid from manifest:\n{manifest}");
    }
}

/// Extracts the build ID from the appinfo file contents using regex.
///
/// Expected format (simplified): `"buildid"    "123456"`.
fn extract_build_id_from_app_info(app_info: &str) -> &str {
    let re = Regex::new(r#""buildid"\s+"(\d+)""#).unwrap();
    if let Some(caps) = re.captures(app_info) {
        caps.get(1).map_or("", |m| m.as_str())
    } else {
        panic!("Failed to extract buildid from appinfo:\n{app_info}");
    }
}

/// Checks if an update is available by comparing the build IDs from the manifest and appinfo files.
pub fn update_is_available(
    manifest_path: &Path,
    appinfo_path: &Path,
) -> Result<bool, InstanceError> {
    let update_info = UpdateInfo::new(manifest_path, appinfo_path)?;
    debug!("Update info: {:?}", update_info);
    Ok(update_info.update_available())
}

/// Updates the server installation using SteamCMD.
///
/// # Parameters
/// - `app_id`: The Steam App ID of the server.
/// - `install_dir`: The directory where the server is installed.
/// - `extra_args`: Additional arguments to pass to SteamCMD during update.
///
/// # Behavior
/// Builds a SteamCMD command to update the app (with validation) and executes it.
/// Returns an error if the command fails.
///
/// # Example
/// ```rust,no_run
/// # use std::path::Path;
/// # use gsm_instance::update::update_server;
/// update_server(123456, Path::new("/home/steam/myserver"), false, &vec!["verbose".to_string()]).expect("Update failed");
/// ```
pub fn update_server<P: AsRef<Path>>(
    app_id: u32,
    install_dir: P,
    force_windows: bool,
    extra_args: &[String],
) -> Result<(), InstanceError> {
    info!(
        "Updating app {} in {}",
        app_id,
        install_dir.as_ref().display()
    );
    let login = "+login anonymous".to_string();
    let force_install_dir = format!("+force_install_dir {}", install_dir.as_ref().display());
    let app_update = format!("+app_update {app_id} validate");
    let mut args = vec![force_install_dir, login, app_update];

    if force_windows {
        let platform = "windows";
        args.insert(0, format!("+@sSteamCmdForcePlatformType {platform}"));
    }

    args.extend_from_slice(extra_args);
    args.push(String::from("+quit"));

    let mut steamcmd = steamcmd_command();
    let command = steamcmd.args(&args);
    debug!("Executing update command: {:?}", command);
    let output = command
        .output()
        .map_err(|e| InstanceError::CommandExecutionError(e.to_string()))?;
    if output.status.success() {
        info!("Update successful.");
        Ok(())
    } else {
        Err(InstanceError::CommandExecutionError(format!(
            "Update failed with status: {:?}",
            output.status
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    const SAMPLE_MANIFEST: &str = r#"
"AppState"
{
    "appid"        "123456"
    "buildid"      "1000"
}
"#;

    const SAMPLE_APPINFO: &str = r#"
"appinfo"
{
    "buildid"      "1001"
}
"#;

    #[test]
    fn test_extract_build_id_from_manifest() {
        let build_id = extract_build_id_from_manifest(SAMPLE_MANIFEST);
        assert_eq!(build_id, "1000");
    }

    #[test]
    fn test_extract_build_id_from_app_info() {
        let build_id = extract_build_id_from_app_info(SAMPLE_APPINFO);
        assert_eq!(build_id, "1001");
    }

    #[test]
    fn test_update_info_update_available() {
        let temp_dir = tempdir().unwrap();
        let manifest_path = temp_dir.path().join("appmanifest.acf");
        let appinfo_path = temp_dir.path().join("appinfo.txt");
        fs::write(&manifest_path, SAMPLE_MANIFEST).unwrap();
        fs::write(&appinfo_path, SAMPLE_APPINFO).unwrap();

        let available = update_is_available(&manifest_path, &appinfo_path).unwrap();
        assert!(available);
    }

    #[test]
    fn test_update_info_no_update() {
        let temp_dir = tempdir().unwrap();
        let manifest_path = temp_dir.path().join("appmanifest.acf");
        let appinfo_path = temp_dir.path().join("appinfo.txt");
        let sample = r#"
"AppState"
{
    "appid"        "123456"
    "buildid"      "2000"
}
"#;
        fs::write(&manifest_path, sample).unwrap();
        fs::write(&appinfo_path, sample).unwrap();

        let available = update_is_available(&manifest_path, &appinfo_path).unwrap();
        assert!(!available);
    }
}
