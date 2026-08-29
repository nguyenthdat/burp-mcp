use crate::model::NodeMetadata;
use crate::storage::StorageError;
#[cfg(test)]
mod tests;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;

const MAX_ANALYSIS_NODES: usize = 250_000;
const MAX_PATH_DEPTH: usize = 16;
const MAX_RESULT_ITEMS: usize = 500;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathStep {
    pub node_id: String,
    pub edge_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShortestPath {
    pub items: Vec<PathStep>,
    pub depth: usize,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cluster {
    pub key: String,
    pub endpoint_ids: Vec<String>,
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImpactNode {
    pub node_id: String,
    pub depth: usize,
}

pub(crate) async fn shortest_path(
    pool: &SqlitePool,
    from_id: &str,
    to_id: &str,
    max_depth: usize,
) -> Result<ShortestPath, StorageError> {
    if from_id == to_id {
        return Ok(ShortestPath {
            items: vec![PathStep {
                node_id: from_id.to_owned(),
                edge_id: None,
            }],
            depth: 0,
            truncated: false,
        });
    }

    let depth_limit = max_depth.clamp(1, MAX_PATH_DEPTH);
    let row = sqlx::query(
        "WITH RECURSIVE bfs(node_id, edge_id, path_nodes, path_edges, depth, visited) AS (
            SELECT ?1, '', ?1, '', 0, '|' || ?1 || '|'
            UNION ALL
            SELECT e.to_id, e.id,
                   bfs.path_nodes || ',' || e.to_id,
                   CASE WHEN bfs.path_edges = '' THEN e.id ELSE bfs.path_edges || ',' || e.id END,
                   bfs.depth + 1,
                   bfs.visited || e.to_id || '|'
            FROM edges e
            JOIN bfs ON e.from_id = bfs.node_id
            WHERE bfs.depth < ?3
              AND instr(bfs.visited, '|' || e.to_id || '|') = 0
              AND bfs.node_id != ?2
        )
        SELECT depth, path_nodes, path_edges
        FROM bfs
        WHERE node_id = ?2
        ORDER BY depth ASC
        LIMIT 1",
    )
    .bind(from_id)
    .bind(to_id)
    .bind(depth_limit as i64)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(ShortestPath {
            items: Vec::new(),
            depth: 0,
            truncated: false,
        });
    };

    let depth: i64 = row.get("depth");
    let path_nodes: String = row.get("path_nodes");
    let path_edges: String = row.get("path_edges");

    let nodes_list: Vec<&str> = path_nodes.split(',').collect();
    let edges_list: Vec<&str> = if path_edges.is_empty() {
        Vec::new()
    } else {
        path_edges.split(',').collect()
    };

    let mut items = Vec::with_capacity(nodes_list.len());
    for (i, node) in nodes_list.iter().enumerate() {
        let edge_id = if i == 0 {
            None
        } else {
            edges_list.get(i - 1).map(|&e| e.to_owned())
        };
        items.push(PathStep {
            node_id: (*node).to_owned(),
            edge_id,
        });
    }

    Ok(ShortestPath {
        items,
        depth: depth as usize,
        truncated: false,
    })
}

pub(crate) async fn clusters(
    pool: &SqlitePool,
    limit: usize,
) -> Result<Vec<Cluster>, StorageError> {
    let limit = limit.clamp(1, MAX_RESULT_ITEMS);
    let rows =
        sqlx::query("SELECT id, metadata FROM nodes WHERE kind='endpoint' ORDER BY id LIMIT ?1")
            .bind(i64::try_from(MAX_ANALYSIS_NODES).unwrap_or(i64::MAX))
            .fetch_all(pool)
            .await?;
    let mut clusters: HashMap<String, Vec<String>> = HashMap::new();
    for row in rows {
        let id = row.get::<String, _>("id");
        let metadata: NodeMetadata = serde_json::from_str(&row.get::<String, _>("metadata"))?;
        let origin = metadata.origin.as_str();
        let prefix = metadata
            .path
            .split('/')
            .find(|segment| !segment.is_empty())
            .unwrap_or("/");
        clusters
            .entry(format!("{origin}/{prefix}"))
            .or_default()
            .push(id);
    }
    let mut result = clusters
        .into_iter()
        .map(|(key, mut endpoint_ids)| {
            endpoint_ids.sort_unstable();
            let total = endpoint_ids.len();
            endpoint_ids.truncate(MAX_RESULT_ITEMS);
            Cluster {
                key,
                endpoint_ids,
                total,
            }
        })
        .collect::<Vec<_>>();
    result.sort_unstable_by(|left, right| {
        right
            .total
            .cmp(&left.total)
            .then_with(|| left.key.cmp(&right.key))
    });
    result.truncate(limit);
    Ok(result)
}

pub(crate) async fn impact(
    pool: &SqlitePool,
    start_id: &str,
    max_depth: usize,
    limit: usize,
) -> Result<Vec<ImpactNode>, StorageError> {
    let depth_limit = max_depth.clamp(1, MAX_PATH_DEPTH);
    let result_limit = limit.clamp(1, MAX_RESULT_ITEMS);

    let rows = sqlx::query(
        "WITH RECURSIVE impact_walk(node_id, depth, visited) AS (
            SELECT e.to_id, 1, '|' || ?1 || '|' || e.to_id || '|'
            FROM edges e
            WHERE e.from_id = ?1
            UNION ALL
            SELECT e.to_id, impact_walk.depth + 1, impact_walk.visited || e.to_id || '|'
            FROM edges e
            JOIN impact_walk ON e.from_id = impact_walk.node_id
            WHERE impact_walk.depth < ?2
              AND instr(impact_walk.visited, '|' || e.to_id || '|') = 0
        )
        SELECT node_id, min(depth) as depth
        FROM impact_walk
        GROUP BY node_id
        ORDER BY depth ASC, node_id ASC
        LIMIT ?3",
    )
    .bind(start_id)
    .bind(depth_limit as i64)
    .bind(result_limit as i64)
    .fetch_all(pool)
    .await?;

    let items = rows
        .into_iter()
        .map(|r| {
            let node_id: String = r.get("node_id");
            let depth: i64 = r.get("depth");
            ImpactNode {
                node_id,
                depth: depth as usize,
            }
        })
        .collect();

    Ok(items)
}

