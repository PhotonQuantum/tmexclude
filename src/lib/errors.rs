//! Error types.
use std::error::Error;
use std::fmt::{Display, Formatter};

use eyre::Report;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use template_eyre::ext::Section;
use thiserror::Error;

/// Error that may occur when loading a config.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Error returned by figment.
    #[error("Error when loading config file")]
    Figment(#[from] figment::Error),
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
    /// Error occur in factory.
    #[error("{0}")]
    Factory(Report),
}

/// Represents a end-user friendly serializable error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedError {
    desc: String,
    cause: Option<Box<SerializedError>>,
    suggestion: Option<String>,
}

impl SerializedError {
    /// Convert `SerializedError` into `eyre::Report`.
    #[must_use]
    pub fn into_report(self) -> Report {
        let suggestion: Value = self.suggestion.clone().map(Into::into).unwrap_or_default();
        Report::new(self).section("suggestion", suggestion)
    }

    /// Convert an error into `SerializedError`.
    #[must_use]
    pub fn from_error(e: impl Into<Report>) -> Self {
        let report = e.into();
        let e = report
            .chain()
            .rfold(None, |acc, x| {
                acc.map_or_else(
                    || {
                        Some(Self {
                            desc: x.to_string(),
                            cause: None,
                            suggestion: None,
                        })
                    },
                    |acc| {
                        Some(Self {
                            desc: x.to_string(),
                            cause: Some(Box::new(acc)),
                            suggestion: None,
                        })
                    },
                )
            })
            .expect("must have one error");
        Self {
            desc: e.desc,
            cause: e.cause,
            suggestion: report.get_suggestion().map(ToString::to_string),
        }
    }
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

/// Helper trait to attach suggestions into error reports.
pub trait SuggestionExt: Section {
    /// Attach suggestion to errors.
    fn suggestion(self, suggestion: impl AsRef<str>) -> Self::Output;
    /// Attach suggestion to errors.
    fn with_suggestion<F, S>(self, suggestion: F) -> Self::Output
    where
        F: FnOnce() -> S,
        S: AsRef<str>;
    /// Get suggestion from errors.
    fn get_suggestion(&self) -> Option<&str>;
}

impl<T> SuggestionExt for T
where
    T: Section,
{
    fn suggestion(self, suggestion: impl AsRef<str>) -> Self::Output {
        self.section("suggestion", suggestion.as_ref())
    }
    fn with_suggestion<F, S>(self, suggestion: F) -> Self::Output
    where
        F: FnOnce() -> S,
        S: AsRef<str>,
    {
        self.section("suggestion", suggestion().as_ref())
    }
    fn get_suggestion(&self) -> Option<&str> {
        self.get_section_str("suggestion")
    }
}
