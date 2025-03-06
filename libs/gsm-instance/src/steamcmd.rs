//! # steamcmd Module
//!
//! This module provides helper functions to interact with the SteamCMD binary.
//! It allows constructing a command to run SteamCMD, optionally using a custom path
//! provided via the `STEAMCMD_PATH` environment variable. If not set, it defaults to `"steamcmd"`.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use gsm_instance::steamcmd::{steamcmd_command, run_steamcmd};
//!
//! // Build a steamcmd command with appropriate arguments.
//! let mut cmd = steamcmd_command();
//! cmd.arg("+login").arg("anonymous")
//!    .arg("+force_install_dir").arg("/home/steam/server")
//!    .arg("+app_update").arg("2278520").arg("validate")
//!    .arg("+quit");
//! let output = cmd.output().expect("Failed to execute steamcmd");
//! println!("SteamCMD output: {:?}", output);
//!
//! // Alternatively, use `run_steamcmd` to run with arguments directly:
//! let args = &[
//!     "+login", "anonymous",
//!     "+force_install_dir", "/home/steam/server",
//!     "+app_update", "2278520", "validate",
//!     "+quit",
//! ];
//! let output = run_steamcmd(args).expect("Failed to run steamcmd");
//! println!("SteamCMD output: {:?}", output);
//! ```

use std::process::Command;
use tracing::debug;

/// Returns a `Command` configured to execute SteamCMD.
///
/// It checks the `STEAMCMD_PATH` environment variable to override the default location.
/// If not set, it defaults to `"steamcmd"`.
pub fn steamcmd_command() -> Command {
    let cmd = std::env::var("STEAMCMD_PATH").unwrap_or_else(|_| "steamcmd".to_string());
    debug!("Using steamcmd executable: {}", cmd);
    Command::new(cmd)
}

/// Runs SteamCMD with the provided arguments and returns its output.
///
/// # Parameters
///
/// - `args`: A slice of string slices representing the arguments to pass to SteamCMD.
///
/// # Returns
///
/// A `Result` containing the output of the command if successful, or an `std::io::Error` otherwise.
///
/// # Example
///
/// ```rust,no_run
/// use gsm_instance::steamcmd::run_steamcmd;
///
/// let args = &[
///     "+login", "anonymous",
///     "+force_install_dir", "/home/steam/server",
///     "+app_update", "2278520", "validate",
///     "+quit",
/// ];
/// let output = run_steamcmd(args).expect("Failed to run steamcmd");
/// println!("SteamCMD output: {:?}", output);
/// ```
pub fn run_steamcmd(args: &[&str]) -> Result<std::process::Output, std::io::Error> {
    let output = steamcmd_command().args(args).output()?;
    Ok(output)
}
