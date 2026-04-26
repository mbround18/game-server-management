//! # Game Server Startup
//!
//! This module provides functionality for starting a game server process,
//! particularly focusing on daemonizing the process and managing its lifecycle.
//! It handles the creation of necessary directories and the redirection of
//! standard output/error to log files.
use crate::config::InstanceConfig;
use crate::errors::InstanceError;
use crate::launcher::launch_server;
use std::fs;
use std::fs::create_dir_all;
use std::path::Path;
use std::process::Child;
use std::thread;
use std::time::Duration;
use tracing::info;

/// Ensures the log directory exists under the given working directory.
///
/// This helper function creates the `logs` subdirectory within the specified
/// `working_dir` if it doesn't already exist. This is where the server's
/// standard output and error streams will be redirected.
///
/// # Arguments
///
/// * `working_dir`: The root working directory of the game server instance.
///
/// # Returns
///
/// Returns `Ok(())` on success, or an `InstanceError::IoError` if the directory
/// cannot be created.
fn ensure_log_dir(working_dir: &Path) -> Result<(), InstanceError> {
    let logs_dir = working_dir.join("logs");
    create_dir_all(&logs_dir)?;
    Ok(())
}

/// Starts the game server as a daemonized process.
///
/// This function orchestrates the launch of the game server. It first prepares
/// the logging environment, then uses the `launcher` module to construct the
/// appropriate command. The server process is then spawned, its PID is recorded
/// in a file, and the function waits briefly to catch immediate startup failures.
///
/// # Arguments
///
/// * `config`: The `InstanceConfig` containing all necessary settings for the server.
///
/// # Returns
///
/// Returns a `Result` containing a `Child` handle to the spawned server process
/// on success, or an `InstanceError` if the startup fails at any stage.
///
/// # Behavior
///
/// - Creates a `logs` directory within the `working_dir` if it doesn't exist.
/// - Constructs the launch command using `launcher::launch_server`.
/// - Spawns the server process in the background.
/// - Writes the process ID (PID) of the spawned server to an `instance.pid` file
///   within the `working_dir`. This PID file is crucial for managing the server's
///   lifecycle (e.g., stopping it).
/// - Waits for a short duration (10 seconds) after spawning to detect if the server
///   process immediately exits, indicating a startup failure. If it exits, the PID
///   file is removed, and an error is returned.
/// - Standard output and error of the child process are redirected to `server.log`
///   and `server.err` files in the `logs` directory.
pub fn start_daemonized(config: InstanceConfig) -> Result<Child, InstanceError> {
    info!("Starting server as a daemonized process...");
    let working_dir = config.working_dir.clone();
    ensure_log_dir(&working_dir)?;

    match launch_server(&config) {
        Ok(mut cmd) => match cmd.spawn() {
            Ok(mut child) => {
                let pid = child.id();
                let pid_file = working_dir.join("instance.pid");

                if pid_file.exists() {
                    fs::remove_file(&pid_file)?;
                }

                fs::write(pid_file, pid.to_string())?;

                // Surface immediate startup failures so callers do not assume
                // a zombie/failed process is a healthy server start.
                // Some proton/wine launch failures occur a few seconds after
                // process creation; wait briefly to catch those as start errors.
                thread::sleep(Duration::from_secs(10));
                if let Some(status) = child
                    .try_wait()
                    .map_err(|e| InstanceError::CommandExecutionError(e.to_string()))?
                {
                    let _ = fs::remove_file(working_dir.join("instance.pid"));
                    return Err(InstanceError::CommandExecutionError(format!(
                        "Server process exited immediately with status {status}"
                    )));
                }

                Ok(child)
            }
            Err(e) => Err(InstanceError::CommandExecutionError(e.to_string())),
        },
        Err(e) => Err(InstanceError::CommandExecutionError(e.to_string())),
    }
}
