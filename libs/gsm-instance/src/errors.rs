//! # Instance Errors
//!
//! This module defines the custom error type for the `gsm-instance` crate. All functions
//! in this crate that can fail will return a `Result` with the `InstanceError` type.
//!
//! The `InstanceError` enum consolidates all possible errors that can occur during the
//! management of a game server instance, from SteamCMD operations to process management.
use std::io;
use std::num::ParseIntError;
use thiserror::Error;

/// Represents all possible errors that can occur within the `gsm-instance` crate.
///
/// This enum is designed to provide a clear and comprehensive set of error types that
/// can be handled by the user of the crate. It uses `thiserror` to derive the `Error`
/// trait and provide descriptive error messages.
#[derive(Error, Debug)]
pub enum InstanceError {
    /// An error occurred while executing a SteamCMD command. This could be due to a
    /// network issue, an invalid App ID, or other SteamCMD-related problems.
    #[error("SteamCMD error: {0}")]
    SteamCmdError(String),

    /// An error related to managing the server process, such as failing to start,
    /// stop, or check the status of the server process.
    #[error("Process error: {0}")]
    ProcessError(String),

    /// An error indicating a problem with the instance configuration, such as a
    /// missing required field or an invalid value.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// An error that occurred during the execution of an external command, other than
    /// SteamCMD. This is often used for launch or compatibility tool errors.
    #[error("Command execution error: {0}")]
    CommandExecutionError(String),

    /// A general I/O error, which can occur during file operations like reading or
    /// writing configuration files, logs, or the PID file. This variant wraps the
    /// standard `std::io::Error`.
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    /// An error that occurred while parsing an integer. This can happen when reading
    /// a PID from a file, for example.
    #[error("Parse error: {0}")]
    ParseError(#[from] ParseIntError),

    /// A catch-all for any other type of error that does not fit into the other
    /// categories. This helps ensure that all error paths are handled.
    #[error("Unknown error: {0}")]
    Unknown(String),
}
