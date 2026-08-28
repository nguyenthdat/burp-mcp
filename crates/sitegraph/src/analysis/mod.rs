use crate::model::NodeMetadata;
use crate::storage::StorageError;
#[cfg(test)]
mod tests;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::{HashMap, HashSet, VecDeque};

const MAX_ANALYSIS_NODES: usize = 250_000;
const MAX_ANALYSIS_EDGES: usize = 1_000_000;
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
    let graph = load(pool).await?;
    if !graph.nodes.contains(from_id) || !graph.nodes.contains(to_id) {
        return Ok(ShortestPath {
            items: Vec::new(),
            depth: 0,
            truncated: false,
        });
    }
    let depth_limit = max_depth.clamp(1, MAX_PATH_DEPTH);
    let mut queue = VecDeque::from([(from_id.to_owned(), 0_usize)]);
    let mut previous: HashMap<String, (String, String)> = HashMap::new();
    let mut visited = HashSet::from([from_id.to_owned()]);
    let mut truncated = false;
    while let Some((node_id, depth)) = queue.pop_front() {
        if node_id == to_id {
            break;
        }
        if depth == depth_limit {
            truncated = true;
            continue;
        }
        for (next, edge_id) in graph.adjacency.get(&node_id).into_iter().flatten() {
            if visited.insert(next.clone()) {
                previous.insert(next.clone(), (node_id.clone(), edge_id.clone()));
                queue.push_back((next.clone(), depth + 1));
            }
        }
    }
    if !visited.contains(to_id) {
        return Ok(ShortestPath {
            items: Vec::new(),
            depth: 0,
            truncated,
        });
    }
    let mut current = to_id.to_owned();
    let mut reverse = vec![PathStep {
        node_id: current.clone(),
        edge_id: None,
    }];
    while current != from_id {
        let Some((parent, edge_id)) = previous.get(&current) else {
            break;
        };
        reverse.push(PathStep {
            node_id: parent.clone(),
            edge_id: Some(edge_id.clone()),
        });
        current = parent.clone();
    }
    reverse.reverse();
    let depth = reverse.len().saturating_sub(1);
    Ok(ShortestPath {
        items: reverse,
        depth,
        truncated,
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
    let graph = load(pool).await?;
    let depth_limit = max_depth.clamp(1, MAX_PATH_DEPTH);
    let result_limit = limit.clamp(1, MAX_RESULT_ITEMS);
    let mut queue = VecDeque::from([(start_id.to_owned(), 0_usize)]);
    let mut visited = HashSet::from([start_id.to_owned()]);
    let mut result = Vec::new();
    while let Some((node_id, depth)) = queue.pop_front() {
        if depth == depth_limit {
            continue;
        }
        for (next, _) in graph.adjacency.get(&node_id).into_iter().flatten() {
            if visited.insert(next.clone()) {
                result.push(ImpactNode {
                    node_id: next.clone(),
                    depth: depth + 1,
                });
                if result.len() == result_limit {
                    return Ok(result);
                }
                queue.push_back((next.clone(), depth + 1));
            }
        }
    }
    Ok(result)
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

struct LoadedGraph {
    nodes: HashSet<String>,
    adjacency: HashMap<String, Vec<(String, String)>>,
}

async fn load(pool: &SqlitePool) -> Result<LoadedGraph, StorageError> {
    let nodes = sqlx::query("SELECT id FROM nodes ORDER BY id LIMIT ?1")
        .bind(i64::try_from(MAX_ANALYSIS_NODES + 1).unwrap_or(i64::MAX))
        .fetch_all(pool)
        .await?;
    if nodes.len() > MAX_ANALYSIS_NODES {
        return Err(StorageError::InvalidInput(format!(
            "analysis node limit exceeded: {MAX_ANALYSIS_NODES}"
        )));
    }
    let node_ids = nodes
        .into_iter()
        .map(|row| row.get::<String, _>("id"))
        .collect::<HashSet<_>>();
    let edges = sqlx::query("SELECT id, from_id, to_id FROM edges ORDER BY id LIMIT ?1")
        .bind(i64::try_from(MAX_ANALYSIS_EDGES + 1).unwrap_or(i64::MAX))
        .fetch_all(pool)
        .await?;
    if edges.len() > MAX_ANALYSIS_EDGES {
        return Err(StorageError::InvalidInput(format!(
            "analysis edge limit exceeded: {MAX_ANALYSIS_EDGES}"
        )));
    }
    let mut adjacency: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for row in edges {
        adjacency
            .entry(row.get("from_id"))
            .or_default()
            .push((row.get("to_id"), row.get("id")));
    }
    for neighbors in adjacency.values_mut() {
        neighbors.sort_unstable();
    }
    Ok(LoadedGraph {
        nodes: node_ids,
        adjacency,
    })
}
