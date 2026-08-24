use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Neighbor {
    pub edge_id: String,
    pub kind: String,
    pub direction: String,
    pub node_id: String,
    pub node_kind: String,
    pub metadata: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NeighborPage {
    pub items: Vec<Neighbor>,
    pub total: u64,
    pub truncated: bool,
    pub next_cursor: Option<u64>,
    pub last_synced_at: Option<i64>,
    pub evidence: Value,
}
