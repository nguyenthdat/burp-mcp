use super::StorageError;
use crate::model::{Edge, EdgeKind};
use crate::normalize::fingerprint::stable_id;
use sqlx::{Sqlite, Transaction};

pub async fn upsert(
    transaction: &mut Transaction<'_, Sqlite>,
    from_id: &str,
    to_id: &str,
    kind: EdgeKind,
    evidence_id: &str,
    timestamp: i64,
) -> Result<String, StorageError> {
    let id = stable_id(kind.as_str(), &[from_id, to_id, evidence_id]);
    let edge = Edge {
        id: id.clone(),
        from_id: from_id.to_owned(),
        to_id: to_id.to_owned(),
        kind,
        evidence_id: evidence_id.to_owned(),
        created_at: timestamp,
        metadata: serde_json::json!({}),
    };
    sqlx::query(
        "INSERT INTO edges(id, from_id, to_id, kind, evidence_id, created_at, metadata)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(id) DO NOTHING",
    )
    .bind(&edge.id)
    .bind(&edge.from_id)
    .bind(&edge.to_id)
    .bind(edge.kind.as_str())
    .bind(&edge.evidence_id)
    .bind(edge.created_at)
    .bind(serde_json::to_string(&edge.metadata)?)
    .execute(&mut **transaction)
    .await?;
    Ok(id)
}
