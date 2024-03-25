//! Error type for this crate.

use std::io;
use std::num::{ParseFloatError, ParseIntError};

/// Result type used in this crate. Uses the crate's [`Error`] type.
pub type Result<T, E = Error> = core::result::Result<T, E>;

/// Error type used throughout this crate.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// A string was supposed to contain a number but there was a parsing error.
    #[error("invalid number: {}", .0)]
    InvalidNumber(#[from] ParseIntError),

    /// A string was supposed to contain a floating-point number but there was a parsing error.
    #[error("invalid floating-point number: {}", .0)]
    InvalidFloat(#[from] ParseFloatError),

    /// An I/O error occurred.
    #[error("I/O error: {}", .0)]
    Io(#[from] io::Error),

    /// A JSON-related error occurred.
    #[error("JSON error: {}", .0)]
    Json(#[from] serde_json::Error),

    /// A YAML-related error occurred.
    #[cfg(feature = "yaml")]
    #[error("YAML error: {}", .0)]
    Yaml(#[from] serde_yaml::Error),

    /// One or more validation errors occurred.
    #[cfg(feature = "validate")]
    #[error("Validation error(s): {}", .0)]
    Validation(#[from] garde::Report),

    /// Operation is unsupported because a feature is disabled.
    #[error("unsupported operation, requires feature '{}'", .0)]
    UnsupportedOperation(&'static str),

    /// File format is unsupported.
    #[error("unsupported file format: {}", .0)]
    UnsupportedFileFormat(String),

    /// Workflow identifier was requested but was not found in workflow definition.
    #[error("workflow identifier not specified")]
    MissingIdentifier,
}
