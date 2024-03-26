//! Error type for this crate.

use std::convert::Infallible;
use std::io;
use std::num::{ParseFloatError, ParseIntError};

use crate::detail::OptFrom;

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

    /// Error while parsing a URL/URI.
    #[error("URL parsing error: {}", .0)]
    Url(#[from] url::ParseError),

    /// One or more validation errors occurred.
    #[cfg(feature = "validate")]
    #[error("Validation error(s): {}", .0)]
    Validation(#[from] garde::Report),

    /// Operation is unsupported because a feature is disabled.
    #[error("unsupported operation, requires feature '{}'", .required_feature)]
    UnsupportedOperation {
        /// Feature required for the operation to be supported.
        required_feature: &'static str,
    },

    /// File format is unsupported.
    #[error("unsupported file format: .{}", .file_ext)]
    UnsupportedFileFormat {
        /// Extension of file that is unsupported.
        file_ext: String,
    },

    /// URI scheme is unsupported.
    #[error("unsupported URI scheme: {}", .scheme)]
    UnsupportedUriScheme {
        /// URI scheme that is unsupported.
        scheme: String,
    },

    /// A `file://` URI could not be turned into a [`Path`](std::path::Path).
    #[error("invalid file:// URI: {}", .0)]
    InvalidFileUri(String),

    /// Workflow identifier was requested but was not found in workflow definition.
    #[error("workflow identifier not specified")]
    MissingIdentifier,

    /// A definition object was found for a URI but is of the wrong type.
    #[error("invalid downcast")]
    InvalidDowncast {
        /// The type of definition object that was expected (e.g. asked for by the caller).
        expected_type: &'static str,

        /// The type of the actual definition object found.
        actual_type: &'static str,
    },
}

impl<E> OptFrom<E> for Error
where
    E: Into<Error>,
{
    fn opt_from(value: E) -> Option<Self> {
        Some(value.into())
    }
}

impl OptFrom<Infallible> for Error {
    fn opt_from(_value: Infallible) -> Option<Self> {
        None
    }
}