pub async fn security_view(
    pool: &SqlitePool,
    view_name: &str,
    limit: usize,
) -> Result<serde_json::Value, StorageError> {
    let limit = limit.clamp(1, MAX_RESULT_ITEMS);
    match view_name.to_lowercase().as_str() {
        "unauthenticated" => {
            let rows = sqlx::query(
                "SELECT id, metadata FROM nodes 
                 WHERE kind = 'endpoint' AND json_extract(metadata, '$.status') = 200
                 ORDER BY id LIMIT ?1",
            )
            .bind(limit as i64)
            .fetch_all(pool)
            .await?;

            let items: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|row| {
                    let id: String = row.get("id");
                    let meta: String = row.get("metadata");
                    let meta_val: serde_json::Value =
                        serde_json::from_str(&meta).unwrap_or_default();
                    serde_json::json!({
                        "id": id,
                        "origin": meta_val.get("origin"),
                        "method": meta_val.get("method"),
                        "path": meta_val.get("path"),
                        "status": meta_val.get("status"),
                    })
                })
                .collect();
            Ok(
                serde_json::json!({ "view": "unauthenticated", "count": items.len(), "items": items }),
            )
        }
        "sensitive_params" => {
            let rows = sqlx::query(
                "SELECT id, metadata FROM nodes 
                 WHERE kind = 'endpoint' AND (
                   json_extract(metadata, '$.parameter_names') LIKE '%id%' OR
                   json_extract(metadata, '$.parameter_names') LIKE '%user%' OR
                   json_extract(metadata, '$.parameter_names') LIKE '%role%' OR
                   json_extract(metadata, '$.parameter_names') LIKE '%token%' OR
                   json_extract(metadata, '$.parameter_names') LIKE '%admin%' OR
                   json_extract(metadata, '$.parameter_names') LIKE '%file%' OR
                   json_extract(metadata, '$.parameter_names') LIKE '%url%' OR
                   json_extract(metadata, '$.parameter_names') LIKE '%redirect%'
                 )
                 ORDER BY id LIMIT ?1",
            )
            .bind(limit as i64)
            .fetch_all(pool)
            .await?;

            let items: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|row| {
                    let id: String = row.get("id");
                    let meta: String = row.get("metadata");
                    let meta_val: serde_json::Value =
                        serde_json::from_str(&meta).unwrap_or_default();
                    serde_json::json!({
                        "id": id,
                        "method": meta_val.get("method"),
                        "path": meta_val.get("path"),
                        "parameters": meta_val.get("parameter_names"),
                    })
                })
                .collect();
            Ok(
                serde_json::json!({ "view": "sensitive_params", "count": items.len(), "items": items }),
            )
        }
        "idor_candidates" => {
            let rows = sqlx::query(
                "SELECT id, metadata FROM nodes 
                 WHERE kind = 'endpoint' AND (
                   json_extract(metadata, '$.path') LIKE '%{id}%' OR
                   json_extract(metadata, '$.parameter_names') LIKE '%id%' OR
                   json_extract(metadata, '$.parameter_names') LIKE '%user_id%' OR
                   json_extract(metadata, '$.parameter_names') LIKE '%account%'
                 )
                 ORDER BY id LIMIT ?1",
            )
            .bind(limit as i64)
            .fetch_all(pool)
            .await?;

            let items: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|row| {
                    let id: String = row.get("id");
                    let meta: String = row.get("metadata");
                    let meta_val: serde_json::Value =
                        serde_json::from_str(&meta).unwrap_or_default();
                    serde_json::json!({
                        "id": id,
                        "method": meta_val.get("method"),
                        "path": meta_val.get("path"),
                        "parameters": meta_val.get("parameter_names"),
                    })
                })
                .collect();
            Ok(
                serde_json::json!({ "view": "idor_candidates", "count": items.len(), "items": items }),
            )
        }
        "untested_routes" => {
            let rows = sqlx::query(
                "SELECT n.id, n.metadata FROM nodes n
                 JOIN edges e ON n.id = e.to_id AND e.kind = 'discovers_route'
                 WHERE n.kind = 'endpoint'
                 ORDER BY n.id LIMIT ?1",
            )
            .bind(limit as i64)
            .fetch_all(pool)
            .await?;

            let items: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|row| {
                    let id: String = row.get("id");
                    let meta: String = row.get("metadata");
                    let meta_val: serde_json::Value =
                        serde_json::from_str(&meta).unwrap_or_default();
                    serde_json::json!({
                        "id": id,
                        "method": meta_val.get("method"),
                        "path": meta_val.get("path"),
                        "source": "client_js_route"
                    })
                })
                .collect();
            Ok(
                serde_json::json!({ "view": "untested_routes", "count": items.len(), "items": items }),
            )
        }
        _ => Err(StorageError::InvalidInput(format!(
            "unknown security view: {view_name}"
        ))),
    }
}

#[allow(dead_code)]
pub fn format_as_mermaid(edges: &[(String, String, String)]) -> String {
    let mut mermaid = String::from("```mermaid\ngraph LR\n");
    for (from, to, kind) in edges {
        mermaid.push_str(&format!("    \"{}\" -->|{}| \"{}\"\n", from, kind, to));
    }
    mermaid.push_str("```\n");
    mermaid
}

#[allow(dead_code)]
pub fn format_as_ascii_tree(paths: &[String]) -> String {
    let mut tree = String::from(".\n");
    for p in paths {
        tree.push_str(&format!("├── {}\n", p));
    }
    tree
}
