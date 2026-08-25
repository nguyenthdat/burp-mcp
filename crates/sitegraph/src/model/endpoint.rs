use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SitemapObservation {
    pub url: String,
    pub method: String,
    pub status: u32,
    pub content_type: String,
    #[serde(skip)]
    pub response_body: Vec<u8>,
    #[serde(skip)]
    pub request_bytes: Vec<u8>,
    #[serde(skip)]
    pub response_bytes: Vec<u8>,
    pub redirect_url: String,
    pub response_links: Vec<String>,
    pub form_actions: Vec<String>,
    pub script_sources: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebSocketObservation {
    pub id: String,
    pub web_socket_id: String,
    pub direction: String,
    pub upgrade_url: String,
    #[serde(skip)]
    pub payload: Vec<u8>,
    #[serde(skip)]
    pub edited_payload: Vec<u8>,
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
    pub websocket_messages: Vec<WebSocketObservation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncContext {
    pub graph_id: String,
    pub source: String,
    pub scope: String,
    pub run_id: String,
    pub cursor: Option<String>,
    pub source_total: Option<u64>,
    pub pages_seen: u64,
    pub items_seen: u64,
    pub complete: bool,
}

impl SyncContext {
    pub fn snapshot(graph_id: impl Into<String>, scope: impl Into<String>) -> Self {
        Self {
            graph_id: graph_id.into(),
            source: "burp_sitemap".to_owned(),
            scope: scope.into(),
            run_id: String::new(),
            cursor: None,
            source_total: None,
            pages_seen: 1,
            items_seen: 0,
            complete: true,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncCoverage {
    pub complete: bool,
    pub items_indexed: u64,
    pub source_total: Option<u64>,
    pub pages_read: u64,
    pub end_of_source: bool,
    pub cancelled: bool,
    pub last_cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncSummary {
    pub sync_id: String,
    pub upserted_nodes: u64,
    pub upserted_edges: u64,
    pub total_nodes: u64,
    pub total_edges: u64,
    pub last_synced_at: i64,
    pub complete: bool,
    pub items_seen: u64,
    pub pages_seen: u64,
    pub tombstoned_nodes: u64,
    pub tombstoned_edges: u64,
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
    pub graph_id: String,
    pub schema_version: i64,
    pub state: String,
    pub freshness: String,
    pub total_nodes: u64,
    pub total_edges: u64,
    pub active_nodes: u64,
    pub active_edges: u64,
    pub last_synced_at: Option<i64>,
    pub last_success_at: Option<i64>,
    pub current_run_id: Option<String>,
    pub coverage: SyncCoverage,
    pub last_error: Option<String>,
}
