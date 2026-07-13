//! This module provides functionality for monitoring log files.
//!
//! The monitor continuously reads from a log file and processes each new line using the log rules
//! defined in the `rules` module. It also detects if the file has been truncated or rotated and reopens it accordingly.

use crate::LogRule;
use crate::constants::INSTANCE_TARGET;
use crate::rules::LogRules;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use tracing::{debug, error, info, trace};

/// Represents a monitor that continuously reads a log file and processes its lines using provided rules.
#[derive(Clone)]
pub struct Monitor {
    rules: LogRules,
}

impl Monitor {
    /// Creates a new `Monitor` instance with the specified log rules.
    pub fn new(rules: LogRules) -> Self {
        trace!("Creating a new Monitor instance");
        Self { rules }
    }

    fn process_rules(&self, line: &str) {
        trace!("Processing rules for line: {line}");
        let mut rules = self.rules.get_rules();

        trace!("Sorting rules by ranking");
        rules.sort_by_key(|rule| rule.ranking);

        let filtered_rules: Vec<&LogRule> =
            rules.iter().filter(|rule| (rule.matcher)(line)).collect();

        trace!("Filtered rules count: {}", filtered_rules.len());
        for rule in filtered_rules {
            trace!("Applying rule action for line");
            (rule.action)(line);

            if rule.stop {
                break;
            }
        }
    }

    pub fn run(&self, path: &Path) {
        info!(target: INSTANCE_TARGET, "Starting watch on {}", path.display());

        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to open log file {}: {}", path.display(), e);
                return;
            }
        };

        let mut reader = BufReader::new(file);
        if let Err(e) = reader.seek(SeekFrom::End(0)) {
            error!("Failed to seek to end of {}: {}", path.display(), e);
            return;
        }

        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    if let Ok(metadata) = reader.get_ref().metadata()
                        && let Ok(current_pos) = reader.stream_position()
                        && metadata.len() < current_pos
                    {
                        info!(target: INSTANCE_TARGET,
                            "Log file {} was truncated/rotated. Re-opening.",
                            path.display()
                        );
                        match File::open(path) {
                            Ok(new_file) => {
                                trace!("Successfully reopened log file");
                                reader = BufReader::new(new_file);
                                if let Err(e) = reader.seek(SeekFrom::Start(0)) {
                                    error!("Failed to seek to start of {}: {}", path.display(), e);
                                }
                            }
                            Err(e) => {
                                error!("Failed to re-open log file {}: {}", path.display(), e);
                            }
                        }
                    }
                    thread::sleep(Duration::from_millis(100));
                }
                Ok(_) => {
                    trace!("Read line from file: {line}");
                    self.process_rules(line.trim_end());
                }
                Err(e) => {
                    error!("Error reading from {}: {}", path.display(), e);
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }
    }
}

pub fn start_monitor_in_thread(log_file: PathBuf, rules: LogRules) {
    info!(target: INSTANCE_TARGET,
        "Spawning new log monitor thread for file: {}",
        log_file.display()
    );
    let monitor = Monitor::new(rules);

    let spawn_result = thread::Builder::new()
        .name(format!("log-monitor-{}", log_file.display()))
        .spawn(move || {
            trace!("Log monitor thread started");
            monitor.run(&log_file);
        });

    match spawn_result {
        Ok(_) => trace!("Log monitor thread successfully spawned"),
        Err(e) => error!("Failed to spawn log monitor thread: {}", e),
    }
}

