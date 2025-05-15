use crate::config::InstanceConfig;
use crate::errors::InstanceError;
use std::fs::File;
use std::process::{Command, Stdio};
use tracing::debug;
use which::which;

fn find_proton() -> Result<String, String> {
    // Common Proton paths
    let proton_paths = [
        "/usr/bin/proton",
        "~/.local/share/Steam/steamapps/common/Proton",
        "/usr/local/bin/proton",
    ];

    for path in &proton_paths {
        if let Ok(path) = which(path) {
            return path
                .to_str()
                .map(|s| s.to_string())
                .ok_or_else(|| "Failed to convert proton path to string.".to_string());
        }
    }
    Err("No Proton installation found.".to_string())
}

fn fine_wine() -> Result<String, String> {
    // Attempt to find 'wine64' first
    if let Ok(path) = which("wine64") {
        return path
            .to_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "Failed to convert wine64 path to string.".to_string());
    }
    // If 'wine64' is not found, attempt to find 'wine'
    if let Ok(path) = which("wine") {
        return path
            .to_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "Failed to convert wine path to string.".to_string());
    }
    // If neither is found, return an error
    Err("Neither 'wine64' nor 'wine' was found in the system's PATH.".to_string())
}

/// Constructs the server process command according to the given configuration.
///
/// # Behavior
///
/// - If `force_windows` is true, the command is prefixed with `"wine64"` to run a Windows executable via Wine.
/// - All additional launch arguments (from `launch_args`) are appended to the command.
/// - The working directory is set to `config.working_dir`.
///
/// Instead of spawning the process immediately, this function returns the constructed `Command` so that
/// the caller can further configure it (for example, piping stdout/stderr) before spawning it.
///
/// # Errors
///
/// Returns an `InstanceError::CommandExecutionError` if building the command fails.
///
/// # Examples
///
/// ```rust,no_run
/// use gsm_instance::config::InstanceConfig;
/// use gsm_instance::launcher::launch_server;
/// use std::path::PathBuf;
///
/// let config = InstanceConfig {
///     app_id: 123456,
///     name: "My Server".to_string(),
///     command: "server_executable".to_string(),
///     install_args: vec![],
///     launch_args: vec!["-nographics".to_string(), "-batchmode".to_string()],
///     force_windows: false,
///     working_dir: PathBuf::from("/home/steam/myserver"),
/// };
///
/// let mut command = launch_server(&config).expect("Failed to build command");
/// // Configure stdout/stderr and spawn the command:
/// let child = command
///     .spawn()
///     .expect("Failed to spawn server process");
/// ```
pub fn launch_server(config: &InstanceConfig) -> Result<Command, InstanceError> {
    let mut command = if config.force_windows {
        // When force_windows is true, try to use Proton first, then fall back to Wine
        match find_proton() {
            Ok(proton_path) => {
                debug!("Using Proton at: {}", proton_path);
                let mut cmd = Command::new(proton_path);
                cmd.arg("run").arg(&config.command);
                cmd
            }
            Err(_) => {
                debug!("Proton not found, falling back to Wine");
                let wine_path = fine_wine().map_err(InstanceError::Unknown)?;
                debug!("Using Wine at: {}", wine_path);
                let mut cmd = Command::new(wine_path);
                cmd.arg(&config.command);
                cmd
            }
        }
    } else {
        Command::new(&config.command)
    };

    // Append additional launch arguments.
    for arg in &config.launch_args {
        command.arg(arg);
    }

    // Set the working directory.
    command.current_dir(&config.working_dir);

    let log_file = File::create(config.stdout()).map_err(InstanceError::IoError)?;

    command.stdout(Stdio::from(
        log_file.try_clone().map_err(InstanceError::IoError)?,
    ));
    command.stderr(Stdio::from(log_file));

    debug!("Launching server with command: {:?}", command);

    Ok(command)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::InstanceConfig;
    use crate::errors::InstanceError;
    use std::process::Child;
    use tempfile::tempdir;

    // On Unix systems, use "/bin/sleep" as a dummy command.
    #[cfg(unix)]
    fn dummy_command() -> String {
        "/bin/sleep".to_string()
    }

    #[cfg(unix)]
    fn dummy_arg() -> String {
        "1".to_string()
    }

    // On Windows, use "timeout" as a dummy command.
    #[cfg(windows)]
    fn dummy_command() -> String {
        "timeout".to_string()
    }
    #[cfg(windows)]
    fn dummy_arg() -> String {
        "1".to_string()
    }

    /// Creates a basic InstanceConfig for testing the launcher.
    fn test_config(force_windows: bool) -> InstanceConfig {
        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path().to_path_buf();
        InstanceConfig {
            app_id: 123456,
            name: "TestServer".to_string(),
            command: dummy_command(),
            install_args: vec![],
            launch_args: vec![dummy_arg()],
            force_windows,
            working_dir: dir_path,
        }
    }

    #[test]
    fn test_launch_server_with_force_windows() {
        // For testing force_windows, check if "wine64" is available.
        if which::which("wine64").is_err() {
            eprintln!("wine64 not found, skipping test_launch_server_with_force_windows");
            return;
        }
        let config = test_config(true);
        let command_result: Result<Command, InstanceError> = launch_server(&config);
        assert!(command_result.is_ok());
        let mut command = command_result.unwrap();
        let mut child: Child = command.spawn().expect("Failed to spawn child process");
        let status = child.wait().expect("Failed to wait on child process");
        assert!(status.success());
    }
}
