use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;
use gsm_serde::serde_ini::{IniHeader, to_string};

/// Represents game settings in the server configuration.
#[derive(Serialize, Deserialize, Debug, Clone, IniSerialize)]
#[INIHeader(name = "/Game/Blueprints/GameModes/BP_GameInstance.BP_GameInstance_C")]
pub struct GameSettings {
    #[serde(rename = "ded_srv_owner_steam_id")]
    pub ded_srv_owner_steam_id: String,
    #[serde(rename = "ded_srv_max_players")]
    pub ded_srv_max_players: u32,
    #[serde(rename = "ded_srv_days_until_reset")]
    pub ded_srv_days_until_reset: u32,
    #[serde(rename = "ded_srv_reset_time_of_day")]
    pub ded_srv_reset_time_of_day: String,
    #[serde(rename = "ded_srv_reset_warnings_in_minutes")]
    pub ded_srv_reset_warnings_in_minutes: Vec<u32>,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            ded_srv_owner_steam_id: env::var("SNM_OWNER_STEAM_ID").unwrap_or_else(|_| "00000000000000000".to_string()),
            ded_srv_max_players: env::var("SNM_MAX_PLAYERS").unwrap_or_else(|_| "16".to_string()).parse().unwrap_or(16),
            ded_srv_days_until_reset: env::var("SNM_DAYS_UNTIL_RESET").unwrap_or_else(|_| "1".to_string()).parse().unwrap_or(1),
            ded_srv_reset_time_of_day: env::var("SNM_RESET_TIME_OF_DAY").unwrap_or_else(|_| "00:00".to_string()),
            ded_srv_reset_warnings_in_minutes: vec![60, 30, 5, 1],
        }
    }
}

/// Loads the configuration from a file or creates a new one with defaults.
pub fn load_or_create_config(path: &Path) -> GameSettings {
    if path.exists() {
        match fs::read_to_string(path) {
            Ok(contents) => serde_ini::from_str::<GameSettings>(&contents).unwrap_or_else(|_| {
                eprintln!("Warning: Corrupt config file detected. Using defaults.");
                GameSettings::default()
            }),
            Err(_) => {
                eprintln!("Failed to read config file. Using defaults.");
                GameSettings::default()
            }
        }
    } else {
        eprintln!("Config file not found. Creating default config.");
        GameSettings::default()
    }
}

/// Saves the configuration to a file.
pub fn save_config(path: &Path, config: &GameSettings) {
    if let Ok(ini) = to_string(config) {
        let _ = fs::write(path, ini);
    }
}
