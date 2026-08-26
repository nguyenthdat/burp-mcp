use super::{Error, Result, sitegraph_capnp};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sitegraph::{
    ArtifactObservation, Cluster, CsvExport, Endpoint, EndpointPage, GraphDiff, GraphStatus,
    HistorySearchPage, ImpactNode, IssueObservation, JsonExport, NeighborPage, ShortestPath,
    SiteGraph, SitemapObservation, SyncBatch, SyncContext, SyncCoverage, SyncSummary,
    TechnologyObservation, TracePage, WebSocketObservation,
};

pub(crate) fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let mut payload = Vec::new();
    ciborium::ser::into_writer(value, &mut payload)
        .map_err(|error| Error::Codec(error.to_string()))?;
    Ok(payload)
}

pub(crate) fn decode<T: DeserializeOwned>(payload: &[u8]) -> Result<T> {
    let mut reader = std::io::Cursor::new(payload);
    let value =
        ciborium::de::from_reader(&mut reader).map_err(|error| Error::Codec(error.to_string()))?;
    if reader.position() != payload.len() as u64 {
        return Err(Error::Protocol(
            "typed payload contains trailing bytes".to_owned(),
        ));
    }
    Ok(value)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SyncTask {
    batch: SyncBatchTask,
    context: SyncContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SyncBatchTask {
    sitemap: Vec<SitemapTask>,
    issues: Vec<IssueObservation>,
    technologies: Vec<TechnologyObservation>,
    artifacts: Vec<ArtifactObservation>,
    websocket_messages: Vec<WebSocketTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SitemapTask {
    url: String,
    method: String,
    status: u32,
    content_type: String,
    response_body: Vec<u8>,
    request_bytes: Vec<u8>,
    response_bytes: Vec<u8>,
    redirect_url: String,
    response_links: Vec<String>,
    form_actions: Vec<String>,
    script_sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WebSocketTask {
    id: String,
    web_socket_id: String,
    direction: String,
    upgrade_url: String,
    payload: Vec<u8>,
    edited_payload: Vec<u8>,
}

impl SyncTask {
    pub(crate) fn new(batch: &SyncBatch, context: &SyncContext) -> Self {
        Self {
            batch: SyncBatchTask {
                sitemap: batch.sitemap.iter().map(SitemapTask::from).collect(),
                issues: batch.issues.clone(),
                technologies: batch.technologies.clone(),
                artifacts: batch.artifacts.clone(),
                websocket_messages: batch
                    .websocket_messages
                    .iter()
                    .map(WebSocketTask::from)
                    .collect(),
            },
            context: context.clone(),
        }
    }

    fn into_parts(self) -> (SyncBatch, SyncContext) {
        (
            SyncBatch {
                sitemap: self.batch.sitemap.into_iter().map(Into::into).collect(),
                issues: self.batch.issues,
                technologies: self.batch.technologies,
                artifacts: self.batch.artifacts,
                websocket_messages: self
                    .batch
                    .websocket_messages
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            },
            self.context,
        )
    }
}

impl From<&SitemapObservation> for SitemapTask {
    fn from(value: &SitemapObservation) -> Self {
        Self {
            url: value.url.clone(),
            method: value.method.clone(),
            status: value.status,
            content_type: value.content_type.clone(),
            response_body: value.response_body.clone(),
            request_bytes: value.request_bytes.clone(),
            response_bytes: value.response_bytes.clone(),
            redirect_url: value.redirect_url.clone(),
            response_links: value.response_links.clone(),
            form_actions: value.form_actions.clone(),
            script_sources: value.script_sources.clone(),
        }
    }
}

impl From<SitemapTask> for SitemapObservation {
    fn from(value: SitemapTask) -> Self {
        Self {
            url: value.url,
            method: value.method,
            status: value.status,
            content_type: value.content_type,
            response_body: value.response_body,
            request_bytes: value.request_bytes,
            response_bytes: value.response_bytes,
            redirect_url: value.redirect_url,
            response_links: value.response_links,
            form_actions: value.form_actions,
            script_sources: value.script_sources,
        }
    }
}

impl From<&WebSocketObservation> for WebSocketTask {
    fn from(value: &WebSocketObservation) -> Self {
        Self {
            id: value.id.clone(),
            web_socket_id: value.web_socket_id.clone(),
            direction: value.direction.clone(),
            upgrade_url: value.upgrade_url.clone(),
            payload: value.payload.clone(),
            edited_payload: value.edited_payload.clone(),
        }
    }
}

impl From<WebSocketTask> for WebSocketObservation {
    fn from(value: WebSocketTask) -> Self {
        Self {
            id: value.id,
            web_socket_id: value.web_socket_id,
            direction: value.direction,
            upgrade_url: value.upgrade_url,
            payload: value.payload,
            edited_payload: value.edited_payload,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CheckpointTask {
    pub(crate) source: String,
    pub(crate) scope: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct SearchTask {
    pub(crate) query: String,
    pub(crate) cursor: u64,
    pub(crate) limit: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct SearchHistoryTask {
    pub(crate) query: String,
    pub(crate) source: Option<String>,
    pub(crate) cursor: u64,
    pub(crate) limit: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct EndpointTask {
    pub(crate) id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct NeighborsTask {
    pub(crate) node_id: String,
    pub(crate) cursor: u64,
    pub(crate) limit: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct DiffTask {
    pub(crate) since: i64,
    pub(crate) cursor: u64,
    pub(crate) limit: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PageTask {
    pub(crate) cursor: u64,
    pub(crate) limit: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct TraceTask {
    pub(crate) start_id: String,
    pub(crate) max_depth: u32,
    pub(crate) limit: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ShortestPathTask {
    pub(crate) from_id: String,
    pub(crate) to_id: String,
    pub(crate) max_depth: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct LimitTask {
    pub(crate) limit: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ImpactTask {
    pub(crate) node_id: String,
    pub(crate) max_depth: usize,
    pub(crate) limit: usize,
}

pub(crate) enum Task {
    Status,
    Checkpoint(CheckpointTask),
    SyncWithContext(Box<SyncTask>),
    Search(SearchTask),
    SearchHistory(SearchHistoryTask),
    Endpoint(EndpointTask),
    Neighbors(NeighborsTask),
    Trace(TraceTask),
    ShortestPath(ShortestPathTask),
    Diff(DiffTask),
    ExportJson(PageTask),
    ExportCsv(PageTask),
    EndpointClusters(LimitTask),
    Impact(ImpactTask),
    ExportExactJson(PageTask),
}

impl Task {
    pub(crate) fn write_capnp(&self, mut output: sitegraph_capnp::task::Builder<'_>) -> Result<()> {
        match self {
            Self::Status => output.set_status(()),
            Self::Checkpoint(task) => {
                let mut value = output.init_checkpoint();
                value.set_source(task.source.as_str());
                value.set_scope(task.scope.as_str());
            }
            Self::SyncWithContext(task) => {
                output.set_sync_with_context(&encode(task)?);
            }
            Self::Search(task) => {
                let mut value = output.init_search();
                value.set_query(task.query.as_str());
                value.set_cursor(task.cursor);
                value.set_limit(task.limit);
            }
            Self::SearchHistory(task) => {
                let mut value = output.init_search_history();
                value.set_query(task.query.as_str());
                value.set_source(task.source.as_deref().unwrap_or_default());
                value.set_cursor(task.cursor);
                value.set_limit(task.limit);
            }
            Self::Endpoint(task) => {
                output.init_endpoint().set_id(task.id.as_str());
            }
            Self::Neighbors(task) => {
                let mut value = output.init_neighbors();
                value.set_node_id(task.node_id.as_str());
                value.set_cursor(task.cursor);
                value.set_limit(task.limit);
            }
            Self::Trace(task) => {
                let mut value = output.init_trace();
                value.set_start_id(task.start_id.as_str());
                value.set_max_depth(task.max_depth);
                value.set_limit(task.limit);
            }
            Self::ShortestPath(task) => {
                let mut value = output.init_shortest_path();
                value.set_from_id(task.from_id.as_str());
                value.set_to_id(task.to_id.as_str());
                value.set_max_depth(task.max_depth as u64);
            }
            Self::Diff(task) => {
                let mut value = output.init_diff();
                value.set_since(task.since);
                value.set_cursor(task.cursor);
                value.set_limit(task.limit);
            }
            Self::ExportJson(task) => {
                let mut value = output.init_export_json();
                value.set_cursor(task.cursor);
                value.set_limit(task.limit);
            }
            Self::ExportCsv(task) => {
                let mut value = output.init_export_csv();
                value.set_cursor(task.cursor);
                value.set_limit(task.limit);
            }
            Self::EndpointClusters(task) => {
                output.init_endpoint_clusters().set_limit(task.limit as u64);
            }
            Self::Impact(task) => {
                let mut value = output.init_impact();
                value.set_node_id(task.node_id.as_str());
                value.set_max_depth(task.max_depth as u64);
                value.set_limit(task.limit as u64);
            }
            Self::ExportExactJson(task) => {
                let mut value = output.init_export_exact_json();
                value.set_cursor(task.cursor);
                value.set_limit(task.limit);
            }
        }
        Ok(())
    }

    pub(crate) fn read_capnp(input: sitegraph_capnp::task::Reader<'_>) -> Result<Self> {
        use sitegraph_capnp::task::{
            Checkpoint, Diff, Endpoint, EndpointClusters, ExportCsv, ExportExactJson, ExportJson,
            Impact, Neighbors, Search, SearchHistory, ShortestPath, Status, SyncWithContext, Trace,
        };
        match input
            .which()
            .map_err(|error| Error::Protocol(error.to_string()))?
        {
            Status(()) => Ok(Self::Status),
            Checkpoint(value) => {
                let value = value.map_err(Error::Capnp)?;
                Ok(Self::Checkpoint(CheckpointTask {
                    source: text(value.get_source())?,
                    scope: text(value.get_scope())?,
                }))
            }
            SyncWithContext(value) => Ok(Self::SyncWithContext(Box::new(decode(
                value.map_err(Error::Capnp)?,
            )?))),
            Search(value) => {
                let value = value.map_err(Error::Capnp)?;
                Ok(Self::Search(SearchTask {
                    query: text(value.get_query())?,
                    cursor: value.get_cursor(),
                    limit: value.get_limit(),
                }))
            }
            SearchHistory(value) => {
                let value = value.map_err(Error::Capnp)?;
                let source = text(value.get_source())?;
                Ok(Self::SearchHistory(SearchHistoryTask {
                    query: text(value.get_query())?,
                    source: (!source.is_empty()).then_some(source),
                    cursor: value.get_cursor(),
                    limit: value.get_limit(),
                }))
            }
            Endpoint(value) => {
                let value = value.map_err(Error::Capnp)?;
                Ok(Self::Endpoint(EndpointTask {
                    id: text(value.get_id())?,
                }))
            }
            Neighbors(value) => {
                let value = value.map_err(Error::Capnp)?;
                Ok(Self::Neighbors(NeighborsTask {
                    node_id: text(value.get_node_id())?,
                    cursor: value.get_cursor(),
                    limit: value.get_limit(),
                }))
            }
            Trace(value) => {
                let value = value.map_err(Error::Capnp)?;
                Ok(Self::Trace(TraceTask {
                    start_id: text(value.get_start_id())?,
                    max_depth: value.get_max_depth(),
                    limit: value.get_limit(),
                }))
            }
            ShortestPath(value) => {
                let value = value.map_err(Error::Capnp)?;
                Ok(Self::ShortestPath(ShortestPathTask {
                    from_id: text(value.get_from_id())?,
                    to_id: text(value.get_to_id())?,
                    max_depth: usize_value(value.get_max_depth(), "max_depth")?,
                }))
            }
            Diff(value) => {
                let value = value.map_err(Error::Capnp)?;
                Ok(Self::Diff(DiffTask {
                    since: value.get_since(),
                    cursor: value.get_cursor(),
                    limit: value.get_limit(),
                }))
            }
            ExportJson(value) => {
                let value = value.map_err(Error::Capnp)?;
                Ok(Self::ExportJson(PageTask {
                    cursor: value.get_cursor(),
                    limit: value.get_limit(),
                }))
            }
            ExportCsv(value) => {
                let value = value.map_err(Error::Capnp)?;
                Ok(Self::ExportCsv(PageTask {
                    cursor: value.get_cursor(),
                    limit: value.get_limit(),
                }))
            }
            EndpointClusters(value) => {
                let value = value.map_err(Error::Capnp)?;
                Ok(Self::EndpointClusters(LimitTask {
                    limit: usize_value(value.get_limit(), "limit")?,
                }))
            }
            Impact(value) => {
                let value = value.map_err(Error::Capnp)?;
                Ok(Self::Impact(ImpactTask {
                    node_id: text(value.get_node_id())?,
                    max_depth: usize_value(value.get_max_depth(), "max_depth")?,
                    limit: usize_value(value.get_limit(), "limit")?,
                }))
            }
            ExportExactJson(value) => {
                let value = value.map_err(Error::Capnp)?;
                Ok(Self::ExportExactJson(PageTask {
                    cursor: value.get_cursor(),
                    limit: value.get_limit(),
                }))
            }
        }
    }
}

fn text(value: capnp::Result<capnp::text::Reader<'_>>) -> Result<String> {
    value
        .map_err(Error::Capnp)?
        .to_str()
        .map(str::to_owned)
        .map_err(|error| Error::Protocol(error.to_string()))
}

fn usize_value(value: u64, name: &str) -> Result<usize> {
    usize::try_from(value).map_err(|_| Error::Protocol(format!("{name} exceeds usize")))
}

pub(crate) enum DispatchResponse {
    Status(GraphStatus),
    Checkpoint(Option<(String, SyncCoverage)>),
    SyncWithContext(SyncSummary),
    Search(EndpointPage),
    SearchHistory(HistorySearchPage),
    Endpoint(Option<Endpoint>),
    Neighbors(NeighborPage),
    Diff(GraphDiff),
    ExportJson(JsonExport),
    ExportExactJson(JsonExport),
    ExportCsv(CsvExport),
    Trace(TracePage),
    ShortestPath(ShortestPath),
    EndpointClusters(Vec<Cluster>),
    Impact(Vec<ImpactNode>),
}

impl DispatchResponse {
    pub(crate) fn encode(self) -> Result<Vec<u8>> {
        match self {
            Self::Status(value) => encode(&value),
            Self::Checkpoint(value) => encode(&value),
            Self::SyncWithContext(value) => encode(&value),
            Self::Search(value) => encode(&value),
            Self::SearchHistory(value) => encode(&value),
            Self::Endpoint(value) => encode(&value),
            Self::Neighbors(value) => encode(&value),
            Self::Diff(value) => encode(&value),
            Self::ExportJson(value) => encode(&value),
            Self::ExportExactJson(value) => encode(&value),
            Self::ExportCsv(value) => encode(&value),
            Self::Trace(value) => encode(&value),
            Self::ShortestPath(value) => encode(&value),
            Self::EndpointClusters(value) => encode(&value),
            Self::Impact(value) => encode(&value),
        }
    }
}

pub(crate) async fn dispatch(graph: &SiteGraph, task: Task) -> Result<DispatchResponse> {
    match task {
        Task::Status => Ok(DispatchResponse::Status(graph.status().await?)),
        Task::Checkpoint(task) => Ok(DispatchResponse::Checkpoint(
            graph.checkpoint(&task.source, &task.scope).await?,
        )),
        Task::SyncWithContext(task) => {
            let (batch, context) = (*task).into_parts();
            Ok(DispatchResponse::SyncWithContext(
                graph.sync_with_context(&batch, &context).await?,
            ))
        }
        Task::Search(task) => Ok(DispatchResponse::Search(
            graph.search(&task.query, task.cursor, task.limit).await?,
        )),
        Task::SearchHistory(task) => Ok(DispatchResponse::SearchHistory(
            graph
                .search_history(&task.query, task.source.as_deref(), task.cursor, task.limit)
                .await?,
        )),
        Task::Endpoint(task) => Ok(DispatchResponse::Endpoint(graph.endpoint(&task.id).await?)),
        Task::Neighbors(task) => Ok(DispatchResponse::Neighbors(
            graph
                .neighbors(&task.node_id, task.cursor, task.limit)
                .await?,
        )),
        Task::Diff(task) => Ok(DispatchResponse::Diff(
            graph.diff(task.since, task.cursor, task.limit).await?,
        )),
        Task::ExportJson(task) => Ok(DispatchResponse::ExportJson(
            graph.export_json(task.cursor, task.limit).await?,
        )),
        Task::ExportExactJson(task) => Ok(DispatchResponse::ExportExactJson(
            graph.export_exact_json(task.cursor, task.limit).await?,
        )),
        Task::ExportCsv(task) => Ok(DispatchResponse::ExportCsv(
            graph.export_csv(task.cursor, task.limit).await?,
        )),
        Task::Trace(task) => Ok(DispatchResponse::Trace(
            graph
                .trace(&task.start_id, task.max_depth, task.limit)
                .await?,
        )),
        Task::ShortestPath(task) => Ok(DispatchResponse::ShortestPath(
            graph
                .shortest_path(&task.from_id, &task.to_id, task.max_depth)
                .await?,
        )),
        Task::EndpointClusters(task) => Ok(DispatchResponse::EndpointClusters(
            graph.endpoint_clusters(task.limit).await?,
        )),
        Task::Impact(task) => Ok(DispatchResponse::Impact(
            graph
                .impact(&task.node_id, task.max_depth, task.limit)
                .await?,
        )),
    }
}
