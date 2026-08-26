use crate::model::NodeMetadata;
use crate::storage::StorageError;
use serde::{Deserialize, Serialize};
use sqlx::{Acquire, Row, SqlitePool};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportNode {
    pub id: String,
    pub kind: String,
    pub metadata: NodeMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportEdge {
    pub id: String,
    pub from_id: String,
    pub to_id: String,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExactEvidence {
    pub id: String,
    pub source_entry_id: Option<String>,
    pub surface: String,
    pub direction: Option<String>,
    pub content_type: Option<String>,
    pub payload_base64: String,
    pub byte_length: i64,
    pub observed_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportEvidence {
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<ExactEvidence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    pub source: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonExport {
    pub nodes: Vec<ExportNode>,
    pub edges: Vec<ExportEdge>,
    pub total: u64,
    pub truncated: bool,
    pub next_cursor: Option<u64>,
    pub last_synced_at: Option<i64>,
    pub snapshot_id: Option<String>,
    pub evidence: ExportEvidence,
    pub profile: String,
}

pub async fn page(
    pool: &SqlitePool,
    cursor: u64,
    limit: u64,
    last_synced_at: Option<i64>,
) -> Result<JsonExport, StorageError> {
    let mut transaction = pool.begin().await?;
    let connection = transaction.acquire().await?;
    let node_total = sqlx::query("SELECT count(*) AS count FROM nodes")
        .fetch_one(&mut *connection)
        .await?
        .get::<i64, _>("count") as u64;
    let edge_total = sqlx::query("SELECT count(*) AS count FROM edges")
        .fetch_one(&mut *connection)
        .await?
        .get::<i64, _>("count") as u64;
    let rows = sqlx::query("SELECT id, kind, metadata FROM nodes ORDER BY id LIMIT ?1 OFFSET ?2")
        .bind(limit as i64)
        .bind(cursor as i64)
        .fetch_all(&mut *connection)
        .await?;
    let nodes = rows
        .into_iter()
        .map(|row| {
            Ok(ExportNode {
                id: row.get("id"),
                kind: row.get("kind"),
                metadata: serde_json::from_str(&row.get::<String, _>("metadata"))?,
            })
        })
        .collect::<Result<Vec<_>, StorageError>>()?;
    let mut edges = Vec::new();
    for node in &nodes {
        for row in
            sqlx::query("SELECT id, from_id, to_id, kind FROM edges WHERE from_id=?1 ORDER BY id")
                .bind(&node.id)
                .fetch_all(&mut *connection)
                .await?
        {
            edges.push(ExportEdge {
                id: row.get("id"),
                from_id: row.get("from_id"),
                to_id: row.get("to_id"),
                kind: row.get("kind"),
            });
        }
    }
    let snapshot_id = sqlx::query("SELECT value FROM graph_metadata WHERE key='last_synced_at'")
        .fetch_optional(&mut *connection)
        .await?
        .map(|row| row.get::<String, _>("value"));
    transaction.commit().await?;
    let next = cursor + nodes.len() as u64;
    Ok(JsonExport {
        nodes,
        edges,
        total: node_total + edge_total,
        truncated: next < node_total,
        next_cursor: (next < node_total).then_some(next),
        last_synced_at,
        snapshot_id,
        profile: "metadata".to_owned(),
        evidence: ExportEvidence {
            items: Vec::new(),
            profile: None,
            source: "metadata-only SQLite read transaction".to_owned(),
        },
    })
}

pub async fn exact_page(
    pool: &SqlitePool,
    cursor: u64,
    limit: u64,
    last_synced_at: Option<i64>,
) -> Result<JsonExport, StorageError> {
    let mut transaction = pool.begin().await?;
    let connection = transaction.acquire().await?;
    let total = sqlx::query("SELECT count(*) AS count FROM evidence_blobs")
        .fetch_one(&mut *connection)
        .await?
        .get::<i64, _>("count") as u64;
    let rows = sqlx::query("SELECT id, source_entry_id, surface, direction, content_type, payload, byte_length, observed_at FROM evidence_blobs ORDER BY id LIMIT ?1 OFFSET ?2")
        .bind(limit as i64)
        .bind(cursor as i64)
        .fetch_all(&mut *connection)
        .await?;
    let evidence = rows
        .into_iter()
        .map(|row| ExactEvidence {
            id: row.get("id"),
            source_entry_id: row.get("source_entry_id"),
            surface: row.get("surface"),
            direction: row.get("direction"),
            content_type: row.get("content_type"),
            payload_base64: base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                row.get::<Vec<u8>, _>("payload"),
            ),
            byte_length: row.get("byte_length"),
            observed_at: row.get("observed_at"),
        })
        .collect::<Vec<_>>();
    transaction.commit().await?;
    let next = cursor + evidence.len() as u64;
    Ok(JsonExport {
        nodes: Vec::new(),
        edges: Vec::new(),
        total,
        truncated: next < total,
        next_cursor: (next < total).then_some(next),
        last_synced_at,
        snapshot_id: None,
        profile: "exact".to_owned(),
        evidence: ExportEvidence {
            items: evidence,
            profile: Some("exact".to_owned()),
            source: "project evidence_blobs".to_owned(),
        },
    })
}
