use crate::sitegraph_sync::SiteGraphSynchronizer;
use burp_protocol::BurpClient;
use serde::Serialize;
use sitegraph::{GraphStatus, SiteGraph, SyncSummary};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, watch};

const DEFAULT_QUEUE_CAPACITY: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum IndexerState {
    Disabled,
    Ready,
    CatchingUp,
    Degraded,
    Stopped,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct IndexerStatus {
    pub state: IndexerState,
    pub pending_commands: usize,
    pub last_error: Option<String>,
    pub graph: GraphStatus,
}

struct StatusState {
    state: IndexerState,
    pending_commands: usize,
    last_error: Option<String>,
}

enum Command {
    Sync {
        prefix: String,
        response: oneshot::Sender<Result<SyncSummary, String>>,
    },
    Shutdown {
        response: oneshot::Sender<()>,
    },
}

#[derive(Clone)]
pub(crate) struct SitegraphIndexer {
    sender: mpsc::Sender<Command>,
    status: watch::Receiver<StatusState>,
    graph: Arc<SiteGraph>,
}

impl SitegraphIndexer {
    pub(crate) fn spawn(client: BurpClient, graph: Arc<SiteGraph>) -> Self {
        let (sender, mut receiver) = mpsc::channel(DEFAULT_QUEUE_CAPACITY);
        let (status_sender, status) = watch::channel(StatusState {
            state: IndexerState::Disabled,
            pending_commands: 0,
            last_error: None,
        });
        let synchronizer = SiteGraphSynchronizer::new(client, Arc::clone(&graph));
        tokio::spawn(async move {
            while let Some(command) = receiver.recv().await {
                match command {
                    Command::Sync { prefix, response } => {
                        let pending_commands = receiver.len();
                        status_sender.send_modify(|status| {
                            status.state = IndexerState::CatchingUp;
                            status.pending_commands = pending_commands;
                            status.last_error = None;
                        });
                        let result = synchronizer.run(prefix).await;
                        status_sender.send_modify(|status| {
                            status.state = if result.is_ok() {
                                IndexerState::Ready
                            } else {
                                IndexerState::Degraded
                            };
                            status.pending_commands = receiver.len();
                            status.last_error = result.as_ref().err().cloned();
                        });
                        let _ = response.send(result);
                    }
                    Command::Shutdown { response } => {
                        let _ = response.send(());
                        break;
                    }
                }
            }
            status_sender.send_modify(|status| status.state = IndexerState::Stopped);
        });
        Self { sender, status, graph }
    }

    pub(crate) async fn sync(&self, prefix: String) -> Result<SyncSummary, String> {
        let (response, receiver) = oneshot::channel();
        self.sender
            .try_send(Command::Sync { prefix, response })
            .map_err(|error| match error {
                mpsc::error::TrySendError::Full(_) => "sitegraph indexer queue is full".to_owned(),
                mpsc::error::TrySendError::Closed(_) => "sitegraph indexer is stopped".to_owned(),
            })?;
        receiver
            .await
            .map_err(|_| "sitegraph indexer stopped before completing sync".to_owned())?
    }

    pub(crate) async fn status(&self) -> Result<IndexerStatus, String> {
        let (state, pending_commands, last_error) = {
            let status = self.status.borrow();
            (status.state, status.pending_commands, status.last_error.clone())
        };
        let graph = self.graph.status().await.map_err(|error| error.to_string())?;
        Ok(IndexerStatus {
            state,
            pending_commands,
            last_error,
            graph,
        })
    }

    pub(crate) async fn shutdown(&self) {
        let (response, receiver) = oneshot::channel();
        if self.sender.send(Command::Shutdown { response }).await.is_ok() {
            let _ = tokio::time::timeout(std::time::Duration::from_secs(5), receiver).await;
        }
    }
}
