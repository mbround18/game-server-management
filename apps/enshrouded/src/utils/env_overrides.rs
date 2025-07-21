use crate::game_settings::ServerConfig;
use std::env;

/// Applies environment variable overrides to the config.
pub fn apply_env_overrides(config: &mut ServerConfig) {
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
