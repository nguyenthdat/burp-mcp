use base64::{Engine as _, engine::general_purpose::STANDARD};
use fs2::FileExt as _;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use sitegraph::{
    Cluster, CsvExport, Endpoint, EndpointPage, GraphDiff, GraphStatus, ImpactNode, JsonExport,
    NeighborPage, ShortestPath, SiteGraph, SyncBatch, SyncContext, SyncCoverage, SyncSummary,
    TracePage, enrichment::RulePack,
};
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt as _, AsyncWriteExt as _, BufReader};
use tokio::net::{TcpListener, TcpStream};

const MAX_FRAME_BYTES: usize = 128 * 1024 * 1024;
const STARTUP_ATTEMPTS: usize = 50;
const STARTUP_DELAY: Duration = Duration::from_millis(100);

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("sitegraph daemon I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("sitegraph daemon protocol failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("sitegraph daemon storage failed: {0}")]
    Storage(#[from] sitegraph::StorageError),
    #[error("sitegraph daemon rejected request: {0}")]
    Remote(String),
    #[error("sitegraph daemon protocol violation: {0}")]
    Protocol(String),
    #[error("another sitegraph daemon already owns {0}")]
    AlreadyRunning(PathBuf),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointFile {
    pub address: String,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Request {
    token: String,
    operation: String,
    #[serde(default)]
    payload: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    ok: bool,
    payload: Value,
    #[serde(default)]
    error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SyncRequest {
    batch: SyncBatch,
    context: SyncContext,
}

#[derive(Clone)]
pub enum GraphBackend {
    Local(Arc<SiteGraph>),
    Remote(Client),
}

impl GraphBackend {
    pub async fn status(&self) -> Result<GraphStatus> {
        match self {
            Self::Local(graph) => Ok(graph.status().await?),
            Self::Remote(client) => client.request("status", Value::Null).await,
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
                    .request("checkpoint", json!({"source": source, "scope": scope}))
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
                    .request(
                        "sync_with_context",
                        serde_json::to_value(SyncRequest {
                            batch: batch.clone(),
                            context: context.clone(),
                        })?,
                    )
                    .await
            }
        }
    }

    pub async fn search(&self, query: &str, cursor: u64, limit: u64) -> Result<EndpointPage> {
        match self {
            Self::Local(graph) => Ok(graph.search(query, cursor, limit).await?),
            Self::Remote(client) => {
                client
                    .request(
                        "search",
                        json!({"query": query, "cursor": cursor, "limit": limit}),
                    )
                    .await
            }
        }
    }

    pub async fn endpoint(&self, id: &str) -> Result<Option<Endpoint>> {
        match self {
            Self::Local(graph) => Ok(graph.endpoint(id).await?),
            Self::Remote(client) => client.request("endpoint", json!({"id": id})).await,
        }
    }

    pub async fn neighbors(&self, node_id: &str, cursor: u64, limit: u64) -> Result<NeighborPage> {
        match self {
            Self::Local(graph) => Ok(graph.neighbors(node_id, cursor, limit).await?),
            Self::Remote(client) => {
                client
                    .request(
                        "neighbors",
                        json!({"node_id": node_id, "cursor": cursor, "limit": limit}),
                    )
                    .await
            }
        }
    }

    pub async fn diff(&self, since: i64, cursor: u64, limit: u64) -> Result<GraphDiff> {
        match self {
            Self::Local(graph) => Ok(graph.diff(since, cursor, limit).await?),
            Self::Remote(client) => {
                client
                    .request(
                        "diff",
                        json!({"since": since, "cursor": cursor, "limit": limit}),
                    )
                    .await
            }
        }
    }

    pub async fn export_json(&self, cursor: u64, limit: u64) -> Result<JsonExport> {
        match self {
            Self::Local(graph) => Ok(graph.export_json(cursor, limit).await?),
            Self::Remote(client) => {
                client
                    .request("export_json", json!({"cursor": cursor, "limit": limit}))
                    .await
            }
        }
    }

    pub async fn export_exact_json(&self, cursor: u64, limit: u64) -> Result<JsonExport> {
        match self {
            Self::Local(graph) => Ok(graph.export_exact_json(cursor, limit).await?),
            Self::Remote(client) => {
                client
                    .request(
                        "export_exact_json",
                        json!({"cursor": cursor, "limit": limit}),
                    )
                    .await
            }
        }
    }

    pub async fn export_csv(&self, cursor: u64, limit: u64) -> Result<CsvExport> {
        match self {
            Self::Local(graph) => Ok(graph.export_csv(cursor, limit).await?),
            Self::Remote(client) => {
                client
                    .request("export_csv", json!({"cursor": cursor, "limit": limit}))
                    .await
            }
        }
    }

    pub async fn trace(&self, start_id: &str, max_depth: u32, limit: u32) -> Result<TracePage> {
        match self {
            Self::Local(graph) => Ok(graph.trace(start_id, max_depth, limit).await?),
            Self::Remote(client) => {
                client
                    .request(
                        "trace",
                        json!({"start_id": start_id, "max_depth": max_depth, "limit": limit}),
                    )
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
                    .request(
                        "shortest_path",
                        json!({"from_id": from_id, "to_id": to_id, "max_depth": max_depth}),
                    )
                    .await
            }
        }
    }

    pub async fn endpoint_clusters(&self, limit: usize) -> Result<Vec<Cluster>> {
        match self {
            Self::Local(graph) => Ok(graph.endpoint_clusters(limit).await?),
            Self::Remote(client) => {
                client
                    .request("endpoint_clusters", json!({"limit": limit}))
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
                    .request(
                        "impact",
                        json!({"node_id": node_id, "max_depth": max_depth, "limit": limit}),
                    )
                    .await
            }
        }
    }
}

#[derive(Clone)]
pub struct Client {
    endpoint_file: Arc<PathBuf>,
}

impl Client {
    pub fn new(endpoint_file: impl Into<PathBuf>) -> Self {
        Self {
            endpoint_file: Arc::new(endpoint_file.into()),
        }
    }

    pub fn endpoint_file(&self) -> &Path {
        &self.endpoint_file
    }

    async fn request<T: DeserializeOwned>(&self, operation: &str, payload: Value) -> Result<T> {
        let endpoint = read_endpoint_file(&self.endpoint_file).await?;
        let mut stream = TcpStream::connect(&endpoint.address).await?;
        let request = Request {
            token: endpoint.token,
            operation: operation.to_owned(),
            payload,
        };
        let mut frame = serde_json::to_vec(&request)?;
        if frame.len() > MAX_FRAME_BYTES {
            return Err(Error::Protocol("request exceeds 128 MiB".to_owned()));
        }
        frame.push(b'\n');
        stream.write_all(&frame).await?;
        stream.flush().await?;

        let mut reader = BufReader::new(stream);
        let mut response = Vec::new();
        let bytes = reader.read_until(b'\n', &mut response).await?;
        if bytes == 0 || bytes > MAX_FRAME_BYTES {
            return Err(Error::Protocol("invalid response frame size".to_owned()));
        }
        let response: Response = serde_json::from_slice(&response)?;
        if !response.ok {
            return Err(Error::Remote(response.error));
        }
        Ok(serde_json::from_value(response.payload)?)
    }
}

pub fn endpoint_path(graph_path: &Path) -> PathBuf {
    graph_path.with_extension("daemon.json")
}

pub async fn connect_or_spawn(
    graph_path: &Path,
    graph_id: &str,
    rules_path: &Path,
) -> Result<Client> {
    let endpoint_file = endpoint_path(graph_path);
    let client = Client::new(endpoint_file.clone());
    if client_matches(&client, graph_id).await {
        return Ok(client);
    }
    let executable = std::env::current_exe()?;
    let sibling = executable.with_file_name(if cfg!(windows) {
        "sitegraph-daemon.exe"
    } else {
        "sitegraph-daemon"
    });
    let mut command = if sibling.is_file() {
        std::process::Command::new(sibling)
    } else {
        let mut command = std::process::Command::new(executable);
        command.arg("__sitegraph-daemon");
        command
    };
    command
        .arg("--graph-path")
        .arg(graph_path)
        .arg("--graph-id")
        .arg(graph_id)
        .arg("--endpoint-file")
        .arg(&endpoint_file)
        .arg("--rules-path")
        .arg(rules_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()?;

    for _ in 0..STARTUP_ATTEMPTS {
        if client_matches(&client, graph_id).await {
            return Ok(client);
        }
        tokio::time::sleep(STARTUP_DELAY).await;
    }
    Err(Error::Protocol(format!(
        "daemon did not become ready at {}",
        endpoint_file.display()
    )))
}

async fn client_matches(client: &Client, graph_id: &str) -> bool {
    matches!(
        client.request::<GraphStatus>("status", Value::Null).await,
        Ok(status) if status.graph_id == graph_id
    )
}

pub struct Server {
    endpoint_file: PathBuf,
    token: Arc<str>,
    graph: Arc<SiteGraph>,
    listener: TcpListener,
    _lock: std::fs::File,
}

impl Server {
    pub async fn bind(
        graph_path: &Path,
        graph_id: &str,
        endpoint_file: PathBuf,
        rules_path: &Path,
    ) -> Result<Self> {
        if let Some(parent) = endpoint_file.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let lock_path = graph_path.with_extension("daemon.lock");
        let lock = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&lock_path)?;
        lock.try_lock_exclusive()
            .map_err(|_| Error::AlreadyRunning(lock_path))?;

        let rule_pack = RulePack::from_path(rules_path).map_err(Error::Protocol)?;
        let graph = Arc::new(SiteGraph::open_with_rules(graph_path, graph_id, rule_pack).await?);
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let mut token_bytes = [0_u8; 32];
        getrandom::fill(&mut token_bytes)
            .map_err(|error| Error::Protocol(format!("could not create daemon token: {error}")))?;
        let token: Arc<str> = STANDARD.encode(token_bytes).into();
        write_endpoint_file(
            &endpoint_file,
            &EndpointFile {
                address: listener.local_addr()?.to_string(),
                token: token.to_string(),
            },
        )
        .await?;
        Ok(Self {
            endpoint_file,
            token,
            graph,
            listener,
            _lock: lock,
        })
    }

    pub async fn run(self) -> Result<()> {
        let result = self.accept_loop().await;
        let _ = tokio::fs::remove_file(&self.endpoint_file).await;
        result
    }

    async fn accept_loop(&self) -> Result<()> {
        loop {
            let (stream, _) = self.listener.accept().await?;
            let graph = Arc::clone(&self.graph);
            let token = Arc::clone(&self.token);
            tokio::spawn(async move {
                if let Err(error) = handle_connection(stream, graph, token).await {
                    tracing::warn!(%error, "sitegraph daemon request failed");
                }
            });
        }
    }
}

pub async fn read_endpoint_file(path: &Path) -> Result<EndpointFile> {
    Ok(serde_json::from_slice(&tokio::fs::read(path).await?)?)
}

async fn write_endpoint_file(path: &Path, endpoint: &EndpointFile) -> Result<()> {
    let temporary = path.with_extension("json.tmp");
    tokio::fs::write(&temporary, serde_json::to_vec(endpoint)?).await?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(&temporary, std::fs::Permissions::from_mode(0o600))?;
    }
    tokio::fs::rename(temporary, path).await?;
    Ok(())
}

async fn handle_connection(
    stream: TcpStream,
    graph: Arc<SiteGraph>,
    token: Arc<str>,
) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut frame = Vec::new();
    let bytes = reader.read_until(b'\n', &mut frame).await?;
    let response = if bytes == 0 || bytes > MAX_FRAME_BYTES {
        Response::error("invalid request frame size")
    } else {
        match serde_json::from_slice::<Request>(&frame) {
            Ok(request) if request.token == token.as_ref() => dispatch(&graph, request).await,
            Ok(_) => Response::error("unauthorized"),
            Err(error) => Response::error(format!("invalid request: {error}")),
        }
    };
    let mut encoded = serde_json::to_vec(&response)?;
    encoded.push(b'\n');
    writer.write_all(&encoded).await?;
    writer.shutdown().await?;
    Ok(())
}

