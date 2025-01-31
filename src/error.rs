use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Client error: {0}")]
    Client(#[from] ClientError),

    #[error("Proxy error: {0}")]
    Proxy(#[from] ProxyError),

    #[error("Scraping error: {0}")]
    Scraper(#[from] ScraperError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Request error: {0}")]
    Request(#[from] rquest::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    FileRead(#[from] std::io::Error),

    #[error("Failed to parse config: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("Missing required configuration: {0}")]
    MissingField(String),

    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Failed to build client: {0}")]
    BuildError(String),

    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("Response error {status_code}")]
    ResponseError { status_code: u16, message: String },

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}

#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("No working proxies available")]
    NoWorkingProxies,

    #[error("All proxies exhausted")]
    AllProxiesExhausted {
        failed_proxies: Vec<(String, String)>,
    },

    #[error("Proxy validation failed: {0}")]
    ValidationFailed(String),

    #[error("Proxy timeout: {0}")]
    TimeoutError(String),
}

#[derive(Error, Debug)]
pub enum ScraperError {
    #[error("Failed to parse HTML: {0}")]
    ParseError(String),

    #[error("Selector error: {0}")]
    SelectorError(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
