use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

macro_rules! env_parse {
    ($env_var:expr, $default:expr, $t:ty) => {
        std::env::var($env_var)
            .ok()
            .and_then(|s| s.parse::<$t>().ok())
            .unwrap_or($default)
    };
}

/// Represents game settings for Palworld server.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameSettings {
    pub difficulty: String,
    pub day_time_speed_rate: f32,
    pub night_time_speed_rate: f32,
    pub exp_rate: f32,
    pub pal_capture_rate: f32,
    pub pal_spawn_num_rate: f32,
    pub pal_damage_rate_attack: f32,
    pub pal_damage_rate_defense: f32,
    pub player_damage_rate_attack: f32,
    pub player_damage_rate_defense: f32,
    pub player_stomach_decrease_rate: f32,
    pub player_stamina_decrease_rate: f32,
    pub player_auto_hp_regen_rate: f32,
    pub player_auto_hp_regen_rate_in_sleep: f32,
    pub pal_auto_hp_regen_rate: f32,
    pub pal_auto_hp_regen_rate_in_sleep: f32,
    pub build_object_damage_rate: f32,
    pub build_object_deterioration_damage_rate: f32,
    pub collection_drop_rate: f32,
    pub collection_object_hp_rate: f32,
    pub collection_object_respawn_speed_rate: f32,
    pub enemy_drop_item_rate: f32,
    pub death_penalty: String,
    pub enable_pvp: bool,
    pub enable_friendly_fire: bool,
    pub enable_invader_enemy: bool,
    pub enable_aim_assist_pad: bool,
    pub enable_aim_assist_keyboard: bool,
    pub server_player_max_num: u16,
    pub coop_player_max_num: u16,
    pub server_name: String,
    pub server_description: String,
    pub admin_password: String,
    pub server_password: String,
    pub public_port: u16,
    pub public_ip: String,
    pub rcon_enabled: bool,
    pub rcon_port: u16,
    pub use_auth: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            difficulty: std::env::var("DIFFICULTY").unwrap_or_else(|_| "None".to_string()),
            day_time_speed_rate: env_parse!("DAY_TIME_SPEED_RATE", 1.0, f32),
            night_time_speed_rate: env_parse!("NIGHT_TIME_SPEED_RATE", 1.0, f32),
            exp_rate: env_parse!("EXP_RATE", 1.0, f32),
            pal_capture_rate: env_parse!("PAL_CAPTURE_RATE", 1.2, f32),
            pal_spawn_num_rate: env_parse!("PAL_SPAWN_NUM_RATE", 1.0, f32),
            pal_damage_rate_attack: env_parse!("PAL_DAMAGE_RATE_ATTACK", 1.0, f32),
            pal_damage_rate_defense: env_parse!("PAL_DAMAGE_RATE_DEFENSE", 1.0, f32),
            player_damage_rate_attack: env_parse!("PLAYER_DAMAGE_RATE_ATTACK", 1.0, f32),
            player_damage_rate_defense: env_parse!("PLAYER_DAMAGE_RATE_DEFENSE", 1.0, f32),
            player_stomach_decrease_rate: env_parse!("PLAYER_STOMACH_DECREASE_RATE", 1.0, f32),
            player_stamina_decrease_rate: env_parse!("PLAYER_STAMINA_DECREASE_RATE", 1.0, f32),
            player_auto_hp_regen_rate: env_parse!("PLAYER_AUTO_HP_REGEN_RATE", 1.0, f32),
            player_auto_hp_regen_rate_in_sleep: env_parse!(
                "PLAYER_AUTO_HP_REGEN_RATE_IN_SLEEP",
                2.0,
                f32
            ),
            pal_auto_hp_regen_rate: env_parse!("PAL_AUTO_HP_REGEN_RATE", 1.0, f32),
            pal_auto_hp_regen_rate_in_sleep: env_parse!(
                "PAL_AUTO_HP_REGEN_RATE_IN_SLEEP",
                2.0,
                f32
            ),
            build_object_damage_rate: env_parse!("BUILD_OBJECT_DAMAGE_RATE", 1.0, f32),
            build_object_deterioration_damage_rate: env_parse!(
                "BUILD_OBJECT_DETERIORATION_DAMAGE_RATE",
                1.0,
                f32
            ),
            collection_drop_rate: env_parse!("COLLECTION_DROP_RATE", 1.5, f32),
            collection_object_hp_rate: env_parse!("COLLECTION_OBJECT_HP_RATE", 1.0, f32),
            collection_object_respawn_speed_rate: env_parse!(
                "COLLECTION_OBJECT_RESPAWN_SPEED_RATE",
                1.0,
                f32
            ),
            enemy_drop_item_rate: env_parse!("ENEMY_DROP_ITEM_RATE", 1.2, f32),
            death_penalty: std::env::var("DEATH_PENALTY").unwrap_or_else(|_| "None".to_string()),
            enable_pvp: env_parse!("ENABLE_PVP", false, bool),
            enable_friendly_fire: env_parse!("ENABLE_FRIENDLY_FIRE", false, bool),
            enable_invader_enemy: env_parse!("ENABLE_INVADER_ENEMY", true, bool),
            enable_aim_assist_pad: env_parse!("ENABLE_AIM_ASSIST_PAD", true, bool),
            enable_aim_assist_keyboard: env_parse!("ENABLE_AIM_ASSIST_KEYBOARD", false, bool),
            server_player_max_num: env_parse!("SERVER_PLAYER_MAX_NUM", 16, u16),
            coop_player_max_num: env_parse!("COOP_PLAYER_MAX_NUM", 16, u16),
            server_name: std::env::var("SERVER_NAME")
                .unwrap_or_else(|_| "My Palworld Server".to_string()),
            server_description: std::env::var("SERVER_DESCRIPTION").unwrap_or_default(),
            admin_password: std::env::var("ADMIN_PASSWORD").unwrap_or_default(),
            server_password: std::env::var("SERVER_PASSWORD").unwrap_or_default(),
            public_port: env_parse!("PUBLIC_PORT", 8211, u16),
            public_ip: std::env::var("PUBLIC_IP").unwrap_or_default(),
            rcon_enabled: env_parse!("RCON_ENABLED", false, bool),
            rcon_port: env_parse!("RCON_PORT", 25575, u16),
            use_auth: env_parse!("USE_AUTH", true, bool),
        }
    }
}

