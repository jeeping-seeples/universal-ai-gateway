use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    9471
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    #[serde(default = "default_default_model")]
    pub default_model: String,
    #[serde(default = "default_request_timeout_secs")]
    pub request_timeout_secs: u64,
}

fn default_default_model() -> String {
    "mock-gpt".to_string()
}

fn default_request_timeout_secs() -> u64 {
    30
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            default_model: default_default_model(),
            request_timeout_secs: default_request_timeout_secs(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub gateway: GatewayConfig,
}

impl AppConfig {
    pub fn load_from_str(content: &str) -> Result<Self, AppError> {
        let mut config: AppConfig = toml::from_str(content)
            .map_err(|e| AppError::Config(format!("Failed to parse config TOML: {}", e)))?;
        config.apply_env_overrides();
        config.validate()?;
        Ok(config)
    }

    pub fn load_from_file_or_default(path: Option<&Path>) -> Result<Self, AppError> {
        let mut config = if let Some(p) = path {
            if p.exists() {
                let content = fs::read_to_string(p).map_err(|e| {
                    AppError::Config(format!("Failed to read config file {}: {}", p.display(), e))
                })?;
                Self::load_from_str(&content)?
            } else {
                Self::default()
            }
        } else {
            Self::default()
        };

        config.apply_env_overrides();
        config.validate()?;
        Ok(config)
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(host) = env::var("GATEWAY_HOST") {
            if !host.trim().is_empty() {
                self.server.host = host.trim().to_string();
            }
        }

        if let Ok(port_str) = env::var("GATEWAY_PORT") {
            if let Ok(port) = port_str.trim().parse::<u16>() {
                self.server.port = port;
            }
        }

        if let Ok(model) = env::var("GATEWAY_DEFAULT_MODEL") {
            if !model.trim().is_empty() {
                self.gateway.default_model = model.trim().to_string();
            }
        }

        if let Ok(timeout_str) = env::var("GATEWAY_REQUEST_TIMEOUT_SECS") {
            if let Ok(timeout) = timeout_str.trim().parse::<u64>() {
                self.gateway.request_timeout_secs = timeout;
            }
        }
    }

    pub fn socket_addr(&self) -> Result<SocketAddr, AppError> {
        let addr_str = format!("{}:{}", self.server.host, self.server.port);
        addr_str
            .parse::<SocketAddr>()
            .map_err(|e| AppError::Config(format!("Invalid server address '{}': {}", addr_str, e)))
    }

    pub fn validate(&self) -> Result<(), AppError> {
        self.socket_addr()?;
        if self.gateway.default_model.trim().is_empty() {
            return Err(AppError::Config(
                "default_model cannot be empty".to_string(),
            ));
        }
        if self.gateway.request_timeout_secs == 0 {
            return Err(AppError::Config(
                "request_timeout_secs must be greater than 0".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 9471);
        assert_eq!(config.gateway.default_model, "mock-gpt");
        assert_eq!(config.gateway.request_timeout_secs, 30);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_parse_valid_toml() {
        let toml_str = r#"
            [server]
            host = "0.0.0.0"
            port = 9000

            [gateway]
            default_model = "custom-model"
            request_timeout_secs = 60
        "#;
        let config = AppConfig::load_from_str(toml_str).unwrap();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 9000);
        assert_eq!(config.gateway.default_model, "custom-model");
        assert_eq!(config.gateway.request_timeout_secs, 60);
    }

    #[test]
    fn test_invalid_address() {
        let toml_str = r#"
            [server]
            host = "not-an-ip"
            port = 9000
        "#;
        let result = AppConfig::load_from_str(toml_str);
        assert!(result.is_err());
    }
}
