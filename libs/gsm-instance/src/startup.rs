use crate::config::InstanceConfig;
use crate::errors::InstanceError;
use crate::launcher::launch_server;
#[cfg(unix)]
use daemonize::Daemonize;
use std::fs;
use std::fs::{File, create_dir_all, write};
use std::path::Path;
use std::process::Child;
use tracing::info;

/// Creates log files in a "logs" subdirectory under the given working directory.
#[cfg(unix)]
fn create_log_files(working_dir: &Path) -> Result<(File, File), InstanceError> {
    let logs_dir = working_dir.join("logs");
    create_dir_all(&logs_dir)?;
    let stdout_path = logs_dir.join("server.log");
    let stderr_path = logs_dir.join("server.err");
    let stdout = File::create(stdout_path)?;
    let stderr = File::create(stderr_path)?;
    Ok((stdout, stderr))
}

fn write_pid_file(working_dir: &Path, pid: u32) -> Result<(), InstanceError> {
    let pid_file = working_dir.join("instance.pid");
    if pid_file.exists() {
        fs::remove_file(&pid_file)?;
    }
    write(pid_file, pid.to_string())?;
    Ok(())
}

/// Starts the server as a daemonized process using the provided configuration.
/// Returns a handle to the spawned child process.
#[cfg(unix)]
pub fn start_daemonized(config: InstanceConfig) -> Result<Child, InstanceError> {
    info!("Starting server as a daemonized process...");
    let working_dir = config.working_dir.clone();
    let (stdout, stderr) = create_log_files(&working_dir)?;
    let mut cmd = launch_server(&config)?;
    let child = cmd
        .spawn()
        .map_err(|e| InstanceError::CommandExecutionError(e.to_string()))?;

    Daemonize::new()
        .working_directory(&working_dir)
        .stdout(stdout)
        .stderr(stderr)
        .privileged_action(move || {
            info!("Executing privileged actions before launching server");
            write_pid_file(&working_dir, child.id())
                .expect("Failed to write pid file during daemon startup");
            child
        })
        .start()
        .map_err(|e| InstanceError::CommandExecutionError(e.to_string()))
}

/// Starts the server process on Windows without daemonization.
#[cfg(windows)]
pub fn start_daemonized(config: InstanceConfig) -> Result<Child, InstanceError> {
    info!("Starting server process without daemonization...");
    let working_dir = config.working_dir.clone();
    let mut cmd = launch_server(&config)?;
    let child = cmd
        .spawn()
        .map_err(|e| InstanceError::CommandExecutionError(e.to_string()))?;
    write_pid_file(&working_dir, child.id())?;
    Ok(child)
}
