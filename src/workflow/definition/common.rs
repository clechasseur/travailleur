//! Common types used in workflow defitions.
//!
//! Corresponding JSON schema: [common.json](https://github.com/serverlessworkflow/specification/blob/v0.8/schema/common.json).

use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

use num::Zero;
use serde::{Deserialize, Serialize};

#[cfg(feature = "validate")]
use crate::detail::garde::{must_be_a_number, must_be_zero_or_greater};

/// Metadata information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
pub struct Metadata {
    /// Attached metadata, comprised of custom properties.
    #[serde(flatten)]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub meta: HashMap<String, String>,
}

/// A non-negative number, represented either as a number or as a string (that must contain a number).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(untagged)]
pub enum NonNegativeNumber<T>
where
    T: PartialOrd + Zero + Display + FromStr,
{
    /// Number representation
    Number(#[cfg_attr(feature = "validate", garde(custom(must_be_zero_or_greater::<T, _>)))] T),

    /// String representation (that must actually be a number).
    String(
        #[cfg_attr(feature = "validate", garde(length(min = 1), custom(must_be_a_number::<T, _, _>)))]
         String,
    ),
}

impl<T> NonNegativeNumber<T>
where
    T: PartialOrd + Zero + Display + FromStr + Copy,
    <T as FromStr>::Err: Into<crate::Error>,
{
    /// Returns the numeric value if possible.
    ///
    /// * If `self` is a [`Number`](Self::Number), the conversion is always possible.
    /// * If `self` is a [`String`](Self::String), the conversion can fail if the number was not
    ///   validated. If `validate` was called and succeeded, then conversion is always possible.
    pub fn value(&self) -> crate::Result<T> {
        match self {
            NonNegativeNumber::Number(n) => Ok(*n),
            NonNegativeNumber::String(s) => s.parse::<T>().map_err(|e| e.into()),
        }
    }
}

impl<T> Display for NonNegativeNumber<T>
where
    T: PartialOrd + Zero + Display + FromStr,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NonNegativeNumber::Number(n) => n.fmt(f),
            NonNegativeNumber::String(s) => <String as Display>::fmt(s, f),
        }
    }
}

/// Simple wrapper for a non-negative number.
///
/// Used to convert valid [`NonNegativeNumber`]s to valid values.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ValidatedNonNegativeNumber<T>(pub T);

impl<T> ValidatedNonNegativeNumber<T>
where
    T: Copy,
{
    pub fn value(&self) -> T {
        self.0
    }
}

impl<T> Deref for ValidatedNonNegativeNumber<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for ValidatedNonNegativeNumber<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Display for ValidatedNonNegativeNumber<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> From<T> for ValidatedNonNegativeNumber<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> From<&T> for ValidatedNonNegativeNumber<T>
where
    T: Copy,
{
    fn from(value: &T) -> Self {
        Self(*value)
    }
}

impl<T> TryFrom<NonNegativeNumber<T>> for ValidatedNonNegativeNumber<T>
where
    T: PartialOrd + Zero + Display + FromStr + Copy,
    <T as FromStr>::Err: Into<crate::Error>,
{
    type Error = crate::Error;

    fn try_from(value: NonNegativeNumber<T>) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

impl<T> TryFrom<&NonNegativeNumber<T>> for ValidatedNonNegativeNumber<T>
where
    T: PartialOrd + Zero + Display + FromStr + Copy,
    <T as FromStr>::Err: Into<crate::Error>,
{
    type Error = crate::Error;

    fn try_from(value: &NonNegativeNumber<T>) -> Result<Self, Self::Error> {
        Ok(Self(value.value()?))
    }
}

/// Possible execution modes for actions or workflows: either sequentially or in parallel.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionMode {
    /// Sequential execution
    Sequential,

    /// Parallel execution
    Parallel,
}

/// Possible invocation modes for actions or functions: either synchronously or asynchronously.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InvocationMode {
    /// Synchronous invocation
    Sync,

    /// Asynchronous invocation
    Async,
}
