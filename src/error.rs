//! Error type for this crate.

use std::convert::Infallible;
use std::io;
use std::num::{ParseFloatError, ParseIntError};

use url::Url;

use crate::detail::OptFrom;

/// Result type used in this crate. Uses the crate's [`Error`] type.
pub type Result<T, E = Error> = core::result::Result<T, E>;

/// Error type used throughout this crate.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    // --- Errors related to workflow definitions ---
    /// Workflow identifier was requested but was not found in workflow definition.
    #[error("workflow identifier not specified")]
    MissingIdentifier,

    /// A string was supposed to contain an integer number but there was a parsing error.
    #[error("invalid number: {}", .0)]
    InvalidInt(#[from] ParseIntError),

    /// A string was supposed to contain a floating-point number but there was a parsing error.
    #[error("invalid floating-point number: {}", .0)]
    InvalidFloat(#[from] ParseFloatError),

    /// One or more validation errors occurred.
    ///
    /// ### Note
    ///
    /// This variant can only occur if the `validate` feature is enabled.
    #[error("validation error(s): {}", .0)]
    ValidationFailed(
        #[cfg(feature = "validate")]
        #[from]
        garde::Report,
        #[cfg(not(feature = "yaml"))] crate::impossible::Impossible,
    ),

    // --- Errors related to loading/saving workflow definitions ---
    /// Error while parsing a URL/URI.
    #[error("invalid URL: {}", .0)]
    InvalidUrl(#[from] url::ParseError),

    /// A `file://` URI could not be turned into a [`Path`](std::path::Path).
    #[error("file:// URI '{}' contains invalid path", .file_uri)]
    InvalidPathInFileUri {
        /// The invalid `file://` URI.
        file_uri: Url,
    },

    /// URI scheme is unsupported.
    #[error("unsupported URI scheme: {}", .scheme)]
    UnsupportedUriScheme {
        /// URI scheme that is unsupported.
        scheme: String,
    },

    /// File format is unsupported.
    #[error("unsupported file format: .{}", .file_ext)]
    UnsupportedFileFormat {
        /// Extension of file that is unsupported.
        file_ext: String,
    },

    /// Conversion to/from JSON failed.
    #[error("JSON conversion failed: {}", .0)]
    JsonConversionFailed(#[from] serde_json::Error),

    /// Conversion to/from YAML failed.
    ///
    /// ### Note
    ///
    /// This variant can only occur if the `yaml` feature is enabled.
    #[error("YAML conversion failed: {}", .0)]
    YamlConversionFailed(
        #[cfg(feature = "yaml")]
        #[from]
        serde_yaml::Error,
        #[cfg(not(feature = "yaml"))] crate::impossible::Impossible,
    ),

    /// A file I/O error occurred.
    #[error("file I/O error: {}", .0)]
    FileIo(#[from] io::Error),

    // --- Errors related to caching of workflow definition objects ---
    /// A definition object was found in cache for a URI but is of the wrong type.
    #[error("error: cached object was expected to be of type '{}', actual type is '{}'", .expected_type, .actual_type)]
    InvalidCachedObjectType {
        /// The type of definition object that was expected (e.g. asked for by the caller).
        expected_type: &'static str,

        /// The type of the actual definition object found.
        actual_type: &'static str,
    },

    // --- Utility errors ---
    /// Operation is unsupported because a feature is disabled.
    #[error("unsupported operation, requires feature '{}'", .required_feature)]
    FeatureDisabled {
        /// Feature required for the operation to be supported.
        required_feature: &'static str,
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
