use crate::storage::StorageError;
use serde::Serialize;
use serde_json::Value;
use sqlx::{Row, SqlitePool};

#[derive(Debug, Serialize)]
pub struct JsonExport {
    pub nodes: Vec<Value>,
    pub edges: Vec<Value>,
    pub total: u64,
    pub truncated: bool,
    pub next_cursor: Option<u64>,
    pub last_synced_at: Option<i64>,
    pub evidence: Value,
}

pub async fn page(
    pool: &SqlitePool,
    cursor: u64,
    limit: u64,
    last_synced_at: Option<i64>,
) -> Result<JsonExport, StorageError> {
    let limit = limit.clamp(1, 500);
    let node_total = sqlx::query("SELECT count(*) AS count FROM nodes")
        .fetch_one(pool)
        .await?
        .get::<i64, _>("count") as u64;
    let edge_total = sqlx::query("SELECT count(*) AS count FROM edges")
        .fetch_one(pool)
        .await?
        .get::<i64, _>("count") as u64;
    let rows = sqlx::query("SELECT id, kind, metadata FROM nodes ORDER BY id LIMIT ?1 OFFSET ?2")
        .bind(limit as i64)
        .bind(cursor as i64)
        .fetch_all(pool)
        .await?;
    let nodes = rows
        .into_iter()
        .map(|row| {
            Ok(serde_json::json!({
                "id": row.get::<String, _>("id"),
                "kind": row.get::<String, _>("kind"),
                "metadata": serde_json::from_str::<Value>(&row.get::<String, _>("metadata"))?,
            }))
        })
        .collect::<Result<Vec<_>, StorageError>>()?;
    let node_ids = nodes
        .iter()
        .filter_map(|node| node["id"].as_str())
        .collect::<Vec<_>>();
    let mut edges = Vec::new();
    for node_id in node_ids {
        for row in sqlx::query(
            "SELECT id, from_id, to_id, kind FROM edges WHERE from_id=?1 ORDER BY id LIMIT 500",
        )
        .bind(node_id)
        .fetch_all(pool)
        .await?
        {
            edges.push(serde_json::json!({
                "id": row.get::<String, _>("id"),
                "from_id": row.get::<String, _>("from_id"),
                "to_id": row.get::<String, _>("to_id"),
                "kind": row.get::<String, _>("kind"),
            }));
        }
    }
    let next = cursor + nodes.len() as u64;
    Ok(JsonExport {
        nodes,
        edges,
        total: node_total + edge_total,
        truncated: next < node_total,
        next_cursor: (next < node_total).then_some(next),
        last_synced_at,
        evidence: serde_json::json!({"source": "metadata-only SQLite graph export"}),
    })
}
