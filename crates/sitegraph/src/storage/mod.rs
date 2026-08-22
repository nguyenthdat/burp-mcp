mod edges;
mod migrations;
mod nodes;
mod sqlite;

pub use sqlite::SqliteGraph;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("invalid graph input: {0}")]
    InvalidInput(String),
    #[error("graph storage failed: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("graph migration failed: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("graph serialization failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("graph I/O failed: {0}")]
    Io(#[from] std::io::Error),
}
