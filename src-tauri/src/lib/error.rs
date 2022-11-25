use std::error::Error;

use thiserror::Error;

/// Error that may occur when loading a config.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Error returned by serde deserializer.
    #[error("Error when deserializing config file")]
    Deserialize(#[source] Box<dyn Error + Send + Sync>),
    /// Missing rule.
    #[error("Missing rule: {0}")]
    Rule(String),
    /// No directories in config.
    #[error("No directory to scan")]
    NoDirectory,
    /// Specified path is invalid.
    #[error("Specified path is invalid: {path}")]
    InvalidPath {
        /// The invalid path.
        path: String,
        /// The underlying IO error.
        source: std::io::Error,
    },
    /// Missing rule.
    #[error("Loop found in rules. Rendezvous point: {0}")]
    Loop(String),
    #[error("Error when reading/writing config file")]
    Load(#[from] ConfigIOError),
}

#[derive(Debug, Error)]
pub enum ConfigIOError {
    #[error("Home directory not found")]
    MissingHome,
    #[error("Failed to create config directory")]
    CreateConfigDir(#[source] std::io::Error),
    #[error("Failed to read config")]
    ReadConfig(#[source] std::io::Error),
    #[error("Failed to write config")]
    WriteConfig(#[source] std::io::Error),
    #[error("Error when deserializing config file")]
    Deserialize(#[source] Box<dyn Error + Send + Sync>),
    #[error("Error when serializing config file")]
    Serialize(#[source] Box<dyn Error + Send + Sync>),
}
