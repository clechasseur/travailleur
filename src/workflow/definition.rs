//! Serverless Workflow specification - workflow schema
//!
//! Corresponding JSON schema: [workflow.json](https://github.com/serverlessworkflow/specification/blob/v0.8/schema/workflow.json).

pub mod auth;
pub mod common;
pub(crate) mod detail;
pub mod errors;
pub mod events;
pub mod functions;
pub mod retries;
pub mod secrets;
pub mod timeouts;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

#[cfg(feature = "validate")]
use crate::detail::garde::{
    must_be, one_of_three_must_be_set, one_of_two_must_be_set, unique_values,
};
use crate::detail::{all_of, false_value, jq, parallel, sequential, sync, terminate, true_value};
use crate::workflow::definition::auth::Auth;
use crate::workflow::definition::common::{
    ExecutionMode, InvocationMode, Metadata, NonNegativeNumber,
};
#[cfg(feature = "validate")]
use crate::workflow::definition::detail::garde::if_not_used_for_compensation_then_must_have_transition_or_end;
use crate::workflow::definition::errors::Errors;
use crate::workflow::definition::events::Events;
use crate::workflow::definition::functions::Functions;
use crate::workflow::definition::retries::Retries;
use crate::workflow::definition::secrets::Secrets;
use crate::workflow::definition::timeouts::{
    ActionExecTimeout, BranchExecTimeout, EventTimeout, StateExecTimeout, Timeouts,
    WorkflowExecTimeout,
};

/// Workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase")]
pub struct WorkflowDefinition {
    /// Workflow unique identifier
    #[serde(flatten)]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub identifier: Identifier,

    /// Workflow description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub description: Option<String>,

    /// Workflow version
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub version: Option<String>,

    /// List of helpful terms describing the workflows intended purpose, subject areas, or other important qualities
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub annotations: Option<Vec<String>>,

    /// JSON Schema used to validate the workflow data input
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub data_input_schema: Option<DataInputSchema>,

    /// Secrets definitions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub secrets: Option<Secrets>,

    /// Workflow constants
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub constants: Option<Constants>,

    /// Workflow start
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub start: Option<StartDef>,

    /// Serverless Workflow schema version
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub spec_version: String,

    /// Identifies the expression language used for workflow expressions. Default is 'jq'
    #[serde(default = "jq")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub expression_lang: String,

    /// Timeouts definitions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub timeouts: Option<Timeouts>,

    /// Error definitions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub errors: Option<Errors>,

    /// If 'true', workflow instances is not terminated when there are no active execution paths.
    /// Instance can be terminated via 'terminate end definition' or reaching defined 'workflowExecTimeout'
    #[serde(default = "false_value")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub keep_active: bool,

    /// Workflow metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub metadata: Option<Metadata>,

    /// Event definitions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub events: Option<Events>,

    /// Function definitions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub functions: Option<Functions>,

    /// If set to true, actions should automatically be retried on unchecked errors. Default is false
    #[serde(default = "false_value")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub auto_retries: bool,

    /// Retry definitions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub retries: Option<Retries>,

    /// Auth definitions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub auth: Option<Auth>,

    /// State definitions
    #[cfg_attr(feature = "validate", garde(dive, length(min = 1)))]
    pub states: Vec<State>,
}

impl WorkflowDefinition {
    /// Returns the name of the starting workflow [`State`].
    ///
    /// This is either the state pointed to by the [`start`] property or, if the property is
    /// not specified, the first state in the [`states`] array.
    ///
    /// # Return value
    ///
    /// | Value of [`start`] | Value of [`states`]           | Return value                |
    /// |--------------------|-------------------------------|-----------------------------|
    /// | `Some(state_name)` | `_`                           | `Some(state_name)`          |
    /// | `None`             | Non-empty array of [`State`]s | `Some(states.first().name)` |
    /// | `None`             | Empty array of [`State`]s     | `None`[^1]                  |
    ///
    /// [^1]: this is impossible if the workflow has been validated, because the workflow
    ///       would then be guaranteed to have at least one [`State`].
    ///
    /// [`start`]: Self::start
    /// [`states`]: Self::states
    pub fn start_state_name(&self) -> Option<&str> {
        self.start
            .as_ref()
            .map(StartDef::state_name)
            .or_else(|| self.states.first().map(State::name))
    }
}

/// Workflow identifier
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
pub struct Identifier {
    /// Workflow unique identifier
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1), custom(one_of_two_must_be_set("id", "key", self.key.as_ref()))))]
    pub id: Option<String>,

    /// Domain-specific workflow identifier
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub key: Option<String>,
}

impl Identifier {
    /// Returns the unique, domain-specific workflow identifier.
    ///
    /// # Return value
    ///
    /// | Value of [`id`] | Value of [`key`] | Return value             |
    /// |-----------------|------------------|--------------------------|
    /// | Some(id)        | _                | Ok(id)                   |
    /// | None            | Some(key)        | Ok(key)                  |
    /// | None            | None             | Err([MissingIdentifier]) |
    ///
    /// [`id`]: Self::id
    /// [`key`]: Self::key
    /// [MissingIdentifier]: crate::Error::MissingIdentifier
    pub fn id(&self) -> crate::Result<&str> {
        self.id
            .as_ref()
            .or(self.key.as_ref())
            .map(|id| id.as_str())
            .ok_or(crate::Error::MissingIdentifier)
    }
}

