use anyhow::{Context, Result, bail};
use burp_grpc::proto::{EchoBytesRequest, PingRequest, ServerInfoRequest};
use burp_grpc::{DEFAULT_MAX_MESSAGE_BYTES, connect_client};
use std::env;
use std::time::Duration;
use tonic::Request;

const DEFAULT_ENDPOINT: &str = "http://127.0.0.1:9877";

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "probe".to_owned());
    if command != "probe" {
        bail!("Phase 0 supports only: burp-mcp probe [--endpoint http://127.0.0.1:9877]");
    }
    let endpoint = parse_endpoint(args)?;
    run_probe(&endpoint).await
}

fn parse_endpoint(mut args: impl Iterator<Item = String>) -> Result<String> {
    let mut endpoint = DEFAULT_ENDPOINT.to_owned();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--endpoint" => {
                endpoint = args.next().context("--endpoint requires a value")?;
            }
            unknown => bail!("unknown probe argument: {unknown}"),
        }
    }
    if !endpoint.starts_with("http://127.0.0.1:") {
        bail!("Phase 0 probe connects only to http://127.0.0.1:<port>");
    }
    Ok(endpoint)
}

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

    for size in [0, 1, 10 * 1024 * 1024] {
        let payload = patterned_payload(size);
        let mut request = Request::new(EchoBytesRequest {
            payload: payload.clone(),
            delay_millis: 0,
        });
        request.set_timeout(Duration::from_secs(10));
        let echoed = client.echo_bytes(request).await?.into_inner().payload;
        if echoed != payload {
            bail!("byte-exact echo failed for {size} byte payload");
        }
    }

    println!("PASS endpoint={endpoint}");
    println!("server={} version={}", ping.server, ping.version);
    println!("capabilities={}", info.capabilities.join(","));
    println!(
        "limits: message={} response={} page={} concurrency={} timeout={}s",
        info.max_message_bytes,
        info.max_response_bytes,
        info.max_page_size,
        info.max_concurrent_calls_per_connection,
        info.max_rpc_timeout_seconds,
    );
    println!("byte-exact payloads: 0, 1, and 10485760 bytes");
    Ok(())
}

fn patterned_payload(size: usize) -> Vec<u8> {
    (0..size).map(|index| (index % 251) as u8).collect()
}
