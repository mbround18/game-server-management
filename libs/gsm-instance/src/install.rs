//! # Game Server Installation
//!
//! This module provides the core functionality for installing or updating a game server
//! using SteamCMD. It abstracts the process of constructing and executing the necessary
//! SteamCMD command.
//!
//! The main function, `install`, takes care of logging in, setting the installation
//! directory, and running the `app_update` command with validation. It also supports
//! additional arguments and environment variables for more advanced configurations,
//! such as installing a beta branch.
//!
//! # Example
//!
//! ```rust,no_run
//! use std::path::Path;
//! use gsm_instance::install::install;
//!
//! // Install a server with App ID 123456 to a specified directory.
//! let app_id = 123456;
//! let install_dir = Path::new("/home/steam/myserver");
//! let extra_args = vec!["-beta".to_string(), "preview".to_string()];
//!
//! let status = install(app_id, install_dir, false, &extra_args)
//!     .expect("Installation failed");
//!
//! assert!(status.success());
//! ```
use crate::executable::execute_mut;
use crate::steamcmd::steamcmd_command;
use std::env;
use std::io;
use std::path::Path;
use std::process::{ExitStatus, Stdio};
use tracing::{debug, info};

/// Adds additional SteamCMD arguments from the `ADDITIONAL_STEAMCMD_ARGS` environment variable.
///
/// This function checks for the `ADDITIONAL_STEAMCMD_ARGS` environment variable and, if it
/// is set, appends its value to the provided argument list. The value is trimmed of
/// whitespace and surrounding quotes.
fn add_additional_args(args: &mut Vec<String>) {
    if let Ok(extra_args) = env::var("ADDITIONAL_STEAMCMD_ARGS") {
        let trimmed = extra_args.trim_matches('"').trim();
        if !trimmed.is_empty() {
            args.push(trimmed.to_string());
        }
    }
}

/// Installs or updates a game server using SteamCMD.
///
/// This function constructs and executes the SteamCMD command required to install or
/// update a game server. It handles logging in, setting the installation directory,
// and running the `app_update` command.
///
/// # Parameters
///
/// - `app_id`: The Steam App ID of the game server to install or update.
/// - `install_dir`: The directory where the server should be installed.
/// - `force_windows`: If `true`, configures SteamCMD to download the Windows version of
///   the server, which is necessary for running with Wine or Proton.
/// - `extra_args`: A slice of extra arguments to append to the SteamCMD command, which
///   can be used for things like specifying a beta branch.
///
/// # Returns
///
/// Returns an `io::Result<ExitStatus>` that indicates whether the SteamCMD process
/// executed successfully.
///
/// # Behavior
///
/// - The function logs in to Steam as an anonymous user.
/// - It forces the installation to the specified `install_dir`.
/// - It runs `app_update` with the `validate` option to ensure file integrity.
/// - It appends any extra arguments from the `extra_args` parameter and the
///   `ADDITIONAL_STEAMCMD_ARGS` environment variable.
/// - The command's standard output and error are inherited, so they will be displayed
///   in the console.
pub fn install<P: AsRef<Path>>(
    app_id: u32,
    install_dir: P,
    force_windows: bool,
    extra_args: &[String],
) -> io::Result<ExitStatus> {
    info!(
        "Installing app {} to {}",
        app_id,
        install_dir.as_ref().display()
    );

    // Base SteamCMD arguments.
    let login = "+login anonymous".to_string();
    let force_install_dir = format!("+force_install_dir {}", install_dir.as_ref().display());
    let app_update = format!("+app_update {app_id} validate");

    // Start building the argument list.
    let mut args = vec![force_install_dir, login, app_update];

    if force_windows {
        let platform = "windows";
        args.insert(0, format!("+@sSteamCmdForcePlatformType {platform}"));
    }

    // Append any extra installation arguments.
    args.extend_from_slice(extra_args);
    // Append any additional arguments from environment variables.
    add_additional_args(&mut args);

    // Build the full SteamCMD command.
    let mut steamcmd = steamcmd_command();
    let command = steamcmd
        .args(&args)
        .arg("+quit")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    debug!("Launching install command: {:#?}", command);

    // Execute the command using our helper (assumed to be defined in executable.rs)
    execute_mut(command)
}

#[cfg(test)]
mod tests {
    use super::{add_additional_args, install};
    use crate::test_support::env_lock;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    #[cfg(unix)]
    fn write_executable_script(path: &Path, body: &str) {
        use std::os::unix::fs::PermissionsExt;

        fs::write(path, body).unwrap();
        let mut permissions = fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).unwrap();
    }

    #[test]
    fn add_additional_args_ignores_missing_or_blank_values() {
        let _lock = env_lock().lock().unwrap_or_else(|e| e.into_inner());
        let mut args = Vec::new();

        unsafe {
            std::env::remove_var("ADDITIONAL_STEAMCMD_ARGS");
        }
        add_additional_args(&mut args);
        assert!(args.is_empty());

        unsafe {
            std::env::set_var("ADDITIONAL_STEAMCMD_ARGS", "   ");
        }
        add_additional_args(&mut args);
        assert!(args.is_empty());

        unsafe {
            std::env::remove_var("ADDITIONAL_STEAMCMD_ARGS");
        }
    }

    #[test]
    fn add_additional_args_trims_wrapping_quotes() {
        let _lock = env_lock().lock().unwrap_or_else(|e| e.into_inner());
        let mut args = Vec::new();

        unsafe {
            std::env::set_var("ADDITIONAL_STEAMCMD_ARGS", "\"+app_info_update 1\"");
        }

        add_additional_args(&mut args);
        assert_eq!(args, vec![String::from("+app_info_update 1")]);

        unsafe {
            std::env::remove_var("ADDITIONAL_STEAMCMD_ARGS");
        }
    }

    #[cfg(unix)]
    #[test]
    fn install_passes_expected_args_to_steamcmd() {
        let _lock = env_lock().lock().unwrap_or_else(|e| e.into_inner());
        let temp_dir = tempdir().unwrap();
        let args_path = temp_dir.path().join("args.txt");
        let script_path = temp_dir.path().join("fake-steamcmd.sh");
        let script = format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\nexit 0\n",
            args_path.display()
        );
        write_executable_script(&script_path, &script);

        unsafe {
            std::env::set_var("STEAMCMD_PATH", &script_path);
            std::env::set_var("ADDITIONAL_STEAMCMD_ARGS", "\"+app_info_update 1\"");
        }

        let extra_args = vec![String::from("+download_depot 123 456")];
        let status = install(2278520, temp_dir.path(), true, &extra_args).unwrap();
        assert!(status.success());

        let recorded_args = fs::read_to_string(&args_path).unwrap();
        let lines: Vec<&str> = recorded_args.lines().collect();
        assert_eq!(lines[0], "+@sSteamCmdForcePlatformType windows");
        assert_eq!(
            lines[1],
            format!("+force_install_dir {}", temp_dir.path().display())
        );
        assert_eq!(lines[2], "+login anonymous");
        assert_eq!(lines[3], "+app_update 2278520 validate");
        assert_eq!(lines[4], "+download_depot 123 456");
        assert_eq!(lines[5], "+app_info_update 1");
        assert_eq!(lines[6], "+quit");

        unsafe {
            std::env::remove_var("STEAMCMD_PATH");
            std::env::remove_var("ADDITIONAL_STEAMCMD_ARGS");
        }
    }
}
