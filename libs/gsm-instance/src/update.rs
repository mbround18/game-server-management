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
    use crate::test_support::env_lock;
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

    #[cfg(unix)]
    fn write_executable_script(path: &Path, body: &str) {
        use std::os::unix::fs::PermissionsExt;

        fs::write(path, body).unwrap();
        let mut permissions = fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).unwrap();
    }

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
    #[should_panic(expected = "Failed to extract buildid from manifest")]
    fn test_extract_build_id_from_manifest_panics_when_missing() {
        let _ = extract_build_id_from_manifest("\"AppState\" {}\n");
    }

    #[test]
    #[should_panic(expected = "Failed to extract buildid from appinfo")]
    fn test_extract_build_id_from_app_info_panics_when_missing() {
        let _ = extract_build_id_from_app_info("\"appinfo\" {}\n");
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

    #[test]
    fn test_update_info_new_returns_error_for_missing_manifest() {
        let temp_dir = tempdir().unwrap();
        let manifest_path = temp_dir.path().join("missing_manifest.acf");
        let appinfo_path = temp_dir.path().join("appinfo.txt");
        fs::write(&appinfo_path, SAMPLE_APPINFO).unwrap();

        let error = UpdateInfo::new(&manifest_path, &appinfo_path).unwrap_err();
        assert!(matches!(error, InstanceError::CommandExecutionError(_)));
    }

    #[cfg(unix)]
    #[test]
    fn test_update_server_passes_force_windows_and_extra_args() {
        let _lock = env_lock().lock().unwrap_or_else(|e| e.into_inner());
        let temp_dir = tempdir().unwrap();
        let args_path = temp_dir.path().join("args.txt");
        let script_path = temp_dir.path().join("fake-steamcmd.sh");
        let script = format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\n",
            args_path.display()
        );
        write_executable_script(&script_path, &script);

        unsafe {
            std::env::set_var("STEAMCMD_PATH", &script_path);
        }

        let extra_args = vec![String::from("+app_info_update 1")];
        update_server(2278520, temp_dir.path(), true, &extra_args).unwrap();

        let recorded_args = fs::read_to_string(&args_path).unwrap();
        let lines: Vec<&str> = recorded_args.lines().collect();
        assert_eq!(lines[0], "+@sSteamCmdForcePlatformType windows");
        assert_eq!(
            lines[1],
            format!("+force_install_dir {}", temp_dir.path().display())
        );
        assert_eq!(lines[2], "+login anonymous");
        assert_eq!(lines[3], "+app_update 2278520 validate");
        assert_eq!(lines[4], "+app_info_update 1");
        assert_eq!(lines[5], "+quit");

        unsafe {
            std::env::remove_var("STEAMCMD_PATH");
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_update_server_returns_error_when_command_fails() {
        let _lock = env_lock().lock().unwrap_or_else(|e| e.into_inner());
        let temp_dir = tempdir().unwrap();
        let script_path = temp_dir.path().join("failing-steamcmd.sh");
        write_executable_script(&script_path, "#!/bin/sh\nexit 1\n");

        unsafe {
            std::env::set_var("STEAMCMD_PATH", &script_path);
        }

        let error = update_server(2278520, temp_dir.path(), false, &[]).unwrap_err();
        match error {
            InstanceError::CommandExecutionError(message) => {
                assert!(message.contains("Update failed with status"));
            }
            other => panic!("unexpected error: {other:?}"),
        }

        unsafe {
            std::env::remove_var("STEAMCMD_PATH");
        }
    }
}
