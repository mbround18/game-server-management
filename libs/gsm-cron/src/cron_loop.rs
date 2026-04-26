//! # Cron Loop
//!
//! This module provides the main event loop for the cron scheduler.
use std::time::Duration;
use tokio::time::sleep;

/// Begins the main cron loop, which runs indefinitely.
///
/// This function starts an infinite loop that can be used to keep a program running
/// while scheduled cron jobs execute in the background. The loop currently sleeps for
/// 60 seconds at a time, but this can be adapted for other purposes, such as handling
/// signals for graceful shutdown.
///
/// In a typical application, you would spawn your cron jobs using `spawn_scheduled_job`
/// or `register_job`, and then call this function to keep the main thread alive.
///
/// # Example
///
/// ```rust,no_run
/// use gsm_cron::{register_job, begin_cron_loop};
///
/// async fn main() {
///     // Register a job to run every minute.
///     register_job("heartbeat", "* * * * *", || {
///         println!("Cron loop is alive!");
///     });
///
///     // Start the cron loop to keep the application running.
///     begin_cron_loop().await;
/// }
/// ```
pub async fn begin_cron_loop() {
    loop {
        tokio::select! {
            _ = sleep(Duration::from_secs(60)) => {
                // Normal tick every 60 seconds. This loop can be used to
                // integrate with signal handling for graceful shutdown.
            }
        }
    }
}
