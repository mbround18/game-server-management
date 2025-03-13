use gsm_serde::serde_ini::{IniHeader, to_string};
use ini_derive::IniSerialize;
use serde::{Deserialize, Serialize};
use std::fs::create_dir_all;
use std::path::Path;
use std::{env, fs};

macro_rules! env_parse {
    ($env_var:expr, $default:expr, $t:ty) => {
        std::env::var($env_var)
            .ok()
            .and_then(|s| s.parse::<$t>().ok())
            .unwrap_or($default)
    };
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Preset {
    Casual,
    Normal,
    Hard,
}

#[derive(Debug, Clone, Serialize, Deserialize, IniSerialize, Default)]
#[INIHeader(name = "/Script/Pal.PalGameWorldSettings")]
pub struct Settings {
    #[serde(rename = "OptionSettings")]
    option_settings: GameSettings,
}

/// Represents game settings for Palworld server.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameSettings {
    #[serde(rename = "Difficulty")]
    pub difficulty: String,

    #[serde(rename = "DayTimeSpeedRate")]
    pub day_time_speed_rate: f32,

    #[serde(rename = "NightTimeSpeedRate")]
    pub night_time_speed_rate: f32,

    #[serde(rename = "ExpRate")]
    pub exp_rate: f32,

    #[serde(rename = "PalCaptureRate")]
    pub pal_capture_rate: f32,

    #[serde(rename = "PalSpawnNumRate")]
    pub pal_spawn_num_rate: f32,

    #[serde(rename = "PalDamageRateAttack")]
    pub pal_damage_rate_attack: f32,

    #[serde(rename = "PalDamageRateDefense")]
    pub pal_damage_rate_defense: f32,

    #[serde(rename = "PlayerDamageRateAttack")]
    pub player_damage_rate_attack: f32,

    #[serde(rename = "PlayerDamageRateDefense")]
    pub player_damage_rate_defense: f32,

    #[serde(rename = "PlayerStomachDecreaseRate")]
    pub player_stomach_decrease_rate: f32,

    #[serde(rename = "PlayerStaminaDecreaseRate")]
    pub player_stamina_decrease_rate: f32,

    #[serde(rename = "PlayerAutoHPRegeneRate")]
    pub player_auto_hp_regen_rate: f32,

    #[serde(rename = "PlayerAutoHpRegeneRateInSleep")]
    pub player_auto_hp_regen_rate_in_sleep: f32,

    #[serde(rename = "PalAutoHPRegeneRate")]
    pub pal_auto_hp_regen_rate: f32,

    #[serde(rename = "PalAutoHpRegeneRateInSleep")]
    pub pal_auto_hp_regen_rate_in_sleep: f32,

    #[serde(rename = "BuildObjectDamageRate")]
    pub build_object_damage_rate: f32,

    #[serde(rename = "BuildObjectDeteriorationDamageRate")]
    pub build_object_deterioration_damage_rate: f32,

    #[serde(rename = "CollectionDropRate")]
    pub collection_drop_rate: f32,

    #[serde(rename = "CollectionObjectHpRate")]
    pub collection_object_hp_rate: f32,

    #[serde(rename = "CollectionObjectRespawnSpeedRate")]
    pub collection_object_respawn_speed_rate: f32,

    #[serde(rename = "EnemyDropItemRate")]
    pub enemy_drop_item_rate: f32,

    #[serde(rename = "DeathPenalty")]
    pub death_penalty: String,

    #[serde(rename = "bEnablePvP")]
    pub enable_pvp: bool,

    #[serde(rename = "bEnableFriendlyFire")]
    pub enable_friendly_fire: bool,

    #[serde(rename = "bEnableInvaderEnemy")]
    pub enable_invader_enemy: bool,

    #[serde(rename = "bEnableAimAssistPad")]
    pub enable_aim_assist_pad: bool,

    #[serde(rename = "bEnableAimAssistKeyboard")]
    pub enable_aim_assist_keyboard: bool,

    #[serde(rename = "ServerPlayerMaxNum")]
    pub server_player_max_num: u16,

    #[serde(rename = "CoopPlayerMaxNum")]
    pub coop_player_max_num: u16,

    #[serde(rename = "ServerName")]
    pub server_name: String,

    #[serde(rename = "ServerDescription")]
    pub server_description: String,

    #[serde(rename = "AdminPassword")]
    pub admin_password: String,

    #[serde(rename = "ServerPassword")]
    pub server_password: String,

    #[serde(rename = "PublicPort")]
    pub public_port: u16,

    #[serde(rename = "PublicIP")]
    pub public_ip: String,

    #[serde(rename = "RCONEnabled")]
    pub rcon_enabled: bool,

    #[serde(rename = "RCONPort")]
    pub rcon_port: u16,

    #[serde(rename = "bUseAuth")]
    pub use_auth: bool,

    #[serde(rename = "Region")]
    pub region: String,

    #[serde(rename = "BanListURL")]
    pub ban_list_url: String,
}

