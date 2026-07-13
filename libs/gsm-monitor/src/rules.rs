//! This module provides the functionality for defining and managing log rules.
//!
//! A log rule consists of a matcher and an action. The matcher determines if a given log line should trigger
//! the associated action. Log rules are stored and processed in order of their ranking.

use crate::constants::INSTANCE_TARGET;
use std::sync::PoisonError;
use std::sync::{Arc, RwLock};
use tracing::{error, info, trace, warn};

/// The default ranking value for log rules.
pub static DEFAULT_STOP_INT: i32 = 99999;

/// A matcher function that determines whether a log line should trigger a rule.
/// It takes a string slice and returns a boolean.
pub type Matcher = Arc<dyn Fn(&str) -> bool + Send + Sync>;

/// An action function that is executed when its corresponding matcher returns true.
/// It takes a string slice as input.
pub type Action = Arc<dyn Fn(&str) + Send + Sync>;

/// Computes the default ranking for a new rule based on the current count of rules.
fn default_ranking(current_count: usize) -> i32 {
    trace!("Computing default ranking for rule count: {current_count}");
    current_count as i32 - DEFAULT_STOP_INT
}

#[derive(Clone)]
pub struct LogRule {
    pub matcher: Matcher,
    pub action: Action,
    pub ranking: i32,
    pub stop: bool,
}

impl Default for LogRule {
    fn default() -> Self {
        trace!("Creating default LogRule");
        Self {
            matcher: Arc::new(|_| true),
            action: Arc::new(|line| info!(target: INSTANCE_TARGET, "{line}")),
            ranking: DEFAULT_STOP_INT,
            stop: true,
        }
    }
}

impl LogRule {
    pub fn new() -> Self {
        trace!("Creating new LogRule");
        Self::default()
    }
}

#[derive(Clone)]
pub struct LogRules {
    rules: Arc<RwLock<Vec<LogRule>>>,
}

impl LogRules {
    pub fn new() -> Self {
        trace!("Initializing LogRules");
        Self {
            rules: Arc::new(RwLock::new(vec![LogRule::default()])),
        }
    }

    pub fn add_rule<F, G>(&self, matcher: F, action: G, stop: bool, ranking: Option<i32>)
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
        G: Fn(&str) + Send + Sync + 'static,
    {
        trace!("Adding new rule with stop flag: {stop}");
        let mut rules = self.rules.write().unwrap_or_else(PoisonError::into_inner);
        let mut rule = LogRule::new();
        rule.stop = stop;
        rule.matcher = Arc::new(matcher);
        rule.action = Arc::new(action);
        rule.ranking = ranking.unwrap_or_else(|| default_ranking(rules.len()));
        rules.push(rule);
    }

    pub fn get_rules(&self) -> Vec<LogRule> {
        trace!("Retrieving and sorting rules");
        let mut rules = self
            .rules
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        rules.sort_by_key(|r| r.ranking);
        trace!("Sorted rules count: {}", rules.len());
        rules
    }
}

impl Default for LogRules {
    fn default() -> Self {
        trace!("Creating default LogRules instance");
        let rules = Self::new();
        rules.add_rule(
            |line| line.contains("WARNING"),
            |line| warn!(target: INSTANCE_TARGET, "{}", line),
            true,
            None,
        );
        rules.add_rule(
            |line| line.contains("ERROR"),
            |line| error!(target: INSTANCE_TARGET, "{}", line),
            true,
            None,
        );
        rules
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_rule_matches_everything_and_stops() {
        let rule = LogRule::default();
        assert!(rule.stop);
        assert_eq!(rule.ranking, DEFAULT_STOP_INT);
        assert!((rule.matcher)("any line"));
    }

    #[test]
    fn rules_are_sorted_by_ranking() {
        let rules = LogRules::new();
        rules.add_rule(|_| true, |_| {}, false, Some(20));
        rules.add_rule(|_| true, |_| {}, false, Some(5));

        let rankings: Vec<i32> = rules
            .get_rules()
            .into_iter()
            .map(|rule| rule.ranking)
            .collect();
        assert_eq!(rankings, vec![5, 20, DEFAULT_STOP_INT]);
    }

    #[test]
    fn default_rules_include_warning_and_error_handlers() {
        let rules = LogRules::default();
        let mut matched = Vec::new();
        for rule in rules.get_rules() {
            if (rule.matcher)("WARNING: example") || (rule.matcher)("ERROR: example") {
                matched.push(rule.ranking);
            }
        }

        assert!(!matched.is_empty());
    }
}
