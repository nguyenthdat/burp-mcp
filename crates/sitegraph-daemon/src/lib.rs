use base64::{Engine as _, engine::general_purpose::STANDARD};
use fs2::FileExt as _;
use serde::{Deserialize, Serialize};
use sitegraph::{GraphStatus, SiteGraph, enrichment::RulePack};
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};

mod backend;
mod capnp_transport;
mod tasks;
mod wire;

pub use backend::GraphBackend;
pub use wire::Client;

pub mod sitegraph_capnp {
    include!(concat!(env!("OUT_DIR"), "/schema/sitegraph_capnp.rs"));
}

pub(crate) const MAX_FRAME_BYTES: usize = 128 * 1024 * 1024;
const STARTUP_ATTEMPTS: usize = 50;
const STARTUP_DELAY: Duration = Duration::from_millis(100);

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("sitegraph daemon I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("sitegraph daemon Cap'n Proto failed: {0}")]
    Capnp(#[from] capnp::Error),
    #[error("sitegraph daemon typed payload failed: {0}")]
    Codec(String),
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

pub fn endpoint_path(graph_path: &Path) -> PathBuf {
    graph_path.with_extension("daemon.toml")
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
    matches!(client.status().await, Ok(status) if status.graph_id == graph_id)
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
            .truncate(false)
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
            tokio::task::spawn_local(async move {
                if let Err(error) = capnp_transport::serve_connection(stream, graph, token).await {
                    tracing::warn!(%error, "sitegraph daemon request failed");
                }
            });
        }
    }
}

pub async fn read_endpoint_file(path: &Path) -> Result<EndpointFile> {
    let content = tokio::fs::read_to_string(path).await?;
    toml::from_str(&content).map_err(|error| Error::Protocol(error.to_string()))
}

async fn write_endpoint_file(path: &Path, endpoint: &EndpointFile) -> Result<()> {
    let temporary = path.with_extension("toml.tmp");
    let content = toml::to_string(endpoint).map_err(|error| Error::Protocol(error.to_string()))?;
    tokio::fs::write(&temporary, content).await?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(&temporary, std::fs::Permissions::from_mode(0o600))?;
    }
    tokio::fs::rename(temporary, path).await?;
    Ok(())
}

impl Client {
    async fn status(&self) -> Result<GraphStatus> {
        GraphBackend::Remote(self.clone()).status().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sitegraph::{SitemapObservation, SyncBatch, SyncContext};

    #[tokio::test]
    async fn multiple_clients_share_typed_graph_tasks() {
        let temporary = tempfile::tempdir().unwrap();
        let graph_path = temporary.path().join("graph.sqlite");
        let endpoint_file = temporary.path().join("daemon.toml");
        let rules_path = temporary.path().join("default-rules.json");
        std::fs::write(&rules_path, sitegraph::enrichment::DEFAULT_RULE_PACK).unwrap();
        let server = Server::bind(&graph_path, "shared", endpoint_file.clone(), &rules_path)
            .await
            .unwrap();
        let local = tokio::task::LocalSet::new();
        local
            .run_until(async move {
                let server_task = tokio::task::spawn_local(server.run());
                let first = GraphBackend::Remote(Client::new(endpoint_file.clone()));
                let second = GraphBackend::Remote(Client::new(endpoint_file));
                let context = SyncContext::snapshot("shared", "all");
                first
                    .sync_with_context(
                        &SyncBatch {
                            sitemap: vec![SitemapObservation {
                                url: "https://example.test/api".to_owned(),
                                method: "GET".to_owned(),
                                status: 200,
                                request_bytes: b"GET /api HTTP/1.1\r\n\r\ntypedneedle".to_vec(),
                                ..SitemapObservation::default()
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
                let search = second.search("api", 0, 10).await.unwrap();
                assert_eq!(search.total, 1);
                let endpoint_id = search.items[0].id.clone();
                let detail = second.endpoint(&endpoint_id).await.unwrap().unwrap();
                assert_eq!(detail.path, "/api");
                let checkpoint = second.checkpoint("burp_sitemap", "all").await.unwrap();
                assert!(checkpoint.is_some());
                let neighbors = second.neighbors(&endpoint_id, 0, 10).await.unwrap();
                assert!(neighbors.total > 0);
                second.trace(&endpoint_id, 4, 10).await.unwrap();
                let path = second
                    .shortest_path(&endpoint_id, &endpoint_id, 4)
                    .await
                    .unwrap();
                assert_eq!(path.depth, 0);
                assert!(!second.endpoint_clusters(10).await.unwrap().is_empty());
                second.impact(&endpoint_id, 4, 10).await.unwrap();
                assert!(second.diff(0, 0, 10).await.unwrap().total > 0);
                assert!(second.export_json(0, 10).await.unwrap().total > 0);
                assert!(second.export_exact_json(0, 10).await.unwrap().total > 0);
                assert!(second.export_csv(0, 10).await.unwrap().total > 0);
                let history = second
                    .search_history("typedneedle", Some("http"), 0, 10)
                    .await
                    .unwrap();
                assert_eq!(history.total, 1);
                server_task.abort();
                let _ = server_task.await;
            })
            .await;
    }
}