/// JSON Schema used to validate the workflow data input
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(untagged, deny_unknown_fields)]
pub enum DataInputSchema {
    /// URI of the JSON Schema used to validate the workflow data input
    UriOnly(#[cfg_attr(feature = "validate", garde(length(min = 1)))] String),

    /// Workflow data input schema definition
    #[serde(rename_all = "camelCase")]
    Full {
        /// URI of the JSON Schema used to validate the workflow data input
        #[cfg_attr(feature = "validate", garde(length(min = 1)))]
        schema: String,

        /// Determines if workflow execution should continue if there are validation errors
        #[serde(default = "true_value")]
        #[cfg_attr(feature = "validate", garde(skip))]
        fail_on_validation_errors: bool,
    },
}

/// Workflow constants
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(untagged)]
pub enum Constants {
    /// URI to a resource containing constants data (json or yaml)
    One(#[cfg_attr(feature = "validate", garde(skip))] Url),

    /// Workflow constants data (object type)
    Multiple {
        #[serde(flatten)]
        #[cfg_attr(feature = "validate", garde(skip))]
        constants: HashMap<String, Value>,
    },
}

/// Sleep time definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
pub struct Sleep {
    /// Amount of time (ISO 8601 duration format) to sleep before function/subflow invocation. Does not apply if 'eventRef' is defined.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(custom(one_of_two_must_be_set("before", "after", self.after.as_ref()))))]
    before: Option<String>,

    /// Amount of time (ISO 8601 duration format) to sleep after function/subflow invocation. Does not apply if 'eventRef' is defined.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(skip))]
    after: Option<String>,
}

/// Cron definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(untagged, deny_unknown_fields)]
pub enum CronDef {
    /// Cron expression defining when workflow instances should be created (automatically)
    Expr(#[cfg_attr(feature = "validate", garde(length(min = 1)))] String),

    /// Repeating cron definition
    #[serde(rename_all = "camelCase")]
    Repeat {
        /// Repeating interval (cron expression) describing when the workflow instance should be created
        #[cfg_attr(feature = "validate", garde(length(min = 1)))]
        expression: String,

        /// Specific date and time (ISO 8601 format) when the cron expression invocation is no longer valid
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "validate", garde(skip))]
        valid_until: Option<String>,
    },
}

/// "Continue as" definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(untagged)]
pub enum ContinueAsDef {
    Simple(#[cfg_attr(feature = "validate", garde(length(min = 1)))] String),

    #[serde(rename_all = "camelCase")]
    WithData {
        /// Unique id of the workflow to continue execution as
        #[cfg_attr(feature = "validate", garde(skip))]
        workflow_id: String,

        /// Version of the workflow to continue execution as
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "validate", garde(skip))]
        version: Option<String>,

        /// How to pass data to the workflow to continue execution as
        ///
        /// * If [`Expression`](Data::Expression), an expression which selects parts of the states data output to become the workflow data input of continued execution.
        /// * If [`Object`](Data::Object), a custom object to become the workflow data input of the continued execution.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "validate", garde(dive))]
        data: Option<Data>,

        /// Workflow execution timeout to be used by the workflow continuing execution. Overwrites any specific settings set by that workflow
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "validate", garde(dive))]
        workflow_exec_timeout: Option<WorkflowExecTimeout>,
    },
}

/// Data configuration
///
/// Determines how to pass data to an event or workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(untagged)]
pub enum Data {
    /// An expression which selects parts of the state's data to pass to the event or workflow.
    Expression(#[cfg_attr(feature = "validate", garde(skip))] String),

    /// A custom object to become the data to pass to the event or workflow.
    Object {
        #[serde(flatten)]
        #[cfg_attr(feature = "validate", garde(skip))]
        fields: HashMap<String, Value>,
    },
}

/// Transition definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(untagged, deny_unknown_fields)]
pub enum Transition {
    /// Name of state to transition to
    ByName(#[cfg_attr(feature = "validate", garde(length(min = 1)))] String),

    /// Function reference
    #[serde(rename_all = "camelCase")]
    Complex {
        /// Name of state to transition to
        #[cfg_attr(feature = "validate", garde(length(min = 1)))]
        next_state: String,

        /// Array of events to be produced before the transition happens
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "validate", garde(skip))]
        produce_events: Option<Vec<ProduceEventDef>>,

        /// If set to `true`, triggers workflow compensation when before this transition is taken. Default is `false`
        #[serde(default = "false_value")]
        #[cfg_attr(feature = "validate", garde(skip))]
        compensate: bool,
    },
}

/// Error definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Error {
    /// Reference to a unique workflow error definition. Used of errorRefs is not used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(custom(one_of_two_must_be_set("error_ref", "error_refs", self.error_refs.as_ref()))))]
    pub error_ref: Option<String>,

    /// References one or more workflow error definitions. Used if errorRef is not used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub error_refs: Option<Vec<String>>,

    /// Transition to next state to handle the error.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive, custom(one_of_two_must_be_set("transition", "end", self.end.as_ref()))))]
    pub transition: Option<Transition>,

    /// End workflow execution in case of this error.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub end: Option<End>,
}

/// OnEvents definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OnEvents {
    /// References one or more unique event names in the defined workflow events
    #[cfg_attr(feature = "validate", garde(length(min = 1), custom(unique_values)))]
    pub event_refs: Vec<String>,

    /// Specifies how actions are to be performed (in sequence or in parallel)
    #[serde(default = "sequential")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub action_mode: ExecutionMode,

    /// Actions to be performed if expression matches
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub actions: Option<Vec<Action>>,

    /// Event data filter
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub event_data_filter: Option<EventDataFilter>,
}

