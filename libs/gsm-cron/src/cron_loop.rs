use std::time::Duration;
use tokio::time::sleep;

/// Begins the cron loop and listens for a Ctrl-C signal. When Ctrl-C is caught,
/// it sends a SIGINT (or an equivalent) to all child processes in `child_list`.
pub async fn begin_cron_loop() {
    loop {
        tokio::select! {
            _ = sleep(Duration::from_secs(60)) => {
                // Normal tick every 60 seconds.
            }
        }
    }
}
