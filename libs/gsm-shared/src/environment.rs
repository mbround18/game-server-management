use crate::parse_truthy;
use cached::proc_macro::cached;
use std::env;

/// Fetches an environment variable, returning `default` if not set or empty.
pub fn fetch_var(name: &str, default: &str) -> String {
    match env::var(name) {
        Ok(value) if !value.is_empty() => value,
        _ => default.to_string(),
    }
}

/// Fetches an environment variable and, if non-empty, appends a colon to it.
pub fn fetch_multiple_var(name: &str, default: &str) -> String {
    let value = fetch_var(name, default);
    if value.is_empty() {
        value
    } else {
        format!("{value}:")
    }
}

/// Determines if the named environment variable is truthy.
/// Uses caching for improved performance.
#[cached]
pub fn is_env_var_truthy(name: &'static str) -> bool {
    parse_truthy(&fetch_var(name, "0")).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_fetch_var_set() {
        let key = "TEST_FETCH_VAR_SET";
        let expected = "value1";
        unsafe {
            env::set_var(key, expected);
        }
        let result = fetch_var(key, "default");
        assert_eq!(result, expected);
        unsafe {
            env::remove_var(key);
        }
    }

    #[test]
    fn test_fetch_var_not_set() {
        let key = "TEST_FETCH_VAR_NOT_SET";
        unsafe {
            env::remove_var(key);
        }
        let result = fetch_var(key, "default");
        assert_eq!(result, "default");
    }

    #[test]
    fn test_fetch_var_empty() {
        let key = "TEST_FETCH_VAR_EMPTY";
        unsafe {
            env::set_var(key, "");
        }
        let result = fetch_var(key, "default");
        assert_eq!(result, "default");
        unsafe {
            env::remove_var(key);
        }
    }

    #[test]
    fn test_fetch_multiple_var_non_empty() {
        let key = "TEST_FETCH_MULTIPLE_VAR_NON_EMPTY";
        let expected = "foo";
        unsafe {
            env::set_var(key, expected);
        }
        let result = fetch_multiple_var(key, "default");
        assert_eq!(result, format!("{expected}:"));
        unsafe {
            env::remove_var(key);
        }
    }

    #[test]
    fn test_fetch_multiple_var_empty() {
        let key = "TEST_FETCH_MULTIPLE_VAR_EMPTY_QFDIUJFGHNERF";
        unsafe {
            env::set_var(key, "");
        }
        // When the variable is empty, fetch_var returns the default.
        let result = fetch_multiple_var(key, "");
        assert_eq!(result, "");
        unsafe {
            env::remove_var(key);
        }
    }

    #[test]
    fn test_is_env_var_truthy_default() {
        let key = "TEST_IS_ENV_VAR_TRUTHY_DEFAULT";
        unsafe {
            env::remove_var(key);
        }
        // Not set defaults to "0", so false.
        assert!(!is_env_var_truthy(key));
    }

    #[test]
    fn test_is_env_var_truthy_truthy() {
        let key = "TEST_IS_ENV_VAR_TRUTHY_TRUTHY";
        unsafe {
            env::set_var(key, "true");
        }
        assert!(is_env_var_truthy(key));
        unsafe {
            env::remove_var(key);
        }
    }

    #[test]
    fn test_is_env_var_truthy_falsy() {
        let key = "TEST_IS_ENV_VAR_TRUTHY_FALSY";
        unsafe {
            env::set_var(key, "false");
        }
        assert!(!is_env_var_truthy(key));
        unsafe {
            env::remove_var(key);
        }
    }
}