/// Converts game settings to a formatted INI string.
fn to_ini(settings: &GameSettings) -> String {
    let mut ini_string = "[/Script/Pal.PalGameWorldSettings]\n".to_string();

    let settings_map = [
        ("Difficulty", settings.difficulty.clone()),
        ("DayTimeSpeedRate", settings.day_time_speed_rate.to_string()),
        (
            "NightTimeSpeedRate",
            settings.night_time_speed_rate.to_string(),
        ),
        ("ExpRate", settings.exp_rate.to_string()),
        ("PalCaptureRate", settings.pal_capture_rate.to_string()),
        ("PalSpawnNumRate", settings.pal_spawn_num_rate.to_string()),
        (
            "PalDamageRateAttack",
            settings.pal_damage_rate_attack.to_string(),
        ),
        (
            "PalDamageRateDefense",
            settings.pal_damage_rate_defense.to_string(),
        ),
        (
            "PlayerDamageRateAttack",
            settings.player_damage_rate_attack.to_string(),
        ),
        (
            "PlayerDamageRateDefense",
            settings.player_damage_rate_defense.to_string(),
        ),
        (
            "PlayerStomachDecreaseRate",
            settings.player_stomach_decrease_rate.to_string(),
        ),
        (
            "PlayerStaminaDecreaseRate",
            settings.player_stamina_decrease_rate.to_string(),
        ),
        (
            "PlayerAutoHPRegeneRate",
            settings.player_auto_hp_regen_rate.to_string(),
        ),
        (
            "PlayerAutoHpRegeneRateInSleep",
            settings.player_auto_hp_regen_rate_in_sleep.to_string(),
        ),
        (
            "PalAutoHPRegeneRate",
            settings.pal_auto_hp_regen_rate.to_string(),
        ),
        (
            "PalAutoHpRegeneRateInSleep",
            settings.pal_auto_hp_regen_rate_in_sleep.to_string(),
        ),
        (
            "BuildObjectDamageRate",
            settings.build_object_damage_rate.to_string(),
        ),
        (
            "BuildObjectDeteriorationDamageRate",
            settings.build_object_deterioration_damage_rate.to_string(),
        ),
        (
            "CollectionDropRate",
            settings.collection_drop_rate.to_string(),
        ),
        (
            "CollectionObjectHpRate",
            settings.collection_object_hp_rate.to_string(),
        ),
        (
            "CollectionObjectRespawnSpeedRate",
            settings.collection_object_respawn_speed_rate.to_string(),
        ),
        (
            "EnemyDropItemRate",
            settings.enemy_drop_item_rate.to_string(),
        ),
        ("DeathPenalty", settings.death_penalty.clone()),
        ("bIsPvP", settings.enable_pvp.to_string()),
        (
            "bEnableFriendlyFire",
            settings.enable_friendly_fire.to_string(),
        ),
        (
            "bEnableInvaderEnemy",
            settings.enable_invader_enemy.to_string(),
        ),
        (
            "bEnableAimAssistPad",
            settings.enable_aim_assist_pad.to_string(),
        ),
        (
            "bEnableAimAssistKeyboard",
            settings.enable_aim_assist_keyboard.to_string(),
        ),
        (
            "ServerPlayerMaxNum",
            settings.server_player_max_num.to_string(),
        ),
        ("CoopPlayerMaxNum", settings.coop_player_max_num.to_string()),
        ("ServerName", format!("\"{}\"", settings.server_name)),
        (
            "ServerDescription",
            format!("\"{}\"", settings.server_description),
        ),
        ("AdminPassword", format!("\"{}\"", settings.admin_password)),
        (
            "ServerPassword",
            format!("\"{}\"", settings.server_password),
        ),
        ("PublicPort", settings.public_port.to_string()),
        ("PublicIP", format!("\"{}\"", settings.public_ip)),
        ("RCONEnabled", settings.rcon_enabled.to_string()),
        ("RCONPort", settings.rcon_port.to_string()),
        ("bUseAuth", settings.use_auth.to_string()),
    ];

    for (key, value) in settings_map {
        ini_string.push_str(&format!("{}={}\n", key, value));
    }

    ini_string
}

/// Saves the configuration to an INI file.
pub fn save_config(path: &Path, config: &GameSettings) {
    if let Err(e) = fs::write(path, to_ini(config)) {
        eprintln!("Failed to save config: {}", e);
    }
}

/// Loads the configuration from an INI file or returns defaults if the file is missing.
pub fn load_or_create_config(path: &Path) -> GameSettings {
    if path.exists() {
        println!("Loading existing config from {:?}", path);
        // Future: Implement INI parsing here if needed.
    } else {
        println!("Config file not found. Creating default config.");
        let default_config = GameSettings::default();
        save_config(path, &default_config);
        return default_config;
    }
    GameSettings::default()
}
