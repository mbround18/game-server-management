//! # Install Module
//!
//! This module provides a function to install (or update) a game server
//! using SteamCMD. It builds the SteamCMD command based on the provided
//! Steam App ID, installation directory, and additional arguments.
//!
//! The command constructed is similar to:
//!
//! ```sh
//! steamcmd +login anonymous +force_install_dir "<install_dir>" +app_update <app_id> validate [extra args...] +quit
//! ```
//!
//! Environment variables such as `ADDITIONAL_STEAMCMD_ARGS`, `USE_BETA`, `BETA_BRANCH`,
//! and `BETA_BRANCH_PASSWORD` can be used to further customize the install command.
//!
//! # Example
//!
//! ```rust,no_run
//! use std::path::Path;
//! use gsm_instance::install::install;
//!
//! // Install server with app_id 123456 to the specified working directory
//! let extra_args = vec!["verbose".to_string()];
//! let status = install(123456, Path::new("/home/steam/myserver"), false, &extra_args)
//!     .expect("Installation failed");
//! assert!(status.success());
//! ```

use crate::executable::execute_mut;
use crate::steamcmd::steamcmd_command;
use std::env;
use std::io;
use std::path::Path;
use std::process::{ExitStatus, Stdio};
use tracing::{debug, info};

/// Adds any additional SteamCMD arguments from the environment.
fn add_additional_args(args: &mut Vec<String>) {
    if let Ok(extra_args) = env::var("ADDITIONAL_STEAMCMD_ARGS") {
        let trimmed = extra_args.trim_matches('"').trim();
        if !trimmed.is_empty() {
            args.push(trimmed.to_string());
        }
    }
}

/// Installs (or updates) the server using SteamCMD.
///
/// # Parameters
/// - `app_id`: The Steam App ID of the server.
/// - `install_dir`: The directory where the server should be installed.
/// - `extra_args`: A vector of extra arguments to append to the SteamCMD command.
///
/// # Returns
///  an `io::Result<ExitStatus>` indicating the success or failure of the command execution.
///
/// # Behavior
/// - The command logs in as anonymous, forces the install directory, updates the app (with validation),
///   appends any extra arguments and beta-related options, then quits.
/// - Environment variables (`ADDITIONAL_STEAMCMD_ARGS`, `USE_BETA`, etc.) allow further customization.
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
    let app_update = format!("+app_update {} validate", app_id);

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
