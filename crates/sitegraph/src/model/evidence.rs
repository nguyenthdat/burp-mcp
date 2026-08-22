use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Evidence {
    pub id: String,
    pub source: String,
    pub observed_at: i64,
    pub summary: Value,
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
