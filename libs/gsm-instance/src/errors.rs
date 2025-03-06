use thiserror::Error;

/// Errors that can occur during operations in the gsm-instance crate.
#[derive(Error, Debug)]
pub enum InstanceError {
    /// An error occurred when running SteamCMD.
    #[error("SteamCMD error: {0}")]
    SteamCmdError(String),

    /// An error occurred in process management (starting, stopping, etc.).
    #[error("Process error: {0}")]
    ProcessError(String),

    /// There was a problem with the configuration.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// An error occurred during command execution.
    #[error("Command execution error: {0}")]
    CommandExecutionError(String),

    /// A general I/O error occurred.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// An unknown error occurred.
    #[error("Unknown error: {0}")]
    Unknown(String),
}
