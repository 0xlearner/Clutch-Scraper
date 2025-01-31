use crate::error::{AppError, ConfigError, Result};
use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    Layer, Registry,
};

#[derive(Debug)]
pub struct LoggerConfig {
    pub directory: String,
    pub file_name: String,
    pub rotation: Rotation,
    pub level: Level,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            directory: "logs".to_string(),
            file_name: "scraper.log".to_string(),
            rotation: Rotation::DAILY,
            level: Level::INFO,
        }
    }
}

pub fn init_logging(config: LoggerConfig) -> Result<()> {
    // Create the log directory if it doesn't exist
    std::fs::create_dir_all(&config.directory).map_err(|e| {
        AppError::Config(ConfigError::FileRead(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to create log directory: {}", e),
        )))
    })?;

    // Set up file appender
    let file_appender =
        RollingFileAppender::new(config.rotation, config.directory, config.file_name);

    // Create a formatting layer for files
    let file_layer = fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_span_events(FmtSpan::FULL)
        .with_writer(file_appender)
        .with_target(true)
        .with_level(true)
        .with_ansi(false)
        .with_filter(tracing::level_filters::LevelFilter::from_level(
            config.level,
        ));

    // Create a formatting layer for stdout
    let stdout_layer = fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_target(true)
        .with_level(true)
        .with_ansi(true)
        .with_filter(tracing::level_filters::LevelFilter::from_level(
            config.level,
        ));

    // Combine both layers
    let subscriber = Registry::default().with(file_layer).with(stdout_layer);

    // Set the subscriber as the default
    tracing::subscriber::set_global_default(subscriber).map_err(|e| {
        AppError::Config(ConfigError::InvalidValue(format!(
            "Failed to set global subscriber: {}",
            e
        )))
    })?;

    Ok(())
}

// Helper function to parse log level from string
pub fn parse_log_level(level: &str) -> Result<Level> {
    match level.to_lowercase().as_str() {
        "trace" => Ok(Level::TRACE),
        "debug" => Ok(Level::DEBUG),
        "info" => Ok(Level::INFO),
        "warn" => Ok(Level::WARN),
        "error" => Ok(Level::ERROR),
        _ => Err(AppError::Config(ConfigError::InvalidValue(format!(
            "Invalid log level: {}",
            level
        )))),
    }
}

// Helper macros for consistent logging with error handling
#[macro_export]
macro_rules! log_error {
    // Handle AppError variants
    ($err:expr => $($arg:tt)*) => {{
        use tracing::error;
        use $crate::error::AppError;

        match $err {
            err @ AppError::Config(e) => error!(error = %err, kind = "config", $($arg)*),
            err @ AppError::Client(e) => error!(error = %err, kind = "client", $($arg)*),
            err @ AppError::Proxy(e) => error!(error = %err, kind = "proxy", $($arg)*),
            err @ AppError::Scraper(e) => error!(error = %err, kind = "scraper", $($arg)*),
            err @ AppError::Io(e) => error!(error = %err, kind = "io", $($arg)*),
            err @ AppError::Request(e) => error!(error = %err, kind = "request", $($arg)*),
            err @ AppError::Serde(e) => error!(error = %err, kind = "serde", $($arg)*),
        }
    }};
    // Handle regular string messages
    ($($arg:tt)*) => {
        tracing::error!($($arg)*)
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        tracing::warn!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        tracing::info!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        tracing::debug!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        tracing::trace!($($arg)*);
    };
}
