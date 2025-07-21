use serde::{Serialize, de::DeserializeOwned};
use std::fs;
use std::path::Path;

pub fn load_config_with_defaults<T>(path: &Path) -> T
where
    T: DeserializeOwned + Default,
{
    if path.exists() {
        match fs::read_to_string(path) {
            Ok(contents) => serde_json::from_str::<T>(&contents).unwrap_or_else(|_| T::default()),
            Err(_) => T::default(),
        }
    } else {
        T::default()
    }
}

pub fn save_config<T: Serialize>(path: &Path, config: &T) {
    if let Ok(json) = serde_json::to_string_pretty(config) {
        let _ = fs::write(path, json);
    }
}
