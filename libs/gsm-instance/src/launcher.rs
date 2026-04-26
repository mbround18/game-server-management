//! # Game Server Launcher
//!
//! This module is responsible for launching the game server executable. It handles different
//! launch modes, including native execution, and compatibility layers like Wine and Proton
//! for running Windows servers on Linux.
//!
//! The main entry point is the `launch_server` function, which takes an `InstanceConfig`
//! and constructs a `Command` ready to be executed.
use crate::config::InstanceConfig;
use crate::config::LaunchMode;
use crate::errors::InstanceError;
use crate::proton;
use crate::proton::ProtonConfig;
use std::env;
use std::fs::File;
use std::fs::create_dir_all;
use std::path::Path;
use std::process::{Command, Stdio};
use tracing::{debug, error};
use which::which;

/// Represents the Windows compatibility layer to use for launching the server.
enum WindowsCompat {
    /// Use Proton, with a specific `ProtonConfig`.
    Proton { config: ProtonConfig },
    /// Use Wine, with the specified path to the Wine executable.
    Wine { path: String },
    /// No compatibility layer.
    None,
}

impl WindowsCompat {
    /// Creates a `Command` for the given game executable using the compatibility layer.
    fn create_command(&self, game_exe: &str) -> Option<Command> {
        match self {
            WindowsCompat::Proton { config } => {
                debug!("Creating Proton command for: {}", game_exe);
                Some(config.create_command(game_exe))
            }
            WindowsCompat::Wine { path } => {
                debug!("Creating Wine command for: {}", game_exe);
                let mut cmd = Command::new(path);
                cmd.arg(game_exe);
                Some(cmd)
            }
            WindowsCompat::None => {
                debug!("No compatibility layer available for: {}", game_exe);
                None
            }
        }
    }
}

/// Tries to find a Proton installation, either a specific version or any available version.
fn try_find_proton(
    version_option: Option<&str>,
    force_proton: bool,
    app_id: u32,
) -> Result<WindowsCompat, String> {
    match proton::find_proton(version_option) {
        Ok(mut config) => {
            let version_desc = version_option.unwrap_or("any version");
            debug!("Found Proton {} at {}", version_desc, config.path);
            config.app_id = app_id.to_string();
            setup_proton_config(config)
        }
        Err(e) => {
            let err_msg = match version_option {
                Some(v) => format!("Failed to find or download Proton {}: {}", v, e),
                None => format!("Proton not found: {}", e),
            };

            if version_option.is_some() {
                error!("{}", err_msg);
            } else {
                debug!("{}", err_msg);
            }

            if force_proton {
                Err(err_msg)
            } else {
                Err(format!("Proton unavailable: {}", e))
            }
        }
    }
}

/// Sets up the Proton prefix and environment variables for a given `ProtonConfig`.
fn setup_proton_config(mut config: ProtonConfig) -> Result<WindowsCompat, String> {
    if let Ok(home) = env::var("HOME") {
        let prefix_path = format!("{}/.proton/prefixes/gsm", home);
        debug!("Setting up Proton prefix at: {}", prefix_path);
        if let Err(e) = proton::setup_prefix(&mut config, &prefix_path) {
            error!("Failed to set up Proton prefix: {}", e);
        } else {
            debug!("Successfully set up Proton prefix");
        }
    }

    debug!("Initializing Proton environment variables");
    if let Err(e) = proton::init_proton_env(&mut config) {
        error!("Failed to initialize Proton environment: {}", e);
    } else {
        debug!("Successfully initialized Proton environment");
    }

    Ok(WindowsCompat::Proton { config })
}

/// Checks if a string value represents a truthy value.
fn is_truthy(val: &str) -> bool {
    val == "1" || val == "true" || val == "yes"
}

/// Finds a suitable Windows compatibility layer (Proton or Wine) based on the launch mode
/// and environment variables.
fn find_windows_compatibility(
    app_id: u32,
    launch_mode: &LaunchMode,
) -> Result<WindowsCompat, String> {
    debug!("Searching for Windows compatibility layers");
    let force_proton = env::var("FORCE_PROTON")
        .map(|v| is_truthy(&v))
        .unwrap_or(false);

    if let LaunchMode::Proton = launch_mode {
        // Check if PROTON_VERSION is set
        if let Ok(version) = env::var("PROTON_VERSION") {
            debug!("PROTON_VERSION is set to: {}", version);

            let parsed_version = match proton::parse_version(&version) {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to parse PROTON_VERSION: {}", e);
                    version
                }
            };

            let result = try_find_proton(Some(&parsed_version), force_proton, app_id);
            if result.is_ok() || force_proton {
                return result;
            }
        }

        // If no specific version requested, try to find any version
        let result = try_find_proton(None, force_proton, app_id);
        if result.is_ok() || force_proton {
            return result;
        }
    }

    if let LaunchMode::Wine = launch_mode {
        if let Ok(wine_path) = find_wine() {
            debug!("Found Wine at: {}", wine_path);
            return Ok(WindowsCompat::Wine { path: wine_path });
        } else {
            debug!("Wine not found");
        }
    }

    if force_proton {
        return Err("FORCE_PROTON is set but no Proton installation was found".to_string());
    }

    debug!("No Windows compatibility layer found");
    Ok(WindowsCompat::None)
}

/// Finds the path to the Wine executable (`wine64` or `wine`).
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