impl Response {
    fn success(payload: Value) -> Self {
        Self {
            ok: true,
            payload,
            error: String::new(),
        }
    }

    fn error(error: impl Into<String>) -> Self {
        Self {
            ok: false,
            payload: Value::Null,
            error: error.into(),
        }
    }
}

async fn dispatch(graph: &SiteGraph, request: Request) -> Response {
    match dispatch_inner(graph, &request.operation, request.payload).await {
        Ok(payload) => Response::success(payload),
        Err(error) => Response::error(error.to_string()),
    }
}

async fn dispatch_inner(graph: &SiteGraph, operation: &str, payload: Value) -> Result<Value> {
    let string = |name: &str| {
        payload
            .get(name)
            .and_then(Value::as_str)
            .ok_or_else(|| Error::Protocol(format!("missing string field {name}")))
    };
    let unsigned = |name: &str| {
        payload
            .get(name)
            .and_then(Value::as_u64)
            .ok_or_else(|| Error::Protocol(format!("missing unsigned field {name}")))
    };
    match operation {
        "status" => Ok(serde_json::to_value(graph.status().await?)?),
        "checkpoint" => Ok(serde_json::to_value(
            graph
                .checkpoint(string("source")?, string("scope")?)
                .await?,
        )?),
        "sync_with_context" => {
            let request: SyncRequest = serde_json::from_value(payload)?;
            Ok(serde_json::to_value(
                graph
                    .sync_with_context(&request.batch, &request.context)
                    .await?,
            )?)
        }
        "search" => Ok(serde_json::to_value(
            graph
                .search(string("query")?, unsigned("cursor")?, unsigned("limit")?)
                .await?,
        )?),
        "endpoint" => Ok(serde_json::to_value(graph.endpoint(string("id")?).await?)?),
        "neighbors" => Ok(serde_json::to_value(
            graph
                .neighbors(string("node_id")?, unsigned("cursor")?, unsigned("limit")?)
                .await?,
        )?),
        "diff" => Ok(serde_json::to_value(
            graph
                .diff(
                    signed(&payload, "since")?,
                    unsigned("cursor")?,
                    unsigned("limit")?,
                )
                .await?,
        )?),
        "export_json" => Ok(serde_json::to_value(
            graph
                .export_json(unsigned("cursor")?, unsigned("limit")?)
                .await?,
        )?),
        "export_exact_json" => Ok(serde_json::to_value(
            graph
                .export_exact_json(unsigned("cursor")?, unsigned("limit")?)
                .await?,
        )?),
        "export_csv" => Ok(serde_json::to_value(
            graph
                .export_csv(unsigned("cursor")?, unsigned("limit")?)
                .await?,
        )?),
        "trace" => Ok(serde_json::to_value(
            graph
                .trace(
                    string("start_id")?,
                    to_u32(unsigned("max_depth")?, "max_depth")?,
                    to_u32(unsigned("limit")?, "limit")?,
                )
                .await?,
        )?),
        "shortest_path" => Ok(serde_json::to_value(
            graph
                .shortest_path(
                    string("from_id")?,
                    string("to_id")?,
                    to_usize(unsigned("max_depth")?, "max_depth")?,
                )
                .await?,
        )?),
        "endpoint_clusters" => Ok(serde_json::to_value(
            graph
                .endpoint_clusters(to_usize(unsigned("limit")?, "limit")?)
                .await?,
        )?),
        "impact" => Ok(serde_json::to_value(
            graph
                .impact(
                    string("node_id")?,
                    to_usize(unsigned("max_depth")?, "max_depth")?,
                    to_usize(unsigned("limit")?, "limit")?,
                )
                .await?,
        )?),
        _ => Err(Error::Protocol(format!(
            "unsupported operation: {operation}"
        ))),
    }
}

