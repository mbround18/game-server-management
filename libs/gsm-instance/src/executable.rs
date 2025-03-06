use std::io;
use std::process::{Command, ExitStatus};

/// Spawns the given command and waits for it to finish.
pub fn execute_mut(cmd: &mut Command) -> io::Result<ExitStatus> {
    cmd.spawn()?.wait()
}
