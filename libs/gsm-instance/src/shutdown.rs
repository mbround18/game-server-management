//! # Shutdown Module
//!
//! This module provides functionality to gracefully shut down a running game server instance.
//! It sends an interrupt signal to all running server processes (identified by a specific substring in their
//! executable path) and waits until they have terminated.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use gsm_instance::shutdown::blocking_shutdown;
//!
//! // Gracefully shut down the server.
//! // Replace "my_game_server.exe" with the actual executable name.
//! blocking_shutdown("my_game_server.exe");
//! ```

use std::{thread, time::Duration};
use tracing::{debug, info};

use crate::process::ServerProcess;

/// Sends an interrupt signal to all running server processes and waits until they terminate.
///
/// This function is designed to perform a graceful shutdown of all processes associated
/// with a game server instance. It identifies running processes by checking their executable
/// path against a provided string.
///
/// # Arguments
///
/// * `executable`: A string slice representing a unique part of the server executable's
///   path or name. This is used to identify the target processes to shut down.
///
/// # Behavior
///
/// 1. A `ServerProcess` instance is created to manage process operations.
/// 2. An interrupt signal (e.g., SIGINT on Unix-like systems) is sent to all processes
///    whose executable path or name contains the `executable` string.
/// 3. The function then waits for a short period (5 seconds) to allow processes to begin
///    their shutdown sequence.
/// 4. It enters a loop, periodically checking (every 5 seconds) if any matching server
///    processes are still running.
/// 5. The loop continues until no matching processes are found, at which point the function
///    concludes that the server has been successfully stopped.
///
/// # Panics
///
/// This function does not explicitly panic, but underlying `ServerProcess` operations
/// might in extreme cases of system resource exhaustion.
pub fn blocking_shutdown(executable: &str) {
    let mut server_process = ServerProcess::new();
    info!("Sending interrupt signal to server processes...");
    server_process.send_interrupt(executable);
    // Wait a short while for processes to begin termination.
    thread::sleep(Duration::from_secs(5));
    loop {
        let mut sp = server_process.clone();
        debug!("Checking if server processes are still running...");
        if !sp.are_processes_running(executable) {
            info!("Server processes have been stopped successfully!");
            break;
        } else {
            debug!("Server processes still running. Waiting for 5 seconds...");
            thread::sleep(Duration::from_secs(5));
        }
    }
}
