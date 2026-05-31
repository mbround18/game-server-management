//! # gsm-monitor
//!
//! Real-time log-file monitoring for GSM game server instances.
//!
//! This crate continuously tails server log files and dispatches each new line through a
//! configurable set of [`LogRules`]. Rules can match on arbitrary patterns and execute
//! any action (e.g. emitting a structured log event or triggering a notification).
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use std::path::PathBuf;
//! use gsm_monitor::{LogRules, start_instance_log_monitor};
//!
//! // Use the default rule set (warnings and errors are highlighted automatically).
//! let rules = LogRules::default();
//!
//! // Start background threads that tail `<working_dir>/logs/server.log` and
//! // `<working_dir>/logs/server.err`.
//! start_instance_log_monitor(PathBuf::from("/home/steam/server"), rules);
//! ```
#![warn(missing_docs)]

mod constants;
mod monitor;
mod rules;

pub use monitor::{Monitor, start_instance_log_monitor, start_monitor_in_thread};
pub use rules::{LogRule, LogRules};
