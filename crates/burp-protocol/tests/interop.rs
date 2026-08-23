use anyhow::{Context, Result, bail};
use burp_protocol::interop_proto::{
    EchoBytesRequest, PageRequest, PingRequest, ProxyHistoryRequest, ServerInfoRequest,
};
use burp_protocol::{BurpClientConfig, DEFAULT_MAX_MESSAGE_BYTES, connect_client, spawn_client};
use std::env;
use std::path::PathBuf;
use std::time::Duration;
use tonic::{Code, Request};

fn endpoint() -> Option<String> {
    env::var("BURP_MCP_INTEROP_ENDPOINT").ok()
}

#[tokio::test]
async fn kotlin_server_echoes_binary_payloads_and_handles_concurrency() -> Result<()> {
    let Some(endpoint) = endpoint() else {
        eprintln!(
            "skipped: set BURP_MCP_INTEROP_ENDPOINT to run against the Kotlin Phase 0 server"
        );
        return Ok(());
    };
    let mut client = connect_client(
        &endpoint,
        Duration::from_secs(10),
        DEFAULT_MAX_MESSAGE_BYTES,
    )
    .await
    .context("connect to Kotlin gRPC server")?;

    let mut info_request = Request::new(ServerInfoRequest {});
    info_request.set_timeout(Duration::from_secs(2));
    let info = client.server_info(info_request).await?.into_inner();
    assert_eq!(
        DEFAULT_MAX_MESSAGE_BYTES,
        usize::try_from(info.max_message_bytes)?
    );
    assert_eq!(32, info.max_concurrent_calls_per_connection);
    assert_eq!(30, info.max_rpc_timeout_seconds);

    let mut page_request = Request::new(ProxyHistoryRequest {
        page: Some(PageRequest {
            limit: 10,
            cursor: String::new(),
        }),
        url_filter: String::new(),
        method_filter: String::new(),
        status_filter: None,
        has_notes: false,
        color: String::new(),
    });
    page_request.set_timeout(Duration::from_secs(2));
    let history = client.proxy_history(page_request).await?.into_inner();
    assert!(history.items.len() <= 10);
    assert_eq!(0, history.page.as_ref().map_or(0, |page| page.total));

    for payload in [Vec::new(), vec![0xa5], patterned_payload(10 * 1024 * 1024)] {
        let mut request = Request::new(EchoBytesRequest {
            payload: payload.clone(),
            delay_millis: 0,
        });
        request.set_timeout(Duration::from_secs(10));
        let response = client.clone().echo_bytes(request).await?.into_inner();
        assert_eq!(payload, response.payload);
    }

    let mut tasks = Vec::with_capacity(32);
    for index in 0..32_u8 {
        let mut concurrent_client = client.clone();
        tasks.push(tokio::spawn(async move {
            let mut request = Request::new(EchoBytesRequest {
                payload: vec![index; 4096],
                delay_millis: 0,
            });
            request.set_timeout(Duration::from_secs(5));
            concurrent_client
                .echo_bytes(request)
                .await
                .map(|response| response.into_inner().payload)
        }));
    }
    for (index, task) in tasks.into_iter().enumerate() {
        assert_eq!(vec![u8::try_from(index)?; 4096], task.await??);
    }
    Ok(())
}

#[tokio::test]
async fn kotlin_server_honors_deadlines_and_reconnects_after_restart() -> Result<()> {
    let Some(endpoint) = endpoint() else {
        eprintln!(
            "skipped: set BURP_MCP_INTEROP_ENDPOINT to run against the Kotlin Phase 0 server"
        );
        return Ok(());
    };
    let mut client =
        connect_client(&endpoint, Duration::from_secs(2), DEFAULT_MAX_MESSAGE_BYTES).await?;
    let mut delayed = Request::new(EchoBytesRequest {
        payload: vec![],
        delay_millis: 500,
    });
    delayed.set_timeout(Duration::from_millis(25));
    let status = client
        .echo_bytes(delayed)
        .await
        .expect_err("delayed call must time out");
    assert!(matches!(
        status.code(),
        Code::DeadlineExceeded | Code::Cancelled
    ));

    let control =
        PathBuf::from(env::var("BURP_MCP_INTEROP_CONTROL").context("interop control directory")?);
    std::fs::write(control.join("stop"), b"")?;
    wait_for(&control.join("stopped"), Duration::from_secs(5)).await?;

    let actor = spawn_client(BurpClientConfig {
        endpoint,
        call_timeout: Duration::from_millis(500),
        queue_capacity: 8,
        max_message_bytes: DEFAULT_MAX_MESSAGE_BYTES,
    })?;
    assert!(
        actor
            .ping(PingRequest {
                client: "rust-interop".to_owned(),
            })
            .await
            .is_err()
    );
    std::fs::write(control.join("start"), b"")?;
    wait_for(&control.join("ready"), Duration::from_secs(5)).await?;

    for _ in 0..60 {
        if actor
            .ping(PingRequest {
                client: "rust-interop".to_owned(),
            })
            .await
            .is_ok()
        {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    bail!("Kotlin server did not become available after restart")
}

async fn wait_for(path: &std::path::Path, timeout: Duration) -> Result<()> {
    let deadline = tokio::time::Instant::now() + timeout;
    while tokio::time::Instant::now() < deadline {
        if path.exists() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    bail!("timed out waiting for {}", path.display())
}

fn patterned_payload(size: usize) -> Vec<u8> {
    (0..size).map(|index| (index % 251) as u8).collect()
}
