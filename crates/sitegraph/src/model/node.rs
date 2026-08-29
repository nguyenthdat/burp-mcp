use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Origin,
    Endpoint,
    PathSegment,
    Parameter,
    ResponseFingerprint,
    Technology,
    Issue,
    Artifact,
    Role,
    AuthContext,
    Finding,
}

impl NodeKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Origin => "origin",
            Self::Endpoint => "endpoint",
            Self::PathSegment => "path_segment",
            Self::Parameter => "parameter",
            Self::ResponseFingerprint => "response_fingerprint",
            Self::Technology => "technology",
            Self::Issue => "issue",
            Self::Artifact => "artifact",
            Self::Role => "role",
            Self::AuthContext => "auth_context",
            Self::Finding => "finding",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct NodeMetadata {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub origin: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub method: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u32>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parameter_names: Vec<String>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub segment: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub location: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub fingerprint: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub severity: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub confidence: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub kind: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub artifact_kind: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub web_socket_id: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub upgrade_url: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub direction: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub url: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub path_template: String,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub is_template: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub kind: NodeKind,
    pub stable_hash: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub metadata: NodeMetadata,
}
