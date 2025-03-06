use crate::environment::name;
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
            player_health_factor: 1.0,
            player_mana_factor: 1.0,
            player_stamina_factor: 1.0,
            player_body_heat_factor: 1.0,
            enable_durability: true,
            enable_starving_debuff: false,
            food_buff_duration_factor: 1.0,
            from_hunger_to_starving: 600_000_000_000,
            shroud_time_factor: 1.0,
            tombstone_mode: "AddBackpackMaterials".to_string(),
            enable_glider_turbulences: true,
            weather_frequency: "Normal".to_string(),
            mining_damage_factor: 1.0,
            plant_growth_speed_factor: 1.0,
            resource_drop_stack_amount_factor: 1.0,
            factory_production_speed_factor: 1.0,
            perk_upgrade_recycling_factor: 0.5,
            perk_cost_factor: 1.0,
            experience_combat_factor: 1.0,
            experience_mining_factor: 1.0,
            experience_exploration_quests_factor: 1.0,
            random_spawner_amount: "Normal".to_string(),
            aggro_pool_amount: "Normal".to_string(),
            enemy_damage_factor: 1.0,
            enemy_health_factor: 1.0,
            enemy_stamina_factor: 1.0,
            enemy_perception_range_factor: 1.0,
            boss_damage_factor: 1.0,
            boss_health_factor: 1.0,
            threat_bonus: 1.0,
            pacify_all_enemies: false,
            taming_startle_repercussion: "LoseSomeProgress".to_string(),
            day_time_duration: 1_800_000_000_000,
            night_time_duration: 720_000_000_000,
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
    let mut config = if path.exists() {
        match fs::read_to_string(path) {
            Ok(contents) => serde_json::from_str::<ServerConfig>(&contents).unwrap_or_else(|_| {
                eprintln!("Warning: Corrupt config file detected. Using defaults.");
                ServerConfig::default()
            }),
            Err(_) => {
                eprintln!("Failed to read config file. Using defaults.");
                ServerConfig::default()
            }
        }
    } else {
        eprintln!("Config file not found. Creating default config.");
        ServerConfig::default()
    };

    apply_env_overrides(&mut config);

    save_config(path, &config);

    config
}

/// Overrides configuration values using environment variables.
fn apply_env_overrides(config: &mut ServerConfig) {
    for (key, value) in env::vars() {
        if let Some(stripped) = key.strip_prefix("SET_GROUP_") {
            let mut parts = stripped.splitn(2, '_');
            if let (Some(group_name), Some(field_name)) = (parts.next(), parts.next()) {
                if let Some(group) = config
                    .user_groups
                    .iter_mut()
                    .find(|g| g.name.eq_ignore_ascii_case(group_name))
                {
                    match field_name.to_lowercase().as_str() {
                        "password" => group.password = value,
                        "can_kick_ban" => {
                            group.can_kick_ban = value.parse().unwrap_or(group.can_kick_ban)
                        }
                        "can_access_inventories" => {
                            group.can_access_inventories =
                                value.parse().unwrap_or(group.can_access_inventories)
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

/// Saves the configuration to a file.
pub fn save_config(path: &Path, config: &ServerConfig) {
    if let Ok(json) = serde_json::to_string_pretty(config) {
        let _ = fs::write(path, json);
    }
}
