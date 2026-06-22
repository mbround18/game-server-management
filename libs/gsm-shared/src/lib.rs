//! # gsm-shared
//!
//! Common utilities shared across the Game Server Management (GSM) workspace.
//!
//! This crate provides:
//!
//! - **Environment helpers** ([`fetch_var`], [`fetch_multiple_var`], [`is_env_var_truthy`]) for
//!   reading and interpreting environment variables.
//! - **Path utilities** ([`normalize_paths`], [`get_working_dir`]) for working with file-system
//!   paths in a cross-platform way.
//! - **Network helpers** ([`fetch_public_address`], [`is_valid_url`]) for resolving the server's
//!   public IP address and validating URLs.
//! - **String utilities** ([`parse_truthy`], [`get_md5_hash`], [`parse_file_name`],
//!   [`url_parse_file_type`]) for common string transformations.
#![warn(missing_docs)]

mod fetch_public_ip_address;

pub use fetch_public_ip_address::*;
use reqwest::Url;
use std::env;
use std::path::Path;
use tracing::debug;

mod is_valid_url;
pub use is_valid_url::*;

mod normalize_paths;
pub use normalize_paths::*;

mod parse_truthy;
pub use parse_truthy::*;

mod environment;
pub use environment::*;

mod constants;

/// Returns the configured working directory for the game server.
///
/// Reads the `WORKING_DIR` environment variable and falls back to the
/// compile-time default ([`constants::WORKING_DIR`]) when the variable is absent or empty.
pub fn get_working_dir() -> String {
    environment::fetch_var(
        constants::WORKING_DIR,
        env::current_dir().unwrap().to_str().unwrap(),
    )
}

/// Returns `true` if the file-system path `path` exists.
///
/// Logs a debug message indicating whether the path was found.
pub fn path_exists(path: &str) -> bool {
    let state = Path::new(path).exists();
    debug!(
        "Path {} {}",
        path,
        if state { "exists" } else { "does not exist" }
    );
    state
}

/// Extracts the last path segment from a URL to use as a file name.
///
/// Falls back to `default` when the URL has no path segments or the last segment is empty.
pub fn parse_file_name(url: &Url, default: &str) -> String {
    String::from(
        url.path_segments()
            .and_then(|mut segments| segments.next_back())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or(default),
    )
}

/// Computes the MD5 hash of `context` and returns it as a lowercase hex string.
pub fn get_md5_hash(context: &str) -> String {
    format!("{:x}", md5::compute(context.as_bytes()))
}

/// Returns the file extension of a URL by extracting the last `.`-delimited segment.
///
/// # Panics
///
/// Panics if `url` contains no `.` character (i.e. has no extension).
pub fn url_parse_file_type(url: &str) -> String {
    url.split('.').next_back().unwrap().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_str() {
        assert_eq!(
            get_md5_hash("abcdefghijklmnopqrstuvwxyz"),
            "c3fcd3d76192e4007dfb496cca67e13b"
        );
    }
}
