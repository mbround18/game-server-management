use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Write;

/// Trait for types that require a custom INI header.
///
/// The procedural macro from the `ini-derive` crate will automatically implement this trait
/// based on the `#[INIHeader(name = "...")]` attribute.
///
/// # Example
/// ```rust
/// use gsm_serde::serde_ini::{ IniHeader};
///
/// struct MySettings;
///
/// impl IniHeader for MySettings {
///     fn ini_header() -> &'static str {
///         "my_section"
///     }
/// }
///
/// // Now you can use `MySettings::ini_header()` to retrieve the section name.
/// assert_eq!(MySettings::ini_header(), "my_section");
/// ```
pub trait IniHeader {
    fn ini_header() -> &'static str;
}

/// Helper: Format a serde_json number with up to 5 decimal places, trimming trailing zeros.
fn format_number(n: &serde_json::Number) -> String {
    if let Some(f) = n.as_f64() {
        // Format to 5 decimals.
        let s = format!("{f:.5}");
        // Trim trailing zeros and possible trailing dot.
        let s = s.trim_end_matches('0').trim_end_matches('.');
        if s.is_empty() {
            "0".to_string()
        } else {
            s.to_string()
        }
    } else {
        n.to_string()
    }
}

/// Helper: Format a JSON value appropriately.
fn format_json_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => format!("\"{s}\""),
        serde_json::Value::Bool(b) => format!("{b}"),
        serde_json::Value::Number(n) => format_number(n),
        _ => format!("{value}"),
    }
}

fn serialize_value(value: &serde_json::Value, indent: usize) -> String {
    let mut output = String::new();
    let indent_str = "\t".repeat(indent);
    match value {
        serde_json::Value::Object(map) => {
            // Collect and sort keys alphabetically.
            let mut entries: Vec<_> = map.iter().collect();
            entries.sort_by(|(a, _), (b, _)| a.cmp(b));
            for (key, val) in entries {
                match val {
                    serde_json::Value::Object(_) => {
                        // Start a new nested block.
                        output.push_str(&format!("{indent_str}{key}=(\n"));
                        output.push_str(&serialize_value(val, indent + 1));
                        output.push_str(&format!("{indent_str})\n"));
                    }
                    _ => {
                        output.push_str(&format!(
                            "{}{}={},\n",
                            indent_str,
                            key,
                            format_json_value(val)
                        ));
                    }
                }
            }
        }
        _ => {
            output.push_str(&format!("{}{}", indent_str, format_json_value(value)));
        }
    }
    output
}

/// Serializes a struct into an INI-formatted string.
///
/// For nested JSON objects, it outputs them as a block with a surrounding parenthesis.
///
/// # Examples
///
/// Serializing a simple settings struct:
///
/// ```rust
/// use serde::Serialize;
/// use gsm_serde::serde_ini::{to_string, IniHeader};
///
/// #[derive(Serialize)]
/// struct Settings {
///     key: String,
/// }
///
/// impl IniHeader for Settings {
///     fn ini_header() -> &'static str {
///         "my_section"
///     }
/// }
///
/// let settings = Settings { key: "value".into() };
/// let ini_str = to_string(&settings).unwrap();
/// println!("{}", ini_str);
/// // Expected output:
/// // [my_section]
/// // key="value",
/// ```
///
/// Serializing a struct with nested objects (e.g., OptionSettings) will output a block:
///
/// ```rust
/// use serde::Serialize;
/// use gsm_serde::serde_ini::{to_string, IniHeader};
///
/// #[derive(Serialize)]
/// struct OptionSettings {
///     #[serde(rename = "Difficulty")]
///     difficulty: String,
///     #[serde(rename = "DayTimeSpeedRate")]
///     day_time_speed_rate: f32,
///     #[serde(rename = "NightTimeSpeedRate")]
///     night_time_speed_rate: f32,
/// }
///
/// #[derive(Serialize)]
/// #[serde(rename = "/Script/Pal.PalGameWorldSettings")]
/// struct GameSettings {
///     #[serde(rename = "OptionSettings")]
///     option_settings: OptionSettings,
/// }
///
/// impl IniHeader for GameSettings {
///     fn ini_header() -> &'static str {
///         "/Script/Pal.PalGameWorldSettings"
///     }
/// }
///
/// let settings = GameSettings {
///     option_settings: OptionSettings {
///         difficulty: "Hard".into(),
///         day_time_speed_rate: 1.5,
///         night_time_speed_rate: 0.8,
///     },
/// };
///
/// let ini_str = to_string(&settings).unwrap();
/// println!("{}", ini_str);
/// // Expected output:
/// // [/Script/Pal.PalGameWorldSettings]
/// // OptionSettings=(
/// // Difficulty="Hard",
/// // DayTimeSpeedRate=1.5,
/// // NightTimeSpeedRate=0.8,
/// // )
/// ```
pub fn to_string<T: Serialize + IniHeader>(value: &T) -> Result<String, serde_json::Error> {
    let mut output = String::new();

    // Write the header section.
    let section = T::ini_header();
    writeln!(&mut output, "[{section}]").unwrap();

    // Convert the value into a serde_json::Value.
    let serialized = serde_json::to_value(value)?;
    if let serde_json::Value::Object(map) = serialized {
        // Sort top-level keys alphabetically.
        let mut entries: Vec<(String, serde_json::Value)> = map.into_iter().collect();
        entries.sort_by(|(a, _), (b, _)| a.cmp(b));
        for (key, val) in entries {
            match val {
                serde_json::Value::Object(_) => {
                    // For nested objects, use the recursive helper with indent level 1.
                    writeln!(&mut output, "{key}=(").unwrap();
                    output.push_str(&serialize_value(&val, 1));
                    writeln!(&mut output, ")").unwrap();
                }
                _ => {
                    writeln!(&mut output, "{}={},", key, format_json_value(&val)).unwrap();
                }
            }
        }
    }

    Ok(output)
}