fn signed(payload: &Value, name: &str) -> Result<i64> {
    payload
        .get(name)
        .and_then(Value::as_i64)
        .ok_or_else(|| Error::Protocol(format!("missing signed field {name}")))
}

fn to_u32(value: u64, name: &str) -> Result<u32> {
    u32::try_from(value).map_err(|_| Error::Protocol(format!("{name} exceeds u32")))
}

fn to_usize(value: u64, name: &str) -> Result<usize> {
    usize::try_from(value).map_err(|_| Error::Protocol(format!("{name} exceeds usize")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn multiple_clients_share_one_graph_server() {
        let temporary = tempfile::tempdir().unwrap();
        let graph_path = temporary.path().join("graph.sqlite");
        let endpoint_file = temporary.path().join("daemon.json");
        let rules_path = temporary.path().join("default-rules.json");
        std::fs::write(&rules_path, sitegraph::enrichment::DEFAULT_RULE_PACK).unwrap();
        let server = Server::bind(&graph_path, "shared", endpoint_file.clone(), &rules_path)
            .await
            .unwrap();
        let task = tokio::spawn(server.run());
        let first = GraphBackend::Remote(Client::new(endpoint_file.clone()));
        let second = GraphBackend::Remote(Client::new(endpoint_file));

        let context = SyncContext::snapshot("shared", "all");
        first
            .sync_with_context(
                &SyncBatch {
                    sitemap: vec![sitegraph::SitemapObservation {
                        url: "https://example.test/api".to_owned(),
                        method: "GET".to_owned(),
                        status: 200,
                        ..sitegraph::SitemapObservation::default()
                    }],
                    ..SyncBatch::default()
                },
                &context,
            )
            .await
            .unwrap();
        let status = second.status().await.unwrap();
        assert_eq!(status.graph_id, "shared");
        assert!(status.total_nodes > 0);

        task.abort();
        let _ = task.await;
    }
}
