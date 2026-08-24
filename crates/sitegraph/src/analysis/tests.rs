use super::*;
use sqlx::SqlitePool;

#[tokio::test]
async fn shortest_path_terminates_on_cycle_and_returns_deterministic_path() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::query("CREATE TABLE nodes(id TEXT PRIMARY KEY, metadata TEXT NOT NULL)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "CREATE TABLE edges(id TEXT PRIMARY KEY, from_id TEXT NOT NULL, to_id TEXT NOT NULL)",
    )
    .execute(&pool)
    .await
    .unwrap();
    for id in ["a", "b", "c"] {
        sqlx::query("INSERT INTO nodes(id, metadata) VALUES(?1, '{}')")
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
    }
    for (id, from, to) in [("ab", "a", "b"), ("bc", "b", "c"), ("ca", "c", "a")] {
        sqlx::query("INSERT INTO edges(id, from_id, to_id) VALUES(?1, ?2, ?3)")
            .bind(id)
            .bind(from)
            .bind(to)
            .execute(&pool)
            .await
            .unwrap();
    }
    let result = shortest_path(&pool, "a", "c", 16).await.unwrap();
    assert_eq!(result.depth, 2);
    assert_eq!(
        result
            .items
            .iter()
            .map(|item| item.node_id.as_str())
            .collect::<Vec<_>>(),
        vec!["a", "b", "c"]
    );
    assert!(!result.truncated);
}
