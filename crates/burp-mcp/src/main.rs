mod cli;
mod tools;
mod utility;

use crate::cli::{Cli, Command, ServeArgs};
use crate::tools::BurpTools;
use anyhow::{Context, Result, anyhow, bail};
use burp_protocol::proto::{
    EchoBytesRequest, PageRequest, PingRequest, ProxyHistoryRequest, ServerInfoRequest,
};
use burp_protocol::{BurpClientConfig, DEFAULT_MAX_MESSAGE_BYTES, connect_client, spawn_client};
use clap::Parser;
use rmcp::{ServiceExt, transport::stdio};
use std::time::Duration;
use tonic::Request;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    match Cli::parse().command {
        None => run_server(ServeArgs::default()).await,
        Some(Command::Serve(config)) => run_server(config).await,
        Some(Command::Probe(config)) => {
            run_probe(&config.resolved_endpoint().map_err(|error| anyhow!(error))?).await
        }
    }
}

async fn run_server(config: ServeArgs) -> Result<()> {
    if !config.stdio {
        bail!("serve currently requires --stdio");
    }
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .try_init()
        .ok();
    let actor = spawn_client(BurpClientConfig {
        endpoint: config.resolved_endpoint().map_err(|error| anyhow!(error))?,
        ..BurpClientConfig::default()
    })?;
    let graph_path = config.resolved_graph_path();
    let service = BurpTools::new(actor, &graph_path)
        .await
        .map_err(|error| anyhow!(error))?
        .serve(stdio())
        .await?;
    service.waiting().await?;
    Ok(())
}

// MCP tool implementations live in `tools.rs`; this binary owns only CLI setup and probing.

async fn run_probe(endpoint: &str) -> Result<()> {
    let mut client = connect_client(endpoint, Duration::from_secs(5), DEFAULT_MAX_MESSAGE_BYTES)
        .await
        .with_context(|| format!("connect to Burp gRPC endpoint {endpoint}"))?;

    let mut ping = Request::new(PingRequest {
        client: "burp-mcp-phase0-probe".to_owned(),
    });
    ping.set_timeout(Duration::from_secs(5));
    let ping = client.ping(ping).await?.into_inner();

    let mut info = Request::new(ServerInfoRequest {});
    info.set_timeout(Duration::from_secs(5));
    let info = client.server_info(info).await?.into_inner();
    if usize::try_from(info.max_message_bytes)? != DEFAULT_MAX_MESSAGE_BYTES {
        bail!("server message limit differs from Rust client limit");
    }

    for payload in [Vec::new(), vec![0xa5], patterned_payload(10 * 1024 * 1024)] {
        let expected = payload.clone();
        let mut request = Request::new(EchoBytesRequest {
            payload,
            delay_millis: 0,
        });
        request.set_timeout(Duration::from_secs(10));
        let echoed = client.echo_bytes(request).await?.into_inner().payload;
        if echoed != expected {
            bail!("byte round trip failed for {} bytes", expected.len());
        }
    }

    let mut history = Request::new(ProxyHistoryRequest {
        page: Some(PageRequest {
            limit: 1,
            cursor: String::new(),
        }),
        url_filter: String::new(),
        method_filter: String::new(),
        status_filter: None,
        has_notes: false,
        color: String::new(),
    });
    history.set_timeout(Duration::from_secs(5));
    client.proxy_history(history).await?;

    println!("PASS endpoint={endpoint}");
    println!("server={} version={}", ping.server, ping.version);
    println!("capabilities={}", info.capabilities.join(","));
    println!(
        "limits: message={} response={} page={} concurrency={} timeout={}s",
        info.max_message_bytes,
        info.max_response_bytes,
        info.max_page_size,
        info.max_concurrent_calls_per_connection,
        info.max_rpc_timeout_seconds
    );
    println!("byte-exact payloads: 0, 1, and {} bytes", 10 * 1024 * 1024);
    Ok(())
}

fn patterned_payload(size: usize) -> Vec<u8> {
    (0..size).map(|index| (index % 251) as u8).collect()
}
