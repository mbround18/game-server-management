use crate::config::InstanceConfig;
use crate::errors::InstanceError;
use glob::glob;
use std::fs::File;
use std::process::{Command, Stdio};
use tracing::debug;
use which::which;

enum WindowsCompat {
    Proton { path: String },
    Wine { path: String },
    UmuProton { path: String },
    None,
}

impl WindowsCompat {
    fn create_command(&self, game_exe: &str) -> Option<Command> {
        match self {
            WindowsCompat::Proton { path } => {
                let mut cmd = Command::new(path);
                cmd.arg("run").arg(game_exe);
                Some(cmd)
            }
            WindowsCompat::Wine { path } => {
                let mut cmd = Command::new(path);
                cmd.arg(game_exe);
                Some(cmd)
            }
            WindowsCompat::UmuProton { path } => {
                let mut cmd = Command::new(path);
                cmd.arg(game_exe);
                Some(cmd)
            }
            WindowsCompat::None => None,
        }
    }
}

fn find_windows_compatibility() -> WindowsCompat {
    debug!("Searching for Windows compatibility layers");
    if let Some(umu_launcher) = find_umu_launcher() {
        debug!("Found UMU launcher at: {}", umu_launcher);
        // if let Ok(proton_dir) = find_proton_dir() {
        //     debug!("Found Proton directory at: {}", proton_dir);
        return WindowsCompat::UmuProton {
            path: umu_launcher,
            // dir: proton_dir,
        };
        // } else {
        //     debug!("UMU launcher found but no Proton directory");
        // }
    } else {
        debug!("UMU launcher not found");
    }

    if let Ok(proton_path) = find_proton() {
        debug!("Found Proton at: {}", proton_path);
        return WindowsCompat::Proton { path: proton_path };
    } else {
        debug!("Proton not found");
    }

    if let Ok(wine_path) = find_wine() {
        debug!("Found Wine at: {}", wine_path);
        return WindowsCompat::Wine { path: wine_path };
    } else {
        debug!("Wine not found");
    }

    debug!("No Windows compatibility layer found");
    WindowsCompat::None
}

fn find_umu_launcher() -> Option<String> {
    // Try to find umu-run first (the main launcher binary)
    if let Ok(path) = which("umu-run") {
        return path.to_str().map(String::from);
    }

    // Fall back to umu-launcher if umu-run is not found
    which("umu-launcher")
        .ok()
        .and_then(|path| path.to_str().map(String::from))
}

fn find_proton() -> Result<String, String> {
    // Try glob search in common compatibility tools directories first
    let glob_patterns = [
        "/home/steam/.steam/root/compatibilitytools.d/*Proton*/proton",
        "/home/steam/.steam/steam/compatibilitytools.d/*Proton*/proton",
        "~/.local/share/Steam/compatibilitytools.d/*Proton*/proton",
        "~/.steam/root/compatibilitytools.d/*Proton*/proton",
        "~/.steam/steam/compatibilitytools.d/*Proton*/proton",
    ];

    debug!("Searching for Proton using glob patterns");
    for pattern in &glob_patterns {
        debug!("Trying pattern: {}", pattern);
        if let Ok(paths) = glob(pattern) {
            for path in paths.flatten() {
                if path.is_file() {
                    debug!("Found potential Proton at: {:?}", path);
                    return path
                        .to_str()
                        .map(|s| s.to_string())
                        .ok_or_else(|| "Failed to convert proton path to string.".to_string());
                }
            }
        }
    }

    // If glob search failed, try specific paths
    let fallback_paths = [
        "/usr/bin/proton",
        "~/.local/share/Steam/steamapps/common/Proton/proton",
        "/usr/local/bin/proton",
    ];

    debug!("Glob search failed, trying specific paths");
    for path in &fallback_paths {
        debug!("Checking path: {}", path);
        if let Ok(path) = which(path) {
            debug!("Found Proton at: {:?}", path);
            return path
                .to_str()
                .map(|s| s.to_string())
                .ok_or_else(|| "Failed to convert proton path to string.".to_string());
        }
    }

    Err("No Proton installation found.".to_string())
}

// fn find_proton_dir() -> Result<String, String> {
//     let glob_patterns = [
//         "/home/steam/.steam/root/compatibilitytools.d/*Proton*",
//         "/home/steam/.steam/steam/compatibilitytools.d/*Proton*",
//         "~/.local/share/Steam/compatibilitytools.d/*Proton*",
//         "~/.steam/root/compatibilitytools.d/*Proton*",
//         "~/.steam/steam/compatibilitytools.d/*Proton*",
//         "~/.local/share/Steam/steamapps/common/*Proton*",
//     ];

//     debug!("Searching for Proton directories");
//     for pattern in &glob_patterns {
//         debug!("Checking pattern: {}", pattern);
//         if let Ok(paths) = glob(pattern) {
//             for path in paths.flatten() {
//                 if path.is_dir() {
//                     debug!("Found Proton directory at: {:?}", path);
//                     return path
//                         .to_str()
//                         .map(|s| s.to_string())
//                         .ok_or_else(|| "Failed to convert proton dir path to string.".to_string());
//                 }
//             }
//         }
//     }

//     Err("No Proton directory found.".to_string())
// }

fn find_wine() -> Result<String, String> {
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
        // When force_windows is true, try to find a suitable Windows compatibility layer
        let compat = find_windows_compatibility();

        match &compat {
            WindowsCompat::Proton { path } => debug!("Using Proton at: {}", path),
            WindowsCompat::Wine { path } => debug!("Using Wine at: {}", path),
            WindowsCompat::UmuProton { path } => {
                debug!("Using UMU Launcher at: {}", path)
            }
            WindowsCompat::None => {}
        }

        compat.create_command(&config.command).ok_or_else(|| {
            InstanceError::CommandExecutionError(
                "No suitable Windows compatibility layer found.".to_string(),
            )
        })?
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

    #[test]
    fn test_umu_proton_command_creation() {
        let umu_path = "umu-launcher";
        let proton_dir = "/path/to/proton";
        let game_exe = "/path/to/game.exe";

        let umu_proton = WindowsCompat::UmuProton {
            path: umu_path.to_string(),
            dir: proton_dir.to_string(),
        };

        let cmd_option = umu_proton.create_command(game_exe);
        assert!(cmd_option.is_some());

        let cmd = cmd_option.unwrap();
        let args: Vec<_> = cmd.get_args().collect();

        assert_eq!(args.len(), 1);
        assert_eq!(args[0].to_str().unwrap(), game_exe);
    }
}
