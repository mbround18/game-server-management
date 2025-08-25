use crate::config::InstanceConfig;
use crate::config::LaunchMode;
use crate::errors::InstanceError;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tracing::debug;
use which::which; // Make sure LaunchMode is accessible

// Define a common function to find the Steam installation path
fn find_steam_root() -> Result<PathBuf, String> {
    let path = PathBuf::from("/home/steam/.steam/steam");
    if path.exists() {
        Ok(path)
    } else {
        Err("Steam installation not found.".to_string())
    }
}

// Function to find the Proton executable
fn find_proton() -> Result<PathBuf, String> {
    let steam_root = find_steam_root()?;
    let common_dir = steam_root.join("steamapps/common");

    // Find the latest version of Proton
    let proton_path = fs::read_dir(common_dir)
        .map_err(|e| format!("Failed to read common directory: {}", e))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            let file_name = path.file_name()?.to_str()?;
            if file_name.starts_with("Proton") {
                Some(path)
            } else {
                None
            }
        })
        .max() // max will get the latest version by lexicographical sort
        .ok_or_else(|| "No Proton installation found.".to_string())?;

    Ok(proton_path.join("proton"))
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
pub fn launch_server(config: &InstanceConfig) -> Result<Command, InstanceError> {
    let mut command = match &config.launch_mode {
        LaunchMode::Wine => {
            let mut cmd = Command::new(fine_wine().map_err(InstanceError::Unknown)?);
            cmd.arg(&config.command);
            cmd
        }
        LaunchMode::Proton => {
            let proton_path = find_proton().map_err(InstanceError::Unknown)?;
            let mut cmd = Command::new(proton_path);
            cmd.env(
                "STEAM_COMPAT_DATA_PATH",
                &config.working_dir.join("compatdata"),
            ); // Use a separate compatdata dir
            cmd.arg("run"); // Tell Proton to run an executable
            cmd.arg(&config.command);
            cmd
        }
        LaunchMode::Native => Command::new(&config.command),
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
    use crate::config::LaunchMode; // Also need to use LaunchMode in tests
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
    fn test_config(launch_mode: LaunchMode) -> InstanceConfig {
        InstanceConfig {
            app_id: 123456,
            name: "TestServer".to_string(),
            command: dummy_command(),
            install_args: vec![],
            launch_args: vec![dummy_arg()],
            launch_mode,
            working_dir: tempdir().unwrap().keep(),
            force_windows: false,
            // Ensure you have a stdout() method for testing
        }
    }

    #[test]
    fn test_launch_server_with_wine() {
        // For testing force_windows, check if "wine64" is available.
        if which::which("wine64").is_err() {
            eprintln!("wine64 not found, skipping test_launch_server_with_wine");
            return;
        }
        let config = test_config(LaunchMode::Wine);
        let command_result: Result<Command, InstanceError> = launch_server(&config);
        assert!(command_result.is_ok());
        let mut command = command_result.unwrap();
        let mut child: Child = command.spawn().expect("Failed to spawn child process");
        let status = child.wait().expect("Failed to wait on child process");
        assert!(status.success());
    }

    #[test]
    fn test_launch_server_with_proton() {
        // Check if proton can be found
        if find_proton().is_err() {
            eprintln!("Proton not found, skipping test_launch_server_with_proton");
            return;
        }

        let config = test_config(LaunchMode::Proton);
        let command_result: Result<Command, InstanceError> = launch_server(&config);
        assert!(command_result.is_ok());

        // This test would require a more complex setup to actually run a real proton command.
        // The assertion for `command_result.is_ok()` is still valid to check if the
        // Command was successfully constructed.
    }
}
