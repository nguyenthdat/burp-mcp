use crate::storage::StorageError;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

#[derive(Debug, Serialize, Deserialize)]
pub struct CsvExport {
    pub csv: String,
    pub total: u64,
    pub truncated: bool,
    pub next_cursor: Option<u64>,
    pub last_synced_at: Option<i64>,
    pub evidence: serde_json::Value,
}

pub async fn page(
    pool: &SqlitePool,
    cursor: u64,
    limit: u64,
    last_synced_at: Option<i64>,
) -> Result<CsvExport, StorageError> {
    let total = sqlx::query("SELECT count(*) AS count FROM nodes")
        .fetch_one(pool)
        .await?
        .get::<i64, _>("count") as u64;
    let rows = sqlx::query("SELECT id, kind, metadata FROM nodes ORDER BY id LIMIT ?1 OFFSET ?2")
        .bind(limit as i64)
        .bind(cursor as i64)
        .fetch_all(pool)
        .await?;
    let mut csv = String::from("id,kind,metadata\n");
    for row in &rows {
        let values = [
            row.get::<String, _>("id"),
            row.get::<String, _>("kind"),
            row.get::<String, _>("metadata"),
        ];
        csv.push_str(&values.map(|value| quote(&value)).join(","));
        csv.push('\n');
    }
    let next = cursor + rows.len() as u64;
    Ok(CsvExport {
        csv,
        total,
        truncated: next < total,
        next_cursor: (next < total).then_some(next),
        last_synced_at,
        evidence: serde_json::json!({"source": "metadata-only SQLite graph export"}),
    })
}

fn quote(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}