/// Workflow action definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Action {
    /// Unique action identifier
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub id: Option<String>,

    /// Unique action definition name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub name: Option<String>,

    /// References a function to be invoked
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive, custom(one_of_three_must_be_set(
        "function_ref",
        "event_ref",
        "sub_flow_ref",
        self.event_ref.as_ref(),
        self.sub_flow_ref.as_ref(),
    ))))]
    pub function_ref: Option<FunctionRef>,

    /// References a 'trigger' and 'result' reusable event definitions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub event_ref: Option<EventRef>,

    /// References a sub-workflow to invoke
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub sub_flow_ref: Option<SubflowRef>,

    /// Defines time periods workflow execution should sleep before / after function execution
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub sleep: Option<Sleep>,

    /// References a defined workflow retry definition.
    ///
    /// If not defined the default retry policy is assumed
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub retry_ref: Option<String>,

    /// List of unique references to defined workflow errors for which the action should not be retried.
    ///
    /// Used only when [`auto_retries`](WorkflowDefinition::auto_retries) is set to `true`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub non_retryable_errors: Option<Vec<String>>,

    /// List of unique references to defined workflow errors for which the action should be retried.
    ///
    /// Used only when [`auto_retries`](WorkflowDefinition::auto_retries) is set to `false`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub retryable_errors: Option<Vec<String>>,

    /// Action data filter
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub action_data_filter: Option<ActionDataFilter>,

    /// Expression, if defined, must evaluate to `true` for this action to be performed. If `false`, action is disregarded
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub condition: Option<String>,
}

/// Function reference definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(untagged, deny_unknown_fields)]
pub enum FunctionRef {
    /// Name of the referenced function
    ByName(#[cfg_attr(feature = "validate", garde(length(min = 1)))] String),

    /// Function reference
    #[serde(rename_all = "camelCase")]
    Complex {
        /// Name of the referenced function
        #[cfg_attr(feature = "validate", garde(skip))]
        ref_name: String,

        /// Function arguments/inputs
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "validate", garde(dive))]
        arguments: Option<FunctionArguments>,

        /// Only used if function type is 'graphql'. A string containing a valid GraphQL selection set
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "validate", garde(skip))]
        selection_set: Option<String>,

        /// Specifies if the function should be invoked sync or async
        #[serde(default = "sync")]
        #[cfg_attr(feature = "validate", garde(skip))]
        invoke: InvocationMode,
    },
}

/// Arguments passed to a function
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
pub struct FunctionArguments {
    #[serde(flatten)]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub arguments: HashMap<String, Value>,
}

/// Event References
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EventRef {
    /// Reference to the unique name of a 'produced' event definition
    #[cfg_attr(feature = "validate", garde(skip))]
    pub trigger_event_ref: String,

    /// Reference to the unique name of a 'consumed' event definition
    #[cfg_attr(feature = "validate", garde(skip))]
    pub result_event_ref: String,

    /// Maximum amount of time (ISO 8601 format) to wait for the result event.
    ///
    /// If not defined it should default to the `actionExecutionTimeout`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub result_event_timeout: Option<String>,

    /// How to pass data to the result event
    ///
    /// * If [`Expression`], an expression which selects parts of the states data output to become the data (payload) of the event referenced by [`trigger_event_ref`].
    /// * If [`Object`], a custom object to become the data (payload) of the event referenced by [`trigger_event_ref`].
    ///
    /// [`Expression`]: Data::Expression
    /// [`Object`]: Data::Object
    /// [`trigger_event_ref`]: EventRef::trigger_event_ref
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub data: Option<Data>,

    /// Add additional extension context attributes to the produced event
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub context_attributes: Option<ContextAttributes>,

    /// Specifies if the function should be invoked [`Sync`] or [`Async`]. Default is [`Sync`].
    ///
    /// [`Sync`]: InvocationMode::Sync
    /// [`Async`]: InvocationMode::Async
    #[serde(default = "sync")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub invoke: InvocationMode,
}

/// Event context attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
pub struct ContextAttributes {
    /// Context attributes
    #[serde(flatten)]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub attributes: HashMap<String, String>,
}

/// Sub-workflow reference definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(untagged)]
pub enum SubflowRef {
    /// Unique id of the sub-workflow to be invoked
    ById(#[cfg_attr(feature = "validate", garde(length(min = 1)))] String),

    /// Specifies a sub-workflow to be invoked
    #[serde(rename_all = "camelCase")]
    Complex {
        /// Unique id of the sub-workflow to be invoked
        #[cfg_attr(feature = "validate", garde(skip))]
        workflow_id: String,

        /// Version of the sub-workflow to be invoked
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "validate", garde(length(min = 1)))]
        version: Option<String>,

        /// If [`invoke`] is [`Async`], specifies how sub-workflow execution should behave when parent workflow completes. Default is [`Terminate`]
        ///
        /// [`invoke`]: Self::Complex::invoke
        /// [`Async`]: InvocationMode::Async
        /// [`Terminate`]: OnComplete::Terminate
        #[serde(default = "terminate")]
        #[cfg_attr(feature = "validate", garde(skip))]
        on_parent_complete: OnComplete,

        /// Specifies if the subflow should be invoked sync or async
        #[serde(default = "sync")]
        #[cfg_attr(feature = "validate", garde(skip))]
        invoke: InvocationMode,
    },
}

