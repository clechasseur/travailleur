//! Serverless Workflow specification - retries schema
//!
//! Corresponding JSON schema: [retries.json](https://github.com/serverlessworkflow/specification/blob/v0.8/schema/retries.json).

use serde::{Deserialize, Serialize};

#[cfg(feature = "validate")]
use crate::detail::garde::must_be_optional_multiple_of;
use crate::workflow::definition::common::NonNegativeNumber;

/// Workflow Retry definitions.
///
/// Define retry strategies that can be referenced in states onError definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(untagged)]
pub enum Retries {
    /// URI to a resource containing retry definitions (json or yaml)
    Uri(#[cfg_attr(feature = "validate", garde(url))] String),

    /// Inline retry definitions
    Inline(#[cfg_attr(feature = "validate", garde(dive, length(min = 1)))] Vec<RetryDef>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RetryDef {
    /// Unique retry strategy name
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub name: String,

    /// Time delay between retry attempts (ISO 8601 duration format)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub delay: Option<String>,

    /// Maximum time delay between retry attempts (ISO 8601 duration format)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub max_delay: Option<String>,

    /// Static value by which the delay increases during each attempt (ISO 8601 time format)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub increment: Option<String>,

    /// Numeric value, if specified the delay between retries is multiplied by this value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive, custom(must_be_optional_multiple_of(0.01))))]
    pub multiplier: Option<NonNegativeNumber<f64>>,

    /// Maximum number of retry attempts.
    #[cfg_attr(feature = "validate", garde(dive))]
    pub max_attempts: NonNegativeNumber<i64>,

    /// Jitter value (see [`Jitter`]).
    #[cfg_attr(feature = "validate", garde(dive))]
    pub jitter: Option<Jitter>,
}

/// Retry definition jitter value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(untagged)]
pub enum Jitter {
    /// Maximum amount of random time added or subtracted from the delay between each retry relative to total delay (between 0 and 1)
    Float(#[cfg_attr(feature = "validate", garde(range(min = 0.0, max = 1.0)))] f64),

    /// Absolute maximum amount of random time added or subtracted from the delay between each retry (ISO 8601 duration format)
    Duration(#[cfg_attr(feature = "validate", garde(skip))] String),
}
