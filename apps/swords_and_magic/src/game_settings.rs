use gsm_serde::serde_ini::{IniHeader, from_str, to_string};
use ini_derive::IniSerialize;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;

/// Represents game settings in the server configuration.
#[derive(Serialize, Deserialize, Debug, Clone, IniSerialize)]
#[INIHeader(name = "/Game/Blueprints/GameModes/BP_GameInstance.BP_GameInstance_C")]
pub struct GameSettings {
    #[serde(rename = "DedSrv_OwnerSteamID")]
    pub owner_steam_id: String,

    #[serde(rename = "DedSrv_MaxPlayers")]
    pub max_players: u32,

    #[serde(rename = "DedSrv_DaysUntilReset")]
    pub days_until_reset: u32,

    #[serde(rename = "DedSrv_ResetTimeOfDay")]
    pub reset_time_of_day: String,

    #[serde(rename = "DedSrv_ResetWarningsInMinutes")]
    pub reset_warnings_in_minutes: Vec<u32>,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            owner_steam_id: env::var("SNM_OWNER_STEAM_ID")
                .unwrap_or_else(|_| "00000000000000000".to_string()),
            max_players: env::var("SNM_MAX_PLAYERS")
                .unwrap_or_else(|_| "16".to_string())
                .parse()
                .unwrap_or(16),
            days_until_reset: env::var("SNM_DAYS_UNTIL_RESET")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .unwrap_or(1),
            reset_time_of_day: env::var("SNM_RESET_TIME_OF_DAY")
                .unwrap_or_else(|_| "00:00".to_string()),
            reset_warnings_in_minutes: vec![60, 30, 5, 1],
        }
    }
}

/// Loads the configuration from a file or creates a new one with defaults.
pub fn load_or_create_config(path: &Path) -> GameSettings {
    if path.exists() {
        match fs::read_to_string(path) {
            Ok(contents) => from_str::<GameSettings>(&contents).unwrap_or_else(|_| {
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
        let default_config = GameSettings::default();
        save_config(path, &default_config);
        default_config
    }
}

/// Saves the configuration to a file.
pub fn save_config(path: &Path, config: &GameSettings) {
    match to_string(config) {
        Ok(ini) => {
            if let Err(e) = fs::write(path, ini) {
                eprintln!("Failed to write config file: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to serialize config: {}", e);
        }
    }
}
