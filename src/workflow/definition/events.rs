//! Serverless Workflow specification - events schema
//!
//! Corresponding JSON schema: [events.json](https://github.com/serverlessworkflow/specification/blob/v0.8/schema/events.json).

use serde::{Deserialize, Serialize};

use crate::detail::{consumed, true_value};
use crate::workflow::definition::common::Metadata;
#[cfg(feature = "validate")]
use crate::workflow::definition::detail::garde::mandatory_for_consumed_events;

/// Workflow CloudEvent definitions. Defines CloudEvents that can be consumed or produced
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(untagged)]
pub enum Events {
    /// URI to a resource containing event definitions (json or yaml)
    Uri(#[cfg_attr(feature = "validate", garde(url))] String),

    /// Inline event definitions
    Inline(#[cfg_attr(feature = "validate", garde(length(min = 1)))] Vec<EventDef>),
}

/// Event definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EventDef {
    /// Unique event name
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub name: String,

    /// CloudEvent source
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(custom(mandatory_for_consumed_events(self.kind))))]
    pub source: Option<String>,

    /// CloudEvent type
    #[serde(rename = "type")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub event_type: String,

    /// Defines the CloudEvent as either [`Consumed`] or [`Produced`] by the workflow. Default is [`Consumed`]
    ///
    /// [`Consumed`]: EventKind::Consumed
    /// [`Produced`]: EventKind::Produced
    #[serde(default = "consumed")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub kind: EventKind,

    /// CloudEvent correlation definitions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive, length(min = 1)))]
    pub correlation: Option<Vec<CorrelationDef>>,

    /// If `true`, only the Event payload is accessible to consuming Workflow states.
    /// If `false`, both event payload and context attributes should be accessible.
    #[serde(default = "true_value")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub data_only: bool,

    /// Metadata information
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub metadata: Option<Metadata>,
}

/// CloudEvent kind
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventKind {
    /// CloudEvent is consumed
    Consumed,

    /// CloudEvent is produced
    Produced,
}

/// CloudEvent correlation definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CorrelationDef {
    /// CloudEvent Extension Context Attribute name
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub context_attribute_name: String,

    /// CloudEvent Extension Context Attribute value
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub context_attribute_value: Option<String>,
}
