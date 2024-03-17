//! Workflow error definitions
//!
//! Corresponding JSON schema: [errors.json](https://github.com/serverlessworkflow/specification/blob/v0.8/schema/errors.json).

use serde::{Deserialize, Serialize};

/// Workflow Error definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(untagged)]
pub enum Errors {
    /// URI to a resource containing error definitions (json or yaml)
    Uri(#[cfg_attr(feature = "validate", garde(url))] String),

    /// Workflow Error definitions.
    ///
    /// Defines checked errors that can be explicitly handled during workflow execution
    Inlined(#[cfg_attr(feature = "validate", garde(dive, length(min = 1)))] Vec<ErrorDef>),
}

/// Workflow Error definition.
///
/// Defines a checked error that can be explicitly handled during workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "validate", garde(allow_unvalidated))]
pub struct ErrorDef {
    /// Domain-specific error name
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub name: String,

    /// Error code.
    ///
    /// Can be used in addition to the name to help runtimes resolve to technical errors/exceptions.
    /// Should not be defined if error is set to '*'
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub code: Option<String>,

    /// Error description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
