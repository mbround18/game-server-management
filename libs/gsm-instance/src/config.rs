use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LaunchMode {
    Native,
    Wine,
    Proton,
}

/// Configuration for a game server instance managed by gsm-instance.
///
/// This struct holds all parameters needed to configure the game server:
/// - `app_id`: The Steam App ID for the game server.
/// - `name`: The display name of the server.
/// - `command`: The executable command or path to launch the server.
/// - `install_args`: Additional arguments to pass to steamcmd during install/update.
/// - `launch_args`: Additional arguments to pass when launching the server.
/// - `force_windows`: If true, forces the Windows version to be installed/used,
///   which may be needed for launching with Wine64.
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
///     launch_mode: gsm_instance::config::LaunchMode::Native,
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

    pub launch_mode: LaunchMode,
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
            launch_mode: LaunchMode::Native,
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