/// "On complete" sub-workflow behavior
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OnComplete {
    /// Sub-workflow should complete when parent workflow completes
    Continue,

    /// Sub-workflow should terminate when parent workflow completes
    Terminate,
}

/// Branch Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(deny_unknown_fields)]
pub struct Branch {
    /// Branch name
    #[cfg_attr(feature = "validate", garde(skip))]
    pub name: String,

    /// State specific timeouts
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub timeouts: Option<BranchTimeouts>,

    /// Actions to be executed in this branch
    #[cfg_attr(feature = "validate", garde(dive))]
    pub actions: Vec<Action>,
}

/// [`Branch`]-specific timeouts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase")]
pub struct BranchTimeouts {
    /// Action exec timeout
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub action_exec_timeout: Option<ActionExecTimeout>,

    /// Branch exec timeout
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub branch_exec_timeout: Option<BranchExecTimeout>,
}

/// Possible workflow states
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(tag = "type", rename_all = "lowercase")]
#[allow(clippy::large_enum_variant)] // All variants are similarly huge, so no big difference
pub enum State {
    /// Sleep state
    Sleep(#[cfg_attr(feature = "validate", garde(dive))] SleepState),

    /// Event state
    Event(#[cfg_attr(feature = "validate", garde(dive))] EventState),

    /// Operation state
    Operation(#[cfg_attr(feature = "validate", garde(dive))] OperationState),

    /// Parallel state
    Parallel(#[cfg_attr(feature = "validate", garde(dive))] ParallelState),

    /// Switch state
    Switch(#[cfg_attr(feature = "validate", garde(dive))] SwitchState),

    /// Inject state
    Inject(#[cfg_attr(feature = "validate", garde(dive))] InjectState),

    /// For-each state
    ForEach(#[cfg_attr(feature = "validate", garde(dive))] ForEachState),

    /// Callback state
    Callback(#[cfg_attr(feature = "validate", garde(dive))] CallbackState),
}

impl State {
    /// Returns the state name.
    pub fn name(&self) -> &str {
        match self {
            Self::Sleep(state) => state.name.as_str(),
            Self::Event(state) => state.name.as_str(),
            Self::Operation(state) => state.name.as_str(),
            Self::Parallel(state) => state.name.as_str(),
            Self::Switch(state) => match state {
                SwitchState::DataBased(state) => state.name.as_str(),
                SwitchState::EventBased(state) => state.name.as_str(),
            },
            Self::Inject(state) => state.name.as_str(),
            Self::ForEach(state) => state.name.as_str(),
            Self::Callback(state) => state.name.as_str(),
        }
    }
}

/// Causes the workflow execution to sleep for a specified duration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SleepState {
    /// Unique State id
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub id: Option<String>,

    /// State name
    #[cfg_attr(feature = "validate", garde(skip))]
    pub name: String,

    /// State end definition
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub end: Option<End>,

    /// State data filter
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub state_data_filter: Option<StateDataFilter>,

    /// Duration (ISO 8601 duration format) to sleep
    #[cfg_attr(feature = "validate", garde(skip))]
    pub duration: String,

    /// State specific timeouts
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub timeouts: Option<SleepStateTimeouts>,

    /// States error handling definitions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub on_errors: Option<Vec<Error>>,

    /// Next transition of the workflow after the workflow sleep
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub transition: Option<Transition>,

    /// Unique Name of a workflow state which is responsible for compensation of this state
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub compensated_by: Option<String>,

    /// If `true`, this state is used to compensate another state. Default is `false`
    #[serde(default = "false_value")]
    #[cfg_attr(feature = "validate", garde(
        custom(if_not_used_for_compensation_then_must_have_transition_or_end(&self.transition, &self.end))
    ))]
    pub used_for_compensation: bool,

    /// State metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub metadata: Option<Metadata>,
}

/// [`SleepState`]-specific timeouts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase")]
pub struct SleepStateTimeouts {
    /// State exec timeout
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub state_exec_timeout: Option<StateExecTimeout>,
}

/// This state is used to wait for events from event sources, then consumes them and invoke one or more actions to run in sequence or parallel
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EventState {
    /// Unique State id
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub id: Option<String>,

    /// State name
    #[cfg_attr(feature = "validate", garde(skip))]
    pub name: String,

    /// How events must be consumed for actions to be triggered
    ///
    /// * If `true`, consuming one of the defined events causes its associated actions to be performed.
    /// * If `false`, all of the defined events must be consumed in order for actions to be performed.
    #[serde(default = "true_value")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub exclusive: bool,

    /// Define the events to be consumed and optional actions to be performed
    #[cfg_attr(feature = "validate", garde(dive))]
    pub on_events: Vec<OnEvents>,

    /// State specific timeouts
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub timeouts: Option<EventStateTimeouts>,

    /// State data filter
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub state_data_filter: Option<StateDataFilter>,

    /// States error handling definitions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub on_errors: Option<Vec<Error>>,

    /// Next transition of the workflow after all the actions have been performed
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive, custom(one_of_two_must_be_set("transition", "end", self.end.as_ref()))))]
    pub transition: Option<Transition>,

    /// State end definition
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub end: Option<End>,

    /// Unique Name of a workflow state which is responsible for compensation of this state
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub compensated_by: Option<String>,

    /// State metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub metadata: Option<Metadata>,
}

/// [`EventState`]-specific timeouts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase")]
pub struct EventStateTimeouts {
    /// State exec timeout
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub state_exec_timeout: Option<StateExecTimeout>,

