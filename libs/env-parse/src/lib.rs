extern crate proc_macro;

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
