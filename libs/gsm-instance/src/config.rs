//! # Instance Configuration
//!
//! This module defines the structures and enumerations used to configure a game server instance.
//! The central piece is the `InstanceConfig` struct, which holds all the necessary settings
//! for installing, running, and managing a game server.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Defines the launch mode for the game server.
///
/// This enum allows specifying how the game server executable should be run, which is
/// particularly useful for handling cross-platform compatibility (e.g., running a
/// Windows-based server on Linux).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LaunchMode {
    /// Run the server executable natively. This is the default.
    Native,
    /// Run the server using Wine. This is typically used for Windows executables on Linux.
    Wine,
    /// Run the server using Proton, which is Valve's compatibility tool for running
    /// Windows games on Linux.
    Proton,
}

/// Configuration for a game server instance managed by `gsm-instance`.
///
/// This struct holds all the parameters needed to configure and manage a game server,
/// from installation with SteamCMD to launching the server process. It is the primary
/// configuration object used throughout the `gsm-instance` crate.
///
/// # Example
///
/// ```rust
/// use gsm_instance::config::{InstanceConfig, LaunchMode};
/// use std::path::PathBuf;
///
/// let config = InstanceConfig {
///     app_id: 123456,
///     name: "My Awesome Server".to_string(),
///     command: "server_executable".to_string(),
///     install_args: vec!["+beta".to_string(), "preview".to_string()],
///     launch_args: vec!["-nographics".to_string(), "-batchmode".to_string()],
///     force_windows: true,
///     working_dir: PathBuf::from("/home/steam/myserver"),
///     launch_mode: LaunchMode::Proton,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceConfig {
    /// The Steam App ID for the game server. This is used by SteamCMD to identify which
    /// game server to install or update.
    pub app_id: u32,
    /// A user-friendly name for the server, primarily used for logging and identification.
    pub name: String,
    /// The command or executable path used to launch the server. This can be a simple
    /// executable name (e.g., `valheim_server.x86_64`) or a path relative to the
    /// `working_dir`.
    pub command: String,
    /// A list of additional arguments to pass to SteamCMD during the installation or
    /// update process. This can be used for things like selecting a beta branch.
    pub install_args: Vec<String>,
    /// A list of additional arguments to pass to the server executable when it is launched.
    pub launch_args: Vec<String>,
    /// If `true`, forces the installation and launch of the Windows version of the game
    /// server, typically for use with Wine or Proton on Linux.
    pub force_windows: bool,
    /// The working directory where the server will be installed and run. All server-related
    /// files, logs, and the PID file will be stored here.
    pub working_dir: PathBuf,
    /// The launch mode for the server, which determines how the executable is run.
    pub launch_mode: LaunchMode,
}

impl Default for InstanceConfig {
    /// Creates a default `InstanceConfig` with empty or default values.
    ///
    /// The default configuration is not typically useful on its own, but it provides a
    /// convenient starting point for building a configuration.
    fn default() -> Self {
        Self {
            app_id: 0,
            name: String::new(),
            command: String::new(),
            install_args: Vec::new(),
            launch_args: Vec::new(),
            force_windows: false,
            working_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            launch_mode: LaunchMode::Native,
        }
    }
}

impl InstanceConfig {
    /// Returns the path to the PID file for the instance.
    ///
    /// The PID file is used to store the process ID of the running server, which allows
    /// for managing the server process (e.g., stopping or checking its status).
    pub fn pid_file(&self) -> PathBuf {
        self.working_dir.join("instance.pid")
    }

    /// Returns the path to the log directory for the instance.
    pub fn log_dir(&self) -> PathBuf {
        self.working_dir.join("logs")
    }

    /// Returns the path to the standard output log file for the server.
    pub fn stdout(&self) -> PathBuf {
        self.log_dir().join("server.log")
    }

    /// Returns the path to the standard error log file for the server.
    pub fn stderr(&self) -> PathBuf {
        self.log_dir().join("server.err")
    }
}

#[cfg(test)]
mod tests {
    use super::{InstanceConfig, LaunchMode};

    #[test]
    fn default_config_uses_empty_values_and_native_mode() {
        let config = InstanceConfig::default();

        assert_eq!(config.app_id, 0);
        assert_eq!(config.name, "");
        assert_eq!(config.command, "");
        assert!(config.install_args.is_empty());
        assert!(config.launch_args.is_empty());
        assert!(!config.force_windows);
        assert!(matches!(config.launch_mode, LaunchMode::Native));
    }

    #[test]
    fn path_helpers_are_relative_to_working_dir() {
        let working_dir = std::env::temp_dir().join("gsm-instance-config-tests");
        let config = InstanceConfig {
            working_dir: working_dir.clone(),
            ..InstanceConfig::default()
        };

        assert_eq!(config.pid_file(), working_dir.join("instance.pid"));
        assert_eq!(config.log_dir(), working_dir.join("logs"));
        assert_eq!(config.stdout(), working_dir.join("logs").join("server.log"));
        assert_eq!(config.stderr(), working_dir.join("logs").join("server.err"));
    }

    #[test]
    fn serde_round_trip_preserves_non_default_fields() {
        let config = InstanceConfig {
            app_id: 2278520,
            name: String::from("Test Server"),
            command: String::from("./server"),
            install_args: vec![String::from("+beta"), String::from("staging")],
            launch_args: vec![String::from("-log"), String::from("-port=27015")],
            force_windows: true,
            working_dir: std::path::PathBuf::from("/srv/server"),
            launch_mode: LaunchMode::Proton,
        };

        let serialized = serde_json::to_string(&config).expect("serialize config");
        let deserialized: InstanceConfig =
            serde_json::from_str(&serialized).expect("deserialize config");

        assert_eq!(deserialized.app_id, 2278520);
        assert_eq!(deserialized.name, "Test Server");
        assert_eq!(deserialized.command, "./server");
        assert_eq!(deserialized.install_args, vec!["+beta", "staging"]);
        assert_eq!(deserialized.launch_args, vec!["-log", "-port=27015"]);
        assert!(deserialized.force_windows);
        assert_eq!(
            deserialized.working_dir,
            std::path::PathBuf::from("/srv/server")
        );
        assert!(matches!(deserialized.launch_mode, LaunchMode::Proton));
    }
}
