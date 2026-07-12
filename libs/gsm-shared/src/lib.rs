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

pub fn get_working_dir() -> String {
    let default_working_dir = env::current_dir()
        .ok().map_or_else(|| String::from("."), |path| path.to_string_lossy().into_owned());
    environment::fetch_var(constants::WORKING_DIR, default_working_dir.as_str())
}

pub fn path_exists(path: &str) -> bool {
    let state = Path::new(path).exists();
    debug!(
        "Path {} {}",
        path,
        if state { "exists" } else { "does not exist" }
    );
    state
}

pub fn parse_file_name(url: &Url, default: &str) -> String {
    url.path_segments()
        .and_then(|mut segments| segments.next_back())
        .filter(|name| !name.is_empty())
        .map_or_else(|| default.to_owned(), std::borrow::ToOwned::to_owned)
}

pub fn get_md5_hash(context: &str) -> String {
    format!("{:x}", md5::compute(context.as_bytes()))
}

pub fn url_parse_file_type(url: &str) -> String {
    url.rsplit('.')
        .next()
        .filter(|part| !part.is_empty())
        .map_or_else(String::new, std::borrow::ToOwned::to_owned)
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
