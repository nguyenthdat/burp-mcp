use crate::storage::StorageError;
use serde::Serialize;
use sqlx::{Row, SqlitePool};

#[derive(Debug, Serialize)]
pub struct GraphDiff {
    pub added_node_ids: Vec<String>,
    pub updated_node_ids: Vec<String>,
    pub removed_node_ids: Vec<String>,
    pub added_edge_ids: Vec<String>,
    pub removed_edge_ids: Vec<String>,
    pub total: u64,
    pub truncated: bool,
    pub next_cursor: Option<u64>,
    pub last_synced_at: Option<i64>,
    pub evidence: serde_json::Value,
}

pub async fn since(
    pool: &SqlitePool,
    since: i64,
    cursor: u64,
    limit: u64,
    last_synced_at: Option<i64>,
) -> Result<GraphDiff, StorageError> {
    let total = sqlx::query(
        "SELECT (SELECT count(*) FROM nodes WHERE updated_at>?1)
              + (SELECT count(*) FROM edges WHERE updated_at>?1)
              + (SELECT count(*) FROM tombstones WHERE last_confirmed_at>?1) AS count",
    )
    .bind(since)
    .fetch_one(pool)
    .await?
    .get::<i64, _>("count") as u64;
    let rows = sqlx::query(
        "SELECT 'node' AS entity_type, id AS entity_id, created_at AS changed_at, 'active' AS status
         FROM nodes WHERE updated_at>?1
         UNION ALL
         SELECT 'edge' AS entity_type, id AS entity_id, updated_at AS changed_at, 'active' AS status
         FROM edges WHERE updated_at>?1
         UNION ALL
         SELECT entity_type, entity_id, last_confirmed_at AS changed_at, 'removed' AS status
         FROM tombstones WHERE last_confirmed_at>?1
         ORDER BY changed_at, entity_type, entity_id LIMIT ?2 OFFSET ?3",
    )
    .bind(since)
    .bind(limit as i64)
    .bind(cursor as i64)
    .fetch_all(pool)
    .await?;
    let mut added_node_ids = Vec::new();
    let mut updated_node_ids = Vec::new();
    let mut removed_node_ids = Vec::new();
    let mut added_edge_ids = Vec::new();
    let mut removed_edge_ids = Vec::new();
    for row in &rows {
        let entity_type = row.get::<String, _>("entity_type");
        let entity_id = row.get::<String, _>("entity_id");
        let status = row.get::<String, _>("status");
        match (entity_type.as_str(), status.as_str()) {
            ("node", "removed") => removed_node_ids.push(entity_id),
            ("node", _) if row.get::<i64, _>("changed_at") > since => {
                added_node_ids.push(entity_id)
            }
            ("edge", "removed") => removed_edge_ids.push(entity_id),
            ("edge", _) => added_edge_ids.push(entity_id),
            _ => updated_node_ids.push(entity_id),
        }
    }
    let next = cursor + rows.len() as u64;
    Ok(GraphDiff {
        added_node_ids,
        updated_node_ids,
        removed_node_ids,
        added_edge_ids,
        removed_edge_ids,
        total,
        truncated: next < total,
        next_cursor: (next < total).then_some(next),
        last_synced_at,
        evidence: serde_json::json!({"source": "node, edge and tombstone revisions", "since": since}),
    })
}
