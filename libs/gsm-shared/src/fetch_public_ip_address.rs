use log::{debug, error};
use reqwest::blocking::Client;
use std::env::VarError;
use std::{env, fmt};

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
    fn new(ip: String, port: u16) -> IPConfig {
        IPConfig { ip, port }
    }

    fn default() -> IPConfig {
        IPConfig::new("127.0.0.1".to_string(), 2456)
    }

    fn get_ip_from_env(&self) -> Result<String, VarError> {
        env::var("ADDRESS")
    }

    fn get_port_from_env(&self) -> Result<u16, VarError> {
        env::var("PORT").map(|port| port.parse().unwrap())
    }

    pub fn to_string_from_env(&self) -> Result<IPConfig, VarError> {
        match self.get_ip_from_env() {
            Ok(ip) => match self.get_port_from_env() {
                Ok(port) => {
                    if ip.is_empty() {
                        error!("IP address is empty");
                        Err(VarError::NotPresent)
                    } else if port.to_string().is_empty() {
                        error!("Port is empty");
                        Err(VarError::NotPresent)
                    } else {
                        Ok(IPConfig::new(ip, port))
                    }
                }
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }

    pub fn fetch_ip_from_api(&self, client: &Client) -> Result<String, Box<dyn std::error::Error>> {
        let urls = [
            "https://api.ipify.org?format=json",
            "https://api.seeip.org/jsonip?",
            "https://ipinfo.io",
        ];

        for url in urls {
            match client.get(url).send() {
                Ok(response) => match response.json::<IPResponse>() {
                    Ok(json) => return Ok(json.ip.to_string()),
                    Err(e) => {
                        debug!("Failed to parse JSON from {}: {}", url, e);
                        continue;
                    }
                },
                Err(e) => {
                    debug!("Request to {} failed: {}", url, e);
                    continue;
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
    use super::*;
    use std::env;

    #[test]
    fn test_to_string_from_env_success() {
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
}
