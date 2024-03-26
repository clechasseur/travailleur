//! Workflow instance type

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::workflow::definition::{Identifier, WorkflowDefinition};

/// Workflow instance container.
///
/// TODO expand
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInstance {
    /// Workflow instance ID. Unique among all workflow instances.
    pub id: String,

    /// Workflow identifier.
    pub workflow_identifier: Identifier,

    /// Name of current workflow state, or `None` if workflow has completed.
    pub state: Option<String>,

    /// Workflow data (a JSON object), passed between states.
    ///
    /// If [`state`](Self::state) is `None`, this is the final workflow output.
    pub data: Map<String, Value>,

    /// Whether workflow has terminated prematurely.
    pub terminated: bool,
}

impl WorkflowInstance {
    /// Generates a new workflow instance from a [`WorkflowDefinition`].
    ///
    /// The instance will have a new, randomly-generated [`id`], will start at the workflow's
    /// [start state] with the provided workflow input (or an empty JSON object if no initial
    /// input is provided).
    ///
    /// [`id`]: Self::id
    /// [start state]: WorkflowDefinition::start_state_name
    pub fn for_definition(
        definition: &WorkflowDefinition,
        input: Option<Map<String, Value>>,
    ) -> Self {
        Self {
            id: Self::generate_id(),
            workflow_identifier: definition.identifier.clone(),
            state: definition.start_state_name().map(|name| name.into()),
            data: input.unwrap_or_default(),
            terminated: false,
        }
    }

    /// Generates a new workflow instance for a workflow identified via its [`Identifier`].
    ///
    /// The instance will have a new, randomly-generated [`id`], will point to the given `state`
    /// and contain the given `data` (or an empty JSON object if no data is provided).
    ///
    /// [`id`]: Self::id
    pub fn for_workflow_identifier<I>(
        identifier: I,
        state: Option<String>,
        data: Option<Map<String, Value>>,
    ) -> Self
    where
        I: Into<Identifier>,
    {
        Self {
            id: Self::generate_id(),
            workflow_identifier: identifier.into(),
            state,
            data: data.unwrap_or_default(),
            terminated: false,
        }
    }

    fn generate_id() -> String {
        Uuid::new_v4().into()
    }
}
