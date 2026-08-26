use super::StorageError;
use crate::model::{Node, NodeKind, NodeMetadata};
use sqlx::{Sqlite, Transaction};

pub async fn upsert(
    transaction: &mut Transaction<'_, Sqlite>,
    node: &Node,
    search: SearchFields<'_>,
) -> Result<(), StorageError> {
    let metadata = serde_json::to_string(&node.metadata)?;
    sqlx::query(
        "INSERT INTO nodes(id, kind, stable_hash, created_at, updated_at, metadata)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(id) DO UPDATE SET updated_at=excluded.updated_at, metadata=excluded.metadata",
    )
    .bind(&node.id)
    .bind(node.kind.as_str())
    .bind(&node.stable_hash)
    .bind(node.created_at)
    .bind(node.updated_at)
    .bind(metadata)
    .execute(&mut **transaction)
    .await?;
    sqlx::query("DELETE FROM node_search WHERE node_id=?1")
        .bind(&node.id)
        .execute(&mut **transaction)
        .await?;
    sqlx::query(
        "INSERT INTO node_search(node_id, kind, origin, method, path, name) VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
    )
    .bind(&node.id)
    .bind(node.kind.as_str())
    .bind(search.origin)
    .bind(search.method)
    .bind(search.path)
    .bind(search.name)
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

#[derive(Clone, Copy, Default)]
pub struct SearchFields<'a> {
    pub origin: &'a str,
    pub method: &'a str,
    pub path: &'a str,
    pub name: &'a str,
}

pub fn node(kind: NodeKind, stable_hash: String, timestamp: i64, metadata: NodeMetadata) -> Node {
    Node {
        id: stable_hash.clone(),
        kind,
        stable_hash,
        created_at: timestamp,
        updated_at: timestamp,
        metadata,
    }
}