/// Creates a `Command` for a Windows executable, using a compatibility layer if available.
fn get_command_for_windows(
    exe_path: &str,
    app_id: u32,
    launch_mode: &LaunchMode,
) -> Result<Command, InstanceError> {
    debug!("Getting Windows command for: {}", exe_path);

    // Try to find a suitable Windows compatibility layer
    let compat = find_windows_compatibility(app_id, launch_mode).map_err(|e| {
        // Check if we need to exit immediately due to FORCE_PROTON
        if env::var("FORCE_PROTON")
            .map(|v| is_truthy(&v))
            .unwrap_or(false)
        {
            error!("FORCE_PROTON set but Proton setup failed: {}", e);
            std::process::exit(1);
        }
        error!("Failed to find Windows compatibility layer: {}", e);
        InstanceError::CommandExecutionError(e)
    })?;

    match &compat {
        WindowsCompat::Proton { config } => {
            let cmd_exists = Path::new(&config.path).exists();
            debug!("Using Proton at: {} (exists: {})", config.path, cmd_exists);
            if !cmd_exists {
                error!("Proton executable not found at: {}", config.path);
            }
        }
        WindowsCompat::Wine { path } => {
            let cmd_exists = Path::new(path).exists();
            debug!("Using Wine at: {} (exists: {})", path, cmd_exists);
            if !cmd_exists {
                error!("Wine executable not found at: {}", path);
            }
        }
        WindowsCompat::None => debug!("No Windows compatibility layer selected"),
    }

    let cmd = compat.create_command(exe_path).ok_or_else(|| {
        error!("Failed to create command for {}", exe_path);
        InstanceError::CommandExecutionError(
            "No suitable Windows compatibility layer found.".to_string(),
        )
    })?;

    debug!("Created Windows command: {:?}", cmd);
    Ok(cmd)
}

/// Prepares a `Command` to launch the game server based on the provided configuration.
///
/// This function constructs a `Command` that is ready to be spawned as a child process.
/// It sets up the executable, arguments, working directory, and log files based on the
/// `InstanceConfig`.
///
/// # Arguments
///
/// * `config`: A reference to the `InstanceConfig` for the server.
///
/// # Returns
///
/// A `Result` containing the configured `Command`, or an `InstanceError` if something
/// goes wrong (e.g., a log file cannot be created).
///
/// # Behavior
///
/// - If `launch_mode` is `Native`, it creates a simple command for the executable.
/// - If `launch_mode` is `Proton` or `Wine`, it attempts to find a suitable compatibility
///   layer and constructs the command accordingly.
/// - It appends any `launch_args` from the configuration.
/// - It sets the working directory to `config.working_dir`.
/// - It creates the log directory and redirects the command's `stdout` and `stderr` to
///   log files (`server.log` and `server.err`).
pub fn launch_server(config: &InstanceConfig) -> Result<Command, InstanceError> {
    debug!("Launching server with config: {:?}", config);

    let mut command = match config.launch_mode {
        LaunchMode::Native => {
            debug!("Using native command: {}", config.command);
            Command::new(&config.command)
        }
        LaunchMode::Proton | LaunchMode::Wine => {
            debug!("Windows executable detected, finding compatibility layer");
            get_command_for_windows(&config.command, config.app_id, &config.launch_mode)?
        }
    };

    // Append additional launch arguments.
    if !config.launch_args.is_empty() {
        debug!("Adding launch args: {:?}", config.launch_args);
        for arg in &config.launch_args {
            command.arg(arg);
        }
    }

    // Set the working directory.
    debug!("Setting working directory: {:?}", config.working_dir);
    command.current_dir(&config.working_dir);

    if let Err(e) = create_dir_all(config.log_dir()) {
        error!("Failed to create log directory: {}", e);
        return Err(InstanceError::IoError(e));
    }

    debug!("Creating stdout log file at: {:?}", config.stdout());
    let stdout_file = match File::create(config.stdout()) {
        Ok(file) => file,
        Err(e) => {
            error!("Failed to create stdout log file: {}", e);
            return Err(InstanceError::IoError(e));
        }
    };

    debug!("Creating stderr log file at: {:?}", config.stderr());
    let stderr_file = match File::create(config.stderr()) {
        Ok(file) => file,
        Err(e) => {
            error!("Failed to create stderr log file: {}", e);
            return Err(InstanceError::IoError(e));
        }
    };

    command.stdout(Stdio::from(stdout_file));
    command.stderr(Stdio::from(stderr_file));

    debug!("Final command: {:?}", command);

    Ok(command)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::InstanceConfig;
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
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.keep();
        InstanceConfig {
            app_id: 123456,
            name: "TestServer".to_string(),
            command: dummy_command(),
            install_args: vec![],
            launch_args: vec![dummy_arg()],
            launch_mode,
            working_dir: path,
            force_windows: false,
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
        let command_result = launch_server(&config);
        assert!(command_result.is_ok());
        let mut command = command_result.unwrap();
        let mut child = command.spawn().expect("Failed to spawn child process");
        let status = child.wait().expect("Failed to wait on child process");
        assert!(status.success());
    }

    #[test]
    fn launch_server_creates_expected_log_files() {
        let config = test_config(LaunchMode::Native);

        let command_result = launch_server(&config);
        assert!(command_result.is_ok());
        assert!(config.stdout().exists());
        assert!(config.stderr().exists());
    }

    #[test]
    fn test_is_truthy() {
        assert!(is_truthy("1"));
        assert!(is_truthy("true"));
        assert!(is_truthy("yes"));
        assert!(!is_truthy("0"));
        assert!(!is_truthy("false"));
        assert!(!is_truthy("no"));
    }
}
