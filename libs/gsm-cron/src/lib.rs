mod cron_loop;

use chrono::Utc;
use cron::Schedule;
use std::str::FromStr;
use tokio::time::{Duration, sleep};
use tracing::{debug, error, info};

pub use cron_loop::begin_cron_loop;

/// Spawns a job using cron-like scheduling asynchronously
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

/// A simple helper to register a job with a name and schedule.
pub fn register_job<F>(name: &str, schedule: &str, job: F)
where
    F: Fn() + Send + Sync + 'static,
{
    let name_owned = name.to_owned();
    let field_count = schedule.split_whitespace().count();
    let adjusted_schedule = if field_count == 5 {
        let new_schedule = format!("0 {}", schedule);
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
