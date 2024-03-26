//! Serverless Workflow specification - functions schema
//!
//! Corresponding JSON schema: [functions.json](https://github.com/serverlessworkflow/specification/blob/v0.8/schema/functions.json).

use serde::{Deserialize, Serialize};
use url::Url;

use crate::detail::rest;
use crate::workflow::definition::common::Metadata;

/// Workflow function definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(untagged)]
pub enum Functions {
    /// URI to a resource containing function definitions (json or yaml)
    Uri(#[cfg_attr(feature = "validate", garde(skip))] Url),

    /// Inline function definitions
    Inline(#[cfg_attr(feature = "validate", garde(length(min = 1)))] Vec<Function>),
}

/// Function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Function {
    /// Unique function name
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub name: String,

    /// Operation specification. Format depends on [`function_type`]:
    ///
    /// | [`function_type`] value | [`operation`] format                                                               |
    /// |-------------------------|------------------------------------------------------------------------------------|
    /// | [`Rest`]                | <path_to_openapi_definition>#<operation_id>                                        |
    /// | [`AsyncApi`]            | <path_to_asyncapi_definition>#<operation_id>                                       |
    /// | [`GRpc`]                | <path_to_grpc_proto_file>#<service_name>#<service_method>                          |
    /// | [`GraphQL`]             | <url_to_graphql_endpoint>#<literal `mutation` or `query`>#<query_or_mutation_name> |
    /// | [`OData`]               | <URI_to_odata_service>#<Entity_Set_Name>                                           |
    /// | [`Expression`]          | Language-dependent expression (see [`expression_lang`])                            |
    /// | [`Custom`]              | Runtime-specific format                                                            |
    ///
    /// [`function_type`]: Function::function_type
    /// [`operation`]: Function::operation
    /// [`Rest`]: FunctionType::Rest
    /// [`AsyncApi`]: FunctionType::AsyncApi
    /// [`GRpc`]: FunctionType::GRpc
    /// [`GraphQL`]: FunctionType::GraphQL
    /// [`OData`]: FunctionType::OData
    /// [`Expression`]: FunctionType::Expression
    /// [`Custom`]: FunctionType::Custom
    /// [`expression_lang`]: crate::workflow::definition::WorkflowDefinition::expression_lang
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub operation: String,

    /// Defines the function type. Default is [`Rest`](FunctionType::Rest).
    #[serde(rename = "type", default = "rest")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub function_type: FunctionType,

    /// References an auth definition name to be used to access to resource defined in the operation parameter
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub auth_ref: Option<String>,

    /// Function metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    pub metadata: Option<Metadata>,
}

/// Function type
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FunctionType {
    /// REST endpoint
    Rest,

    /// Asynchronous API defined by an [AsyncApi](https://www.asyncapi.com) specification
    AsyncApi,

    /// [gRPC](https://grpc.io) endpoint
    #[serde(rename = "rpc")]
    GRpc,

    /// [GraphQL](https://graphql.org) service method
    GraphQL,

    /// [OData](https://www.odata.org) service
    OData,

    /// An inlined function expression in the workflow's expression language (default is `jq`).
    Expression,

    /// Custom function type (runtime-specific)
    Custom,
}
