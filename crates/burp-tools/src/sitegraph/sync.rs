use ::sitegraph::enrichment::RulePack;
use ::sitegraph::{
    IssueObservation, SitemapObservation, SyncBatch, SyncContext, SyncSummary,
    TechnologyObservation, WebSocketObservation,
};
use burp_protocol::protocol::{
    AnnotateProxyEntriesRequest, PageRequest, ProxyAnnotation, ProxyHistoryRequest,
    ProxyWebSocketHistoryRequest, ScanIssuesRequest, SitemapSnapshotRequest,
};
use burp_protocol::{BurpClient, ClientError};
use sitegraph_daemon::GraphBackend;
use std::sync::{Arc, Mutex};

const PAGE_SIZE: u32 = 500;
pub(crate) const MAX_HTTP_ANNOTATIONS_PER_RUN: usize = 50;

#[derive(Clone)]
pub(crate) struct SiteGraphSynchronizer {
    client: BurpClient,
    graph: GraphBackend,
    rule_pack: Arc<RulePack>,
    last_http_id: Arc<Mutex<Option<u32>>>,
    last_websocket_id: Arc<Mutex<Option<u32>>>,
}

impl SiteGraphSynchronizer {
    pub(crate) fn new(client: BurpClient, graph: GraphBackend, rule_pack: Arc<RulePack>) -> Self {
        Self {
            client,
            graph,
            rule_pack,
            last_http_id: Arc::new(Mutex::new(None)),
            last_websocket_id: Arc::new(Mutex::new(None)),
        }
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
        let http_after_id = *self
            .last_http_id
            .lock()
            .map_err(|_| "HTTP history cursor lock poisoned")?;
        let mut http_cursor = String::new();
        let mut max_http_id = http_after_id;
        let mut annotations = Vec::with_capacity(MAX_HTTP_ANNOTATIONS_PER_RUN);
        loop {
            let response = self
                .client
                .proxy_history(ProxyHistoryRequest {
                    page: Some(PageRequest {
                        limit: PAGE_SIZE,
                        cursor: http_cursor.clone(),
                    }),
                    url_filter: prefix.clone(),
                    method_filter: String::new(),
                    status_filter: None,
                    has_notes: false,
                    color: String::new(),
                    after_id: http_after_id,
                })
                .await
                .map_err(|error| error.to_string())?;
            let page = response.page.unwrap_or_default();
            let raw_entries = response.items;
            let sitemap = raw_entries
                .iter()
                .map(|entry| {
                    max_http_id = Some(max_http_id.unwrap_or_default().max(entry.id));
                    SitemapObservation {
                        url: entry.url.clone(),
                        method: entry.method.clone(),
                        status: entry.status,
                        content_type: entry.content_type.clone(),
                        response_body: entry.response.clone(),
                        request_bytes: entry.request.clone(),
                        response_bytes: entry.response.clone(),
                        redirect_url: String::new(),
                        response_links: Vec::new(),
                        form_actions: Vec::new(),
                        script_sources: Vec::new(),
                    }
                })
                .collect::<Vec<_>>();
            if !sitemap.is_empty() {
                let mut context = SyncContext::snapshot(&graph_id, format!("{scope}:http_history"));
                context.run_id = format!("{run_id}-http-history-{http_cursor}");
                context.source = "burp_http_history".to_owned();
                context.complete = false;
                context.items_seen = sitemap.len() as u64;
                last_summary = Some(
                    self.graph
                        .sync_with_context(
                            &SyncBatch {
                                sitemap,
                                ..SyncBatch::default()
                            },
                            &context,
                        )
                        .await
                        .map_err(|error| error.to_string())?,
                );

                for entry in &raw_entries {
                    if annotations.len() >= MAX_HTTP_ANNOTATIONS_PER_RUN {
                        break;
                    }
                    if let Some(candidate) =
                        evaluate_http_annotation(&self.rule_pack, &entry.request, &entry.response)
                    {
                        annotations.push(ProxyAnnotation {
                            id: entry.id,
                            highlight: candidate.color,
                            sitegraph_marker: candidate.marker,
                        });
                    }
                }
            }
            if !page.truncated || page.next_cursor.is_empty() {
                break;
            }
            http_cursor = page.next_cursor;
        }
        let annotations_applied = if annotations.is_empty() {
            true
        } else {
            match self
                .client
                .annotate_proxy_entries(AnnotateProxyEntriesRequest {
                    entries: annotations,
                })
                .await
            {
                Ok(_) => true,
                Err(error) => {
                    tracing::warn!(%error, "sitegraph indexing completed but Burp annotation sync failed; entries will be retried on the next sync");
                    false
                }
            }
        };
        if annotations_applied {
            *self
                .last_http_id
                .lock()
                .map_err(|_| "HTTP history cursor lock poisoned")? = max_http_id;
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
        let websocket_after_id = *self
            .last_websocket_id
            .lock()
            .map_err(|_| "WebSocket history cursor lock poisoned")?;
        let mut websocket_cursor = String::new();
        let mut max_websocket_id = websocket_after_id;
        loop {
            let response = self
                .client
                .proxy_websocket_history(ProxyWebSocketHistoryRequest {
                    page: Some(PageRequest {
                        limit: PAGE_SIZE,
                        cursor: websocket_cursor.clone(),
                    }),
                    after_id: websocket_after_id,
                })
                .await
                .map_err(|error| error.to_string())?;
            let page = response.page.unwrap_or_default();
            let websocket_messages = response
                .items
                .into_iter()
                .map(|entry| {
                    max_websocket_id = Some(max_websocket_id.unwrap_or_default().max(entry.id));
                    WebSocketObservation {
                        id: entry.id.to_string(),
                        web_socket_id: entry.web_socket_id.to_string(),
                        direction: entry.direction,
                        upgrade_url: entry.upgrade_url,
                        payload: entry.payload,
                        edited_payload: entry.edited_payload,
                    }
                })
                .collect::<Vec<_>>();
            if !websocket_messages.is_empty() {
                let mut context = SyncContext::snapshot(&graph_id, format!("{scope}:websocket"));
                context.run_id = format!("{run_id}-websocket-{websocket_cursor}");
                context.source = "burp_websocket_history".to_owned();
                context.complete = false;
                context.items_seen = websocket_messages.len() as u64;
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
        *self
            .last_websocket_id
            .lock()
            .map_err(|_| "WebSocket history cursor lock poisoned")? = max_websocket_id;
        last_summary.ok_or_else(|| "Burp history sources returned no page".to_owned())
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AnnotationCandidate {
    pub severity: String,
    pub marker: String,
    pub color: String,
}

fn split_http_body(bytes: &[u8]) -> &[u8] {
    if let Some(pos) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
        &bytes[pos + 4..]
    } else if let Some(pos) = bytes.windows(2).position(|w| w == b"\n\n") {
        &bytes[pos + 2..]
    } else {
        b""
    }
}

fn is_medium_or_higher(severity: &str) -> bool {
    matches!(
        severity.to_ascii_lowercase().as_str(),
        "medium" | "high" | "critical"
    )
}

fn severity_rank(sev: &str) -> u8 {
    match sev.to_ascii_lowercase().as_str() {
        "critical" => 3,
        "high" => 2,
        "medium" => 1,
        _ => 0,
    }
}

pub(crate) fn evaluate_http_annotation(
    rule_pack: &RulePack,
    request: &[u8],
    response: &[u8],
) -> Option<AnnotationCandidate> {
    let req_matches = rule_pack.matches("request_message", request);
    let resp_msg_matches = rule_pack.matches("response_message", response);
    let body = split_http_body(response);
    let resp_body_matches = rule_pack.matches("response_body", body);

    let req_hits: Vec<_> = req_matches
        .into_iter()
        .filter(|m| is_medium_or_higher(&m.severity))
        .collect();
    let resp_hits: Vec<_> = resp_msg_matches
        .into_iter()
        .chain(resp_body_matches)
        .filter(|m| is_medium_or_higher(&m.severity))
        .collect();

    if req_hits.is_empty() && resp_hits.is_empty() {
        return None;
    }

    let direction = match (!req_hits.is_empty(), !resp_hits.is_empty()) {
        (true, false) => "request",
        (false, true) => "response",
        (true, true) => "both",
        (false, false) => unreachable!(),
    };

    let mut highest_rank = 0u8;
    let mut highest_severity = "medium";
    for hit in req_hits.iter().chain(resp_hits.iter()) {
        let rank = severity_rank(&hit.severity);
        if rank > highest_rank {
            highest_rank = rank;
            highest_severity = match rank {
                3 => "critical",
                2 => "high",
                _ => "medium",
            };
        }
    }

    let color = match highest_severity {
        "critical" | "high" => "RED",
        "medium" => "ORANGE",
        _ => "YELLOW",
    }
    .to_owned();

    let mut rule_ids: Vec<String> = req_hits
        .iter()
        .chain(resp_hits.iter())
        .map(|m| m.rule_id.clone())
        .collect();
    rule_ids.sort();
    rule_ids.dedup();
    let rules_joined = rule_ids.join(",");

    let marker = format!(
        "[SiteGraph] severity={highest_severity} rules={rules_joined} direction={direction}"
    );

    Some(AnnotationCandidate {
        severity: highest_severity.to_owned(),
        marker,
        color,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_http_body() {
        let msg = b"HTTP/1.1 200 OK\r\nHeader: val\r\n\r\nHello World";
        assert_eq!(split_http_body(msg), b"Hello World");

        let lf_msg = b"HTTP/1.1 200 OK\nHeader: val\n\nHello LF";
        assert_eq!(split_http_body(lf_msg), b"Hello LF");

        let no_body = b"HTTP/1.1 200 OK\r\nHeader: val";
        assert_eq!(split_http_body(no_body), b"");
    }

    #[test]
    fn test_evaluate_annotation_medium_plus_filter() {
        let dsl = r#"
pack {
    id = "test_pack"
    version = "1.0.0"
    max_matches = 100
}
rule "low_rule" {
    pattern = "low_marker"
    capture_group = 0
    severity = "low"
    surfaces = ["request_message", "response_body"]
}
rule "med_rule" {
    pattern = "med_marker"
    capture_group = 0
    severity = "medium"
    surfaces = ["response_body"]
}
rule "crit_rule" {
    pattern = "crit_marker"
    capture_group = 0
    severity = "critical"
    surfaces = ["request_message"]
}
"#;
        let pack = RulePack::from_dsl(dsl).expect("dsl parses");

        // Low-only should return None
        let res_low =
            evaluate_http_annotation(&pack, b"low_marker", b"HTTP/1.1 200 OK\r\n\r\nlow_marker");
        assert!(res_low.is_none());

        // Medium should return ORANGE and response direction
        let res_med =
            evaluate_http_annotation(&pack, b"nothing", b"HTTP/1.1 200 OK\r\n\r\nmed_marker");
        assert!(res_med.is_some());
        let c_med = res_med.unwrap();
        assert_eq!(c_med.severity, "medium");
        assert_eq!(c_med.color, "ORANGE");
        assert_eq!(
            c_med.marker,
            "[SiteGraph] severity=medium rules=med_rule direction=response"
        );

        // Critical should return RED and request direction
        let res_crit =
            evaluate_http_annotation(&pack, b"crit_marker", b"HTTP/1.1 200 OK\r\n\r\nnothing");
        assert!(res_crit.is_some());
        let c_crit = res_crit.unwrap();
        assert_eq!(c_crit.severity, "critical");
        assert_eq!(c_crit.color, "RED");
        assert_eq!(
            c_crit.marker,
            "[SiteGraph] severity=critical rules=crit_rule direction=request"
        );

        // Both directions with Critical in request and Medium in response -> Critical, RED, direction=both
        let res_both =
            evaluate_http_annotation(&pack, b"crit_marker", b"HTTP/1.1 200 OK\r\n\r\nmed_marker");
        assert!(res_both.is_some());
        let c_both = res_both.unwrap();
        assert_eq!(c_both.severity, "critical");
        assert_eq!(c_both.color, "RED");
        assert_eq!(
            c_both.marker,
            "[SiteGraph] severity=critical rules=crit_rule,med_rule direction=both"
        );
    }
}
