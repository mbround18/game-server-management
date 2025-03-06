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
//! blocking_shutdown("test.exe");
//! ```

use std::{thread, time::Duration};
use tracing::{debug, info};

use crate::process::ServerProcess;

/// Sends an interrupt signal to all running server processes and waits until they terminate.
///
/// This function:
/// 1. Creates a new `ServerProcess` instance.
/// 2. Sends an interrupt signal to all processes whose executable contains `SERVER_EXECUTABLE`.
/// 3. Waits 5 seconds for the processes to begin shutting down.
/// 4. Continuously checks every 5 seconds until no matching processes are running.
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
