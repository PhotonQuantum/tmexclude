//! Error types.
use thiserror::Error;

/// Error that may occur when loading a config.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Error returned by figment.
    #[error("Error when loading config file: {0}")]
    Figment(#[from] figment::Error),
    /// Missing rule.
    #[error("Missing rule: {0}")]
    Rule(String),
    /// Specified path is invalid.
    #[error("Specified path is invalid: {path}")]
    InvalidPath {
        /// The invalid path.
        path: String,
        /// The underlying IO error.
        source: std::io::Error,
    },
}

/// Error that may occur when persisting state to disk.
#[derive(Debug, Error)]
pub enum PersistentError {
    /// Invalid json.
    #[error("Invalid json: {0}")]
    Json(#[from] serde_json::Error),
    /// IO error.
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
}