pub fn start_instance_log_monitor(working_dir: &Path, rules: LogRules) {
    let log_dir = working_dir.join("logs");
    let server_log = log_dir.join("server.log");
    let server_err = log_dir.join("server.err");

    info!(target: INSTANCE_TARGET,
        "Starting instance log monitor for logs in: {}",
        log_dir.display()
    );
    debug!(target: INSTANCE_TARGET, "Debugging log monitor startup");

    start_monitor_in_thread(server_log, rules.clone());
    start_monitor_in_thread(server_err, rules);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use std::fs;
    use std::sync::atomic::AtomicBool;
    use tempfile::tempdir;

    #[test]
    fn monitor_new_creates_instance() {
        let rules = LogRules::new();
        let _monitor = Monitor::new(rules);
    }

    #[test]
    fn run_returns_early_when_file_does_not_exist() {
        let rules = LogRules::new();
        let monitor = Monitor::new(rules);
        // A path that does not exist: run() should open-fail and return immediately.
        monitor.run(std::path::Path::new(
            "/tmp/gsm-test-nonexistent-log-file-xyz.log",
        ));
    }

    #[test]
    fn run_processes_lines_appended_to_log_file() {
        let temp = tempdir().unwrap();
        let log_path = temp.path().join("server.log");
        fs::write(&log_path, "").unwrap();

        let hit = Arc::new(AtomicBool::new(false));
        let hit_clone = Arc::clone(&hit);

        let rules = LogRules::new();
        rules.add_rule(
            |line| line.contains("SENTINEL"),
            move |_| {
                hit_clone.store(true, Ordering::SeqCst);
            },
            true,
            None,
        );

        let monitor = Monitor::new(rules);
        let path = log_path.clone();
        let handle = thread::spawn(move || monitor.run(&path));

        thread::sleep(Duration::from_millis(50));
        fs::write(&log_path, "line with SENTINEL keyword\n").unwrap();
        thread::sleep(Duration::from_millis(300));

        assert!(hit.load(Ordering::SeqCst), "rule action should have fired");
        drop(handle); // thread runs forever; let it be reaped by the process
    }

    #[test]
    fn start_monitor_in_thread_does_not_panic_for_missing_file() {
        let temp = tempdir().unwrap();
        let missing = temp.path().join("no-such-file.log");
        let rules = LogRules::new();
        // Should spawn a thread that opens/fails and exits cleanly.
        start_monitor_in_thread(missing, rules);
        thread::sleep(Duration::from_millis(50));
    }

    #[test]
    fn start_instance_log_monitor_spawns_without_panic() {
        let temp = tempdir().unwrap();
        start_instance_log_monitor(temp.path(), LogRules::default());
        thread::sleep(Duration::from_millis(50));
    }

    #[test]
    fn process_rules_applies_matching_rules_in_ranking_order() {
        let hits = Arc::new(AtomicUsize::new(0));
        let rules = LogRules::new();

        {
            let hits = Arc::clone(&hits);
            rules.add_rule(
                |line| line.contains("match"),
                move |_| {
                    hits.fetch_add(1, Ordering::SeqCst);
                },
                false,
                Some(5),
            );
        }

        {
            let hits = Arc::clone(&hits);
            rules.add_rule(
                |line| line.contains("match"),
                move |_| {
                    hits.fetch_add(10, Ordering::SeqCst);
                },
                true,
                Some(20),
            );
        }

        let monitor = Monitor::new(rules);
        monitor.process_rules("match this line");
        assert_eq!(hits.load(Ordering::SeqCst), 11);
    }

    #[test]
    fn process_rules_stops_after_first_stop_rule() {
        let hits = Arc::new(AtomicUsize::new(0));
        let rules = LogRules::new();

        {
            let hits = Arc::clone(&hits);
            rules.add_rule(
                |_| true,
                move |_| {
                    hits.fetch_add(1, Ordering::SeqCst);
                },
                true,
                Some(1),
            );
        }

        {
            let hits = Arc::clone(&hits);
            rules.add_rule(
                |_| true,
                move |_| {
                    hits.fetch_add(100, Ordering::SeqCst);
                },
                false,
                Some(10),
            );
        }

        let monitor = Monitor::new(rules);
        monitor.process_rules("any line");
        assert_eq!(hits.load(Ordering::SeqCst), 1);
    }
}
