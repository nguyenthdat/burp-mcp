use ::sitegraph::{
    IssueObservation, SitemapObservation, SyncBatch, SyncContext, SyncSummary,
    TechnologyObservation, WebSocketObservation,
};
use burp_protocol::protocol::{
    PageRequest, ProxyWebSocketHistoryRequest, ScanIssuesRequest, SitemapSnapshotRequest,
};
use burp_protocol::{BurpClient, ClientError};
use sitegraph_daemon::GraphBackend;

const PAGE_SIZE: u32 = 500;

#[derive(Clone)]
pub(crate) struct SiteGraphSynchronizer {
    client: BurpClient,
    graph: GraphBackend,
}

impl SiteGraphSynchronizer {
    pub(crate) fn new(client: BurpClient, graph: GraphBackend) -> Self {
        Self { client, graph }
    }

    pub(crate) async fn run(&self, prefix: String) -> Result<SyncSummary, String> {
        let graph_id = self
            .graph
            .status()
            .await
            .map_err(|error| error.to_string())?
            .graph_id;
        let scope = if prefix.is_empty() {
            "all".to_owned()
        } else {
            prefix.clone()
        };
        let checkpoint = self
            .graph
            .checkpoint("burp_sitemap", &scope)
            .await
            .map_err(|error| error.to_string())?;
        let resumed = checkpoint
            .as_ref()
            .filter(|(_, coverage)| !coverage.complete);
        let mut cursor = resumed
            .and_then(|(_, coverage)| coverage.last_cursor.clone())
            .unwrap_or_default();
        let run_id = resumed.map_or_else(
            || {
                blake3::hash(
                    format!(
                        "{graph_id}\\0{scope}\\0{}",
                        time::OffsetDateTime::now_utc().unix_timestamp_nanos()
                    )
                    .as_bytes(),
                )
                .to_hex()
                .to_string()
            },
            |(run_id, _)| run_id.clone(),
        );
        let mut pages_seen = resumed.map_or(0, |(_, coverage)| coverage.pages_read);
        let mut items_seen = resumed.map_or(0, |(_, coverage)| coverage.items_indexed);
        let mut last_summary;
        loop {
            let response = self
                .client
                .sitemap_snapshot(SitemapSnapshotRequest {
                    url_prefix: prefix.clone(),
                    page: Some(PageRequest {
                        limit: PAGE_SIZE,
                        cursor: cursor.clone(),
                    }),
                })
                .await
                .map_err(|error| error.to_string())?;
            let page = response.page.unwrap_or_default();
            let sitemap = response
                .items
                .into_iter()
                .map(|entry| SitemapObservation {
                    url: entry.url,
                    method: entry.method,
                    status: entry.status,
                    content_type: entry.content_type,
                    response_body: entry.response_body,
                    request_bytes: entry.request_bytes,
                    response_bytes: entry.response_bytes,
                    redirect_url: entry.redirect_url,
                    response_links: entry.response_links,
                    form_actions: entry.form_actions,
                    script_sources: entry.script_sources,
                })
                .collect::<Vec<_>>();
            let technologies = sitemap
                .iter()
                .flat_map(::sitegraph::ingest::detect_technologies)
                .collect::<Vec<TechnologyObservation>>();
            pages_seen += 1;
            items_seen = items_seen.saturating_add(sitemap.len() as u64);
            let end_of_source = !page.truncated || page.next_cursor.is_empty();
            let mut context = SyncContext::snapshot(&graph_id, &scope);
            context.run_id = run_id.clone();
            context.cursor = (!end_of_source).then(|| page.next_cursor.clone());
            context.source_total = Some(u64::from(page.total));
            context.pages_seen = pages_seen;
            last_summary = Some(
                self.graph
                    .sync_with_context(
                        &SyncBatch {
                            sitemap,
                            technologies,
                            ..SyncBatch::default()
                        },
                        &context,
                    )
                    .await
                    .map_err(|error| error.to_string())?,
            );
            if end_of_source {
                break;
            }
            cursor = page.next_cursor;
        }
        let issues = self
            .fetch_issues()
            .await
            .map_err(|error| error.to_string())?;
        if !issues.is_empty() {
            let mut context = SyncContext::snapshot(&graph_id, format!("{scope}:issues"));
            context.run_id = format!("{run_id}-issues");
            context.source = "burp_scanner_issues".to_owned();
            context.source_total = Some(issues.len() as u64);
            context.items_seen = issues.len() as u64;
            last_summary = Some(
                self.graph
                    .sync_with_context(
                        &SyncBatch {
                            issues,
                            ..SyncBatch::default()
                        },
                        &context,
                    )
                    .await
                    .map_err(|error| error.to_string())?,
            );
        }
        let mut websocket_cursor = String::new();
        loop {
            let response = self
                .client
                .proxy_websocket_history(ProxyWebSocketHistoryRequest {
                    page: Some(PageRequest {
                        limit: PAGE_SIZE,
                        cursor: websocket_cursor.clone(),
                    }),
                })
                .await
                .map_err(|error| error.to_string())?;
            let page = response.page.unwrap_or_default();
            let websocket_messages = response
                .items
                .into_iter()
                .map(|entry| WebSocketObservation {
                    id: entry.id.to_string(),
                    web_socket_id: entry.web_socket_id.to_string(),
                    direction: entry.direction,
                    upgrade_url: entry.upgrade_url,
                    payload: entry.payload,
                    edited_payload: entry.edited_payload,
                })
                .collect::<Vec<_>>();
            if !websocket_messages.is_empty() {
                let mut context = SyncContext::snapshot(&graph_id, format!("{scope}:websocket"));
                context.run_id = format!("{run_id}-websocket-{websocket_cursor}");
                context.source = "burp_websocket_history".to_owned();
                context.complete = !page.truncated || page.next_cursor.is_empty();
                last_summary = Some(
                    self.graph
                        .sync_with_context(
                            &SyncBatch {
                                websocket_messages,
                                ..SyncBatch::default()
                            },
                            &context,
                        )
                        .await
                        .map_err(|error| error.to_string())?,
                );
            }
            if !page.truncated || page.next_cursor.is_empty() {
                break;
            }
            websocket_cursor = page.next_cursor;
        }
        last_summary.ok_or_else(|| "sitemap source returned no page".to_owned())
    }

    async fn fetch_issues(&self) -> Result<Vec<IssueObservation>, ClientError> {
        let mut cursor = String::new();
        let mut issues = Vec::new();
        loop {
            let response = self
                .client
                .scan_issues(ScanIssuesRequest {
                    page: Some(PageRequest {
                        limit: PAGE_SIZE,
                        cursor,
                    }),
                })
                .await?;
            issues.extend(response.items.into_iter().map(|issue| IssueObservation {
                name: issue.name,
                url: issue.url,
                severity: issue.severity,
                confidence: issue.confidence,
            }));
            let page = response.page.unwrap_or_default();
            if !page.truncated || page.next_cursor.is_empty() {
                break;
            }
            cursor = page.next_cursor;
        }
        Ok(issues)
    }
}
