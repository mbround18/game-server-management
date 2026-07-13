use reqwest::blocking::Client;
use std::env::VarError;
use std::{env, fmt};
use tracing::{debug, error};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct IPResponse {
    ip: String,
}

pub struct IPConfig {
    pub(crate) ip: String,
    pub(crate) port: u16,
}

impl fmt::Display for IPConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.ip, self.port)
    }
}

impl IPConfig {
    const fn new(ip: String, port: u16) -> Self {
        Self { ip, port }
    }

    fn default() -> Self {
        Self::new("127.0.0.1".to_owned(), 2456)
    }

    fn get_ip_from_env() -> Result<String, VarError> {
        env::var("ADDRESS")
    }

    fn get_port_from_env() -> Result<u16, VarError> {
        env::var("PORT").and_then(|port| port.parse().map_err(|_| VarError::NotPresent))
    }

    /// Builds an [`IPConfig`] from `ADDRESS` and `PORT` environment variables.
    ///
    /// # Errors
    ///
    /// Returns `VarError::NotPresent` when either variable is absent/invalid, or when
    /// the resulting values are empty.
    pub fn to_string_from_env(&self) -> Result<Self, VarError> {
        match Self::get_ip_from_env() {
            Ok(ip) => match Self::get_port_from_env() {
                Ok(port) => {
                    if ip.is_empty() {
                        error!("IP address is empty");
                        Err(VarError::NotPresent)
                    } else {
                        Ok(Self::new(ip, port))
                    }
                }
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }

    /// Fetches the public IP from known API endpoints.
    ///
    /// # Errors
    ///
    /// Returns an error when all configured endpoints fail to return a parseable
    /// response.
    pub fn fetch_ip_from_api(&self, client: &Client) -> Result<String, Box<dyn std::error::Error>> {
        let urls = [
            "https://api.ipify.org?format=json",
            "https://api.seeip.org/jsonip?",
            "https://ipinfo.io",
        ];

        for url in urls {
            match client.get(url).send() {
                Ok(response) => match response.json::<IPResponse>() {
                    Ok(json) => return Ok(json.ip),
                    Err(e) => {
                        debug!("Failed to parse JSON from {}: {}", url, e);
                    }
                },
                Err(e) => {
                    debug!("Request to {} failed: {}", url, e);
                }
            }
        }

        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "All IP fetch attempts failed",
        )))
    }
}

// Standardized way of fetching public address.
pub fn fetch_public_address() -> IPConfig {
    let client = Client::new();
    let mut ip_config = IPConfig::default();
    debug!("Checking for address in env");
    match ip_config.to_string_from_env() {
        Ok(ip) => {
            debug!("Fetched IP from env: {}", ip);
            ip
        }
        Err(_) => match ip_config.fetch_ip_from_api(&client) {
            Ok(ip) => {
                debug!("Fetched IP from API: {}", ip);
                ip_config.ip = ip;
                ip_config
            }
            Err(e) => {
                debug!("Failed to fetch IP from API: {}", e);
                ip_config
            }
        },
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use std::env;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn test_to_string_from_env_success() {
        let _lock = env_lock().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let key_address = "ADDRESS";
        let key_port = "PORT";
        let expected_address = "192.168.1.100";
        let expected_port = "8080";
        // Unsafe blocks required for setting env vars in Rust 2024.
        unsafe {
            env::set_var(key_address, expected_address);
            env::set_var(key_port, expected_port);
        }
        let ip_config = IPConfig::default();
        let config = ip_config.to_string_from_env().unwrap();
        assert_eq!(config.ip, expected_address);
        assert_eq!(config.port, expected_port.parse::<u16>().unwrap());
        unsafe {
            env::remove_var(key_address);
            env::remove_var(key_port);
        }
    }

    #[test]
    fn test_to_string_from_env_rejects_invalid_values() {
        let _lock = env_lock().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let key_address = "ADDRESS";
        let key_port = "PORT";

        unsafe {
            env::set_var(key_address, "");
            env::set_var(key_port, "not-a-port");
        }

        let ip_config = IPConfig::default();
        assert!(ip_config.to_string_from_env().is_err());

        unsafe {
            env::remove_var(key_address);
            env::remove_var(key_port);
        }
    }

    #[test]
    fn test_fetch_public_address_uses_env_values() {
        let _lock = env_lock().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let key_address = "ADDRESS";
        let key_port = "PORT";

        unsafe {
            env::set_var(key_address, "10.0.0.12");
            env::set_var(key_port, "25565");
        }

        let config = fetch_public_address();
        assert_eq!(config.to_string(), "10.0.0.12:25565");

        unsafe {
            env::remove_var(key_address);
            env::remove_var(key_port);
        }
    }

    #[test]
    fn test_display_formats_ip_and_port() {
        let config = IPConfig::new("1.2.3.4".to_owned(), 1234);
        assert_eq!(config.to_string(), "1.2.3.4:1234");
    }
}
