//! Workflow timeouts definitions
//!
//! Corresponding JSON schema: [timeouts.json](https://github.com/serverlessworkflow/specification/blob/v0.8/schema/timeouts.json).

use serde::{Deserialize, Serialize};

use crate::detail::true_value;

/// Workflow default timeouts definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(untagged, deny_unknown_fields)]
pub enum Timeouts {
    /// URI to a resource containing timeouts definitions (json or yaml)
    Uri(#[cfg_attr(feature = "validate", garde(url))] String),

    /// Workflow default timeouts
    #[serde(rename_all = "camelCase")]
    Complex {
        /// Workflow execution timeouts
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "validate", garde(dive))]
        workflow_exec_timeout: Option<WorkflowExecTimeout>,

        /// State execution timeouts
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "validate", garde(dive))]
        state_exec_timeout: Option<StateExecTimeout>,

        /// Action execution timeouts
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "validate", garde(dive))]
        action_exec_timeout: Option<ActionExecTimeout>,

        /// Branch execution timeouts
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "validate", garde(dive))]
        branch_exec_timeout: Option<BranchExecTimeout>,

        /// Event timeouts
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "validate", garde(dive))]
        event_timeout: Option<EventTimeout>,
    },
}

/// Workflow execution timeouts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(untagged, deny_unknown_fields)]
pub enum WorkflowExecTimeout {
    /// Workflow execution timeout duration (ISO 8601 duration format).
    ///
    /// If not specified should be 'unlimited'
    Simple(#[cfg_attr(feature = "validate", garde(length(min = 1)))] String),

    /// Workflow execution timeouts
    #[serde(rename_all = "camelCase")]
    Complex {
        /// Workflow execution timeout duration (ISO 8601 duration format).
        ///
        /// If not specified should be 'unlimited'
        #[cfg_attr(feature = "validate", garde(length(min = 1)))]
        duration: String,

        /// If `false`, workflow instance is allowed to finish current execution. If `true`, current workflow execution is abrupted.
        #[serde(default = "true_value")]
        #[cfg_attr(feature = "validate", garde(skip))]
        interrupt: bool,

        /// Name of a workflow state to be executed before workflow instance is terminated
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "validate", garde(length(min = 1)))]
        run_before: Option<String>,
    },
}

/// State execution timeouts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(untagged, deny_unknown_fields)]
pub enum StateExecTimeout {
    /// Total state execution timeout (including retries) (ISO 8601 duration format)
    Simple(#[cfg_attr(feature = "validate", garde(length(min = 1)))] String),

    /// Workflow default timeouts
    Complex {
        /// Single state execution timeout, not including retries (ISO 8601 duration format)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "validate", garde(length(min = 1)))]
        single: Option<String>,

        /// Total state execution timeout, including retries (ISO 8601 duration format)
        #[cfg_attr(feature = "validate", garde(length(min = 1)))]
        total: String,
    },
}

/// Single actions definition execution timeout duration (ISO 8601 duration format)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(transparent)]
pub struct ActionExecTimeout(#[cfg_attr(feature = "validate", garde(length(min = 1)))] pub String);

/// Single branch execution timeout duration (ISO 8601 duration format)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(transparent)]
pub struct BranchExecTimeout(#[cfg_attr(feature = "validate", garde(length(min = 1)))] pub String);

/// Timeout duration to wait for consuming defined events (ISO 8601 duration format)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(transparent)]
pub struct EventTimeout(#[cfg_attr(feature = "validate", garde(length(min = 1)))] pub String);
