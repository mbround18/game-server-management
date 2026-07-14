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

fn normalize_schedule(schedule: &str) -> String {
    let field_count = schedule.split_whitespace().count();
    if field_count == 5 {
        format!("0 {schedule}")
    } else {
        schedule.to_owned()
    }
}

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
/// spawn_scheduled_job("0 * * * * *", || {
///     println!("This job runs every minute!");
/// });
/// ```
pub fn spawn_scheduled_job(schedule_str: &str, job: impl Fn() + Send + Sync + 'static) {
    debug!("Attempting to parse schedule: {}", schedule_str);
    let schedule = match Schedule::from_str(schedule_str) {
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
    let adjusted_schedule = normalize_schedule(schedule);
    if schedule.split_whitespace().count() == 5 {
        debug!(
            "Adjusted schedule from 5-field to 6-field for job '{}': {} (original: {})",
            name_owned, adjusted_schedule, schedule
        );
    } else {
        debug!(
            "Schedule for job '{}' is already 6-field: {}",
            name_owned, schedule
        );
    }

    info!(
        "Registering job '{}' with schedule: {}",
        name_owned, adjusted_schedule
    );

    spawn_scheduled_job(&adjusted_schedule, move || {
        info!("Executing job: {}", name_owned);
        job();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_schedule_prepends_seconds_field_for_five_part_cron() {
        assert_eq!(normalize_schedule("* * * * *"), "0 * * * * *");
    }

    #[test]
    fn normalize_schedule_leaves_six_part_cron_unchanged() {
        assert_eq!(normalize_schedule("0 * * * * *"), "0 * * * * *");
    }

    #[tokio::test]
    async fn spawn_scheduled_job_with_invalid_schedule_does_not_panic() {
        // Invalid schedule must be silently rejected (error logged, no panic).
        spawn_scheduled_job("not-a-cron-expression", || {});
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    #[tokio::test]
    async fn register_job_accepts_six_field_schedule_without_panic() {
        register_job("test-6field", "0 59 23 31 12 *", || {});
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    #[tokio::test]
    async fn register_job_adjusts_five_field_schedule_without_panic() {
        register_job("test-5field", "59 23 31 12 *", || {});
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    #[tokio::test]
    async fn register_job_with_invalid_schedule_does_not_panic() {
        register_job("test-invalid", "garbage schedule", || {});
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
}
