use crate::config::InstanceConfig;
use crate::errors::InstanceError;
use crate::process::send_interrupt_to_pid;
use crate::{install, shutdown, startup, update};
use std::fs;
use std::path::PathBuf;
use std::process::Child; // Using synchronous std process Child

/// The main struct representing a game server instance.
///
/// This struct holds the configuration for the instance and provides
/// methods to install, update, start, stop, and restart the server.
#[derive(Clone, Debug)]
pub struct Instance {
    pub config: InstanceConfig,
}

impl Instance {
    /// Creates a new instance with the given configuration.
    pub const fn new(config: InstanceConfig) -> Self {
        Self { config }
    }

    /// Reads and parses the current server PID from the pid file.
    ///
    /// # Errors
    ///
    /// Returns an error when the pid file is missing, unreadable, or contains
    /// an invalid integer PID.
    pub fn pid(&self) -> Result<u32, InstanceError> {
        let pid_file = self.config.pid_file();
        if pid_file.exists() {
            // Read the PID from the file
            return fs::read_to_string(&pid_file)
                .map_err(InstanceError::IoError)?
                .trim()
                .parse::<u32>()
                .map_err(InstanceError::ParseError);
        }
        Err(InstanceError::Unknown("Failed to find pid".to_owned()))
    }

    /// Installs the server using SteamCMD.
    ///
    /// # Errors
    ///
    /// Returns an error when SteamCMD cannot be launched or exits with a failure status.
    pub fn install(&self) -> Result<(), InstanceError> {
        let status = install::install(
            self.config.app_id,
            &self.config.working_dir,
            self.config.force_windows,
            &self.config.install_args,
        )
        .map_err(|e| InstanceError::CommandExecutionError(e.to_string()))?;
        if status.success() {
            Ok(())
        } else {
            Err(InstanceError::CommandExecutionError(format!(
                "Install failed with status {status:?}"
            )))
        }
    }

    /// Updates the server installation.
    ///
    /// # Errors
    ///
    /// Returns an error when update command execution fails.
    pub fn update(&self) -> Result<(), InstanceError> {
        update::update_server(
            self.config.app_id,
            &self.config.working_dir,
            self.config.force_windows,
            &self.config.install_args,
        )?;
        Ok(())
    }

    /// Checks whether an update is available for the server.
    pub fn update_available(&self) -> bool {
        let manifest_path: PathBuf = self
            .config
            .working_dir
            .join("steamapps")
            .join(format!("appmanifest_{}.acf", self.config.app_id));
        let appinfo_path: PathBuf = std::env::var("STEAM_APPINFO_PATH").map_or_else(|_| PathBuf::from("/home/steam/Steam/appcache/appinfo.vdf"), PathBuf::from);

        update::update_is_available(&manifest_path, &appinfo_path).unwrap_or(false)
    }

    /// Starts the server as a daemonized process.
    ///
    /// This method uses the synchronous startup function from startup.rs.
    /// # Returns
    /// A handle to the spawned child process.
    ///
    /// # Errors
    ///
    /// Returns an error when process launch or startup verification fails.
    pub fn start(&self) -> Result<Child, InstanceError> {
        startup::start_daemonized(&self.config)
            .map_err(|e| InstanceError::CommandExecutionError(e.to_string()))
    }

    /// Stops the server gracefully.
    ///
    /// # Errors
    ///
    /// Returns an error when the pid file cannot be removed after signalling the process.
    pub fn stop(&self) -> Result<(), InstanceError> {
        if let Ok(pid) = self.pid() {
            send_interrupt_to_pid(pid);
            fs::remove_file(self.config.pid_file()).map_err(InstanceError::IoError)?;
        } else {
            let file_name = std::path::Path::new(&self.config.command)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(self.config.command.as_str());
            shutdown::blocking_shutdown(file_name);
        }
        Ok(())
    }

    /// Restarts the server by stopping and then starting it.
    ///
    /// # Errors
    ///
    /// Returns an error when either stopping or starting the server fails.
    pub fn restart(&self) -> Result<(), InstanceError> {
        self.stop()?;
        // Optionally, insert a delay if needed.
        self.start()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn pid_reads_pid_file_and_reports_missing_files() {
        let temp_dir = tempdir().unwrap();
        let pid_path = temp_dir.path().join("instance.pid");
        fs::write(&pid_path, "12345\n").unwrap();

        let instance = Instance::new(InstanceConfig {
            working_dir: temp_dir.path().to_path_buf(),
            ..InstanceConfig::default()
        });

        assert_eq!(instance.pid().unwrap(), 12345);

        fs::remove_file(&pid_path).unwrap();
        assert!(instance.pid().is_err());
    }

    #[test]
    fn update_available_uses_environment_override() {
        let temp_dir = tempdir().unwrap();
        let manifest_path = temp_dir
            .path()
            .join("steamapps/appmanifest_2278520.acf");
        fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
        fs::write(
            &manifest_path,
            r#""AppState" { "buildid" "1000" }"#,
        )
        .unwrap();

        let appinfo_path = temp_dir.path().join("appinfo.vdf");
        fs::write(&appinfo_path, r#""appinfo" { "buildid" "2000" }"#).unwrap();

        unsafe {
            std::env::set_var("STEAM_APPINFO_PATH", &appinfo_path);
        }

        let instance = Instance::new(InstanceConfig {
            app_id: 2_278_520,
            working_dir: temp_dir.path().to_path_buf(),
            ..InstanceConfig::default()
        });

        assert!(instance.update_available());

        unsafe {
            std::env::remove_var("STEAM_APPINFO_PATH");
        }
    }

    #[test]
    fn stop_removes_pid_file_when_present() {
        let temp_dir = tempdir().unwrap();
        let pid_path = temp_dir.path().join("instance.pid");
        fs::write(&pid_path, "999999\n").unwrap();

        let instance = Instance::new(InstanceConfig {
            command: "nonexistent-command".to_owned(),
            working_dir: temp_dir.path().to_path_buf(),
            ..InstanceConfig::default()
        });

        instance.stop().unwrap();
        assert!(!pid_path.exists());
    }
}
