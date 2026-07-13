use regex::Regex;
use std::sync::LazyLock;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joined_extracts_name_from_log_line() {
        let log = "[server] Player 'mbround18' logged in with Permissions:";
        assert_eq!(
            extract_player_joined_name(log),
            Some("mbround18".to_owned())
        );
    }

    #[test]
    fn joined_handles_names_with_spaces_and_symbols() {
        let log = "Player 'Cool Player_123' logged in";
        assert_eq!(
            extract_player_joined_name(log),
            Some("Cool Player_123".to_owned())
        );
    }

    #[test]
    fn joined_returns_none_when_pattern_absent() {
        assert_eq!(extract_player_joined_name("[server] Some other log line"), None);
        assert_eq!(extract_player_joined_name(""), None);
    }

    #[test]
    fn left_extracts_name_from_log_line() {
        let log = "[server] Remove Entity for Player 'mbround18'";
        assert_eq!(
            extract_player_left_name(log),
            Some("mbround18".to_owned())
        );
    }

    #[test]
    fn left_handles_names_with_spaces() {
        let log = "Remove Entity for Player 'Cool Player'";
        assert_eq!(
            extract_player_left_name(log),
            Some("Cool Player".to_owned())
        );
    }

    #[test]
    fn left_returns_none_when_pattern_absent() {
        assert_eq!(extract_player_left_name("[server] Server started."), None);
        assert_eq!(extract_player_left_name(""), None);
    }
}

#[allow(clippy::expect_used)]
static JOINED_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Player\s+'([^']+)'").expect("joined-player regex should compile")
});

#[allow(clippy::expect_used)]
static LEFT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Remove Entity for Player\s+'([^']+)'")
        .expect("left-player regex should compile")
});

/// Extracts the player name from a log line.
///
/// The log line is expected to contain a player name wrapped in single quotes,
/// e.g., "[server] Player 'mbround18' logged in with Permissions:".
pub fn extract_player_joined_name(log: &str) -> Option<String> {
    JOINED_RE
        .captures(log)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_owned()))
}

/// Extracts the player name from a log line when a player leaves.
///
/// The log line is expected to follow the format:
/// `[server] Remove Player 'mbround18'`
pub fn extract_player_left_name(log: &str) -> Option<String> {
    LEFT_RE
        .captures(log)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_owned()))
}
