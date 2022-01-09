//! Error types.
use std::error::Error;
use std::fmt::{Display, Formatter};

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
    /// Error occur in factory.
    #[error("{0}")]
    Factory(#[source] Box<dyn Error + Send + Sync>),
}
use serde::{Deserialize, Serialize};

/// Represents a end-user friendly serializable error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedError {
    desc: String,
    cause: Option<Box<SerializedError>>,
}

impl Display for SerializedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.desc.as_str())
    }
}

impl Error for SerializedError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause.as_ref().map(|e| &**e as &dyn Error)
    }
}

impl<E> From<&E> for SerializedError
where
    E: Error + 'static,
{
    fn from(e: &E) -> Self {
        eyre::Chain::new(e)
            .rfold(None, |x, acc| {
                x.map_or_else(
                    || {
                        Some(Self {
                            desc: acc.to_string(),
                            cause: None,
                        })
                    },
                    |x| {
                        Some(Self {
                            desc: acc.to_string(),
                            cause: Some(Box::new(x)),
                        })
                    },
                )
            })
            .expect("must have one error")
    }
}