//! Serverless Workflow specification - auth schema
//!
//! Corresponding JSON schema: [auth.json](https://github.com/serverlessworkflow/specification/blob/v0.8/schema/auth.json).

use serde::{Deserialize, Serialize};

use crate::detail::basic;
use crate::workflow::definition::common::Metadata;

/// Auth definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(untagged)]
pub enum Auth {
    /// URI to a resource containing auth definitions (json or yaml)
    Uri(#[cfg_attr(feature = "validate", garde(url))] String),

    /// Workflow auth definitions
    Definitions(#[cfg_attr(feature = "validate", garde(dive, length(min = 1)))] Vec<AuthDef>),
}

/// Auth definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
pub struct AuthDef {
    /// Unique auth definition name
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    pub name: String,

    /// Defines the auth type
    #[serde(default = "basic")]
    #[cfg_attr(feature = "validate", garde(skip))]
    pub scheme: Scheme,

    /// Auth properties
    #[cfg_attr(feature = "validate", garde(dive))]
    pub properties: AuthDefProperties,
}

/// Auth definition properties
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(untagged)]
pub enum AuthDefProperties {
    /// Expression referencing a workflow secret that contains all needed auth info
    Expression(#[cfg_attr(feature = "validate", garde(skip))] String),

    /// Basic Auth Info
    BasicAuth(#[cfg_attr(feature = "validate", garde(dive))] BasicPropsDef),

    /// Bearer Auth Info State
    BearerAuth(#[cfg_attr(feature = "validate", garde(dive))] BearerPropsDef),

    /// OAuth2 Info
    OAuth2Auth(#[cfg_attr(feature = "validate", garde(dive))] OAuth2PropsDef),
}

/// Auth scheme
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scheme {
    /// Basic authentication
    Basic,

    /// Authentication with bearer token
    Bearer,

    /// OAuth2 authentication
    OAuth2,
}

/// Basic auth properties definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(untagged, deny_unknown_fields)]
pub enum BasicPropsDef {
    /// Expression referencing a workflow secret that contains all needed basic auth info
    Secret(#[cfg_attr(feature = "validate", garde(skip))] String),

    /// Basic auth information
    AuthInfo(#[cfg_attr(feature = "validate", garde(dive))] Box<BasicPropsDefAuthInfo>),
}

/// Basic auth properties definition auth info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
pub struct BasicPropsDefAuthInfo {
    /// String or a workflow expression. Contains the user name
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    username: String,

    /// String or a workflow expression. Contains the user password
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    password: String,

    /// Auth metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    metadata: Option<Metadata>,
}

/// Bearer auth properties definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(untagged, deny_unknown_fields)]
pub enum BearerPropsDef {
    /// Expression referencing a workflow secret that contains all needed bearer auth info
    Secret(#[cfg_attr(feature = "validate", garde(skip))] String),

    /// Bearer auth information
    AuthInfo(#[cfg_attr(feature = "validate", garde(dive))] Box<BearerPropsDefAuthInfo>),
}

/// Bearer auth properties definition auth info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
pub struct BearerPropsDefAuthInfo {
    /// String or a workflow expression. Contains the token
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    token: String,

    /// Auth metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(dive))]
    metadata: Option<Metadata>,
}

/// OAuth2 auth properties definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(untagged)]
pub enum OAuth2PropsDef {
    /// Expression referencing a workflow secret that contains all needed OAuth2 auth info
    Secret(#[cfg_attr(feature = "validate", garde(skip))] String),

    /// OAuth2 information
    AuthInfo(#[cfg_attr(feature = "validate", garde(dive))] Box<OAuth2PropsDefAuthInfo>),
}

/// OAuth2 auth properties definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validate", derive(garde::Validate))]
#[serde(rename_all = "camelCase")]
pub struct OAuth2PropsDefAuthInfo {
    /// String or a workflow expression. Contains the authority information
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    authority: Option<String>,

    /// Defines the grant type
    #[cfg_attr(feature = "validate", garde(skip))]
    grant_type: GrantType,

    /// String or a workflow expression. Contains the client identifier
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    client_id: String,

    /// String or a workflow expression. Contains the client secret
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    client_secret: Option<String>,

    /// Array containing strings or workflow expressions. Contains the OAuth2 scopes
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    scopes: Option<Vec<String>>,

    /// String or a workflow expression. Contains the user name. Used only if grantType is 'resourceOwner'
    ///
    /// TODO 'resourceOwner' is not actually a defined value in the schema for 'grantType'???
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    username: Option<String>,

    /// String or a workflow expression. Contains the user password. Used only if grantType is 'resourceOwner'
    ///
    /// TODO 'resourceOwner' is not actually a defined value in the schema for 'grantType'???
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    password: Option<String>,

    /// Array containing strings or workflow expressions. Contains the OAuth2 audiences
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    audiences: Option<Vec<String>>,

    /// String or a workflow expression. Contains the subject token
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    subject_token: Option<String>,

    /// String or a workflow expression. Contains the requested subject
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    requested_subject: Option<String>,

    /// String or a workflow expression. Contains the requested issuer
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "validate", garde(length(min = 1)))]
    requested_issuer: Option<String>,
}

/// OAuth2 grant type
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GrantType {
    /// Password grant
    Password,

    /// Client credentials grant
    ClientCredentials,

    /// Token exchange grant
    TokenExchange,
}
