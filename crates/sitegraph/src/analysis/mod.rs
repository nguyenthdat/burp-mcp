use crate::storage::StorageError;
#[cfg(test)]
mod tests;
use serde::Serialize;
use serde_json::Value;
use sqlx::{Row, SqlitePool};
use std::collections::{HashMap, HashSet, VecDeque};

const MAX_ANALYSIS_NODES: usize = 25_000;
const MAX_ANALYSIS_EDGES: usize = 100_000;
const MAX_PATH_DEPTH: usize = 16;
const MAX_RESULT_ITEMS: usize = 500;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PathStep {
    pub node_id: String,
    pub edge_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ShortestPath {
    pub items: Vec<PathStep>,
    pub depth: usize,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Cluster {
    pub key: String,
    pub endpoint_ids: Vec<String>,
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
        let metadata: Value = serde_json::from_str(&row.get::<String, _>("metadata"))?;
        let origin = metadata["origin"].as_str().unwrap_or_default();
        let prefix = metadata["path"]
            .as_str()
            .unwrap_or("/")
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
