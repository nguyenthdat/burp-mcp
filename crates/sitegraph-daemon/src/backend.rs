use super::{Client, Result};
use crate::tasks::{
    CheckpointTask, DiffTask, EndpointTask, ImpactTask, LimitTask, NeighborsTask, PageTask,
    SearchHistoryTask, SearchTask, ShortestPathTask, SyncTask, Task, TraceTask,
};
use sitegraph::{
    Cluster, CsvExport, Endpoint, EndpointPage, GraphDiff, GraphStatus, HistorySearchPage,
    ImpactNode, JsonExport, NeighborPage, ShortestPath, SiteGraph, SyncBatch, SyncContext,
    SyncCoverage, SyncSummary, TracePage,
};
use std::sync::Arc;

#[derive(Clone)]
pub enum GraphBackend {
    Local(Arc<SiteGraph>),
    Remote(Client),
}

impl GraphBackend {
    pub async fn status(&self) -> Result<GraphStatus> {
        match self {
            Self::Local(graph) => Ok(graph.status().await?),
            Self::Remote(client) => client.request(Task::Status).await,
        }
    }

    pub async fn checkpoint(
        &self,
        source: &str,
        scope: &str,
    ) -> Result<Option<(String, SyncCoverage)>> {
        match self {
            Self::Local(graph) => Ok(graph.checkpoint(source, scope).await?),
            Self::Remote(client) => {
                client
                    .request(Task::Checkpoint(CheckpointTask {
                        source: source.to_owned(),
                        scope: scope.to_owned(),
                    }))
                    .await
            }
        }
    }

    pub async fn sync_with_context(
        &self,
        batch: &SyncBatch,
        context: &SyncContext,
    ) -> Result<SyncSummary> {
        match self {
            Self::Local(graph) => Ok(graph.sync_with_context(batch, context).await?),
            Self::Remote(client) => {
                client
                    .request(Task::SyncWithContext(Box::new(SyncTask::new(
                        batch, context,
                    ))))
                    .await
            }
        }
    }

    pub async fn search(&self, query: &str, cursor: u64, limit: u64) -> Result<EndpointPage> {
        match self {
            Self::Local(graph) => Ok(graph.search(query, cursor, limit).await?),
            Self::Remote(client) => {
                client
                    .request(Task::Search(SearchTask {
                        query: query.to_owned(),
                        cursor,
                        limit,
                    }))
                    .await
            }
        }
    }

    pub async fn search_history(
        &self,
        query: &str,
        source: Option<&str>,
        cursor: u64,
        limit: u64,
    ) -> Result<HistorySearchPage> {
        match self {
            Self::Local(graph) => Ok(graph.search_history(query, source, cursor, limit).await?),
            Self::Remote(client) => {
                client
                    .request(Task::SearchHistory(SearchHistoryTask {
                        query: query.to_owned(),
                        source: source.map(str::to_owned),
                        cursor,
                        limit,
                    }))
                    .await
            }
        }
    }

    pub async fn endpoint(&self, id: &str) -> Result<Option<Endpoint>> {
        match self {
            Self::Local(graph) => Ok(graph.endpoint(id).await?),
            Self::Remote(client) => {
                client
                    .request(Task::Endpoint(EndpointTask { id: id.to_owned() }))
                    .await
            }
        }
    }

    pub async fn neighbors(&self, node_id: &str, cursor: u64, limit: u64) -> Result<NeighborPage> {
        match self {
            Self::Local(graph) => Ok(graph.neighbors(node_id, cursor, limit).await?),
            Self::Remote(client) => {
                client
                    .request(Task::Neighbors(NeighborsTask {
                        node_id: node_id.to_owned(),
                        cursor,
                        limit,
                    }))
                    .await
            }
        }
    }

    pub async fn diff(&self, since: i64, cursor: u64, limit: u64) -> Result<GraphDiff> {
        match self {
            Self::Local(graph) => Ok(graph.diff(since, cursor, limit).await?),
            Self::Remote(client) => {
                client
                    .request(Task::Diff(DiffTask {
                        since,
                        cursor,
                        limit,
                    }))
                    .await
            }
        }
    }

    pub async fn export_json(&self, cursor: u64, limit: u64) -> Result<JsonExport> {
        match self {
            Self::Local(graph) => Ok(graph.export_json(cursor, limit).await?),
            Self::Remote(client) => {
                client
                    .request(Task::ExportJson(PageTask { cursor, limit }))
                    .await
            }
        }
    }

    pub async fn export_exact_json(&self, cursor: u64, limit: u64) -> Result<JsonExport> {
        match self {
            Self::Local(graph) => Ok(graph.export_exact_json(cursor, limit).await?),
            Self::Remote(client) => {
                client
                    .request(Task::ExportExactJson(PageTask { cursor, limit }))
                    .await
            }
        }
    }

    pub async fn export_csv(&self, cursor: u64, limit: u64) -> Result<CsvExport> {
        match self {
            Self::Local(graph) => Ok(graph.export_csv(cursor, limit).await?),
            Self::Remote(client) => {
                client
                    .request(Task::ExportCsv(PageTask { cursor, limit }))
                    .await
            }
        }
    }

    pub async fn trace(&self, start_id: &str, max_depth: u32, limit: u32) -> Result<TracePage> {
        match self {
            Self::Local(graph) => Ok(graph.trace(start_id, max_depth, limit).await?),
            Self::Remote(client) => {
                client
                    .request(Task::Trace(TraceTask {
                        start_id: start_id.to_owned(),
                        max_depth,
                        limit,
                    }))
                    .await
            }
        }
    }

    pub async fn shortest_path(
        &self,
        from_id: &str,
        to_id: &str,
        max_depth: usize,
    ) -> Result<ShortestPath> {
        match self {
            Self::Local(graph) => Ok(graph.shortest_path(from_id, to_id, max_depth).await?),
            Self::Remote(client) => {
                client
                    .request(Task::ShortestPath(ShortestPathTask {
                        from_id: from_id.to_owned(),
                        to_id: to_id.to_owned(),
                        max_depth,
                    }))
                    .await
            }
        }
    }

    pub async fn endpoint_clusters(&self, limit: usize) -> Result<Vec<Cluster>> {
        match self {
            Self::Local(graph) => Ok(graph.endpoint_clusters(limit).await?),
            Self::Remote(client) => {
                client
                    .request(Task::EndpointClusters(LimitTask { limit }))
                    .await
            }
        }
    }

    pub async fn impact(
        &self,
        node_id: &str,
        max_depth: usize,
        limit: usize,
    ) -> Result<Vec<ImpactNode>> {
        match self {
            Self::Local(graph) => Ok(graph.impact(node_id, max_depth, limit).await?),
            Self::Remote(client) => {
                client
                    .request(Task::Impact(ImpactTask {
                        node_id: node_id.to_owned(),
                        max_depth,
                        limit,
                    }))
                    .await
            }
        }
    }

    pub async fn security_view(&self, view_name: &str, limit: usize) -> Result<serde_json::Value> {
        match self {
            Self::Local(graph) => Ok(graph.security_view(view_name, limit).await?),
            Self::Remote(_) => Ok(serde_json::json!({ "view": view_name, "items": [] })),
        }
    }

    pub async fn import_openapi(&self, content: &str, base_url: &str) -> Result<SyncSummary> {
        match self {
            Self::Local(graph) => Ok(graph.import_openapi(content, base_url).await?),
            Self::Remote(_) => Err(crate::Error::Protocol(
                "remote import_openapi not supported".to_string(),
            )),
        }
    }
}
