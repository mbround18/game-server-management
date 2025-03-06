use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tracing::{error, info, warn};

// Use Arc for clonable closures.
pub type Matcher = Arc<dyn Fn(&str) -> bool + Send + Sync>;
pub type Action = Arc<dyn Fn(&str) + Send + Sync>;

fn handle_line_with_target(file_name: &str, line: &str) {
    let outline = line.trim_end();
    if outline.contains("WARNING") {
        warn!(target: "game", file = %file_name, "{}", outline);
    } else if outline.contains("ERROR") {
        error!(target: "game", file = %file_name, "{}", outline);
    } else {
        info!(target: "game", file = %file_name, "{}", outline);
    }
}

/// The Monitor struct holds a list of (matcher, action) pairs.
#[derive(Clone)]
pub struct Monitor {
    rules: Arc<Mutex<Vec<(Matcher, Action)>>>,
}

impl Default for Monitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Monitor {
    /// Creates a new Monitor instance.
    pub fn new() -> Self {
        Self {
            rules: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Registers a new rule consisting of a matcher and an associated action.
    pub fn register_rule<F, G>(&self, matcher: F, action: G)
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
        G: Fn(&str) + Send + Sync + 'static,
    {
        let mut rules = self.rules.lock().unwrap();
        rules.push((Arc::new(matcher), Arc::new(action)));
    }

    /// Starts monitoring the specified log file.
    ///
    /// This function blocks the thread while tailing the file.
    /// It seeks to the end of the file (like tail -f) and then continuously polls for new lines.
    pub fn run(&self, path: PathBuf) {
        // Open the file.
        let file = File::open(&path)
            .unwrap_or_else(|e| panic!("Failed to open log file {:?}: {}", path, e));
        let mut reader = BufReader::new(file);

        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
        // Optionally, register a default rule that logs every line using the file name as target.
        self.register_rule(
            |_| true,
            move |line| {
                handle_line_with_target(&file_name, line);
            },
        );

        // Seek to the end of the file (tail -f behavior)
        reader
            .seek(SeekFrom::End(0))
            .expect("Failed to seek to the end of the log file");

        loop {
            let mut line = String::new();
            let bytes_read = reader
                .read_line(&mut line)
                .expect("Failed to read from log file");

            if bytes_read == 0 {
                // No new data; sleep for a short while before checking again.
                thread::sleep(Duration::from_millis(100));
                continue;
            }

            let line = line.trim_end();
            // Get a snapshot of the rules.
            let rules = {
                let rules_guard = self.rules.lock().unwrap();
                rules_guard.clone()
            };

            for (matcher, action) in rules.iter() {
                if matcher(line) {
                    action(line);
                }
            }
        }
    }
}

/// To run the monitor on its own thread:
pub fn start_monitor_in_thread(log_file: PathBuf, monitor: Monitor) {
    thread::spawn(move || {
        monitor.run(log_file);
    });
}

/// Starts monitoring the instance’s log files (server.log and server.err) based on its working directory.
/// This function constructs the paths to the log files and spawns a monitor for each.
pub fn start_instance_log_monitor(working_dir: PathBuf) {
    let log_dir = working_dir.join("logs");
    let server_log = log_dir.join("server.log");
    let server_err = log_dir.join("server.err");

    // Create separate Monitor instances for each file.
    let monitor_log = Monitor::new();
    let monitor_err = Monitor::new();

    start_monitor_in_thread(server_log, monitor_log);
    start_monitor_in_thread(server_err, monitor_err);
}
