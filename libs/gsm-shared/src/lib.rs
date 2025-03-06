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
    environment::fetch_var(
        constants::WORKING_DIR,
        env::current_dir().unwrap().to_str().unwrap(),
    )
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
    String::from(
        url.path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or(default),
    )
}

pub fn get_md5_hash(context: &str) -> String {
    format!("{:x}", md5::compute(context.as_bytes()))
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

pub fn url_parse_file_type(url: &str) -> String {
    url.split('.').last().unwrap().to_string()
}
