//! Error type for this crate.

use std::num::{ParseFloatError, ParseIntError};

/// Result type used in this crate. Uses the crate's [`Error`] type.
pub type Result<T, E = Error> = core::result::Result<T, E>;

/// Error type used throughout this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A string was supposed to contain a number but there was a parsing error.
    #[error("invalid number: {}", .0)]
    InvalidNumber(#[from] ParseIntError),

    /// A string was supposed to contain a floating-point number but there was a parsing error.
    #[error("invalid floating-point number: {}", .0)]
    InvalidFloat(#[from] ParseFloatError),
}
