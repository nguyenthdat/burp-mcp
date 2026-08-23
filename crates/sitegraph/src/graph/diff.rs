use crate::storage::StorageError;
use serde::Serialize;
use sqlx::{Row, SqlitePool};

#[derive(Debug, Serialize)]
pub struct GraphDiff {
    pub added_node_ids: Vec<String>,
    pub updated_node_ids: Vec<String>,
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
    let total = sqlx::query("SELECT count(*) AS count FROM nodes WHERE updated_at>?1")
        .bind(since)
        .fetch_one(pool)
        .await?
        .get::<i64, _>("count") as u64;
    let rows = sqlx::query("SELECT id, created_at, updated_at FROM nodes WHERE updated_at>?1 ORDER BY updated_at, id LIMIT ?2 OFFSET ?3")
        .bind(since)
        .bind(limit as i64)
        .bind(cursor as i64)
        .fetch_all(pool)
        .await?;
    let mut added_node_ids = Vec::new();
    let mut updated_node_ids = Vec::new();
    for row in &rows {
        let id = row.get::<String, _>("id");
        if row.get::<i64, _>("created_at") > since {
            added_node_ids.push(id);
        } else {
            updated_node_ids.push(id);
        }
    }
    let next = cursor + rows.len() as u64;
    Ok(GraphDiff {
        added_node_ids,
        updated_node_ids,
        total,
        truncated: next < total,
        next_cursor: (next < total).then_some(next),
        last_synced_at,
        evidence: serde_json::json!({"source": "node timestamps", "since": since}),
    })
}
