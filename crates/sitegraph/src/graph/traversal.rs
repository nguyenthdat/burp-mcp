use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceStep {
    pub depth: u32,
    pub edge_id: String,
    pub edge_kind: String,
    pub from_id: String,
    pub to_id: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TracePage {
    pub items: Vec<TraceStep>,
    pub total: u64,
    pub truncated: bool,
    pub next_cursor: Option<u64>,
    pub last_synced_at: Option<i64>,
}
