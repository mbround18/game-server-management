//! # gsm-instance
//!
//! The **gsm-instance** crate provides functionality to install, update, start, stop, and restart
//! a game server instance using SteamCMD. It offers a flexible configuration system and a CLI
//! interface for managing the server instance.
//!
//! ## Modules
//!
//! - **config**: Defines the `InstanceConfig` struct, which holds configuration options (e.g. app ID,
//!   server name, command, extra arguments, working directory, etc.).
//! - **env_config**: Centralizes environment variable parsing and defaulting. Use this module to
//!   manage environment-based configuration (e.g. beta options, additional arguments).
//! - **errors**: Defines custom error types (`InstanceError`) for the crate.
//! - **instance**: Exposes the main API through the `Instance` struct. Methods include install, update,
//!   start, stop, and restart.
//! - **launcher**: Provides functionality for launching the server process (including support for
//!   running Windows executables via Wine when forced).
//! - **process**: Contains utilities for detecting and managing running server processes.
//! - **shutdown**: Offers functionality to gracefully shut down the server by sending interrupts.
//! - **startup**: Wraps daemonization logic for starting the server process in the background.
//! - **steamcmd**: Provides helper functions for constructing and running SteamCMD commands.
//! - **update**: Contains functions to check for and perform updates by comparing build IDs.
//! - **cli**: Offers a command‑line interface for managing server operations (install, update, start, etc.).
//!

pub mod config;
pub mod errors;
mod executable;
pub mod install;
mod instance;
pub mod launcher;
mod process;
pub mod shutdown;
pub mod startup;
pub mod steamcmd;
pub mod update;

// CLI interface for the crate

// Re-export key types for easier usage.
pub use config::InstanceConfig;
pub use errors::InstanceError;
pub use instance::Instance;
