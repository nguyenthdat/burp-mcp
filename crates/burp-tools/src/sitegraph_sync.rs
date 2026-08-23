use burp_protocol::protocol::{PageRequest, ScanIssuesRequest, SitemapSnapshotRequest};
use burp_protocol::{BurpClient, ClientError};
use sitegraph::{IssueObservation, SiteGraph, SitemapObservation, SyncBatch, SyncSummary};
use std::sync::Arc;

const PAGE_SIZE: u32 = 500;
const MAX_SYNC_ITEMS: usize = 10_000;

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
        let sitemap = self.fetch_sitemap(prefix).await.map_err(|error| error.to_string())?;
        let issues = self.fetch_issues().await.map_err(|error| error.to_string())?;
        self.graph
            .sync(&SyncBatch {
                sitemap,
                issues,
                ..SyncBatch::default()
            })
            .await
            .map_err(|error| error.to_string())
    }

    async fn fetch_sitemap(&self, prefix: String) -> Result<Vec<SitemapObservation>, ClientError> {
        let mut cursor = String::new();
        let mut sitemap = Vec::new();
        loop {
            let response = self
                .client
                .sitemap_snapshot(SitemapSnapshotRequest {
                    url_prefix: prefix.clone(),
                    page: Some(PageRequest { limit: PAGE_SIZE, cursor }),
                })
                .await?;
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
            if !page.truncated || page.next_cursor.is_empty() || sitemap.len() >= MAX_SYNC_ITEMS {
                break;
            }
            cursor = page.next_cursor;
        }
        Ok(sitemap)
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
            if !page.truncated || page.next_cursor.is_empty() || issues.len() >= MAX_SYNC_ITEMS {
                break;
            }
            cursor = page.next_cursor;
        }
        Ok(issues)
    }
}