/// Helper: Parse a string value from INI into a proper JSON value.
///
/// If the value is unquoted, this helper attempts to parse it as an integer, float, or bool.
///
/// This is used during deserialization to recover the original types.
///
/// # Example
///
/// ```rust
/// use gsm_serde::serde_ini::parse_ini_value;
/// use serde_json::Value;
///
/// let v1 = parse_ini_value("1.5");
/// assert_eq!(v1, Value::Number(serde_json::Number::from_f64(1.5).unwrap()));
///
/// let v2 = parse_ini_value("\"Hello\"");
/// assert_eq!(v2, Value::String("Hello".into()));
/// ```
pub fn parse_ini_value(value: &str) -> serde_json::Value {
    let trimmed = value.trim();
    if trimmed.starts_with('\"') && trimmed.ends_with('\"') && trimmed.len() >= 2 {
        // Remove the surrounding quotes.
        let inner = &trimmed[1..trimmed.len() - 1];
        serde_json::Value::String(inner.to_string())
    } else if let Ok(i) = trimmed.parse::<i64>() {
        serde_json::Value::Number(i.into())
    } else if let Ok(f) = trimmed.parse::<f64>() {
        if let Some(n) = serde_json::Number::from_f64(f) {
            serde_json::Value::Number(n)
        } else {
            serde_json::Value::String(trimmed.to_string())
        }
    } else if trimmed.eq_ignore_ascii_case("true") {
        serde_json::Value::Bool(true)
    } else if trimmed.eq_ignore_ascii_case("false") {
        serde_json::Value::Bool(false)
    } else {
        serde_json::Value::String(trimmed.to_string())
    }
}

