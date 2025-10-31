use env_parse::env_parse;
use gsm_serde::serde_ini::{IniHeader, to_string};
use ini_derive::IniSerialize;
use serde::{Deserialize, Serialize};
use std::fs::create_dir_all;
use std::path::Path;
use std::{env, fs};

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameSettings {
    // Core gameplay rates
    #[serde(rename = "Difficulty")]
    pub difficulty: String,

    #[serde(rename = "RandomizerType")]
    pub randomizer_type: String,

    #[serde(rename = "RandomizerSeed")]
    pub randomizer_seed: String,

    #[serde(rename = "bIsRandomizerPalLevelRandom")]
    pub is_randomizer_pal_level_random: bool,

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

    #[serde(rename = "bAllowGlobalPalboxExport")]
    pub allow_global_palbox_export: bool,

    #[serde(rename = "bAllowGlobalPalboxImport")]
    pub allow_global_palbox_import: bool,

    #[serde(rename = "bCharacterRecreateInHardcore")]
    pub character_recreate_in_hardcore: bool,

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

    #[serde(rename = "PalStomachDecreaseRate")]
    pub pal_stomach_decrease_rate: f32,

    #[serde(rename = "PalStaminaDecreaseRate")]
    pub pal_stamina_decrease_rate: f32,

    #[serde(rename = "PalAutoHPRegeneRate")]
    pub pal_auto_hp_regen_rate: f32,

    #[serde(rename = "PalAutoHpRegeneRateInSleep")]
    pub pal_auto_hp_regen_rate_in_sleep: f32,

    // Build and object settings
    #[serde(rename = "BuildObjectHpRate")]
    pub build_object_hp_rate: f32,

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

    // Death penalty and PvP settings
    #[serde(rename = "DeathPenalty")]
    pub death_penalty: String,

    #[serde(rename = "bEnablePlayerToPlayerDamage")]
    pub enable_pvp: bool,

    #[serde(rename = "bEnableFriendlyFire")]
    pub enable_friendly_fire: bool,

    #[serde(rename = "bEnableInvaderEnemy")]
    pub enable_invader_enemy: bool,

    #[serde(rename = "bActiveUNKO")]
    pub active_unko: bool,

    #[serde(rename = "bEnableAimAssistPad")]
    pub enable_aim_assist_pad: bool,

    #[serde(rename = "bEnableAimAssistKeyboard")]
    pub enable_aim_assist_keyboard: bool,

    // Drop and base camp settings
    #[serde(rename = "DropItemMaxNum")]
    pub drop_item_max_num: u32,

    #[serde(rename = "DropItemMaxNum_UNKO")]
    pub drop_item_max_num_unko: u32,

    #[serde(rename = "BaseCampMaxNum")]
    pub base_camp_max_num: u16,

    #[serde(rename = "BaseCampWorkerMaxNum")]
    pub base_camp_worker_max_num: u16,

    #[serde(rename = "DropItemAliveMaxHours")]
    pub drop_item_alive_max_hours: f32,

    // Guild and related settings
    #[serde(rename = "bAutoResetGuildNoOnlinePlayers")]
    pub auto_reset_guild_no_online_players: bool,

    #[serde(rename = "AutoResetGuildTimeNoOnlinePlayers")]
    pub auto_reset_guild_time_no_online_players: f32,

    #[serde(rename = "GuildPlayerMaxNum")]
    pub guild_player_max_num: u16,

    #[serde(rename = "BaseCampMaxNumInGuild")]
    pub base_camp_max_num_in_guild: u16,

    #[serde(rename = "PalEggDefaultHatchingTime")]
    pub pal_egg_default_hatching_time: f32,

    // Other gameplay rates
    #[serde(rename = "WorkSpeedRate")]
    pub work_speed_rate: f32,

    #[serde(rename = "AutoSaveSpan")]
    pub auto_save_span: f32,

    // Multiplayer and PvP modes
    #[serde(rename = "bIsMultiplay")]
    pub is_multiplay: bool,

    #[serde(rename = "bIsPvP")]
    pub is_pvp: bool,

    #[serde(rename = "bHardcore")]
    pub hardcore: bool,

    #[serde(rename = "bPalLost")]
    pub pal_lost: bool,

    #[serde(rename = "bCanPickupOtherGuildDeathPenaltyDrop")]
    pub can_pickup_other_guild_death_penalty_drop: bool,

    #[serde(rename = "bEnableNonLoginPenalty")]
    pub enable_non_login_penalty: bool,

    #[serde(rename = "bEnableFastTravel")]
    pub enable_fast_travel: bool,

    #[serde(rename = "bIsStartLocationSelectByMap")]
    pub is_start_location_select_by_map: bool,

    #[serde(rename = "bExistPlayerAfterLogout")]
    pub exist_player_after_logout: bool,

    #[serde(rename = "bEnableDefenseOtherGuildPlayer")]
    pub enable_defense_other_guild_player: bool,

    #[serde(rename = "bInvisibleOtherGuildBaseCampAreaFX")]
    pub invisible_other_guild_base_camp_area_fx: bool,

    #[serde(rename = "bBuildAreaLimit")]
    pub build_area_limit: bool,

    #[serde(rename = "ItemWeightRate")]
    pub item_weight_rate: f32,

    // Server limits and networking
    #[serde(rename = "CoopPlayerMaxNum")]
    pub coop_player_max_num: u16,

    #[serde(rename = "ServerPlayerMaxNum")]
    pub server_player_max_num: u16,

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

    #[serde(rename = "CrossplayPlatforms")]
    pub crossplay_platforms: String, // Default (Steam,Xbox,PS5,Mac)

    // REST API and additional networking
    #[serde(rename = "RESTAPIEnabled")]
    pub restapi_enabled: bool,

    #[serde(rename = "RESTAPIPort")]
    pub restapi_port: u16,

    #[serde(rename = "bShowPlayerList")]
    pub show_player_list: bool,

    #[serde(rename = "ChatPostLimitPerMinute")]
    pub chat_post_limit_per_minute: u16,

    #[serde(rename = "bIsUseBackupSaveData")]
    pub is_use_backup_save_data: bool,

    #[serde(rename = "LogFormatType")]
    pub log_format_type: String,

    #[serde(rename = "SupplyDropSpan")]
    pub supply_drop_span: f32,

    #[serde(rename = "EnablePredatorBossPal")]
    pub enable_predator_boss_pal: bool,

    #[serde(rename = "MaxBuildingLimitNum")]
    pub max_building_limit_num: u32,

    #[serde(rename = "ServerReplicatePawnCullDistance")]
    pub server_replicate_pawn_cull_distance: f32,
}