    /// Action exec timeout
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub action_exec_timeout: Option<ActionExecTimeout>,

    /// Event timeout
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub event_timeout: Option<EventTimeout>,
}

/// Defines actions be performed. Does not wait for incoming events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OperationState {
    /// Unique State id
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub id: Option<String>,

    /// State name
    #[cfg_attr(feature = "validate", garde(skip))]
    pub name: String,

    /// State end definition
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub end: Option<End>,

    /// State data filter
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub state_data_filter: Option<StateDataFilter>,

    /// Specifies whether actions are performed in sequence or in parallel
    #[serde(default = "sequential")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub action_mode: ExecutionMode,

    /// Actions to be performed
    #[cfg_attr(feature = "validate", garde(dive))]
    pub actions: Vec<Action>,

    /// State specific timeouts
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub timeouts: Option<OperationStateTimeouts>,

    /// States error handling definitions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub on_errors: Option<Vec<Error>>,

    /// Next transition of the workflow after all the actions have been performed
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub transition: Option<Transition>,

    /// Unique Name of a workflow state which is responsible for compensation of this state
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub compensated_by: Option<String>,

    /// If `true`, this state is used to compensate another state. Default is `false`
    #[serde(default = "false_value")]
    #[cfg_attr(feature = "validate", garde(custom(
        if_not_used_for_compensation_then_must_have_transition_or_end(&self.transition, &self.end)
    )))]
    pub used_for_compensation: bool,

    /// State metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub metadata: Option<Metadata>,
}

/// [`OperationState`]-specific timeouts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase")]
pub struct OperationStateTimeouts {
    /// State exec timeout
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub state_exec_timeout: Option<StateExecTimeout>,

    /// Action exec timeout
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub action_exec_timeout: Option<ActionExecTimeout>,
}

/// Consists of a number of states that are executed in parallel
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ParallelState {
    /// Unique State id
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub id: Option<String>,

    /// State name
    #[cfg_attr(feature = "validate", garde(skip))]
    pub name: String,

    /// State end definition
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub end: Option<End>,

    /// State data filter
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub state_data_filter: Option<StateDataFilter>,

    /// State specific timeouts
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub timeouts: Option<ParallelStateTimeouts>,

    /// Branch Definitions
    #[cfg_attr(feature = "validate", garde(dive))]
    pub branches: Vec<Branch>,

    /// Option types on how to complete branch execution.
    #[serde(default = "all_of")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub completion_type: CompletionType,

    /// Used when [`completion_type`] is set to [`AtLeast`] to specify the minimum number of branches that must complete before the state will transition.
    ///
    /// [`completion_type`]: ParallelState::completion_type
    /// [`AtLeast`]: CompletionType::AtLeast
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub num_completed: Option<NonNegativeNumber<i64>>,

    /// States error handling definitions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub on_errors: Option<Vec<Error>>,

    /// Next transition of the workflow after all branches have completed execution
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub transition: Option<Transition>,

    /// Unique Name of a workflow state which is responsible for compensation of this state
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub compensated_by: Option<String>,

    /// If `true`, this state is used to compensate another state. Default is `false`
    #[serde(default = "false_value")]
    #[cfg_attr(feature = "validate", garde(custom(
        if_not_used_for_compensation_then_must_have_transition_or_end(&self.transition, &self.end)
    )))]
    pub used_for_compensation: bool,

    /// State metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub metadata: Option<Metadata>,
}

/// [`ParallelState`]-specific timeouts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase")]
pub struct ParallelStateTimeouts {
    /// State exec timeout
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub state_exec_timeout: Option<StateExecTimeout>,

    /// Branch exec timeout
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub branch_exec_timeout: Option<BranchExecTimeout>,
}

/// Completion type values
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CompletionType {
    /// All branches must be completed
    AllOf,

    /// At least a specific number of branches must be completed
    AtLeast,
}

/// Permits transitions to other states based on events or data conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(untagged)]
pub enum SwitchState {
    /// Transition based on events
    EventBased(#[cfg_attr(feature = "validate", garde(dive))] EventBasedSwitchState),

    /// Transition based on data conditions
    DataBased(#[cfg_attr(feature = "validate", garde(dive))] DataBasedSwitchState),
}

/// Permits transitions to other states based on events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EventBasedSwitchState {
    /// Unique State id
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub id: Option<String>,

    /// State name
    #[cfg_attr(feature = "validate", garde(skip))]
    pub name: String,

    /// State data filter
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub state_data_filter: Option<StateDataFilter>,

    /// State specific timeouts
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub timeouts: Option<EventBasedSwitchStateTimeouts>,

    /// Defines conditions evaluated against events
    #[cfg_attr(feature = "validate", garde(dive))]
    pub event_conditions: Vec<EventCondition>,

    /// States error handling definitions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub on_errors: Option<Vec<Error>>,

    /// Default transition of the workflow if there is no matching data conditions.
    /// Can include a [`transition`] or [`end`] definition.
    ///
    /// [`transition`]: DefaultConditionDef::transition
    /// [`end`]: DefaultConditionDef::end
    #[cfg_attr(feature = "validate", garde(dive))]
    pub default_condition: DefaultConditionDef,

    /// Unique Name of a workflow state which is responsible for compensation of this state
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub compensated_by: Option<String>,

    /// If `true`, this state is used to compensate another state. Default is `false`
    #[serde(default = "false_value")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub used_for_compensation: bool,

    /// State metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub metadata: Option<Metadata>,
}

/// [`EventBasedSwitchState`]-specific timeouts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase")]
pub struct EventBasedSwitchStateTimeouts {
    /// State exec timeout
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub state_exec_timeout: Option<StateExecTimeout>,

