use super::{Error, Result, sitegraph_capnp};
use crate::tasks::{Task, decode};
use capnp_rpc::{RpcSystem, rpc_twoparty_capnp, twoparty};
use futures::AsyncReadExt as _;
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot};
use tokio_util::compat::TokioAsyncReadCompatExt as _;

#[derive(Debug)]
enum WireFailure {
    Remote(String),
    Transport(String),
}

struct WireCall {
    endpoint_file: PathBuf,
    task: Task,
    response: oneshot::Sender<std::result::Result<Vec<u8>, WireFailure>>,
}

static WIRE_WORKER: LazyLock<std::result::Result<mpsc::UnboundedSender<WireCall>, Arc<str>>> =
    LazyLock::new(start_wire_worker);

fn start_wire_worker() -> std::result::Result<mpsc::UnboundedSender<WireCall>, Arc<str>> {
    let (sender, mut receiver) = mpsc::unbounded_channel::<WireCall>();
    std::thread::Builder::new()
        .name("sitegraph-capnp-client".to_owned())
        .spawn(move || {
            let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            else {
                return;
            };
            let local = tokio::task::LocalSet::new();
            local.block_on(&runtime, async move {
                while let Some(call) = receiver.recv().await {
                    tokio::task::spawn_local(async move {
                        let result = execute_wire_call(&call.endpoint_file, call.task).await;
                        let _ = call.response.send(result);
                    });
                }
            });
        })
        .map_err(|error| Arc::from(error.to_string()))?;
    Ok(sender)
}

#[derive(Clone)]
pub struct Client {
    endpoint_file: Arc<PathBuf>,
    worker: std::result::Result<mpsc::UnboundedSender<WireCall>, Arc<str>>,
}

impl Client {
    pub fn new(endpoint_file: impl Into<PathBuf>) -> Self {
        Self {
            endpoint_file: Arc::new(endpoint_file.into()),
            worker: WIRE_WORKER.clone(),
        }
    }

    pub fn endpoint_file(&self) -> &Path {
        &self.endpoint_file
    }

    pub(crate) async fn request<T>(&self, task: Task) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let worker = self
            .worker
            .as_ref()
            .map_err(|error| Error::Protocol(error.to_string()))?;
        let (response, receiver) = oneshot::channel();
        worker
            .send(WireCall {
                endpoint_file: self.endpoint_file.as_ref().clone(),
                task,
                response,
            })
            .map_err(|_| Error::Protocol("sitegraph Cap'n Proto client stopped".to_owned()))?;
        let payload = receiver
            .await
            .map_err(|_| Error::Protocol("sitegraph Cap'n Proto client stopped".to_owned()))?
            .map_err(|error| match error {
                WireFailure::Remote(message) => Error::Remote(message),
                WireFailure::Transport(message) => Error::Protocol(message),
            })?;
        decode(&payload)
    }
}

async fn execute_wire_call(
    endpoint_file: &Path,
    task: Task,
) -> std::result::Result<Vec<u8>, WireFailure> {
    let endpoint = super::read_endpoint_file(endpoint_file)
        .await
        .map_err(transport)?;
    let stream = TcpStream::connect(&endpoint.address)
        .await
        .map_err(transport)?;
    stream.set_nodelay(true).map_err(transport)?;
    let (reader, writer) = stream.compat().split();
    let network = Box::new(twoparty::VatNetwork::new(
        futures::io::BufReader::new(reader),
        futures::io::BufWriter::new(writer),
        rpc_twoparty_capnp::Side::Client,
        Default::default(),
    ));
    let mut rpc_system = RpcSystem::new(network, None);
    let client: sitegraph_capnp::sitegraph::Client =
        rpc_system.bootstrap(rpc_twoparty_capnp::Side::Server);
    let mut request = client.call_request();
    {
        let mut params = request.get();
        params.set_token(&endpoint.token);
        task.write_capnp(params.reborrow().init_task())
            .map_err(transport)?;
    }
    let response = request.send().promise;
    let disconnector = rpc_system.get_disconnector();
    let rpc_task = tokio::task::spawn_local(rpc_system);
    let outcome = async {
        let response = response.await.map_err(transport)?;
        let result = response.get().map_err(transport)?;
        if !result.get_ok() {
            let error = result.get_error().map_err(transport)?;
            let message = error.to_str().map_err(transport)?.to_owned();
            return Err(WireFailure::Remote(message));
        }
        let payload = result.get_payload().map_err(transport)?;
        if payload.len() > super::MAX_FRAME_BYTES {
            return Err(WireFailure::Transport(
                "response exceeds 128 MiB".to_owned(),
            ));
        }
        Ok(payload.to_vec())
    }
    .await;
    let disconnect_result = disconnector.await.map_err(transport);
    let rpc_result = rpc_task
        .await
        .map_err(transport)
        .and_then(|result| result.map_err(transport));
    match outcome {
        Err(error) => Err(error),
        Ok(payload) => {
            disconnect_result?;
            rpc_result?;
            Ok(payload)
        }
    }
}

fn transport(error: impl std::fmt::Display) -> WireFailure {
    WireFailure::Transport(error.to_string())
}
