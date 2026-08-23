use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SitemapObservation {
    pub url: String,
    pub method: String,
    pub status: u32,
    pub content_type: String,
    #[serde(skip)]
    pub response_body: Vec<u8>,
    pub redirect_url: String,
    pub response_links: Vec<String>,
    pub form_actions: Vec<String>,
    pub script_sources: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IssueObservation {
    pub name: String,
    pub severity: String,
    pub confidence: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TechnologyObservation {
    pub name: String,
    pub endpoint_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactObservation {
    pub kind: String,
    pub name: String,
    pub endpoint_url: String,
    pub fingerprint: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncBatch {
    pub sitemap: Vec<SitemapObservation>,
    pub issues: Vec<IssueObservation>,
    pub technologies: Vec<TechnologyObservation>,
    pub artifacts: Vec<ArtifactObservation>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncSummary {
    pub sync_id: String,
    pub upserted_nodes: u64,
    pub upserted_edges: u64,
    pub total_nodes: u64,
    pub total_edges: u64,
    pub last_synced_at: i64,
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Endpoint {
    pub id: String,
    pub origin: String,
    pub method: String,
    pub path: String,
    pub status: u32,
    pub content_type: String,
    pub response_fingerprint: Option<String>,
    pub parameter_names: Vec<String>,
    pub last_seen_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EndpointPage {
    pub items: Vec<Endpoint>,
    pub total: u64,
    pub truncated: bool,
    pub next_cursor: Option<u64>,
    pub last_synced_at: Option<i64>,
    pub evidence: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphStatus {
    pub schema_version: i64,
    pub total_nodes: u64,
    pub total_edges: u64,
    pub last_synced_at: Option<i64>,
}