    /// Event timeout
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub event_timeout: Option<EventTimeout>,
}

/// Permits transitions to other states based on data conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DataBasedSwitchState {
    /// Unique State id
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub id: Option<String>,

    /// State name
    #[cfg_attr(feature = "validate", garde(skip))]
    pub name: String,

    /// State data filter
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub state_data_filter: Option<StateDataFilter>,

    /// State specific timeouts
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub timeouts: Option<DataBasedSwitchStateTimeouts>,

    /// Defines conditions evaluated against state data
    #[cfg_attr(feature = "validate", garde(dive))]
    pub data_conditions: Vec<DataCondition>,

    /// States error handling definitions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub on_errors: Option<Vec<Error>>,

    /// Default transition of the workflow if there is no matching data conditions.
    /// Can include a [`transition`] or [`end`] definition.
    ///
    /// [`transition`]: DefaultConditionDef::transition
    /// [`end`]: DefaultConditionDef::end
    #[cfg_attr(feature = "validate", garde(dive))]
    pub default_condition: DefaultConditionDef,

    /// Unique Name of a workflow state which is responsible for compensation of this state
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub compensated_by: Option<String>,

    /// If `true`, this state is used to compensate another state. Default is `false`
    #[serde(default = "false_value")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub used_for_compensation: bool,

    /// State metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub metadata: Option<Metadata>,
}

/// [`DataBasedSwitchState`]-specific timeouts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase")]
pub struct DataBasedSwitchStateTimeouts {
    /// State exec timeout
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub state_exec_timeout: Option<StateExecTimeout>,
}

/// DefaultCondition definition. Can be either a [`transition`] or [`end`] definition
///
/// [`transition`]: Self::transition
/// [`end`]: Self::end
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(deny_unknown_fields)]
pub struct DefaultConditionDef {
    /// Transition definition
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive, custom(one_of_two_must_be_set("transition", "end", self.end.as_ref()))))]
    pub transition: Option<Transition>,

    /// End definition
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub end: Option<End>,
}

/// Switch state data event condition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(untagged)]
pub enum EventCondition {
    /// Transition condition
    Transition(#[cfg_attr(feature = "validate", garde(dive))] TransitionEventCondition),

    /// End condition
    End(#[cfg_attr(feature = "validate", garde(dive))] EndEventCondition),
}

/// Switch state data event condition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TransitionEventCondition {
    /// Event condition name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub name: Option<String>,

    /// References an unique event name in the defined workflow events
    #[cfg_attr(feature = "validate", garde(skip))]
    pub event_ref: String,

    /// Next transition of the workflow if there is valid matches
    #[cfg_attr(feature = "validate", garde(dive))]
    pub transition: Transition,

    /// Event data filter definition
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub event_data_filter: Option<EventDataFilter>,

    /// Condition metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub metadata: Option<Metadata>,
}

/// Switch state data event condition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EndEventCondition {
    /// Event condition name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub name: Option<String>,

    /// References an unique event name in the defined workflow events
    #[cfg_attr(feature = "validate", garde(skip))]
    pub event_ref: String,

    /// Explicit transition to end
    #[cfg_attr(feature = "validate", garde(dive))]
    pub end: End,

    /// Event data filter definition
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub event_data_filter: Option<EventDataFilter>,

    /// Condition metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub metadata: Option<Metadata>,
}

/// Switch state data based condition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(untagged)]
pub enum DataCondition {
    /// Transition condition
    Transition(#[cfg_attr(feature = "validate", garde(dive))] TransitionDataCondition),

    /// End condition
    End(#[cfg_attr(feature = "validate", garde(dive))] EndDataCondition),
}

/// Switch state data based condition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TransitionDataCondition {
    /// Data condition name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub name: Option<String>,

    /// Workflow expression evaluated against state data. Must evaluate to `true` or `false`
    #[cfg_attr(feature = "validate", garde(skip))]
    pub condition: String,

    /// Workflow transition if condition is evaluated to `true`
    #[cfg_attr(feature = "validate", garde(dive))]
    pub transition: Transition,

    /// Condition metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub metadata: Option<Metadata>,
}

/// Switch state data based condition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EndDataCondition {
    /// Data condition name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub name: Option<String>,

    /// Workflow expression evaluated against state data. Must evaluate to `true` or `false`
    #[cfg_attr(feature = "validate", garde(skip))]
    pub condition: String,

    /// Workflow end definition
    #[cfg_attr(feature = "validate", garde(dive))]
    pub end: End,

    /// Condition metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub metadata: Option<Metadata>,
}

/// Inject static data into state data. Does not perform any actions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InjectState {
    /// Unique State id
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub id: Option<String>,

    /// State name
    #[cfg_attr(feature = "validate", garde(skip))]
    pub name: String,

    /// State end definition
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub end: Option<End>,

    /// JSON object which can be set as states data input and can be manipulated via filters
    #[cfg_attr(feature = "validate", garde(dive))]
    pub data: InjectData,

    /// State specific timeouts
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub timeouts: Option<InjectStateTimeouts>,

    /// State data filter
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub state_data_filter: Option<StateDataFilter>,

    /// Next transition of the workflow after injection has completed
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub transition: Option<Transition>,

    /// Unique Name of a workflow state which is responsible for compensation of this state
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub compensated_by: Option<String>,

    /// If `true`, this state is used to compensate another state. Default is `false`
    #[serde(default = "false_value")]
    #[cfg_attr(feature = "validate", garde(custom(
        if_not_used_for_compensation_then_must_have_transition_or_end(&self.transition, &self.end)
    )))]
    pub used_for_compensation: bool,

    /// State metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub metadata: Option<Metadata>,
}

