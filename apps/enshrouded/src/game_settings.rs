use crate::environment::name;
use crate::utils::config_io::{load_config_with_defaults, save_config};
use crate::utils::env_overrides::apply_env_overrides;
use env_parse::env_parse;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;

/// Represents game settings in the server configuration.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameSettings {
    pub player_health_factor: f32,
    pub player_mana_factor: f32,
    pub player_stamina_factor: f32,
    pub player_body_heat_factor: f32,
    pub enable_durability: bool,
    pub enable_starving_debuff: bool,
    pub food_buff_duration_factor: f32,
    pub from_hunger_to_starving: u64,
    pub shroud_time_factor: f32,
    pub tombstone_mode: String,
    pub enable_glider_turbulences: bool,
    pub weather_frequency: String,
    pub mining_damage_factor: f32,
    pub plant_growth_speed_factor: f32,
    pub resource_drop_stack_amount_factor: f32,
    pub factory_production_speed_factor: f32,
    pub perk_upgrade_recycling_factor: f32,
    pub perk_cost_factor: f32,
    pub experience_combat_factor: f32,
    pub experience_mining_factor: f32,
    pub experience_exploration_quests_factor: f32,
    pub random_spawner_amount: String,
    pub aggro_pool_amount: String,
    pub enemy_damage_factor: f32,
    pub enemy_health_factor: f32,
    pub enemy_stamina_factor: f32,
    pub enemy_perception_range_factor: f32,
    pub boss_damage_factor: f32,
    pub boss_health_factor: f32,
    pub threat_bonus: f32,
    pub pacify_all_enemies: bool,
    pub taming_startle_repercussion: String,
    pub day_time_duration: u64,
    pub night_time_duration: u64,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            player_health_factor: env_parse!("PLAYER_HEALTH_FACTOR", 1.0, f32),
            player_mana_factor: env_parse!("PLAYER_MANA_FACTOR", 1.0, f32),
            player_stamina_factor: env_parse!("PLAYER_STAMINA_FACTOR", 1.0, f32),
            player_body_heat_factor: env_parse!("PLAYER_BODY_HEAT_FACTOR", 1.0, f32),
            enable_durability: env_parse!("ENABLE_DURABILITY", true, bool),
            enable_starving_debuff: env_parse!("ENABLE_STARVING_DEBUFF", false, bool),
            food_buff_duration_factor: env_parse!("FOOD_BUFF_DURATION_FACTOR", 1.0, f32),
            from_hunger_to_starving: env_parse!("FROM_HUNGER_TO_STARVING", 600_000_000_000, u64),
            shroud_time_factor: env_parse!("SHROUD_TIME_FACTOR", 1.0, f32),
            tombstone_mode: std::env::var("TOMBSTONE_MODE")
                .unwrap_or_else(|_| "AddBackpackMaterials".to_string()),
            enable_glider_turbulences: env_parse!("ENABLE_GLIDER_TURBULENCES", true, bool),
            weather_frequency: std::env::var("WEATHER_FREQUENCY")
                .unwrap_or_else(|_| "Normal".to_string()),
            mining_damage_factor: env_parse!("MINING_DAMAGE_FACTOR", 1.0, f32),
            plant_growth_speed_factor: env_parse!("PLANT_GROWTH_SPEED_FACTOR", 1.0, f32),
            resource_drop_stack_amount_factor: env_parse!(
                "RESOURCE_DROP_STACK_AMOUNT_FACTOR",
                1.0,
                f32
            ),
            factory_production_speed_factor: env_parse!(
                "FACTORY_PRODUCTION_SPEED_FACTOR",
                1.0,
                f32
            ),
            perk_upgrade_recycling_factor: env_parse!("PERK_UPGRADE_RECYCLING_FACTOR", 0.5, f32),
            perk_cost_factor: env_parse!("PERK_COST_FACTOR", 1.0, f32),
            experience_combat_factor: env_parse!("EXPERIENCE_COMBAT_FACTOR", 1.0, f32),
            experience_mining_factor: env_parse!("EXPERIENCE_MINING_FACTOR", 1.0, f32),
            experience_exploration_quests_factor: env_parse!(
                "EXPERIENCE_EXPLORATION_QUESTS_FACTOR",
                1.0,
                f32
            ),
            random_spawner_amount: std::env::var("RANDOM_SPAWNER_AMOUNT")
                .unwrap_or_else(|_| "Normal".to_string()),
            aggro_pool_amount: std::env::var("AGGRO_POOL_AMOUNT")
                .unwrap_or_else(|_| "Normal".to_string()),
            enemy_damage_factor: env_parse!("ENEMY_DAMAGE_FACTOR", 1.0, f32),
            enemy_health_factor: env_parse!("ENEMY_HEALTH_FACTOR", 1.0, f32),
            enemy_stamina_factor: env_parse!("ENEMY_STAMINA_FACTOR", 1.0, f32),
            enemy_perception_range_factor: env_parse!("ENEMY_PERCEPTION_RANGE_FACTOR", 1.0, f32),
            boss_damage_factor: env_parse!("BOSS_DAMAGE_FACTOR", 1.0, f32),
            boss_health_factor: env_parse!("BOSS_HEALTH_FACTOR", 1.0, f32),
            threat_bonus: env_parse!("THREAT_BONUS", 1.0, f32),
            pacify_all_enemies: env_parse!("PACIFY_ALL_ENEMIES", false, bool),
            taming_startle_repercussion: std::env::var("TAMING_STARTLE_REPERCUSSION")
                .unwrap_or_else(|_| "LoseSomeProgress".to_string()),
            day_time_duration: env_parse!("DAY_TIME_DURATION", 1_800_000_000_000, u64),
            night_time_duration: env_parse!("NIGHT_TIME_DURATION", 720_000_000_000, u64),
        }
    }
}