/// Deserializes an INI-formatted string into a struct.
///
/// This basic implementation supports a single header and one level of nested fields.
///
/// # Examples
///
/// Deserializing a simple settings struct:
///
/// ```rust
/// use serde::Deserialize;
/// use gsm_serde::serde_ini::from_str;
///
/// #[derive(Deserialize, Debug, PartialEq)]
/// struct Settings {
///     key: String,
/// }
///
/// let ini_str = "[my_section]\nkey=\"value\",\n";
/// let settings: Settings = from_str(ini_str).unwrap();
/// assert_eq!(settings.key, "value");
/// ```
///
/// Deserializing a nested settings struct:
///
/// ```rust
/// use serde::Deserialize;
/// use gsm_serde::serde_ini::from_str;
///
/// #[derive(Deserialize, Debug, PartialEq)]
/// struct OptionSettings {
///     #[serde(rename = "Difficulty")]
///     difficulty: String,
///     #[serde(rename = "DayTimeSpeedRate")]
///     day_time_speed_rate: f32,
///     #[serde(rename = "NightTimeSpeedRate")]
///     night_time_speed_rate: f32,
/// }
///
/// #[derive(Deserialize, Debug, PartialEq)]
/// struct GameSettings {
///     #[serde(rename = "OptionSettings")]
///     option_settings: OptionSettings,
/// }
///
/// let ini_str = "[/Script/Pal.PalGameWorldSettings]\n\
/// OptionSettings=(\n\
/// Difficulty=\"Hard\",\n\
/// DayTimeSpeedRate=1.5,\n\
/// NightTimeSpeedRate=0.8,\n\
/// )\n";
/// let settings: GameSettings = from_str(ini_str).unwrap();
/// assert_eq!(settings.option_settings.difficulty, "Hard");
/// ```
pub fn from_str<T: DeserializeOwned>(ini_str: &str) -> Result<T, serde_json::Error> {
    let mut map = serde_json::Map::new();
    let mut current_key: Option<String> = None;
    let mut nested_map = serde_json::Map::new();
    let mut in_nested = false;

    for line in ini_str.lines() {
        let line = line.trim();
        if line.starts_with('[') || line.is_empty() || line.starts_with(';') {
            continue; // Skip header and comment lines.
        }
        // Detect start of a nested block (e.g., OptionSettings=()
        if line.ends_with("=(") {
            let key = line.trim_end_matches("=(").trim().to_string();
            current_key = Some(key);
            in_nested = true;
            nested_map = serde_json::Map::new();
        } else if in_nested && line == ")" {
            if let Some(key) = current_key.take() {
                map.insert(key, serde_json::Value::Object(nested_map.clone()));
            }
            in_nested = false;
        } else if in_nested {
            // Process nested key=value lines (remove trailing commas).
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().trim_end_matches(',').to_string();
                nested_map.insert(key, parse_ini_value(&value));
            }
        } else if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().trim_end_matches(',').to_string();
            map.insert(key, parse_ini_value(&value));
        }
    }

    let json_value = serde_json::Value::Object(map);
    serde_json::from_value(json_value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ini_derive::IniSerialize;
    use serde::{Deserialize, Serialize};

    // Define the nested configuration.
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct OptionSettings {
        #[serde(rename = "Difficulty")]
        difficulty: String,
        #[serde(rename = "DayTimeSpeedRate")]
        day_time_speed_rate: f32,
        #[serde(rename = "NightTimeSpeedRate")]
        night_time_speed_rate: f32,
    }

    // Top-level configuration that uses the custom attribute.
    #[derive(Serialize, Deserialize, IniSerialize, Debug, PartialEq)]
    #[INIHeader(name = "/Script/Pal.PalGameWorldSettings")]
    struct GameSettings {
        #[serde(rename = "OptionSettings")]
        option_settings: OptionSettings,
    }

    #[test]
    fn test_ini_serialization_with_nested_struct() {
        let settings = GameSettings {
            option_settings: OptionSettings {
                difficulty: "Hard".to_string(),
                day_time_speed_rate: 1.5,
                night_time_speed_rate: 0.8, // even if the actual value is 0.800000011920929,
            },
        };

        let ini_string = to_string(&settings).unwrap();
        let expected_ini = "[/Script/Pal.PalGameWorldSettings]\n\
OptionSettings=(\n\
\tDayTimeSpeedRate=1.5,\n\
\tDifficulty=\"Hard\",\n\
\tNightTimeSpeedRate=0.8,\n\
)\n";
        assert_eq!(ini_string, expected_ini);
    }

    #[test]
    fn test_ini_deserialization_with_nested_struct() {
        let ini_string = "[/Script/Pal.PalGameWorldSettings]\n\
OptionSettings=(\n\
DayTimeSpeedRate=1.5,\n\
Difficulty=\"Hard\",\n\
NightTimeSpeedRate=0.8,\n\
)\n";
        let deserialized: GameSettings = from_str(ini_string).unwrap();
        let expected = GameSettings {
            option_settings: OptionSettings {
                difficulty: "Hard".to_string(),
                day_time_speed_rate: 1.5,
                night_time_speed_rate: 0.8,
            },
        };
        assert_eq!(deserialized, expected);
    }

    #[test]
    fn test_round_trip_with_nested_struct() {
        let settings = GameSettings {
            option_settings: OptionSettings {
                difficulty: "Hard".to_string(),
                day_time_speed_rate: 1.5,
                night_time_speed_rate: 0.8,
            },
        };

        let ini_string = to_string(&settings).unwrap();
        let deserialized: GameSettings = from_str(&ini_string).unwrap();
        assert_eq!(settings, deserialized);
    }
}