/// Data to be injected by an [`InjectState`] (see [`data`]).
///
/// [`data`]: InjectState::data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
pub struct InjectData {
    /// Data fields
    #[serde(flatten)]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub meta: HashMap<String, Value>,
}

/// [`InjectState`]-specific timeouts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase")]
pub struct InjectStateTimeouts {
    /// State exec timeout
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub state_exec_timeout: Option<StateExecTimeout>,
}

/// Execute a set of defined actions or workflows for each element of a data array
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ForEachState {
    /// Unique State id
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub id: Option<String>,

    /// State name
    #[cfg_attr(feature = "validate", garde(skip))]
    pub name: String,

    /// State end definition
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub end: Option<End>,

    /// Workflow expression selecting an array element of the states data
    #[cfg_attr(feature = "validate", garde(skip))]
    pub input_collection: String,

    /// Workflow expression specifying an array element of the states data to add the results of each iteration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub output_collection: Option<String>,

    /// Name of the iteration parameter that can be referenced in actions/workflow.
    ///
    /// For each parallel iteration, this param should contain an unique element of the [`input_collection`] array
    ///
    /// [`input_collection`]: Self::input_collection
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub iteration_param: Option<String>,

    /// Specifies how many iterations may run in parallel at the same time.
    ///
    /// Used if [`mode`] property is set to [`Parallel`] (default)
    ///
    /// [`mode`]: Self::mode
    /// [`Parallel`]: ExecutionMode::Parallel
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub batch_size: Option<NonNegativeNumber<i64>>,

    /// Actions to be executed for each of the elements of [`input_collection`]
    ///
    /// [`input_collection`]: Self::input_collection
    #[cfg_attr(feature = "validate", garde(dive))]
    pub actions: Vec<Action>,

    /// State specific timeouts
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub timeouts: Option<ForEachStateTimeouts>,

    /// State data filter
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub state_data_filter: Option<StateDataFilter>,

    /// States error handling definitions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub on_errors: Option<Vec<Error>>,

    /// Next transition of the workflow after state has completed
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub transition: Option<Transition>,

    /// Unique Name of a workflow state which is responsible for compensation of this state
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub compensated_by: Option<String>,

    /// If `true`, this state is used to compensate another state. Default is `false`
    #[serde(default = "false_value")]
    #[cfg_attr(feature = "validate", garde(custom(
        if_not_used_for_compensation_then_must_have_transition_or_end(&self.transition, &self.end)
    )))]
    pub used_for_compensation: bool,

    /// Specifies how iterations are to be performed (sequentially or in parallel)
    #[serde(default = "parallel")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub mode: ExecutionMode,

    /// State metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub metadata: Option<Metadata>,
}

/// [`ForEachState`]-specific timeouts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase")]
pub struct ForEachStateTimeouts {
    /// State exec timeout
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub state_exec_timeout: Option<StateExecTimeout>,

    /// Action exec timeout
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub action_exec_timeout: Option<ActionExecTimeout>,
}

/// This state performs an action, then waits for the callback event that denotes completion of the action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CallbackState {
    /// Unique State id
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub id: Option<String>,

    /// State name
    #[cfg_attr(feature = "validate", garde(skip))]
    pub name: String,

    /// Defines the action to be executed
    #[cfg_attr(feature = "validate", garde(dive))]
    pub action: Action,

    /// References an unique callback event name in the defined workflow events
    #[cfg_attr(feature = "validate", garde(skip))]
    pub event_ref: String,

    /// State specific timeouts
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub timeouts: Option<CallbackStateTimeouts>,

    /// Event data filter
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub event_data_filter: Option<EventDataFilter>,

    /// State data filter
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub state_data_filter: Option<StateDataFilter>,

    /// States error handling definitions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub on_errors: Option<Vec<Error>>,

    /// Next transition of the workflow after all the actions have been performed
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub transition: Option<Transition>,

    /// State end definition
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub end: Option<End>,

    /// Unique Name of a workflow state which is responsible for compensation of this state
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub compensated_by: Option<String>,

    /// If `true`, this state is used to compensate another state. Default is `false`
    #[serde(default = "false_value")]
    #[cfg_attr(feature = "validate", garde(custom(
        if_not_used_for_compensation_then_must_have_transition_or_end(&self.transition, &self.end)
    )))]
    pub used_for_compensation: bool,

    /// State metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub metadata: Option<Metadata>,
}

/// [`CallbackState`]-specific timeouts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase")]
pub struct CallbackStateTimeouts {
    /// State exec timeout
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub state_exec_timeout: Option<StateExecTimeout>,

    /// Action exec timeout
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub action_exec_timeout: Option<ActionExecTimeout>,

    /// Event timeout
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub event_timeout: Option<EventTimeout>,
}

