#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
mod cli;
mod config;
use crate::cli::{Cli, Command, ServeConfig};
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
    let cli = Cli::parse();
    let file_config = match cli::resolve_config_path(cli.config) {
        Some(path) => config::load(&path)?,
        None => config::Config::default(),
    };
    match cli.command {
        None => {
            run_server(
                cli::ServeArgs::default()
                    .resolve(&file_config)
                    .map_err(|error| anyhow!(error))?,
            )
            .await
        }
        Some(Command::Serve(config)) => {
            run_server(
                config
                    .resolve(&file_config)
                    .map_err(|error| anyhow!(error))?,
            )
            .await
        }
        Some(Command::Probe(config)) => {
            let config = config
                .resolve(&file_config)
                .map_err(|error| anyhow!(error))?;
            run_probe(&config.endpoint, config.tls_dir.as_deref()).await
        }
        Some(Command::SitegraphDaemon(config)) => {
            let server = sitegraph_daemon::Server::bind(
                &config.graph_path,
                &config.graph_id,
                config.endpoint_file,
                &config.rules_path,
            )
            .await?;
            server.run().await?;
            Ok(())
        }
    }
}

async fn run_server(config: ServeConfig) -> Result<()> {
    if !config.stdio {
        bail!("serve currently requires --stdio");
    }
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .try_init()
        .ok();
    let actor = spawn_client(client_config(config.endpoint, config.tls_dir.as_deref())?)?;
    let project_root = config
        .enable_sitegraph
        .then_some(config.sitegraph_project_root);
    if project_root.is_some() {
        config::ensure_rules_file(&config.rules_path)?;
    }
    let mut tools = BurpTools::new(
        actor,
        project_root.as_deref(),
        config.sitegraph_daemon.as_deref(),
        &config.rules_path,
    )
    .await
    .map_err(|error| anyhow!(error))?;
    tools
        .start_auto_index(
            &config.sitegraph_mode,
            std::time::Duration::from_secs(config.sitegraph_interval_seconds.max(1)),
        )
        .await
        .map_err(|error| anyhow!(error))?;
    let service = tools.clone().serve(stdio()).await?;
    service.waiting().await?;
    tools.shutdown().await;
    Ok(())
}

// This binary owns only CLI setup, dependency composition, and probing.

fn client_config(endpoint: String, tls_dir: Option<&Path>) -> Result<BurpClientConfig> {
    let tls = tls_dir.map(load_tls_config).transpose()?;
    if endpoint.starts_with("https://") && tls.is_none() {
        return Err(anyhow!("remote HTTPS endpoint requires an mTLS directory"));
    }
    Ok(BurpClientConfig {
        endpoint,
        tls,
        ..BurpClientConfig::default()
    })
}

fn load_tls_config(directory: &Path) -> Result<burp_protocol::ClientTlsConfig> {
    let read = |name: &str| -> Result<Vec<u8>> {
        let path = directory.join(name);
        if name.ends_with(".key") {
            require_private_key_permissions(&path)?;
        }
        std::fs::read(&path).map_err(|error| anyhow!("failed to read {}: {error}", path.display()))
    };
    Ok(burp_protocol::ClientTlsConfig {
        ca_certificate: read("ca.crt")?,
        client_certificate: read("client.crt")?,
        client_private_key: read("client.key")?,
    })
}

#[cfg(unix)]
fn require_private_key_permissions(path: &Path) -> Result<()> {
    let mode = std::fs::metadata(path)?.permissions().mode() & 0o777;
    if mode != 0o600 {
        bail!(
            "TLS private key {} must have mode 0600; found {:03o}",
            path.display(),
            mode
        );
    }
    Ok(())
}

#[cfg(not(unix))]
fn require_private_key_permissions(_path: &Path) -> Result<()> {
    Ok(())
}

async fn run_probe(endpoint: &str, tls_dir: Option<&Path>) -> Result<()> {
    let mut config = client_config(endpoint.to_owned(), tls_dir)?;
    config.call_timeout = std::time::Duration::from_secs(15);
    burp_protocol::connect_client(&config)
        .await
        .with_context(|| format!("connect to Burp gRPC endpoint {endpoint}"))?;
    let client = spawn_client(config)?;

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