impl GameSettings {
    /// Constructs the base (Normal preset) configuration based on the golden INI.
    pub fn normal() -> Self {
        Self {
            difficulty: "None".to_string(),
            randomizer_type: "None".to_string(),
            randomizer_seed: "".to_string(),
            is_randomizer_pal_level_random: false,
            day_time_speed_rate: 1.0,
            night_time_speed_rate: 1.0,
            exp_rate: 1.0,
            pal_capture_rate: 1.0,
            pal_spawn_num_rate: 1.0,
            pal_damage_rate_attack: 1.0,
            pal_damage_rate_defense: 1.0,
            allow_global_palbox_export: false,
            allow_global_palbox_import: false,
            character_recreate_in_hardcore: false,
            player_damage_rate_attack: 1.0,
            player_damage_rate_defense: 1.0,
            player_stomach_decrease_rate: 1.0,
            player_stamina_decrease_rate: 1.0,
            player_auto_hp_regen_rate: 1.0,
            player_auto_hp_regen_rate_in_sleep: 1.0,
            pal_stomach_decrease_rate: 1.0,
            pal_stamina_decrease_rate: 1.0,
            pal_auto_hp_regen_rate: 1.0,
            pal_auto_hp_regen_rate_in_sleep: 1.0,
            build_object_hp_rate: 1.0,
            build_object_damage_rate: 1.0,
            build_object_deterioration_damage_rate: 1.0,
            collection_drop_rate: 1.0,
            collection_object_hp_rate: 1.0,
            collection_object_respawn_speed_rate: 1.0,
            enemy_drop_item_rate: 1.0,
            death_penalty: "All".to_string(),
            enable_pvp: false,
            enable_friendly_fire: false,
            enable_invader_enemy: true,
            active_unko: false,
            enable_aim_assist_pad: true,
            enable_aim_assist_keyboard: false,
            drop_item_max_num: 3000,
            drop_item_max_num_unko: 100,
            base_camp_max_num: 128,
            base_camp_worker_max_num: 15,
            drop_item_alive_max_hours: 1.0,
            auto_reset_guild_no_online_players: false,
            auto_reset_guild_time_no_online_players: 72.0,
            guild_player_max_num: 20,
            base_camp_max_num_in_guild: 4,
            pal_egg_default_hatching_time: 72.0,
            work_speed_rate: 1.0,
            auto_save_span: 30.0,
            is_multiplay: false,
            is_pvp: false,
            hardcore: false,
            pal_lost: false,
            can_pickup_other_guild_death_penalty_drop: false,
            enable_non_login_penalty: true,
            enable_fast_travel: true,
            is_start_location_select_by_map: true,
            exist_player_after_logout: false,
            enable_defense_other_guild_player: false,
            invisible_other_guild_base_camp_area_fx: false,
            build_area_limit: false,
            item_weight_rate: 1.0,
            coop_player_max_num: 4,
            server_player_max_num: 32,
            server_name: "Default Palworld Server".to_string(),
            server_description: "".to_string(),
            admin_password: "".to_string(),
            server_password: "".to_string(),
            public_port: 8211,
            public_ip: "".to_string(),
            rcon_enabled: false,
            rcon_port: 25575,
            use_auth: true,
            region: "".to_string(),
            ban_list_url: "https://api.palworldgame.com/api/banlist.txt".to_string(),
            restapi_enabled: false,
            restapi_port: 8212,
            show_player_list: false,
            chat_post_limit_per_minute: 10,
            crossplay_platforms: "(Steam,Xbox,PS5,Mac)".to_string(),
            is_use_backup_save_data: true,
            log_format_type: "Text".to_string(),
            supply_drop_span: 180.0,
            enable_predator_boss_pal: true,
            max_building_limit_num: 0,
            server_replicate_pawn_cull_distance: 15000.0,
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
                // Normal preset is our base; no changes.
            }
            Preset::Hard => {
                self.day_time_speed_rate = 1.0;
                self.night_time_speed_rate = 1.0;
                self.exp_rate = 0.5;
                self.pal_capture_rate = 1.0;
                self.pal_spawn_num_rate = 1.0;
                self.pal_damage_rate_attack = 0.5;
                self.pal_damage_rate_defense = 2.0;
                self.player_damage_rate_defense = 4.0;
                self.player_stomach_decrease_rate = 1.0;
                self.player_stamina_decrease_rate = 1.0;
                self.player_damage_rate_attack = 0.7;
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
        if let Ok(preset_str) = env::var("PRESET")
            && let Ok(preset) = serde_plain::from_str::<Preset>(&preset_str)
        {
            settings.apply_preset(preset);
        }

        Self {
            difficulty: env::var("DIFFICULTY").unwrap_or_else(|_| settings.difficulty.clone()),
            randomizer_type: env::var("RANDOMIZER_TYPE")
                .unwrap_or_else(|_| settings.randomizer_type.clone()),
            randomizer_seed: env::var("RANDOMIZER_SEED")
                .unwrap_or_else(|_| settings.randomizer_seed.clone()),
            is_randomizer_pal_level_random: env_parse!(
                "B_IS_RANDOMIZER_PAL_LEVEL_RANDOM",
                settings.is_randomizer_pal_level_random,
                bool
            ),
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
            allow_global_palbox_export: env_parse!(
                "B_ALLOW_GLOBAL_PALBOX_EXPORT",
                settings.allow_global_palbox_export,
                bool
            ),
            allow_global_palbox_import: env_parse!(
                "B_ALLOW_GLOBAL_PALBOX_IMPORT",
                settings.allow_global_palbox_import,
                bool
            ),
            character_recreate_in_hardcore: env_parse!(
                "B_CHARACTER_RECREATE_IN_HARDCORE",
                settings.character_recreate_in_hardcore,
                bool
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
            pal_stomach_decrease_rate: env_parse!(
                "PAL_STOMACH_DECREASE_RATE",
                settings.pal_stomach_decrease_rate,
                f32
            ),
            pal_stamina_decrease_rate: env_parse!(
                "PAL_STAMINA_DECREASE_RATE",
                settings.pal_stamina_decrease_rate,
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
            build_object_hp_rate: env_parse!(
                "BUILD_OBJECT_HP_RATE",
                settings.build_object_hp_rate,
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
            active_unko: env_parse!("ACTIVE_UNKO", settings.active_unko, bool),
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
            drop_item_max_num: env_parse!("DROP_ITEM_MAX_NUM", settings.drop_item_max_num, u32),
            drop_item_max_num_unko: env_parse!(
                "DROP_ITEM_MAX_NUM_UNKO",
                settings.drop_item_max_num_unko,
                u32
            ),
            base_camp_max_num: env_parse!("BASE_CAMP_MAX_NUM", settings.base_camp_max_num, u16),
            base_camp_worker_max_num: env_parse!(
                "BASE_CAMP_WORKER_MAX_NUM",
                settings.base_camp_worker_max_num,
                u16
            ),
            drop_item_alive_max_hours: env_parse!(
                "DROP_ITEM_ALIVE_MAX_HOURS",
                settings.drop_item_alive_max_hours,
                f32
            ),
            auto_reset_guild_no_online_players: env_parse!(
                "AUTO_RESET_GUILD_NO_ONLINE_PLAYERS",
                settings.auto_reset_guild_no_online_players,
                bool
            ),
            auto_reset_guild_time_no_online_players: env_parse!(
                "AUTO_RESET_GUILD_TIME_NO_ONLINE_PLAYERS",
                settings.auto_reset_guild_time_no_online_players,
                f32
            ),
            guild_player_max_num: env_parse!(
                "GUILD_PLAYER_MAX_NUM",
                settings.guild_player_max_num,
                u16
            ),
            base_camp_max_num_in_guild: env_parse!(
                "BASE_CAMP_MAX_NUM_IN_GUILD",
                settings.base_camp_max_num_in_guild,
                u16
            ),
            pal_egg_default_hatching_time: env_parse!(
                "PAL_EGG_DEFAULT_HATCHING_TIME",
                settings.pal_egg_default_hatching_time,
                f32
            ),
            work_speed_rate: env_parse!("WORK_SPEED_RATE", settings.work_speed_rate, f32),
            auto_save_span: env_parse!("AUTO_SAVE_SPAN", settings.auto_save_span, f32),
            is_multiplay: env_parse!("IS_MULTIPLAY", settings.is_multiplay, bool),
            is_pvp: env_parse!("IS_PVP", settings.is_pvp, bool),
            hardcore: env_parse!("HARDCORE", settings.hardcore, bool),
            pal_lost: env_parse!("PAL_LOST", settings.pal_lost, bool),
            can_pickup_other_guild_death_penalty_drop: env_parse!(
                "CAN_PICKUP_OTHER_GUILD_DEATH_PENALTY_DROP",
                settings.can_pickup_other_guild_death_penalty_drop,
                bool
            ),
            enable_non_login_penalty: env_parse!(
                "ENABLE_NON_LOGIN_PENALTY",
                settings.enable_non_login_penalty,
                bool
            ),
            enable_fast_travel: env_parse!("ENABLE_FAST_TRAVEL", settings.enable_fast_travel, bool),
            is_start_location_select_by_map: env_parse!(
                "IS_START_LOCATION_SELECT_BY_MAP",
                settings.is_start_location_select_by_map,
                bool
            ),
            exist_player_after_logout: env_parse!(
                "EXIST_PLAYER_AFTER_LOGOUT",
                settings.exist_player_after_logout,
                bool
            ),
            enable_defense_other_guild_player: env_parse!(
                "ENABLE_DEFENSE_OTHER_GUILD_PLAYER",
                settings.enable_defense_other_guild_player,
                bool
            ),
            invisible_other_guild_base_camp_area_fx: env_parse!(
                "INVISIBLE_OTHER_GUILD_BASE_CAMP_AREA_FX",
                settings.invisible_other_guild_base_camp_area_fx,
                bool
            ),
            build_area_limit: env_parse!("BUILD_AREA_LIMIT", settings.build_area_limit, bool),
            item_weight_rate: env_parse!("ITEM_WEIGHT_RATE", settings.item_weight_rate, f32),
            coop_player_max_num: env_parse!(
                "COOP_PLAYER_MAX_NUM",
                settings.coop_player_max_num,
                u16
            ),
            server_player_max_num: env_parse!(
                "SERVER_PLAYER_MAX_NUM",
                settings.server_player_max_num,
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
            ban_list_url: env::var("BAN_LIST").unwrap_or_else(|_| settings.ban_list_url.clone()),
            restapi_enabled: env_parse!("RESTAPI_ENABLED", settings.restapi_enabled, bool),
            restapi_port: env_parse!("RESTAPI_PORT", settings.restapi_port, u16),
            show_player_list: env_parse!("SHOW_PLAYER_LIST", settings.show_player_list, bool),
            chat_post_limit_per_minute: env_parse!(
                "CHAT_POST_LIMIT_PER_MINUTE",
                settings.chat_post_limit_per_minute,
                u16
            ),
            crossplay_platforms: env::var("CROSSPLAY_PLATFORMS")
                .unwrap_or_else(|_| settings.crossplay_platforms.clone()),
            is_use_backup_save_data: env_parse!(
                "IS_USE_BACKUP_SAVE_DATA",
                settings.is_use_backup_save_data,
                bool
            ),
            log_format_type: env::var("LOG_FORMAT_TYPE")
                .unwrap_or_else(|_| settings.log_format_type.clone()),
            supply_drop_span: env_parse!("SUPPLY_DROP_SPAN", settings.supply_drop_span, f32),
            enable_predator_boss_pal: env_parse!(
                "ENABLE_PREDATOR_BOSS_PAL",
                settings.enable_predator_boss_pal,
                bool
            ),
            max_building_limit_num: env_parse!(
                "MAX_BUILDING_LIMIT_NUM",
                settings.max_building_limit_num,
                u32
            ),
            server_replicate_pawn_cull_distance: env_parse!(
                "SERVER_REPLICATE_PAWN_CULL_DISTANCE",
                settings.server_replicate_pawn_cull_distance,
                f32
            ),
        }
    }
}

/// Saves the configuration to an INI file.
pub fn save_config(path: &Path, settings: &Settings) {
    let ini_config = to_string(&settings).unwrap();

    if let Err(e) = fs::write(path, ini_config) {
        eprintln!("Failed to save config: {e}");
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
        assert_eq!(settings.server_name, "Default Palworld Server");
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
        assert_eq!(loaded_settings.server_name, "Default Palworld Server");
        assert_eq!(loaded_settings.exp_rate, 1.0);
    }
}
