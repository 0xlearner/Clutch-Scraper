use crate::error::{ConfigError, Result};
use serde::Deserialize;
use std::path::Path;
use tracing::info;

#[derive(Debug, Clone, Deserialize)]
pub struct LogConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_directory")]
    pub directory: String,
    #[serde(default = "default_log_filename")]
    pub filename: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProxyConfig {
    #[serde(default = "default_proxy_file")]
    pub file: String,
    #[serde(default = "default_proxy_switch_delay")]
    pub switch_delay: u64,
    #[serde(default = "default_proxy_max_retries")]
    pub max_retries: u32,
    #[serde(default = "default_proxy_request_timeout")]
    pub request_timeout: u64,
    #[serde(default = "default_proxy_concurrent_validations")]
    pub concurrent_validations: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_base_url")]
    pub base_url: String,

    #[serde(default = "default_proxy_file")]
    pub proxy_file: String,

    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    #[serde(default = "default_retry_delay")]
    pub retry_delay: u64,

    #[serde(default = "default_proxy_switch_delay")]
    pub proxy_switch_delay: u64,

    #[serde(default = "default_start_path")]
    pub start_path: String,

    #[serde(default = "default_proxy_max_retries")]
    pub proxy_max_retries: u32,

    #[serde(default = "default_proxy_request_timeout")]
    pub proxy_request_timeout: u64,

    #[serde(default = "default_proxy_concurrent_validations")]
    pub proxy_concurrent_validations: usize,

    #[serde(default)]
    pub logging: LogConfig,

    #[serde(default)]
    pub proxy: ProxyConfig,
}

// Default implementations
impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            file: default_proxy_file(),
            switch_delay: default_proxy_switch_delay(),
            max_retries: default_proxy_max_retries(),
            request_timeout: default_proxy_request_timeout(),
            concurrent_validations: default_proxy_concurrent_validations(),
        }
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            directory: default_log_directory(),
            filename: default_log_filename(),
        }
    }
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(ConfigError::FileRead)?;

        let config: Config = toml::from_str(&content).map_err(ConfigError::Parse)?;

        config.validate()?;
        info!("Configuration loaded successfully");
        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        // Validate base_url
        if self.base_url.is_empty() {
            return Err(ConfigError::MissingField("base_url".to_string()).into());
        }
        if !self.base_url.starts_with("http") {
            return Err(ConfigError::InvalidValue(format!(
                "base_url must start with http(s): {}",
                self.base_url
            ))
            .into());
        }

        // Validate proxy_file if provided
        if !self.proxy_file.is_empty() && !Path::new(&self.proxy_file).exists() {
            return Err(ConfigError::InvalidValue(format!(
                "proxy_file does not exist: {}",
                self.proxy_file
            ))
            .into());
        }

        if self.max_retries == 0 {
            return Err(ConfigError::InvalidValue(
                "max_retries must be greater than 0".to_string(),
            )
            .into());
        }

        if self.retry_delay == 0 {
            return Err(ConfigError::InvalidValue(
                "retry_delay must be greater than 0".to_string(),
            )
            .into());
        }

        if self.proxy_switch_delay == 0 {
            return Err(ConfigError::InvalidValue(
                "proxy_switch_delay must be greater than 0".to_string(),
            )
            .into());
        }

        if self.start_path.is_empty() {
            return Err(ConfigError::InvalidValue("start_path cannot be empty".to_string()).into());
        }

        if self.proxy_max_retries == 0 {
            return Err(ConfigError::InvalidValue(
                "proxy_max_retries must be greater than 0".to_string(),
            )
            .into());
        }

        if self.proxy_request_timeout == 0 {
            return Err(ConfigError::InvalidValue(
                "proxy_request_timeout must be greater than 0".to_string(),
            )
            .into());
        }

        if self.proxy_concurrent_validations == 0 {
            return Err(ConfigError::InvalidValue(
                "proxy_concurrent_validations must be greater than 0".to_string(),
            )
            .into());
        }

        Ok(())
    }
}

fn default_base_url() -> String {
    "https://clutch.co".to_string()
}

fn default_proxy_file() -> String {
    "proxy.txt".to_string()
}

fn default_max_retries() -> u32 {
    3
}

fn default_retry_delay() -> u64 {
    5
}

fn default_proxy_switch_delay() -> u64 {
    2
}

fn default_start_path() -> String {
    "/developers/rust".to_string()
}

fn default_proxy_max_retries() -> u32 {
    2
}

fn default_proxy_request_timeout() -> u64 {
    15
}

fn default_proxy_concurrent_validations() -> usize {
    5
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_directory() -> String {
    "logs".to_string()
}

fn default_log_filename() -> String {
    "scraper.log".to_string()
}
