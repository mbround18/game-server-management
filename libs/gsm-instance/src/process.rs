use std::cmp::Ordering;
use strsim::jaro_winkler;
use sysinfo::{Pid, Signal, System};
use tracing::{debug, error, info}; // Fuzzy matching

/// Sends an interrupt signal (SIGINT) to the process with the given PID.
///
/// # Parameters
/// - `pid`: The process ID to send SIGINT to.
pub fn send_interrupt_to_pid(pid: u32) {
    let mut sys = System::new_all();
    sys.refresh_all();
    let sys_pid = Pid::from(pid as usize);
    if let Some(process) = sys.process(sys_pid) {
        info!("Found process with PID: {}", pid);
        match process.kill_with(Signal::Interrupt) {
            Some(_) => info!("Sent interrupt signal to PID: {}", pid),
            None => error!("Failed to send interrupt signal to PID: {}", pid),
        }
    } else {
        debug!(
            "Process with PID {} not found (it may have already stopped)",
            pid
        );
    }
}

/// A struct for managing server processes.
pub struct ServerProcess {
    system: System,
}

impl ServerProcess {
    /// Creates a new instance of `ServerProcess` by initializing the system information.
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self { system: sys }
    }

    /// Finds all processes whose executable path contains the specified substring.
    ///
    /// # Parameters
    /// - `executable_name`: A substring of the executable name to search for.
    ///
    /// # Returns
    /// A vector of references to matching processes.
    pub fn find_processes(&mut self, executable_name: &str) -> Vec<&sysinfo::Process> {
        self.system.refresh_all();
        let executable_name = executable_name.to_ascii_lowercase();

        debug!(
            "Scanning for processes similar to '{}'. Total processes: {}",
            executable_name,
            self.system.processes().len()
        );

        let mut processes: Vec<(&sysinfo::Process, f64)> = self
            .system
            .processes()
            .values()
            .map(|process| {
                let binding = process.name().to_ascii_lowercase();

                let process_name = binding.to_str().unwrap_or("unknown");
                let similarity = jaro_winkler(&executable_name, process_name);
                (process, similarity)
            })
            .filter(|(_, similarity)| *similarity > 0.75) // Only consider high-confidence matches
            .collect();

        // Sort by confidence score (descending)
        processes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

        // Return only the processes (ignoring similarity scores)
        processes.into_iter().map(|(process, _)| process).collect()
    }

    /// Returns true if any process matching the given executable substring is running.
    pub fn are_processes_running(&mut self, executable_name: &str) -> bool {
        !self.find_processes(executable_name).is_empty()
    }

    /// Sends an interrupt signal (SIGINT) to all processes whose executable path contains the given substring.
    ///
    /// # Parameters
    /// - `executable_name`: The substring to match against the executable paths.
    pub fn send_interrupt(&mut self, executable_name: &str) {
        let processes = self.find_processes(executable_name);
        if processes.is_empty() {
            panic!("Failed to find process with executable name: {executable_name}");
        }

        for process in processes {
            let pid = process.pid().as_u32();
            info!("Sending interrupt to process with PID: {}", pid);
            send_interrupt_to_pid(pid);
        }
    }
}

/// Manual implementation of Clone for ServerProcess.
/// This simply creates a new instance to refresh system information.
impl Clone for ServerProcess {
    fn clone(&self) -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{Child, Command};
    use std::thread;
    use std::time::Duration;

    /// Helper to spawn a dummy process that sleeps for a specified number of seconds.
    #[cfg(unix)]
    fn spawn_dummy_process() -> Child {
        Command::new("sleep")
            .arg("5")
            .spawn()
            .expect("Failed to spawn dummy process")
    }

    #[cfg(windows)]
    fn spawn_dummy_process() -> Child {
        Command::new("timeout")
            .args(&["/T", "5"])
            .spawn()
            .expect("Failed to spawn dummy process")
    }

    #[test]
    fn test_find_processes_none() {
        let mut sp = ServerProcess::new();
        let processes = sp.find_processes("nonexistent_executable_xyz");
        assert!(processes.is_empty());
    }

    #[test]
    fn test_are_processes_running() {
        let mut sp = ServerProcess::new();
        let mut child = spawn_dummy_process();
        // Give the process a moment to start.
        thread::sleep(Duration::from_millis(500));
        let process_name = if cfg!(unix) { "sleep" } else { "timeout" };
        let running = sp.are_processes_running(process_name);
        // Clean up: terminate the dummy process.
        let _ = child.kill();
        // this code is technically unreachable, but kept for clippy purposes
        if child.try_wait().is_ok() {
            thread::sleep(Duration::from_secs(1)); // Wait for the process to terminate.
        }
        assert!(running);
    }

    #[test]
    fn test_send_interrupt_to_pid() {
        let mut child = spawn_dummy_process();
        let pid = child.id();
        // Send interrupt to the dummy process.
        send_interrupt_to_pid(pid);
        // Allow some time for the signal to be delivered.
        thread::sleep(Duration::from_secs(1));
        // Try to wait for the process.
        let result = child.try_wait().expect("Failed to wait on process");
        // The dummy process should have terminated.
        assert!(result.is_some());
    }

    #[test]
    fn test_send_interrupt() {
        // Spawn two dummy processes.
        let mut child1 = spawn_dummy_process();
        let mut child2 = spawn_dummy_process();

        // Give them a moment to start.
        thread::sleep(Duration::from_millis(500));

        // Use the common command name.
        let process_name = if cfg!(unix) { "sleep" } else { "timeout" };
        let mut sp = ServerProcess::new();
        sp.send_interrupt(process_name);

        // Allow time for interrupts.
        thread::sleep(Duration::from_secs(1));

        // Check that at least one of the processes has terminated.
        let result1 = child1.try_wait().expect("Failed to wait on process 1");
        let result2 = child2.try_wait().expect("Failed to wait on process 2");
        assert!(result1.is_some() || result2.is_some());
    }
}