impl GameSettings {
    /// Constructs the base (Normal preset) configuration.
    pub fn normal() -> Self {
        Self {
            difficulty: "None".to_string(),
            day_time_speed_rate: 1.0,
            night_time_speed_rate: 1.0,
            exp_rate: 1.0,
            pal_capture_rate: 1.0,
            pal_spawn_num_rate: 1.0,
            pal_damage_rate_attack: 1.0,
            pal_damage_rate_defense: 1.0,
            player_damage_rate_attack: 1.0,
            player_damage_rate_defense: 1.0,
            player_stomach_decrease_rate: 1.0,
            player_stamina_decrease_rate: 1.0,
            player_auto_hp_regen_rate: 1.0,
            player_auto_hp_regen_rate_in_sleep: 1.0,
            pal_auto_hp_regen_rate: 1.0,
            pal_auto_hp_regen_rate_in_sleep: 1.0,
            build_object_damage_rate: 1.0,
            build_object_deterioration_damage_rate: 1.0,
            collection_drop_rate: 1.0,
            collection_object_hp_rate: 1.0,
            collection_object_respawn_speed_rate: 1.0,
            enemy_drop_item_rate: 1.0,
            death_penalty: "Drop All Items".to_string(),
            enable_pvp: false,
            enable_friendly_fire: false,
            enable_invader_enemy: true,
            enable_aim_assist_pad: true,
            enable_aim_assist_keyboard: false,
            server_player_max_num: 16,
            coop_player_max_num: 16,
            server_name: "My Palworld Server".to_string(),
            server_description: "".to_string(),
            admin_password: "".to_string(),
            server_password: "".to_string(),
            public_port: 8211,
            public_ip: "0.0.0.0".to_string(),
            rcon_enabled: false,
            rcon_port: 25575,
            use_auth: true,
            region: "".to_string(),
            ban_list_url: "https://api.palworldgame.com/api/banlist.txt".to_string(),
        }
    }

    /// Applies preset-specific overrides.
    pub fn apply_preset(&mut self, preset: Preset) {
        match preset {
            Preset::Casual => {
                self.day_time_speed_rate = 1.0;
                self.night_time_speed_rate = 1.0;
                self.exp_rate = 2.0;
                self.pal_capture_rate = 2.0;
                self.pal_spawn_num_rate = 1.0;
                self.pal_damage_rate_attack = 2.0;
                self.pal_damage_rate_defense = 0.5;
                self.player_damage_rate_attack = 2.0;
                self.player_damage_rate_defense = 0.5;
                self.player_stomach_decrease_rate = 0.3;
                self.player_stamina_decrease_rate = 0.3;
                self.player_auto_hp_regen_rate = 2.0;
                self.player_auto_hp_regen_rate_in_sleep = 2.0;
                self.build_object_damage_rate = 2.0;
                self.build_object_deterioration_damage_rate = 0.2;
                self.collection_drop_rate = 3.0;
                self.collection_object_hp_rate = 0.5;
                self.collection_object_respawn_speed_rate = 0.5;
                self.enemy_drop_item_rate = 2.0;
            }
            Preset::Normal => {
                // Normal preset is our base, so nothing to change.
            }
            Preset::Hard => {
                self.day_time_speed_rate = 1.0;
                self.night_time_speed_rate = 1.0;
                self.exp_rate = 0.5;
                self.pal_capture_rate = 1.0;
                self.pal_spawn_num_rate = 1.0;
                self.pal_damage_rate_attack = 0.5;
                self.pal_damage_rate_defense = 2.0;
                self.player_damage_rate_attack = 0.7;
                self.player_damage_rate_defense = 4.0;
                self.player_stomach_decrease_rate = 1.0;
                self.player_stamina_decrease_rate = 1.0;
                self.player_auto_hp_regen_rate = 0.6;
                self.player_auto_hp_regen_rate_in_sleep = 0.6;
                self.build_object_damage_rate = 0.7;
                self.build_object_deterioration_damage_rate = 1.0;
                self.collection_drop_rate = 0.8;
                self.collection_object_hp_rate = 1.0;
                self.collection_object_respawn_speed_rate = 2.0;
                self.enemy_drop_item_rate = 0.7;
                self.death_penalty = "Drop all Items and all Pals on Team".to_string();
            }
        }
    }
}

