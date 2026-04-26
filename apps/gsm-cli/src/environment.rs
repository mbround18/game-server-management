use gsm_instance::config::LaunchMode;
use std::env;
use std::path::PathBuf;

pub fn name() -> String {
    gsm_shared::fetch_var("NAME", "Generic Steam Server")
}

pub fn app_id() -> Option<u32> {
    env::var("APP_ID")
        .ok()
        .and_then(|value| value.trim().parse::<u32>().ok())
}

pub fn install_path() -> Option<PathBuf> {
    env::var("INSTALL_PATH")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
}

pub fn executable() -> Option<String> {
    first_non_empty(["EXECUTABLE", "COMMAND"])
}

pub fn launch_mode() -> Option<LaunchMode> {
    env::var("LAUNCH_MODE")
        .ok()
        .as_deref()
        .and_then(parse_launch_mode)
}

pub fn force_windows() -> bool {
    gsm_shared::is_env_var_truthy("FORCE_WINDOWS")
}

pub fn install_args() -> Vec<String> {
    split_shell_like_values("INSTALL_ARGS")
}

pub fn launch_args() -> Vec<String> {
    split_shell_like_values("LAUNCH_ARGS")
}

fn first_non_empty<const N: usize>(keys: [&str; N]) -> Option<String> {
    keys.into_iter().find_map(|key| {
        env::var(key)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

fn split_shell_like_values(key: &str) -> Vec<String> {
    env::var(key)
        .ok()
        .map(|value| {
            value
                .split_whitespace()
                .map(str::trim)
                .filter(|part| !part.is_empty())
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

pub fn parse_launch_mode(value: &str) -> Option<LaunchMode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "native" => Some(LaunchMode::Native),
        "wine" => Some(LaunchMode::Wine),
        "proton" => Some(LaunchMode::Proton),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn parse_launch_mode_accepts_supported_values() {
        assert!(matches!(
            parse_launch_mode("native"),
            Some(LaunchMode::Native)
        ));
        assert!(matches!(parse_launch_mode("wine"), Some(LaunchMode::Wine)));
        assert!(matches!(
            parse_launch_mode("proton"),
            Some(LaunchMode::Proton)
        ));
    }

    #[test]
    fn install_args_reads_split_env_values() {
        let _lock = env_lock().lock().unwrap_or_else(|error| error.into_inner());

        unsafe {
            env::set_var("INSTALL_ARGS", "+beta staging");
        }

        assert_eq!(install_args(), vec!["+beta", "staging"]);

        unsafe {
            env::remove_var("INSTALL_ARGS");
        }
    }
}