/// Represents a user group and its permissions.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserGroup {
    pub name: String,
    pub password: String,
    pub can_kick_ban: bool,
    pub can_access_inventories: bool,
    pub can_edit_base: bool,
    pub can_extend_base: bool,
    pub reserved_slots: u8,
}

impl Default for UserGroup {
    fn default() -> Self {
        Self {
            name: "Guest".to_string(),
            password: "GuestXXXXXXXX".to_string(),
            can_kick_ban: false,
            can_access_inventories: true,
            can_edit_base: true,
            can_extend_base: true,
            reserved_slots: 0,
        }
    }
}

/// Represents the full server configuration.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServerConfig {
    pub name: String,
    pub save_directory: String,
    pub log_directory: String,
    pub ip: String,
    pub query_port: u16,
    pub slot_count: u8,
    pub voice_chat_mode: String,
    pub enable_voice_chat: bool,
    pub enable_text_chat: bool,
    pub game_settings_preset: String,
    pub game_settings: GameSettings,
    pub user_groups: Vec<UserGroup>,
    pub game_port: i32,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            name: name(),
            save_directory: "./savegame".to_string(),
            log_directory: "./logs".to_string(),
            ip: "0.0.0.0".to_string(),
            game_port: 15636,
            query_port: 15637,
            slot_count: 16,
            voice_chat_mode: "Proximity".to_string(),
            enable_voice_chat: false,
            enable_text_chat: false,
            game_settings_preset: "Default".to_string(),
            game_settings: GameSettings::default(),
            user_groups: vec![
                UserGroup {
                    name: "Admin".to_string(),
                    password: "AdminXXXXXXXX".to_string(),
                    can_kick_ban: true,
                    can_access_inventories: true,
                    can_edit_base: true,
                    can_extend_base: true,
                    reserved_slots: 0,
                },
                UserGroup::default(),
            ],
        }
    }
}

/// Loads the configuration from a file or creates a new one with defaults.
/// Environment variables override both file values and defaults.
pub fn load_or_create_config(path: &Path) -> ServerConfig {
    let mut config = load_config_with_defaults::<ServerConfig>(path);

    apply_env_overrides(&mut config);

    save_config(path, &config);

    config
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;

    lazy_static::lazy_static! {
        static ref TEST_MUTEX: Mutex<()> = Mutex::new(());
    }

    fn clear_env_vars() {
        let vars = [
            "PLAYER_HEALTH_FACTOR",
            "PLAYER_MANA_FACTOR",
            "PLAYER_STAMINA_FACTOR",
            "EXPERIENCE_COMBAT_FACTOR",
            "TOMBSTONE_MODE",
            "THREAT_BONUS",
        ];
        for var in vars {
            unsafe {
                env::remove_var(var);
            }
        }
    }

    #[test]
    fn test_default_settings() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_env_vars();

        let settings = GameSettings::default();

        assert_eq!(settings.player_health_factor, 1.0);
        assert_eq!(settings.player_mana_factor, 1.0);
        assert_eq!(settings.tombstone_mode, "AddBackpackMaterials");
    }

    #[test]
    fn test_env_override_f32() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_env_vars();

        unsafe {
            env::set_var("PLAYER_HEALTH_FACTOR", "1.5");
            env::set_var("EXPERIENCE_COMBAT_FACTOR", "3.0");
        }

        let settings = GameSettings::default();
        assert_eq!(settings.player_health_factor, 1.5);
        assert_eq!(settings.experience_combat_factor, 3.0);
    }

    #[test]
    fn test_env_override_string() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_env_vars();
        unsafe {
            env::set_var("TOMBSTONE_MODE", "Nothing");
        }

        let settings = GameSettings::default();
        assert_eq!(settings.tombstone_mode, "Nothing");
    }

    #[test]
    fn test_save_and_load_config() {
        fs::create_dir_all("./tmp").unwrap();
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_env_vars();

        unsafe {
            env::set_var("PLAYER_HEALTH_FACTOR", "1.5");
            env::set_var("EXPERIENCE_COMBAT_FACTOR", "300.0");
            env::set_var("TOMBSTONE_MODE", "Nothing");
            env::set_var("THREAT_BONUS", "3.14");
        }

        let path = Path::new("./tmp/test_enshrouded_config.json");
        fs::create_dir_all("./tmp").unwrap();
        let _ = fs::remove_file(&path);

        // Load with env override and save
        let config = load_or_create_config(&path);
        assert!(path.exists());

        // Ensure env-injected threat_bonus persisted
        assert!((config.game_settings.threat_bonus - 3.14).abs() < f32::EPSILON);

        let raw = fs::read_to_string(&path).expect("failed to read config");
        let json: serde_json::Value = serde_json::from_str(&raw).expect("invalid JSON");

        let threat_bonus = json["gameSettings"]["threatBonus"]
            .as_f64()
            .expect("missing or invalid threatBonus");

        assert!((threat_bonus - 3.14).abs() < f64::EPSILON);
    }
}
