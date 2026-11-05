use serde::{Deserialize, Serialize};
use std::env;
use std::fmt;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;

use crate::error::AppError;

#[derive(Clone, Serialize, Deserialize)]
pub struct SecretString(String);

impl SecretString {
    pub fn new(secret: impl Into<String>) -> Self {
        Self(secret.into())
    }

    pub fn expose_secret(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.trim().is_empty()
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            write!(f, "[EMPTY]")
        } else {
            write!(f, "[REDACTED]")
        }
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl From<String> for SecretString {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for SecretString {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

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
pub struct HostedProviderConfig {
    #[serde(default = "default_hosted_name")]
    pub name: String,
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default)]
    pub api_key: Option<SecretString>,
    #[serde(default = "default_hosted_models")]
    pub models: Vec<String>,
    #[serde(default)]
    pub enabled: bool,
}

fn default_hosted_name() -> String {
    "openai-compatible".to_string()
}

fn default_base_url() -> String {
    "https://api.openai.com/v1".to_string()
}

fn default_hosted_models() -> Vec<String> {
    vec!["gpt-4o-mini".to_string(), "gpt-4o".to_string()]
}

impl Default for HostedProviderConfig {
    fn default() -> Self {
        Self {
            name: default_hosted_name(),
            base_url: default_base_url(),
            api_key: None,
            models: default_hosted_models(),
            enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    #[serde(default = "default_default_model")]
    pub default_model: String,
    #[serde(default = "default_request_timeout_secs")]
    pub request_timeout_secs: u64,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default)]
    pub fallback_model: Option<String>,
}

fn default_default_model() -> String {
    "mock-gpt".to_string()
}

fn default_request_timeout_secs() -> u64 {
    30
}

fn default_max_retries() -> u32 {
    2
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            default_model: default_default_model(),
            request_timeout_secs: default_request_timeout_secs(),
            max_retries: default_max_retries(),
            fallback_model: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub gateway: GatewayConfig,
    #[serde(default)]
    pub hosted_provider: Option<HostedProviderConfig>,
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

        if let Ok(retries_str) = env::var("GATEWAY_MAX_RETRIES") {
            if let Ok(retries) = retries_str.trim().parse::<u32>() {
                self.gateway.max_retries = retries;
            }
        }

        if let Ok(fallback) = env::var("GATEWAY_FALLBACK_MODEL") {
            if !fallback.trim().is_empty() {
                self.gateway.fallback_model = Some(fallback.trim().to_string());
            }
        }

        // Environment overrides for hosted provider
        let env_api_key = env::var("OPENAI_API_KEY")
            .or_else(|_| env::var("GATEWAY_OPENAI_API_KEY"))
            .ok()
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty());

        let env_base_url = env::var("OPENAI_BASE_URL")
            .or_else(|_| env::var("GATEWAY_OPENAI_BASE_URL"))
            .ok()
            .map(|u| u.trim().to_string())
            .filter(|u| !u.is_empty());

        if env_api_key.is_some() || env_base_url.is_some() {
            let mut hosted = self.hosted_provider.take().unwrap_or_default();
            if let Some(key) = env_api_key {
                if hosted.api_key.is_none()
                    || hosted
                        .api_key
                        .as_ref()
                        .map(|k| k.is_empty())
                        .unwrap_or(true)
                {
                    hosted.api_key = Some(SecretString::new(key));
                }
            }
            if let Some(base_url) = env_base_url {
                hosted.base_url = base_url;
            }
            self.hosted_provider = Some(hosted);
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

        if let Some(ref hosted) = self.hosted_provider {
            if hosted.base_url.trim().is_empty() {
                return Err(AppError::Config(
                    "hosted_provider.base_url cannot be empty".to_string(),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_string_redaction() {
        let secret = SecretString::new("sk-1234567890abcdef");
        assert_eq!(format!("{:?}", secret), "[REDACTED]");
        assert_eq!(format!("{}", secret), "[REDACTED]");
        assert_eq!(secret.expose_secret(), "sk-1234567890abcdef");
    }

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 9471);
        assert_eq!(config.gateway.default_model, "mock-gpt");
        assert_eq!(config.gateway.request_timeout_secs, 30);
        assert_eq!(config.gateway.max_retries, 2);
        assert_eq!(config.gateway.fallback_model, None);
        assert!(config.hosted_provider.is_none());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_parse_hosted_provider_toml() {
        let toml_str = r#"
            [server]
            host = "127.0.0.1"
            port = 9471

            [gateway]
            default_model = "gpt-4o"
            request_timeout_secs = 45
            max_retries = 3
            fallback_model = "mock-gpt"

            [hosted_provider]
            name = "my-openai"
            base_url = "https://api.together.xyz/v1"
            api_key = "sk-custom-secret"
            models = ["gpt-4o", "llama-3"]
            enabled = true
        "#;
        let config = AppConfig::load_from_str(toml_str).unwrap();
        assert_eq!(config.gateway.default_model, "gpt-4o");
        assert_eq!(config.gateway.max_retries, 3);
        assert_eq!(config.gateway.fallback_model, Some("mock-gpt".to_string()));
        let hosted = config.hosted_provider.unwrap();
        assert_eq!(hosted.name, "my-openai");
        assert_eq!(hosted.base_url, "https://api.together.xyz/v1");
        assert_eq!(hosted.models, vec!["gpt-4o", "llama-3"]);
        assert!(hosted.enabled);
        assert_eq!(
            hosted.api_key.as_ref().unwrap().expose_secret(),
            "sk-custom-secret"
        );
        let debug_str = format!("{:?}", hosted);
        assert!(!debug_str.contains("sk-custom-secret"));
        assert!(debug_str.contains("[REDACTED]"));
    }
}
