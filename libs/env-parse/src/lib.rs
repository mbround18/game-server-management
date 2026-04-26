//! # Environment Variable Parsing Macro
//!
//! This crate provides a convenient macro for parsing environment variables into a specified type, with a fallback to a default value.
//!
//! The `env_parse!` macro simplifies the common pattern of reading an environment variable, parsing it, and using a default value if the variable is not set or parsing fails.
extern crate proc_macro;

/// Parses an environment variable into a specified type, falling back to a default value.
///
/// This macro attempts to read an environment variable, parse it into the given type (`$t`),
/// and returns the parsed value. If the environment variable is not set or if parsing fails,
/// it returns the provided default value (`$default`).
///
/// # Arguments
///
/// * `$env_var`: The name of the environment variable to parse (a string literal).
/// * `$default`: The default value to use if the environment variable is not present or parsing fails.
/// * `$t`: The target type to parse the environment variable into.
///
/// # Examples
///
/// ```
/// use env_parse::env_parse;
///
/// // Example 1: Parse a u32 value, with a default.
/// let port = env_parse!("PORT", 8080, u32);
///
/// // Example 2: Parse a String value.
/// let server_name = env_parse!("SERVER_NAME", String::from("localhost"), String);
/// ```
#[macro_export]
macro_rules! env_parse {
    ($env_var:expr, $default:expr, $t:ty) => {
        std::env::var($env_var)
            .ok()
            .and_then(|s| s.parse::<$t>().ok())
            .unwrap_or($default)
    };
}

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn returns_default_when_var_is_missing() {
        let _lock = env_lock().lock().unwrap_or_else(|e| e.into_inner());

        unsafe {
            std::env::remove_var("ENV_PARSE_MISSING_VALUE");
        }

        let value = env_parse!("ENV_PARSE_MISSING_VALUE", 42_u32, u32);
        assert_eq!(value, 42);
    }

    #[test]
    fn parses_value_when_var_is_present() {
        let _lock = env_lock().lock().unwrap_or_else(|e| e.into_inner());

        unsafe {
            std::env::set_var("ENV_PARSE_NUMERIC_VALUE", "99");
        }

        let value = env_parse!("ENV_PARSE_NUMERIC_VALUE", 7_u32, u32);
        assert_eq!(value, 99);

        unsafe {
            std::env::remove_var("ENV_PARSE_NUMERIC_VALUE");
        }
    }

    #[test]
    fn falls_back_to_default_for_invalid_values() {
        let _lock = env_lock().lock().unwrap_or_else(|e| e.into_inner());

        unsafe {
            std::env::set_var("ENV_PARSE_INVALID_VALUE", "not-a-number");
        }

        let value = env_parse!("ENV_PARSE_INVALID_VALUE", 5_i32, i32);
        assert_eq!(value, 5);

        unsafe {
            std::env::remove_var("ENV_PARSE_INVALID_VALUE");
        }
    }

    #[test]
    fn supports_string_values() {
        let _lock = env_lock().lock().unwrap_or_else(|e| e.into_inner());

        unsafe {
            std::env::set_var("ENV_PARSE_STRING_VALUE", "server-name");
        }

        let value = env_parse!("ENV_PARSE_STRING_VALUE", String::from("fallback"), String);
        assert_eq!(value, "server-name");

        unsafe {
            std::env::remove_var("ENV_PARSE_STRING_VALUE");
        }
    }
}
