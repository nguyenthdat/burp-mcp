use burp_protocol::protocol::{PageRequest, ScanIssuesRequest, SitemapSnapshotRequest};
use burp_protocol::{BurpClient, ClientError};
use sitegraph::SyncContext;
use sitegraph::{IssueObservation, SiteGraph, SitemapObservation, SyncBatch, SyncSummary};
use std::sync::Arc;

const PAGE_SIZE: u32 = 500;

#[derive(Clone)]
pub(crate) struct SiteGraphSynchronizer {
    client: BurpClient,
    graph: Arc<SiteGraph>,
}

impl SiteGraphSynchronizer {
    pub(crate) fn new(client: BurpClient, graph: Arc<SiteGraph>) -> Self {
        Self { client, graph }
    }

    pub(crate) async fn run(&self, prefix: String) -> Result<SyncSummary, String> {
        let (sitemap, source_total, pages_seen) = self.fetch_sitemap(prefix.clone()).await.map_err(|error| error.to_string())?;
        let issues = self.fetch_issues().await.map_err(|error| error.to_string())?;
        let items_seen = u64::try_from(sitemap.len() + issues.len()).map_err(|_| "sync item count overflow".to_owned())?;
        let graph_id = self
            .graph
            .status()
            .await
            .map_err(|error| error.to_string())?
            .graph_id;
        let mut context = SyncContext::snapshot(graph_id, if prefix.is_empty() { "all" } else { &prefix });
        context.source_total = source_total;
        context.pages_seen = pages_seen;
        context.items_seen = items_seen;
        self.graph
            .sync_with_context(&SyncBatch { sitemap, issues, ..SyncBatch::default() }, &context)
            .await
            .map_err(|error| error.to_string())
    }

    async fn fetch_sitemap(&self, prefix: String) -> Result<(Vec<SitemapObservation>, Option<u64>, u64), ClientError> {
        let mut cursor = String::new();
        let mut pages_seen = 0_u64;
        let mut source_total;
        let mut sitemap = Vec::new();
        loop {
            let response = self
                .client
                .sitemap_snapshot(SitemapSnapshotRequest {
                    url_prefix: prefix.clone(),
                    page: Some(PageRequest { limit: PAGE_SIZE, cursor }),
                })
                .await?;
            pages_seen += 1;
            source_total = Some(u64::from(response.page.as_ref().map(|page| page.total).unwrap_or(0)));
            sitemap.extend(response.items.into_iter().map(|entry| SitemapObservation {
                url: entry.url,
                method: entry.method,
                status: entry.status,
                content_type: entry.content_type,
                response_body: entry.response_body,
                redirect_url: entry.redirect_url,
                response_links: entry.response_links,
                form_actions: entry.form_actions,
                script_sources: entry.script_sources,
            }));
            let page = response.page.unwrap_or_default();
            if !page.truncated || page.next_cursor.is_empty() {
                break;
            }
            cursor = page.next_cursor;
        }
        Ok((sitemap, source_total, pages_seen))
    }

    async fn fetch_issues(&self) -> Result<Vec<IssueObservation>, ClientError> {
        let mut cursor = String::new();
        let mut issues = Vec::new();
        loop {
            let response = self
                .client
                .scan_issues(ScanIssuesRequest {
                    page: Some(PageRequest { limit: PAGE_SIZE, cursor }),
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
