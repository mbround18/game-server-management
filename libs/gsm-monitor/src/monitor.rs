//! This module provides functionality for monitoring log files.
//!
//! The monitor continuously reads from a log file and processes each new line using the log rules
//! defined in the `rules` module. It also detects if the file has been truncated or rotated and reopens it accordingly.

use crate::LogRule;
use crate::rules::{LOG_TARGET, LogRules};
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tracing::{error, info};

/// Represents a monitor that continuously reads a log file and processes its lines using provided rules.
#[derive(Clone)]
pub struct Monitor {
    rules: LogRules,
}

impl Monitor {
    /// Creates a new `Monitor` instance with the specified log rules.
    ///
    /// # Arguments
    ///
    /// * `rules` - A set of log rules to apply to the log file's lines.
    pub fn new(rules: LogRules) -> Self {
        Self { rules }
    }

    fn process_rules(&self, line: &str) {
        let rules = self.rules.get_rules(); // Store the result to extend its lifetime

        let filtered_rules: Vec<&LogRule> = rules
            .iter() // Now we iterate over a stable reference
            .filter(|rule| (rule.matcher)(line)) // Step 1: Filter matching rules
            .scan(false, |stop_flag, rule| {
                if *stop_flag {
                    return None; // Stop collecting after first stop=true
                }
                if rule.stop {
                    *stop_flag = true;
                }
                Some(rule) // Step 2: Collect until first stop=true
            })
            .collect();

        for rule in filtered_rules {
            (rule.action)(line); // Step 3: Process actions
        }
    }

    /// Runs the log monitor on the specified file path.
    ///
    /// The monitor continuously reads from the log file. If no new data is available,
    /// it waits briefly before trying again. If the file is truncated or rotated,
    /// it will re-open the file to continue monitoring.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the log file to monitor.
    pub fn run(&self, path: PathBuf) {
        // Open the log file.
        let file = match File::open(&path) {
            Ok(f) => f,
            Err(e) => {
                error!(target: LOG_TARGET, "Failed to open log file {}: {}", path.display(), e);
                return;
            }
        };

        let mut reader = BufReader::new(file);
        // Move to the end of the file.
        if let Err(e) = reader.seek(SeekFrom::End(0)) {
            error!(target: LOG_TARGET, "Failed to seek to end of {}: {}", path.display(), e);
            return;
        }

        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    // End of file reached. Check if the file was truncated or rotated.
                    if let Ok(metadata) = reader.get_ref().metadata() {
                        if let Ok(current_pos) = reader.stream_position() {
                            if metadata.len() < current_pos {
                                info!(target: LOG_TARGET, "Log file {} was truncated/rotated. Re-opening.", path.display());
                                // Re-open the file if it has been truncated or rotated.
                                match File::open(&path) {
                                    Ok(new_file) => {
                                        reader = BufReader::new(new_file);
                                        if let Err(e) = reader.seek(SeekFrom::Start(0)) {
                                            error!(target: LOG_TARGET, "Failed to seek to start of {}: {}", path.display(), e);
                                        }
                                    }
                                    Err(e) => {
                                        error!(target: LOG_TARGET, "Failed to re-open log file {}: {}", path.display(), e);
                                    }
                                }
                            }
                        }
                    }
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }
                Ok(_) => {
                    self.process_rules(line.trim_end());
                }
                Err(e) => {
                    error!(target: LOG_TARGET, "Error reading from {}: {}", path.display(), e);
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }
            }
        }
    }
}

/// Starts monitoring a specific log file in a new thread.
///
/// This function spawns a new thread that continuously monitors the given log file.
/// If the thread fails to spawn, an error is logged.
///
/// # Arguments
///
/// * `log_file` - The path to the log file to monitor.
/// * `rules` - The set of log rules to apply.
pub fn start_monitor_in_thread(log_file: PathBuf, rules: LogRules) {
    let monitor = Monitor::new(rules);
    let log_file_clone = log_file.clone();
    let spawn_result = thread::Builder::new()
        .name(format!("log-monitor-{}", log_file_clone.display()))
        .spawn(move || {
            monitor.run(log_file);
        });

    match spawn_result {
        Ok(_handle) => {
            // Optionally, store or use _handle if you need to join later.
        }
        Err(e) => {
            error!(target: LOG_TARGET, "Failed to spawn log monitor thread: {}", e);
        }
    }
}

/// Starts monitoring both the server and error log files in the specified working directory.
///
/// This function expects a `logs` directory within the `working_dir` that contains `server.log` and `server.err`.
///
/// # Arguments
///
/// * `working_dir` - The directory containing the `logs` folder.
/// * `rules` - The set of log rules to apply.
pub fn start_instance_log_monitor(working_dir: PathBuf, rules: LogRules) {
    let log_dir = working_dir.join("logs");
    let server_log = log_dir.join("server.log");
    let server_err = log_dir.join("server.err");

    start_monitor_in_thread(server_log, rules.clone());
    start_monitor_in_thread(server_err, rules);
}
