//! # Game Server Cron Job Scheduler
//!
//! This crate provides a simple, asynchronous cron-like job scheduler for the Game Server Management (GSM) workspace.
//! It is designed to run tasks at specified intervals, such as automated server updates, backups, or restarts.
//!
//! The crate uses the `cron` and `tokio` crates to provide a flexible and efficient scheduling mechanism.
//! It supports standard cron expressions for scheduling jobs.
mod cron_loop;

use chrono::Utc;
use cron::Schedule;
use std::str::FromStr;
use tokio::time::{Duration, sleep};
use tracing::{debug, error, info};

pub use cron_loop::begin_cron_loop;

/// Spawns a job to run on a cron-like schedule asynchronously.
///
/// This function takes a cron schedule string and a closure, and spawns a `tokio` task
/// to execute the closure at the specified times. The schedule is based on UTC.
///
/// # Arguments
///
/// * `schedule_str`: A string representing the cron schedule (e.g., "0 0 * * * *").
/// * `job`: A closure that will be executed when the schedule is met. The closure must be
///   `Send`, `Sync`, and have a `'static` lifetime.
///
/// # Panics
///
/// This function does not panic, but it will log an error if the schedule string is invalid.
///
/// # Example
///
/// ```rust,no_run
/// use gsm_cron::spawn_scheduled_job;
///
/// // Schedule a job to run every minute.
/// spawn_scheduled_job("0 * * * * *".to_string(), || {
///     println!("This job runs every minute!");
/// });
/// ```
pub fn spawn_scheduled_job(schedule_str: String, job: impl Fn() + Send + Sync + 'static) {
    debug!("Attempting to parse schedule: {}", schedule_str);
    let schedule = match Schedule::from_str(&schedule_str) {
        Ok(s) => {
            debug!("Schedule parsed successfully: {:?}", s);
            s
        }
        Err(e) => {
            error!("Invalid cron schedule '{}': {}", schedule_str, e);
            return;
        }
    };

    tokio::spawn(async move {
        for datetime in schedule.upcoming(Utc) {
            let now = Utc::now();
            let wait_time = (datetime - now).to_std().unwrap_or(Duration::ZERO);
            sleep(wait_time).await;
            debug!(
                "Woke up at: {:?} for scheduled time: {:?}",
                Utc::now(),
                datetime
            );
            job();
        }
    });
}

/// A helper function to register a job with a name and a cron schedule.
///
/// This function simplifies the process of scheduling a job by providing a name for logging
/// purposes and handling 5-field cron expressions (by prepending a "0" for seconds).
///
/// # Arguments
///
/// * `name`: A name for the job, used for logging.
/// * `schedule`: The cron schedule string. This can be a standard 6-field cron expression
///   (including seconds) or a 5-field expression (which will be adapted).
/// * `job`: The closure to execute.
///
/// # Example
///
/// ```rust,no_run
/// use gsm_cron::register_job;
///
/// // Register a daily backup job.
/// register_job("daily-backup", "0 0 0 * * *", || {
///     println!("Running daily backup...");
/// });
///
/// // Register a job with a 5-field schedule (runs every minute).
/// register_job("minute-ping", "* * * * *", || {
///     println!("Pinging server...");
/// });
/// ```
pub fn register_job<F>(name: &str, schedule: &str, job: F)
where
    F: Fn() + Send + Sync + 'static,
{
    let name_owned = name.to_owned();
    let field_count = schedule.split_whitespace().count();
    let adjusted_schedule = if field_count == 5 {
        let new_schedule = format!("0 {schedule}");
        debug!(
            "Adjusted schedule from 5-field to 6-field for job '{}': {} (original: {})",
            name_owned, new_schedule, schedule
        );
        new_schedule
    } else {
        debug!(
            "Schedule for job '{}' is already 6-field: {}",
            name_owned, schedule
        );
        schedule.to_string()
    };

    info!(
        "Registering job '{}' with schedule: {}",
        name_owned, adjusted_schedule
    );

    spawn_scheduled_job(adjusted_schedule, move || {
        info!("Executing job: {}", name_owned);
        job();
    });
}
