use std::path::PathBuf;

use thiserror::Error;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error, PartialEq)]
pub enum ConfigError {
    #[error("Error when loading config file: {0}")]
    Figment(#[from] figment::Error),
    #[error("Missing rule: {0}")]
    Rule(String),
    #[error("Specified path does not exist: {0}")]
    NotFound(String),
    #[error("Specified path is not a directory: {0}")]
    NotADirectory(PathBuf),
}

#[derive(Debug, Error)]
pub enum PersistentError {
    #[error("Invalid json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
}