impl Default for GameSettings {
    fn default() -> Self {
        // Start with Normal preset as our base.
        let mut settings = Self::normal();

        // If a PRESET env variable is provided, override our base.
        if let Ok(preset_str) = env::var("PRESET") {
            if let Ok(preset) = serde_plain::from_str::<Preset>(&preset_str) {
                settings.apply_preset(preset);
            }
        }

        Self {
            difficulty: env::var("DIFFICULTY").unwrap_or_else(|_| "None".to_string()),
            day_time_speed_rate: env_parse!(
                "DAY_TIME_SPEED_RATE",
                settings.day_time_speed_rate,
                f32
            ),
            night_time_speed_rate: env_parse!(
                "NIGHT_TIME_SPEED_RATE",
                settings.night_time_speed_rate,
                f32
            ),
            exp_rate: env_parse!("EXP_RATE", settings.exp_rate, f32),
            pal_capture_rate: env_parse!("PAL_CAPTURE_RATE", settings.pal_capture_rate, f32),
            pal_spawn_num_rate: env_parse!("PAL_SPAWN_NUM_RATE", settings.pal_spawn_num_rate, f32),
            pal_damage_rate_attack: env_parse!(
                "PAL_DAMAGE_RATE_ATTACK",
                settings.pal_damage_rate_attack,
                f32
            ),
            pal_damage_rate_defense: env_parse!(
                "PAL_DAMAGE_RATE_DEFENSE",
                settings.pal_damage_rate_defense,
                f32
            ),
            player_damage_rate_attack: env_parse!(
                "PLAYER_DAMAGE_RATE_ATTACK",
                settings.player_damage_rate_attack,
                f32
            ),
            player_damage_rate_defense: env_parse!(
                "PLAYER_DAMAGE_RATE_DEFENSE",
                settings.player_damage_rate_defense,
                f32
            ),
            player_stomach_decrease_rate: env_parse!(
                "PLAYER_STOMACH_DECREASE_RATE",
                settings.player_stomach_decrease_rate,
                f32
            ),
            player_stamina_decrease_rate: env_parse!(
                "PLAYER_STAMINA_DECREASE_RATE",
                settings.player_stamina_decrease_rate,
                f32
            ),
            player_auto_hp_regen_rate: env_parse!(
                "PLAYER_AUTO_HP_REGEN_RATE",
                settings.player_auto_hp_regen_rate,
                f32
            ),
            player_auto_hp_regen_rate_in_sleep: env_parse!(
                "PLAYER_AUTO_HP_REGEN_RATE_IN_SLEEP",
                settings.player_auto_hp_regen_rate_in_sleep,
                f32
            ),
            pal_auto_hp_regen_rate: env_parse!(
                "PAL_AUTO_HP_REGEN_RATE",
                settings.pal_auto_hp_regen_rate,
                f32
            ),
            pal_auto_hp_regen_rate_in_sleep: env_parse!(
                "PAL_AUTO_HP_REGEN_RATE_IN_SLEEP",
                settings.pal_auto_hp_regen_rate_in_sleep,
                f32
            ),
            build_object_damage_rate: env_parse!(
                "BUILD_OBJECT_DAMAGE_RATE",
                settings.build_object_damage_rate,
                f32
            ),
            build_object_deterioration_damage_rate: env_parse!(
                "BUILD_OBJECT_DETERIORATION_DAMAGE_RATE",
                settings.build_object_deterioration_damage_rate,
                f32
            ),
            collection_drop_rate: env_parse!(
                "COLLECTION_DROP_RATE",
                settings.collection_drop_rate,
                f32
            ),
            collection_object_hp_rate: env_parse!(
                "COLLECTION_OBJECT_HP_RATE",
                settings.collection_object_hp_rate,
                f32
            ),
            collection_object_respawn_speed_rate: env_parse!(
                "COLLECTION_OBJECT_RESPAWN_SPEED_RATE",
                settings.collection_object_respawn_speed_rate,
                f32
            ),
            enemy_drop_item_rate: env_parse!(
                "ENEMY_DROP_ITEM_RATE",
                settings.enemy_drop_item_rate,
                f32
            ),
            death_penalty: env::var("DEATH_PENALTY")
                .unwrap_or_else(|_| settings.death_penalty.clone()),

            enable_pvp: env_parse!("ENABLE_PVP", settings.enable_pvp, bool),
            enable_friendly_fire: env_parse!(
                "ENABLE_FRIENDLY_FIRE",
                settings.enable_friendly_fire,
                bool
            ),
            enable_invader_enemy: env_parse!(
                "ENABLE_INVADER_ENEMY",
                settings.enable_invader_enemy,
                bool
            ),
            enable_aim_assist_pad: env_parse!(
                "ENABLE_AIM_ASSIST_PAD",
                settings.enable_aim_assist_pad,
                bool
            ),
            enable_aim_assist_keyboard: env_parse!(
                "ENABLE_AIM_ASSIST_KEYBOARD",
                settings.enable_aim_assist_keyboard,
                bool
            ),

            server_player_max_num: env_parse!(
                "SERVER_PLAYER_MAX_NUM",
                settings.server_player_max_num,
                u16
            ),
            coop_player_max_num: env_parse!(
                "COOP_PLAYER_MAX_NUM",
                settings.coop_player_max_num,
                u16
            ),

            server_name: env::var("SERVER_NAME").unwrap_or_else(|_| settings.server_name.clone()),
            server_description: env::var("SERVER_DESCRIPTION")
                .unwrap_or_else(|_| settings.server_description.clone()),
            admin_password: env::var("ADMIN_PASSWORD")
                .unwrap_or_else(|_| settings.admin_password.clone()),
            server_password: env::var("SERVER_PASSWORD")
                .unwrap_or_else(|_| settings.server_password.clone()),

            public_port: env_parse!("PUBLIC_PORT", settings.public_port, u16),
            public_ip: env::var("PUBLIC_IP").unwrap_or_else(|_| settings.public_ip.clone()),

            rcon_enabled: env_parse!("RCON_ENABLED", settings.rcon_enabled, bool),
            rcon_port: env_parse!("RCON_PORT", settings.rcon_port, u16),
            use_auth: env_parse!("USE_AUTH", settings.use_auth, bool),
            region: env_parse!("REGION", settings.region, String),
            ban_list_url: env_parse!("BAN_LIST", settings.ban_list_url, String),
        }
    }
}

