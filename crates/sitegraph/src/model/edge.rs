use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    Contains,
    PathChild,
    AcceptsParameter,
    RespondedWith,
    LinksTo,
    FormSubmitsTo,
    LoadsScript,
    RedirectsTo,
    HasIssue,
    HasTechnology,
}

impl EdgeKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Contains => "contains",
            Self::PathChild => "path_child",
            Self::AcceptsParameter => "accepts_parameter",
            Self::RespondedWith => "responded_with",
            Self::LinksTo => "links_to",
            Self::FormSubmitsTo => "form_submits_to",
            Self::LoadsScript => "loads_script",
            Self::RedirectsTo => "redirects_to",
            Self::HasIssue => "has_issue",
            Self::HasTechnology => "has_technology",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    pub id: String,
    pub from_id: String,
    pub to_id: String,
    pub kind: EdgeKind,
    pub evidence_id: String,
    pub created_at: i64,
    pub metadata: Value,
}
