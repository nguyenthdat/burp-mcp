mod cli;
use crate::cli::{Cli, Command, ServeArgs};
use anyhow::{Context, Result, anyhow, bail};
use burp_protocol::{
    BurpClientConfig, DEFAULT_MAX_MESSAGE_BYTES, PageRequest, ProxyHistoryQuery, spawn_client,
};
use burp_tools::BurpTools;
use clap::Parser;
use rmcp::{ServiceExt, transport::stdio};
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

// This binary owns only CLI setup, dependency composition, and probing.

async fn run_probe(endpoint: &str) -> Result<()> {
    let client = spawn_client(BurpClientConfig {
        endpoint: endpoint.to_owned(),
        call_timeout: std::time::Duration::from_secs(10),
        ..BurpClientConfig::default()
    })?;

    let ping = client
        .probe_ping("burp-mcp-probe".to_owned())
        .await
        .with_context(|| format!("probe Burp gRPC endpoint {endpoint}"))?;
    let info = client.probe_server_info().await?;
    if usize::try_from(info.max_message_bytes)? != DEFAULT_MAX_MESSAGE_BYTES {
        bail!("server message limit differs from Rust client limit");
    }

    for payload in [Vec::new(), vec![0xa5], patterned_payload(10 * 1024 * 1024)] {
        let expected = payload.clone();
        let echoed = client.probe_echo(payload).await?;
        if echoed != expected {
            bail!("byte round trip failed for {} bytes", expected.len());
        }
    }

    client
        .probe_proxy_history(ProxyHistoryQuery {
            page: PageRequest {
                limit: 1,
                cursor: String::new(),
            },
            ..ProxyHistoryQuery::default()
        })
        .await?;

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
