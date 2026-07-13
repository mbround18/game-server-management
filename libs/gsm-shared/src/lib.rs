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
    let default_working_dir = env::current_dir().ok().map_or_else(
        || String::from("."),
        |path| path.to_string_lossy().into_owned(),
    );
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
    use reqwest::Url;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn hash_str() {
        assert_eq!(
            get_md5_hash("abcdefghijklmnopqrstuvwxyz"),
            "c3fcd3d76192e4007dfb496cca67e13b"
        );
    }

    #[test]
    fn working_dir_prefers_environment_override() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        unsafe {
            std::env::set_var(constants::WORKING_DIR, temp_dir.path());
        }

        assert_eq!(
            get_working_dir(),
            temp_dir.path().to_string_lossy().into_owned()
        );

        unsafe {
            std::env::remove_var(constants::WORKING_DIR);
        }

        Ok(())
    }

    #[test]
    fn path_helpers_cover_basic_cases() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("file.txt");
        fs::write(&file_path, "hello")?;

        let file_path = file_path.to_string_lossy();
        assert!(path_exists(&file_path));
        assert!(!path_exists(
            temp_dir.path().join("missing.txt").to_string_lossy().as_ref()
        ));

        let url = Url::parse("https://example.com/path/to/archive.tar.gz")?;
        assert_eq!(parse_file_name(&url, "default.txt"), "archive.tar.gz");
        assert_eq!(
            parse_file_name(&Url::parse("https://example.com/")?, "default.txt"),
            "default.txt"
        );
        assert_eq!(url_parse_file_type("archive.tar.gz"), "gz");
        assert_eq!(url_parse_file_type("no_extension"), "no_extension");

        Ok(())
    }
}