/// Workflow start definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(untagged, deny_unknown_fields)]
pub enum StartDef {
    ByName(#[cfg_attr(feature = "validate", garde(length(min = 1)))] String),

    #[serde(rename_all = "camelCase")]
    Complex {
        /// Name of the starting workflow state
        #[cfg_attr(feature = "validate", garde(length(min = 1)))]
        state_name: String,

        /// Define the time/repeating intervals or cron at which workflow instances should be automatically started.
        #[cfg_attr(feature = "validate", garde(dive))]
        schedule: Schedule,
    },
}

impl StartDef {
    /// Returns the start state's name.
    pub fn state_name(&self) -> &str {
        match self {
            Self::ByName(state_name) => state_name.as_str(),
            Self::Complex { state_name, .. } => state_name.as_ref(),
        }
    }

    /// Returns the [`Schedule`] to use to start the workflow.
    ///
    /// Will return `None` if the workflow does not have a start schedule.
    pub fn schedule(&self) -> Option<&Schedule> {
        match self {
            Self::ByName(_) => None,
            Self::Complex { schedule, .. } => Some(schedule),
        }
    }
}

/// Schedule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(untagged, deny_unknown_fields)]
pub enum Schedule {
    /// Time interval (must be repeating interval) described with ISO 8601 format.
    ///
    /// Declares when workflow instances will be automatically created.  (UTC timezone is assumed)
    TimeInterval(#[cfg_attr(feature = "validate", garde(length(min = 1)))] String),

    /// Start state schedule definition
    Complex {
        /// Time interval (must be repeating interval) described with ISO 8601 format.
        ///
        /// Declares when workflow instances will be automatically created.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "validate", garde(length(min = 1)))]
        interval: Option<String>,

        /// Cron definition
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "validate", garde(dive, custom(one_of_two_must_be_set("cron", "interval", self.interval()))))]
        cron: Option<CronDef>,

        /// Timezone name used to evaluate the interval & cron-expression. (default: UTC)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "validate", garde(skip))]
        timezone: Option<String>,
    },
}

impl Schedule {
    /// Returns the schedule's interval.
    ///
    /// * If `self` is [`TimeInterval`], returns the contained interval value (in `Some`).
    /// * If `self` is [`Complex`], returns [`Complex::interval`].
    ///
    /// [`TimeInterval`]: Self::TimeInterval
    /// [`Complex`]: Self::Complex
    /// [`Complex::interval`]: Self::Complex::interval
    pub fn interval(&self) -> Option<&String> {
        match self {
            Self::TimeInterval(interval) => Some(interval),
            Self::Complex { interval, .. } => interval.as_ref(),
        }
    }
}

/// State end definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(untagged, deny_unknown_fields)]
pub enum End {
    /// Simple end definition.
    ///
    /// When this is used, the bool value must be `true`.
    Simple(#[cfg_attr(feature = "validate", garde(custom(must_be(true))))] bool),

    /// Complex end definition.
    #[serde(rename_all = "camelCase")]
    Complex {
        /// If `true`, completes all execution flows in the given workflow instance
        #[serde(default = "false_value")]
        #[cfg_attr(feature = "validate", garde(skip))]
        terminate: bool,

        /// Defines events that should be produced
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "validate", garde(dive))]
        produce_events: Option<Vec<ProduceEventDef>>,

        /// If set to `true`, triggers workflow compensation. Default is `false`
        #[serde(default = "false_value")]
        #[cfg_attr(feature = "validate", garde(skip))]
        compensate: bool,

        /// "Continue as" config
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "validate", garde(dive))]
        continue_as: Option<ContinueAsDef>,
    },
}

/// Produce an event and set its data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProduceEventDef {
    /// References a name of a defined event
    #[cfg_attr(feature = "validate", garde(skip))]
    pub event_ref: String,

    /// Event data
    ///
    /// * If [`Expression`](Data::Expression), expression which selects parts of the states data output to become the data of the produced event.
    /// * If [`Object`](Data::Object), a custom object to become the data of produced event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub data: Option<Data>,

    /// Add additional event extension context attributes
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub context_attributes: Option<ContextAttributes>,
}

/// State data filter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(deny_unknown_fields)]
pub struct StateDataFilter {
    /// Workflow expression to filter the state data input
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub input: Option<String>,

    /// Workflow expression that filters the state data output
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub output: Option<String>,
}

/// Event data filter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EventDataFilter {
    /// If set to `false`, event payload is not added/merged to state data.
    /// In this case [`data`] and [`to_state_data`] should be ignored. Default is `true`.
    ///
    /// [`data`]: Self::data
    /// [`to_state_data`]: Self::to_state_data
    #[serde(default = "true_value")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub use_data: bool,

    /// Workflow expression that filters the received event payload (default: `${ . }`)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub data: Option<String>,

    /// Workflow expression that selects a state data element to which the filtered event should be added/merged into.
    ///
    /// If not specified, denotes, the top-level state data element.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub to_state_data: Option<String>,
}

/// Action data filter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ActionDataFilter {
    /// Workflow expression that selects state data that the state action can use
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub from_state_data: Option<String>,

    /// If set to `false`, action data results are not added/merged to state data.
    /// In this case [`results`] and [`to_state_data`] should be ignored. Default is `true`.
    ///
    /// [`results`]: Self::results
    /// [`to_state_data`]: Self::to_state_data
    #[serde(default = "true_value")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub use_results: bool,

    /// Workflow expression that filters the actions data results
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub results: Option<String>,

    /// Workflow expression that selects a state data element to which the action results should be added/merged into.
    ///
    /// If not specified, denote, the top-level state data element
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub to_state_data: Option<String>,
}