/// Saves the configuration to an INI file.
pub fn save_config(path: &Path, settings: &Settings) {
    let ini_config = to_string(&settings).unwrap();

    if let Err(e) = fs::write(path, ini_config) {
        eprintln!("Failed to save config: {}", e);
    }
}

/// Loads the configuration from an INI file or returns defaults if the file is missing.
pub fn load_or_create_config(path: &Path) -> GameSettings {
    if !path.parent().unwrap().exists() {
        create_dir_all(path.parent().unwrap()).unwrap();
    }
    let default_config = Settings::default();
    save_config(path, &default_config);
    default_config.option_settings
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::path::Path;
    use std::sync::Mutex;

    // Define a global mutex to prevent race conditions in tests
    lazy_static::lazy_static! {
        static ref TEST_MUTEX: Mutex<()> = Mutex::new(());
    }

    const TEST_DIR: &str = "./tmp/tests";

    /// Helper function to reset environment variables
    fn clear_env_vars() {
        let vars = [
            "DIFFICULTY",
            "DAY_TIME_SPEED_RATE",
            "NIGHT_TIME_SPEED_RATE",
            "EXP_RATE",
            "PAL_CAPTURE_RATE",
            "PRESET",
            "SERVER_NAME",
        ];
        for var in vars.iter() {
            unsafe { env::remove_var(var) };
        }
    }

    #[test]
    fn test_default_settings() {
        let _lock = TEST_MUTEX.lock().unwrap();

        clear_env_vars();
        let settings = GameSettings::default();

        assert_eq!(settings.day_time_speed_rate, 1.0);
        assert_eq!(settings.night_time_speed_rate, 1.0);
        assert_eq!(settings.exp_rate, 1.0);
        assert_eq!(settings.pal_capture_rate, 1.0);
        assert_eq!(settings.server_name, "My Palworld Server");
    }

    #[test]
    fn test_preset_casual() {
        let _lock = TEST_MUTEX.lock().unwrap();

        clear_env_vars();
        unsafe { env::set_var("PRESET", "casual") };

        let settings = GameSettings::default();
        assert_eq!(settings.exp_rate, 2.0);
        assert_eq!(settings.pal_capture_rate, 2.0);
        assert_eq!(settings.player_damage_rate_attack, 2.0);
        assert_eq!(settings.player_damage_rate_defense, 0.5);
        assert_eq!(settings.collection_drop_rate, 3.0);
    }

    #[test]
    fn test_preset_hard() {
        let _lock = TEST_MUTEX.lock().unwrap();

        clear_env_vars();
        unsafe { env::set_var("PRESET", "hard") };

        let settings = GameSettings::default();
        assert_eq!(settings.exp_rate, 0.5);
        assert_eq!(settings.pal_damage_rate_attack, 0.5);
        assert_eq!(settings.pal_damage_rate_defense, 2.0);
        assert_eq!(settings.player_damage_rate_attack, 0.7);
        assert_eq!(settings.enemy_drop_item_rate, 0.7);
    }

    #[test]
    fn test_env_variable_override() {
        let _lock = TEST_MUTEX.lock().unwrap();

        clear_env_vars();
        unsafe {
            env::set_var("EXP_RATE", "3.5");
            env::set_var("PAL_CAPTURE_RATE", "5.0");
        }

        let settings = GameSettings::default();
        assert_eq!(settings.exp_rate, 3.5);
        assert_eq!(settings.pal_capture_rate, 5.0);
    }

    #[test]
    fn test_preset_with_env_override() {
        let _lock = TEST_MUTEX.lock().unwrap();

        clear_env_vars();
        unsafe {
            env::set_var("PRESET", "casual");
            env::set_var("EXP_RATE", "3.0"); // Override casual's 2.0 exp rate
        }

        let settings = GameSettings::default();
        assert_eq!(settings.exp_rate, 3.0);
        assert_eq!(settings.pal_capture_rate, 2.0); // Preset value remains if not overridden
    }

    #[test]
    fn test_save_and_load_config() {
        let _lock = TEST_MUTEX.lock().unwrap();

        clear_env_vars();
        let test_path = Path::new(TEST_DIR).join("test_config.ini");

        // Ensure the test directory exists
        fs::create_dir_all(TEST_DIR).unwrap();

        let settings = Settings::default();
        save_config(&test_path, &settings);

        assert!(test_path.exists());

        let loaded_settings = load_or_create_config(&test_path);
        assert_eq!(loaded_settings.server_name, "My Palworld Server");
        assert_eq!(loaded_settings.exp_rate, 1.0);
    }
}
