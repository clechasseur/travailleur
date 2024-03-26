//! Serverless Workflow specification - secrets schema
//!
//! Corresponding JSON schema: [secrets.json](https://github.com/serverlessworkflow/specification/blob/v0.8/schema/secrets.json).

use serde::{Deserialize, Serialize};
use url::Url;

/// Workflow secrets definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(untagged)]
pub enum Secrets {
    /// URI to a resource containing secrets definitions (json or yaml)
    Uri(#[cfg_attr(feature = "validate", garde(skip))] Url),

    /// Workflow Secrets definitions
    Inline(#[cfg_attr(feature = "validate", garde(length(min = 1)))] Vec<String>),
}
