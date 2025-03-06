use crate::config::InstanceConfig;
use crate::errors::InstanceError;
use crate::launcher::launch_server;
use daemonize::Daemonize;
use std::fs::{File, create_dir_all};
use std::path::Path;
use std::process::Child;
use tracing::info;

/// Creates log files in a "logs" subdirectory under the given working directory.
fn create_log_files(working_dir: &Path) -> Result<(File, File), InstanceError> {
    let logs_dir = working_dir.join("logs");
    create_dir_all(&logs_dir)?;
    let stdout_path = logs_dir.join("server.log");
    let stderr_path = logs_dir.join("server.err");
    let stdout = File::create(stdout_path)?;
    let stderr = File::create(stderr_path)?;
    Ok((stdout, stderr))
}

/// Starts the server as a daemonized process using the provided configuration.
/// Returns a handle to the spawned child process.
pub fn start_daemonized(config: InstanceConfig) -> Result<Child, InstanceError> {
    info!("Starting server as a daemonized process...");
    let working_dir = config.working_dir.clone();
    let (stdout, stderr) = create_log_files(&working_dir)?;
    match launch_server(&config) {
        Ok(mut cmd) => {
            match cmd.spawn() {
                Ok(child) => {
                    Daemonize::new()
                        .working_directory(&working_dir)
                        .stdout(stdout)
                        .stderr(stderr)
                        .privileged_action(move || {
                            info!("Executing privileged actions before launching server");
                            // Return the command that will be exec'd.
                            child
                        })
                        .start()
                        .map_err(|e| InstanceError::CommandExecutionError(e.to_string()))
                }
                Err(e) => Err(InstanceError::CommandExecutionError(e.to_string())),
            }
        }
        Err(e) => Err(InstanceError::CommandExecutionError(e.to_string())),
    }
}
