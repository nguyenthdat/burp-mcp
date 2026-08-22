use serde::{Deserialize, Serialize};
use serde_json::Value;

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
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub kind: NodeKind,
    pub stable_hash: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub metadata: Value,
}
