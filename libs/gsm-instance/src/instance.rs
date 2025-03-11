use crate::config::InstanceConfig;
use crate::errors::InstanceError;
use crate::{install, shutdown, startup, update};
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
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
    pub fn new(config: InstanceConfig) -> Self {
        Self { config }
    }

    /// Installs the server using SteamCMD.
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
                "Install failed with status {:?}",
                status
            )))
        }
    }

    /// Updates the server installation.
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
        let appinfo_path: PathBuf = std::env::var("STEAM_APPINFO_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/home/steam/Steam/appcache/appinfo.vdf"));

        update::update_is_available(&manifest_path, &appinfo_path).unwrap_or(false)
    }

    /// Starts the server as a daemonized process.
    ///
    /// This method uses the synchronous startup function from startup.rs.
    /// # Returns
    /// A handle to the spawned child process.
    pub fn start(&self) -> Result<Child, InstanceError> {
        startup::start_daemonized(self.config.clone())
            .map_err(|e| InstanceError::CommandExecutionError(e.to_string()))
    }

    /// Stops the server gracefully.
    pub fn stop(&self) -> Result<(), InstanceError> {
        let pid_file = self.config.working_dir.join("instance.pid");
        if pid_file.exists() {
            // Read the PID from the file
            let pid = fs::read_to_string(&pid_file)
                .map_err(|e| InstanceError::IoError(e))?
                .trim()
                .parse::<i32>()
                .map_err(|e| InstanceError::ParseError(e))?;

            // Send SIGINT to the process
            signal::kill(Pid::from_raw(pid), Signal::SIGINT)
                .map_err(|e| InstanceError::SignalError(e))?;
        } else {
            let file_name = std::path::Path::new(&self.config.command)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap();
            shutdown::blocking_shutdown(file_name);
        }
        Ok(())
    }

    /// Restarts the server by stopping and then starting it.
    pub fn restart(&self) -> Result<(), InstanceError> {
        self.stop()?;
        // Optionally, insert a delay if needed.
        self.start()?;
        Ok(())
    }
}
