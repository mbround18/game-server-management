use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for a game server instance managed by gsm-instance.
///
/// This struct holds all parameters needed to configure the game server:
/// - `app_id`: The Steam App ID for the game server.
/// - `name`: The display name of the server.
/// - `command`: The executable command or path to launch the server.
/// - `install_args`: Additional arguments to pass to steamcmd during install/update.
/// - `launch_args`: Additional arguments to pass when launching the server.
/// - `force_windows`: If true, forces the Windows version to be installed/used, which may be needed for launching with Wine64.
/// - `working_dir`: The working directory where the server will be installed and run.
///
/// # Example
///
/// ```rust
/// use gsm_instance::config::InstanceConfig;
/// use std::path::PathBuf;
///
/// let config = InstanceConfig {
///     app_id: 123456,
///     name: "My Awesome Server".to_string(),
///     command: "server_executable".to_string(),
///     install_args: vec!["+install_flag".to_string(), "value".to_string()],
///     launch_args: vec!["-nographics".to_string(), "-batchmode".to_string()],
///     force_windows: true,
///     working_dir: PathBuf::from("/home/steam/myserver"),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceConfig {
    /// The Steam App ID for the game server.
    pub app_id: u32,
    /// The name of the server.
    pub name: String,
    /// The command (or executable path) used to launch the server.
    pub command: String,
    /// Additional arguments for the installation/update process.
    pub install_args: Vec<String>,
    /// Additional arguments for launching the server.
    pub launch_args: Vec<String>,
    /// When set to `true`, forces the installation and launch of the Windows version (via Wine64).
    pub force_windows: bool,
    /// The working directory for the server.
    pub working_dir: PathBuf,
}

impl Default for InstanceConfig {
    fn default() -> Self {
        Self {
            app_id: 0,
            name: String::new(),
            command: String::new(),
            install_args: Vec::new(),
            launch_args: Vec::new(),
            force_windows: false,
            working_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }
}

impl InstanceConfig {
    pub fn pid_file(&self) -> PathBuf {
        self.working_dir.join("instance.pid")
    }

    pub fn log_dir(&self) -> PathBuf {
        self.working_dir.join("logs")
    }

    pub fn stdout(&self) -> PathBuf {
        self.log_dir().join("server.log")
    }

    pub fn stderr(&self) -> PathBuf {
        self.log_dir().join("server.err")
    }
}
