//! This module provides the functionality for defining and managing log rules.
//!
//! A log rule consists of a matcher and an action. The matcher determines if a given log line should trigger
//! the associated action. Log rules are stored and processed in order of their ranking.

use std::sync::{Arc, RwLock};
use tracing::{error, info, warn};

/// The default ranking value for log rules.
pub static DEFAULT_STOP_INT: i32 = 99999;

/// The log target for tracing.
pub static LOG_TARGET: &str = "instance";

/// A matcher function that determines whether a log line should trigger a rule.
/// It takes a string slice and returns a boolean.
pub type Matcher = Arc<dyn Fn(&str) -> bool + Send + Sync>;

/// An action function that is executed when its corresponding matcher returns true.
/// It takes a string slice as input.
pub type Action = Arc<dyn Fn(&str) + Send + Sync>;

/// Computes the default ranking for a new rule based on the current count of rules.
///
/// # Arguments
///
/// * `current_count` - The number of rules currently stored.
///
/// # Returns
///
/// An `i32` representing the default ranking. New rules get a ranking relative to the current count.
fn default_ranking(current_count: usize) -> i32 {
    current_count as i32 - DEFAULT_STOP_INT
}

/// Represents a single log rule consisting of a matcher, an action, a ranking, and a stop flag.
///
/// The `stop` flag indicates whether processing of further rules should be halted if this rule matches.
#[derive(Clone)]
pub struct LogRule {
    /// The matcher function.
    pub matcher: Matcher,
    /// The action function to execute when the matcher returns true.
    pub action: Action,
    /// The ranking determines the order in which rules are evaluated.
    pub ranking: i32,
    /// If true, no further rules are processed after this rule matches.
    pub stop: bool,
}

impl Default for LogRule {
    /// Creates a default `LogRule` with a matcher that always returns true, an action that logs the line,
    /// a default ranking, and a stop flag set to true.
    fn default() -> Self {
        LogRule {
            matcher: Arc::new(|_| true),
            action: Arc::new(|line| info!(target: LOG_TARGET, "{}", line)),
            ranking: DEFAULT_STOP_INT,
            stop: true,
        }
    }
}

impl LogRule {
    /// Creates a new log rule using the default settings.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Manages a collection of log rules.
///
/// Log rules are stored internally in a thread-safe vector and can be added or retrieved in a sorted order.
#[derive(Clone)]
pub struct LogRules {
    rules: Arc<RwLock<Vec<LogRule>>>,
}

impl LogRules {
    /// Creates a new, empty instance of `LogRules`.
    pub fn new() -> Self {
        Self {
            rules: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Adds a new log rule.
    ///
    /// # Arguments
    ///
    /// * `matcher` - A function that determines if a log line should trigger the rule.
    /// * `action` - A function that is executed when the matcher returns true.
    /// * `continue_processing` - If false, stops processing further rules once this rule matches.
    /// * `ranking` - Optional ranking (0 to 99999) that determines the order in which rules are processed.
    ///
    pub fn add_rule<F, G>(&self, matcher: F, action: G, stop: bool, ranking: Option<i32>)
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
        G: Fn(&str) + Send + Sync + 'static,
    {
        let mut rules = self.rules.write().unwrap();
        let mut rule = LogRule::new();
        rule.stop = stop;
        rule.matcher = Arc::new(matcher);
        rule.action = Arc::new(action);
        rule.ranking = ranking.unwrap_or_else(|| default_ranking(rules.len()));
        rules.push(rule);
    }

    /// Retrieves all log rules, sorted by their ranking.
    ///
    /// # Returns
    ///
    /// A vector of `LogRule` instances sorted in ascending order by ranking.
    pub fn get_rules(&self) -> Vec<LogRule> {
        let mut rules = self.rules.read().unwrap().clone();
        rules.sort_by_key(|r| r.ranking);
        rules
    }
}

impl Default for LogRules {
    /// Creates a default `LogRules` instance with pre-defined WARNING and ERROR rules.
    fn default() -> Self {
        let rules = Self::new();
        rules.add_rule(
            |line| line.contains("WARNING"),
            |line| warn!("{}", line),
            true,
            None,
        );
        rules.add_rule(
            |line| line.contains("ERROR"),
            |line| error!("{}", line),
            true,
            None,
        );
        rules
    }
}
