//! Typed Rust client primitives for the loopback Burp gRPC boundary.

pub mod proto {
    tonic::include_proto!("burp.v1");
}

use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tonic::transport::{Channel, Endpoint};
use tonic::{Request, Status};

pub const DEFAULT_MAX_MESSAGE_BYTES: usize = 16 * 1024 * 1024;
pub const DEFAULT_CALL_TIMEOUT: Duration = Duration::from_secs(10);
pub const DEFAULT_QUEUE_CAPACITY: usize = 64;

#[derive(Debug, thiserror::Error)]
pub enum ActorError {
    #[error("gRPC actor queue is full; retry after an in-flight call completes")]
    QueueFull,
    #[error("gRPC actor queue is closed")]
    QueueClosed,
    #[error("gRPC actor request was cancelled")]
    ResponseCancelled,
    #[error("gRPC actor request failed: {0}")]
    Rpc(#[from] Status),
    #[error("invalid gRPC actor configuration: {0}")]
    InvalidConfig(&'static str),
}

pub enum Command {
    Ping {
        request: proto::PingRequest,
        response: oneshot::Sender<Result<proto::PingResponse, ActorError>>,
    },
    EchoBytes {
        request: proto::EchoBytesRequest,
        response: oneshot::Sender<Result<proto::EchoBytesResponse, ActorError>>,
    },
    ProxyHistory {
        request: proto::ProxyHistoryRequest,
        response: oneshot::Sender<Result<proto::ProxyHistoryResponse, ActorError>>,
    },
}

#[derive(Clone)]
pub struct GrpcActorHandle {
    sender: mpsc::Sender<Command>,
}

impl GrpcActorHandle {
    pub async fn ping(
        &self,
        request: proto::PingRequest,
    ) -> Result<proto::PingResponse, ActorError> {
        self.send(|response| Command::Ping { request, response })
            .await
    }

    pub async fn echo_bytes(
        &self,
        request: proto::EchoBytesRequest,
    ) -> Result<proto::EchoBytesResponse, ActorError> {
        self.send(|response| Command::EchoBytes { request, response })
            .await
    }

    pub async fn proxy_history(
        &self,
        request: proto::ProxyHistoryRequest,
    ) -> Result<proto::ProxyHistoryResponse, ActorError> {
        self.send(|response| Command::ProxyHistory { request, response })
            .await
    }

    async fn send<T>(
        &self,
        command: impl FnOnce(oneshot::Sender<Result<T, ActorError>>) -> Command,
    ) -> Result<T, ActorError> {
        let (response, receiver) = oneshot::channel();
        self.sender
            .try_send(command(response))
            .map_err(|error| match error {
                mpsc::error::TrySendError::Full(_) => ActorError::QueueFull,
                mpsc::error::TrySendError::Closed(_) => ActorError::QueueClosed,
            })?;
        receiver.await.map_err(|_| ActorError::ResponseCancelled)?
    }
}

pub struct GrpcActorConfig {
    pub endpoint: String,
    pub call_timeout: Duration,
    pub queue_capacity: usize,
    pub max_message_bytes: usize,
}

impl Default for GrpcActorConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://127.0.0.1:9876".to_owned(),
            call_timeout: DEFAULT_CALL_TIMEOUT,
            queue_capacity: DEFAULT_QUEUE_CAPACITY,
            max_message_bytes: DEFAULT_MAX_MESSAGE_BYTES,
        }
    }
}

pub fn spawn_actor(config: GrpcActorConfig) -> Result<GrpcActorHandle, ActorError> {
    if config.queue_capacity == 0 {
        return Err(ActorError::InvalidConfig("queue capacity must be positive"));
    }
    if config.call_timeout == Duration::ZERO {
        return Err(ActorError::InvalidConfig("call timeout must be positive"));
    }
    if config.max_message_bytes == 0 {
        return Err(ActorError::InvalidConfig("message limit must be positive"));
    }
    let (sender, receiver) = mpsc::channel(config.queue_capacity);
    tokio::spawn(run_actor(config, receiver));
    Ok(GrpcActorHandle { sender })
}

async fn run_actor(config: GrpcActorConfig, mut receiver: mpsc::Receiver<Command>) {
    let mut client: Option<proto::burp_service_client::BurpServiceClient<Channel>> = None;
    while let Some(command) = receiver.recv().await {
        if client.is_none() {
            client = connect(&config).await;
        }
        let Some(current_client) = client.as_mut() else {
            respond_offline(command);
            continue;
        };
        let result = execute(current_client, &config, command).await;
        if result {
            client = None;
        }
    }
}

pub async fn connect_client(
    endpoint: &str,
    timeout: Duration,
    max_message_bytes: usize,
) -> Result<proto::burp_service_client::BurpServiceClient<Channel>, tonic::transport::Error> {
    let channel = Endpoint::from_shared(endpoint.to_owned())?
        .connect_timeout(timeout)
        .timeout(timeout)
        .tcp_keepalive(Some(Duration::from_secs(30)))
        .connect()
        .await?;
    Ok(proto::burp_service_client::BurpServiceClient::new(channel)
        .max_decoding_message_size(max_message_bytes)
        .max_encoding_message_size(max_message_bytes))
}

async fn connect(
    config: &GrpcActorConfig,
) -> Option<proto::burp_service_client::BurpServiceClient<Channel>> {
    connect_client(
        &config.endpoint,
        config.call_timeout,
        config.max_message_bytes,
    )
    .await
    .ok()
}

async fn execute(
    client: &mut proto::burp_service_client::BurpServiceClient<Channel>,
    config: &GrpcActorConfig,
    command: Command,
) -> bool {
    match command {
        Command::Ping { request, response } => {
            let result = client
                .ping(with_deadline(request, config.call_timeout))
                .await
                .map(|r| r.into_inner())
                .map_err(ActorError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::EchoBytes { request, response } => {
            let result = client
                .echo_bytes(with_deadline(request, config.call_timeout))
                .await
                .map(|r| r.into_inner())
                .map_err(ActorError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::ProxyHistory { request, response } => {
            let result = client
                .proxy_history(with_deadline(request, config.call_timeout))
                .await
                .map(|r| r.into_inner())
                .map_err(ActorError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
    }
}

fn with_deadline<T>(message: T, timeout: Duration) -> Request<T> {
    let mut request = Request::new(message);
    request.set_timeout(timeout);
    request
}

fn is_transport_failure(error: &ActorError) -> bool {
    matches!(error, ActorError::Rpc(status) if matches!(status.code(), tonic::Code::Unavailable | tonic::Code::Unknown | tonic::Code::DeadlineExceeded))
}

fn respond_offline(command: Command) {
    let status =
        Status::unavailable("Burp gRPC service is offline; start the Burp extension and retry");
    match command {
        Command::Ping { response, .. } => {
            let _ = response.send(Err(ActorError::Rpc(status)));
        }
        Command::EchoBytes { response, .. } => {
            let _ = response.send(Err(ActorError::Rpc(status)));
        }
        Command::ProxyHistory { response, .. } => {
            let _ = response.send(Err(ActorError::Rpc(status)));
        }
    }
}
