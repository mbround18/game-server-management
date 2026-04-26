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
fn ensure_log_dir(working_dir: &Path) -> Result<(), InstanceError> {
    let logs_dir = working_dir.join("logs");
    create_dir_all(&logs_dir)?;
    Ok(())
}

/// Starts the server as a daemonized process using the provided configuration.
/// Returns a handle to the spawned child process.
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
