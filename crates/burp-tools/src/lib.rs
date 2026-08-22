use burp_protocol::BurpClient;
use burp_protocol::proto::{
    AddIssueRequest, CancelJobRequest, ClearHttpHandlerRequest, ClearProxyRulesRequest,
    CloseWebSocketRequest, ConfigResponse, CookieJarRequest, CreateSessionRuleRequest,
    CreateWebSocketRequest, ExportConfigRequest, ExtensionInfoRequest,
    GenerateCollaboratorPayloadsRequest, GetJobResultRequest, GetJobStatusRequest, HttpHeaderEntry,
    ImportBCheckRequest, ImportBambdaRequest, ImportConfigRequest, InterceptStateRequest,
    ListSessionRulesRequest, ListWebSocketsRequest, MutateScopeRequest, PageRequest,
    PollCollaboratorInteractionsRequest, ProxyDetailRequest, ProxyHistoryRequest,
    ProxyWebSocketHistoryRequest, RegisterHttpHandlerRequest, RegisterProxyRuleRequest,
    RemoveSessionRulesRequest, ScanIssueDetailRequest, ScanIssuesRequest, ScopeCheckRequest,
    SendRequestRequest, SendRequestsRequest, SendToIntruderRequest, SendToRepeaterRequest,
    SendWebSocketBinaryRequest, SendWebSocketTextRequest, SetCookieRequest, SetHighlightRequest,
    SetNoteRequest, SitemapSnapshotRequest, StartAuditRequest, StartBoundedInputMatrixRequest,
    StartConcurrentRequestCheckRequest, StartCrawlRequest, TargetInfoRequest,
};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::{schemars, tool, tool_router};
use serde::{Deserialize, Serialize};
use sitegraph::{IssueObservation, SitemapObservation, SqliteGraph, SyncBatch};
use std::path::Path;
use std::sync::Arc;
use utility_tools::{self as utility, DataValue};

const MAX_PAGE_SIZE: u32 = 500;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ProxyHistoryInput {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub cursor: Option<String>,
    pub url_filter: Option<String>,
    pub method_filter: Option<String>,
    pub status_filter: Option<u32>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct ProxyHistoryItemOutput {
    index: u32,
    method: String,
    url: String,
    status: u32,
    length: u64,
    has_response: bool,
    notes: Option<String>,
    highlight: Option<String>,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
struct ProxyHistoryOutput {
    items: Vec<ProxyHistoryItemOutput>,
    total: u32,
    truncated: bool,
    next_cursor: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ProxyHistoryFilteredInput {
    pub has_notes: Option<bool>,
    pub color: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub cursor: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ProxyDetailInput {
    pub index: u32,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct ProxyDetailOutput {
    index: u32,
    request: String,
    response: Option<String>,
    notes: Option<String>,
    highlight: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SitemapInput {
    pub url_prefix: Option<String>,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct SitemapItemOutput {
    url: String,
    method: String,
    status: u32,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct SitemapOutput {
    items: Vec<SitemapItemOutput>,
    total: u32,
    truncated: bool,
    next_cursor: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TargetInfoInput {
    pub url: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct TargetInfoOutput {
    hosts: Vec<String>,
    technologies: Vec<String>,
    requests_sampled: u32,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ScopeCheckInput {
    pub url: String,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
struct ScanIssueOutput {
    index: u32,
    name: String,
    severity: String,
    confidence: String,
    url: String,
    detail: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct ScanIssuesOutput {
    items: Vec<ScanIssueOutput>,
    total: u32,
    truncated: bool,
    next_cursor: Option<String>,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CookieJarInput {
    pub limit: Option<u32>,
    pub domain: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SetCookieInput {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: Option<String>,
    pub expiration: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AddIssueInput {
    pub name: String,
    pub url: String,
    pub detail: Option<String>,
    pub remediation: Option<String>,
    pub severity: Option<String>,
    pub confidence: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ScanIssueDetailInput {
    pub index: u32,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct InterceptStateInput {
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ProxyWebSocketHistoryInput {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct CookieOutput {
    name: String,
    value: String,
    domain: Option<String>,
    path: Option<String>,
    expiration: Option<String>,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
struct ServerInfoOutput {
    extension: String,
    version: String,
    burp_name: String,
    burp_version: String,
    burp_edition: String,
    burp_build_number: u64,
    capabilities: Vec<String>,
    max_message_bytes: u32,
    max_page_size: u32,
    max_concurrent_calls_per_connection: u32,
    max_rpc_timeout_seconds: u32,
    max_response_bytes: u32,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SendRequestInput {
    pub method: Option<String>,
    pub url: String,
    pub body: Option<String>,
    pub headers: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SendRequestsInput {
    pub requests: Vec<SendRequestInput>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct SendResponseOutput {
    request: String,
    response: Option<String>,
    status: Option<u32>,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SendToRepeaterInput {
    pub request: String,
    pub host: String,
    pub port: Option<u32>,
    pub https: Option<bool>,
    pub tab_name: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SendToIntruderInput {
    pub request: String,
    pub host: String,
    pub port: Option<u32>,
    pub https: Option<bool>,
    pub tab_name: Option<String>,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ConvertRequestInput {
    pub request: String,
    pub convert_to: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ExportRequestInput {
    pub request: String,
    pub host: Option<String>,
    pub format: Option<String>,
    pub https: Option<bool>,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ExtractResponseInput {
    pub index: u32,
    pub regex: String,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct HighlightInput {
    pub index: u32,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AnnotateInput {
    pub index: u32,
    pub note: String,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ScopeMutationInput {
    pub url: String,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ImportConfigInput {
    pub config: String,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RegisterHttpHandlerInput {
    pub header_name: Option<String>,
    pub header_value: Option<String>,
    #[serde(rename = "match")]
    pub match_text: Option<String>,
    pub replace: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RegisterProxyRuleInput {
    pub url_contains: String,
    pub intercept: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateSessionRuleInput {
    pub find: String,
    pub replace: String,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ConcurrentRequestCheckInput {
    pub request: String,
    pub host: String,
    pub port: Option<u32>,
    pub https: Option<bool>,
    pub count: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct BoundedInputMatrixInput {
    pub template: String,
    pub host: String,
    pub port: Option<u32>,
    pub https: Option<bool>,
    pub marker: Option<String>,
    pub wordlist: Vec<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CrawlInput {
    pub url: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AuditInput {
    pub url: String,
    pub mode: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct JobInput {
    pub job_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct JobResultInput {
    pub job_id: String,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CollaboratorGenerateInput {
    pub count: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CollaboratorPollInput {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WebSocketCreateInput {
    pub host: String,
    pub port: Option<u32>,
    pub https: Option<bool>,
    pub path: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WebSocketTextInput {
    pub id: String,
    pub text: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WebSocketBinaryInput {
    pub id: String,
    pub data: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WebSocketIdInput {
    pub id: String,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct BambdaImportInput {
    pub script: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct BCheckImportInput {
    pub script: String,
    pub enabled: bool,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UtilityValueInput {
    Text { value: String },
    Bytes { base64: String },
    Json { value: serde_json::Value },
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DecoderStepInput {
    pub op: String,
    #[serde(default)]
    pub args: serde_json::Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DecoderInput {
    pub input: UtilityValueInput,
    #[serde(default)]
    pub operation: Option<String>,
    #[serde(default)]
    pub args: serde_json::Value,
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub describe: Option<String>,
    #[serde(default)]
    pub magic: bool,
    #[serde(default)]
    pub steps: Vec<DecoderStepInput>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SiteGraphSyncInput {
    pub url_prefix: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SiteGraphSearchInput {
    pub query: String,
    pub limit: Option<u32>,
    pub cursor: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SiteGraphEndpointInput {
    pub id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SiteGraphNeighborsInput {
    pub id: String,
    pub limit: Option<u32>,
    pub cursor: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SiteGraphTraceInput {
    pub id: String,
    pub max_depth: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SiteGraphDiffInput {
    pub since: i64,
    pub limit: Option<u32>,
    pub cursor: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SiteGraphExportInput {
    pub format: Option<String>,
    pub limit: Option<u32>,
    pub cursor: Option<u32>,
}

#[derive(Clone)]
pub struct BurpTools {
    client: BurpClient,
    sitegraph: Arc<SqliteGraph>,
}

#[tool_router(server_handler)]
impl BurpTools {
    pub async fn new(client: BurpClient, graph_path: &Path) -> Result<Self, String> {
        Ok(Self {
            client,
            sitegraph: Arc::new(
                SqliteGraph::open(graph_path)
                    .await
                    .map_err(|error| error.to_string())?,
            ),
        })
    }

    #[tool(
        name = "burp_proxy_history",
        description = "Get a bounded page of Burp proxy history entries"
    )]
    async fn proxy_history(&self, Parameters(input): Parameters<ProxyHistoryInput>) -> String {
        let limit = input.limit.unwrap_or(100);
        if limit > MAX_PAGE_SIZE {
            return serde_json::json!({"error": "limit must be at most 500"}).to_string();
        }
        match self.client.proxy_history(ProxyHistoryRequest {
            page: Some(PageRequest {
                limit,
                cursor: input.cursor.unwrap_or_else(|| input.offset.unwrap_or_default().to_string()),
            }),
            url_filter: input.url_filter.unwrap_or_default(),
            method_filter: input.method_filter.unwrap_or_default(),
            status_filter: input.status_filter,
            has_notes: false,
            color: String::new(),
        }).await {
            Ok(response) => {
                let page = response.page.unwrap_or_default();
                serde_json::to_string(&ProxyHistoryOutput {
                    items: response.items.into_iter().map(|item| ProxyHistoryItemOutput {
                        index: item.index,
                        method: item.method,
                        url: item.url,
                        status: item.status,
                        length: item.length,
                        has_response: item.has_response,
                        notes: (!item.notes.is_empty()).then_some(item.notes),
                        highlight: (!item.highlight.is_empty()).then_some(item.highlight),
                    }).collect(),
                    total: page.total,
                    truncated: page.truncated,
                    next_cursor: (!page.next_cursor.is_empty()).then_some(page.next_cursor),
                }).expect("proxy output must serialize")
            }
            Err(error) => serde_json::json!({"error": error.to_string(), "connected": false, "action": "Start Burp with the Burp MCP extension and retry"}).to_string(),
        }
    }
    #[tool(
        name = "burp_proxy_history_filtered",
        description = "Filter proxy history by annotation color or notes"
    )]
    async fn proxy_history_filtered(
        &self,
        Parameters(input): Parameters<ProxyHistoryFilteredInput>,
    ) -> String {
        let limit = input.limit.unwrap_or(100);
        if limit > MAX_PAGE_SIZE {
            return serde_json::json!({"error": "limit must be at most 500"}).to_string();
        }
        match self
            .client
            .proxy_history(ProxyHistoryRequest {
                page: Some(PageRequest {
                    limit,
                    cursor: input
                        .cursor
                        .unwrap_or_else(|| input.offset.unwrap_or_default().to_string()),
                }),
                url_filter: String::new(),
                method_filter: String::new(),
                status_filter: None,
                has_notes: input.has_notes.unwrap_or(false),
                color: input.color.unwrap_or_default(),
            })
            .await
        {
            Ok(response) => {
                let page = response.page.unwrap_or_default();
                serde_json::to_string(&ProxyHistoryOutput {
                    items: response
                        .items
                        .into_iter()
                        .map(|item| ProxyHistoryItemOutput {
                            index: item.index,
                            method: item.method,
                            url: item.url,
                            status: item.status,
                            length: item.length,
                            has_response: item.has_response,
                            notes: (!item.notes.is_empty()).then_some(item.notes),
                            highlight: (!item.highlight.is_empty()).then_some(item.highlight),
                        })
                        .collect(),
                    total: page.total,
                    truncated: page.truncated,
                    next_cursor: (!page.next_cursor.is_empty()).then_some(page.next_cursor),
                })
                .expect("filtered proxy output must serialize")
            }
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_proxy_detail",
        description = "Get full request and response details for one Burp proxy history index"
    )]
    async fn proxy_detail(&self, Parameters(input): Parameters<ProxyDetailInput>) -> String {
        match self
            .client
            .proxy_detail(ProxyDetailRequest { index: input.index })
            .await
        {
            Ok(detail) => serde_json::to_string(&ProxyDetailOutput {
                index: detail.index,
                request: String::from_utf8_lossy(&detail.request).into_owned(),
                response: (!detail.response.is_empty())
                    .then(|| String::from_utf8_lossy(&detail.response).into_owned()),
                notes: (!detail.notes.is_empty()).then_some(detail.notes),
                highlight: (!detail.highlight.is_empty()).then_some(detail.highlight),
            })
            .expect("proxy detail output must serialize"),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_sitemap",
        description = "Get a bounded page of Burp site map entries with an optional URL prefix"
    )]
    async fn sitemap(&self, Parameters(input): Parameters<SitemapInput>) -> String {
        let limit = input.limit.unwrap_or(100);
        if limit > MAX_PAGE_SIZE {
            return serde_json::json!({"error": "limit must be at most 500"}).to_string();
        }
        match self
            .client
            .sitemap_snapshot(SitemapSnapshotRequest {
                url_prefix: input.url_prefix.unwrap_or_default(),
                page: Some(PageRequest {
                    limit,
                    cursor: input.cursor.unwrap_or_default(),
                }),
            })
            .await
        {
            Ok(response) => {
                let page = response.page.unwrap_or_default();
                serde_json::to_string(&SitemapOutput {
                    items: response
                        .items
                        .into_iter()
                        .map(|item| SitemapItemOutput {
                            url: item.url,
                            method: item.method,
                            status: item.status,
                        })
                        .collect(),
                    total: page.total,
                    truncated: page.truncated,
                    next_cursor: (!page.next_cursor.is_empty()).then_some(page.next_cursor),
                })
                .expect("sitemap output must serialize")
            }
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_target_info",
        description = "Summarize hosts and technology headers from a bounded Burp site map sample"
    )]
    async fn target_info(&self, Parameters(input): Parameters<TargetInfoInput>) -> String {
        match self
            .client
            .target_info(TargetInfoRequest {
                url_prefix: input.url.unwrap_or_default(),
                limit: input.limit.unwrap_or(500).min(500),
            })
            .await
        {
            Ok(info) => serde_json::to_string(&TargetInfoOutput {
                hosts: info.hosts,
                technologies: info.technologies,
                requests_sampled: info.requests_sampled,
            })
            .expect("target info output must serialize"),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_get_scope",
        description = "Check whether one URL is currently in Burp target scope"
    )]
    async fn scope_check(&self, Parameters(input): Parameters<ScopeCheckInput>) -> String {
        match self
            .client
            .scope_check(ScopeCheckRequest { url: input.url })
            .await
        {
            Ok(scope) => {
                serde_json::json!({"url": scope.url, "in_scope": scope.in_scope}).to_string()
            }
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_send_request",
        description = "Send an HTTP request through Burp and get the response"
    )]
    async fn send_request(&self, Parameters(input): Parameters<SendRequestInput>) -> String {
        match self.client.send_request(to_proto_request(input)).await {
            Ok(response) => serde_json::to_string(&to_send_output(response))
                .expect("send output must serialize"),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_send_request_parallel",
        description = "Send parallel HTTP requests"
    )]
    async fn send_request_parallel(
        &self,
        Parameters(input): Parameters<SendRequestsInput>,
    ) -> String {
        if input.requests.len() > 32 {
            return serde_json::json!({"error": "at most 32 requests may be sent in one batch"})
                .to_string();
        }
        match self
            .client
            .send_requests(SendRequestsRequest {
                requests: input.requests.into_iter().map(to_proto_request).collect(),
            })
            .await
        {
            Ok(response) => serde_json::to_string(
                &response
                    .responses
                    .into_iter()
                    .map(to_send_output)
                    .collect::<Vec<_>>(),
            )
            .expect("parallel send output must serialize"),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_send_to_repeater",
        description = "Display a raw HTTP request in Burp Repeater without sending it. tab_name is an optional tab caption, not a tag."
    )]
    async fn send_to_repeater(&self, Parameters(input): Parameters<SendToRepeaterInput>) -> String {
        let https = input.https.unwrap_or(false);
        match self
            .client
            .send_to_repeater(SendToRepeaterRequest {
                request: input.request.into_bytes(),
                host: input.host,
                port: input.port.unwrap_or(if https { 443 } else { 80 }),
                https,
                tab_name: input.tab_name.unwrap_or_else(|| "MCP".to_owned()),
            })
            .await
        {
            Ok(response) => {
                serde_json::json!({"success": response.success, "message": response.message})
                    .to_string()
            }
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }
    #[tool(
        name = "burp_highlight",
        description = "Set the highlight color on an item in the current Burp Proxy HTTP history."
    )]
    async fn highlight(&self, Parameters(input): Parameters<HighlightInput>) -> String {
        match self
            .client
            .set_highlight(SetHighlightRequest {
                index: input.index,
                color: input.color.unwrap_or_default(),
            })
            .await
        {
            Ok(response) => {
                serde_json::json!({"success": response.success, "color": response.message})
                    .to_string()
            }
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_annotate",
        description = "Set notes on an item in the current Burp Proxy HTTP history."
    )]
    async fn annotate(&self, Parameters(input): Parameters<AnnotateInput>) -> String {
        match self
            .client
            .set_note(SetNoteRequest {
                index: input.index,
                note: input.note,
            })
            .await
        {
            Ok(response) => {
                serde_json::json!({"success": response.success, "message": response.message})
                    .to_string()
            }
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }
    #[tool(
        name = "burp_add_to_scope",
        description = "Add a URL to Burp target scope"
    )]
    async fn add_to_scope(&self, Parameters(input): Parameters<ScopeMutationInput>) -> String {
        self.mutate_scope(input.url, true).await
    }

    #[tool(
        name = "burp_remove_from_scope",
        description = "Remove a URL from Burp target scope"
    )]
    async fn remove_from_scope(&self, Parameters(input): Parameters<ScopeMutationInput>) -> String {
        self.mutate_scope(input.url, false).await
    }

    async fn mutate_scope(&self, url: String, include: bool) -> String {
        match self
            .client
            .mutate_scope(MutateScopeRequest { url, include })
            .await
        {
            Ok(response) => {
                serde_json::json!({"success": response.success, "message": response.message})
                    .to_string()
            }
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }
    #[tool(
        name = "burp_export_config",
        description = "Export Burp project configuration as JSON"
    )]
    async fn export_config(&self) -> String {
        match self
            .client
            .export_config(ExportConfigRequest { paths: Vec::new() })
            .await
        {
            Ok(ConfigResponse { config }) => serde_json::json!({"config": config}).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_import_config",
        description = "Import Burp project configuration from JSON"
    )]
    async fn import_config(&self, Parameters(input): Parameters<ImportConfigInput>) -> String {
        match self
            .client
            .import_config(ImportConfigRequest {
                config: input.config,
            })
            .await
        {
            Ok(response) => {
                serde_json::json!({"success": response.success, "message": response.message})
                    .to_string()
            }
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_register_http_handler",
        description = "Register an auto-modify rule for HTTP requests (add header or replace text)"
    )]
    async fn register_http_handler(
        &self,
        Parameters(input): Parameters<RegisterHttpHandlerInput>,
    ) -> String {
        match self
            .client
            .register_http_handler(RegisterHttpHandlerRequest {
                header_name: input.header_name.unwrap_or_default(),
                header_value: input.header_value.unwrap_or_default(),
                r#match: input.match_text.unwrap_or_default(),
                replacement: input.replace.unwrap_or_default(),
            })
            .await
        {
            Ok(response) => {
                serde_json::json!({"success": response.success, "message": response.message})
                    .to_string()
            }
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_remove_http_handler",
        description = "Remove/clear HTTP handler rules"
    )]
    async fn remove_http_handler(&self) -> String {
        action_json(
            self.client
                .clear_http_handler(ClearHttpHandlerRequest {})
                .await,
        )
    }

    #[tool(
        name = "burp_register_proxy_rule",
        description = "Register a proxy intercept rule (intercept URLs containing a string)"
    )]
    async fn register_proxy_rule(
        &self,
        Parameters(input): Parameters<RegisterProxyRuleInput>,
    ) -> String {
        action_json(
            self.client
                .register_proxy_rule(RegisterProxyRuleRequest {
                    url_contains: input.url_contains,
                    intercept: input.intercept.unwrap_or(true),
                })
                .await,
        )
    }

    #[tool(
        name = "burp_remove_proxy_rule",
        description = "Remove/clear proxy intercept rules"
    )]
    async fn remove_proxy_rule(&self) -> String {
        action_json(
            self.client
                .clear_proxy_rules(ClearProxyRulesRequest {})
                .await,
        )
    }

    #[tool(
        name = "burp_session_create_rule",
        description = "Create session handling rule"
    )]
    async fn session_create_rule(
        &self,
        Parameters(input): Parameters<CreateSessionRuleInput>,
    ) -> String {
        action_json(
            self.client
                .create_session_rule(CreateSessionRuleRequest {
                    find: input.find,
                    replacement: input.replace,
                })
                .await,
        )
    }

    #[tool(name = "burp_session_list_rules", description = "List session rules")]
    async fn session_list_rules(&self) -> String {
        match self.client.list_session_rules(ListSessionRulesRequest {}).await {
            Ok(response) => serde_json::json!({"rules": response.items.into_iter().map(|rule| serde_json::json!({"find": rule.find, "replace": rule.replacement})).collect::<Vec<_>>()}).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(name = "burp_session_remove_rule", description = "Remove session rule")]
    async fn session_remove_rule(&self) -> String {
        action_json(
            self.client
                .remove_session_rules(RemoveSessionRulesRequest {})
                .await,
        )
    }

    #[tool(
        name = "burp_race_condition",
        description = "Start a bounded concurrent request comparison job"
    )]
    async fn race_condition(
        &self,
        Parameters(input): Parameters<ConcurrentRequestCheckInput>,
    ) -> String {
        let https = input.https.unwrap_or(true);
        let port = input.port.unwrap_or(if https { 443 } else { 80 });
        job_status_json(
            self.client
                .start_concurrent_request_check(StartConcurrentRequestCheckRequest {
                    request: input.request.into_bytes(),
                    host: input.host,
                    port,
                    https,
                    count: input.count.unwrap_or(10),
                })
                .await,
        )
    }

    #[tool(
        name = "burp_inline_fuzzer",
        description = "Start a bounded input matrix job against an authorized test target"
    )]
    async fn inline_fuzzer(
        &self,
        Parameters(input): Parameters<BoundedInputMatrixInput>,
    ) -> String {
        let https = input.https.unwrap_or(true);
        let port = input.port.unwrap_or(if https { 443 } else { 80 });
        job_status_json(
            self.client
                .start_bounded_input_matrix(StartBoundedInputMatrixRequest {
                    template: input.template.into_bytes(),
                    host: input.host,
                    port,
                    https,
                    marker: input.marker.unwrap_or_else(|| "FUZZ".to_owned()),
                    inputs: input.wordlist,
                })
                .await,
        )
    }

    #[tool(name = "burp_scan", description = "Start a bounded Burp audit job")]
    async fn scan(&self, Parameters(input): Parameters<AuditInput>) -> String {
        let active = input.mode.as_deref().unwrap_or("active") != "passive";
        job_status_json(
            self.client
                .start_audit(StartAuditRequest {
                    url: input.url,
                    active,
                })
                .await,
        )
    }

    #[tool(name = "burp_crawl", description = "Start a bounded Burp crawl job")]
    async fn crawl(&self, Parameters(input): Parameters<CrawlInput>) -> String {
        job_status_json(
            self.client
                .start_crawl(StartCrawlRequest { url: input.url })
                .await,
        )
    }

    #[tool(
        name = "burp_collaborator_generate",
        description = "Generate bounded Collaborator identifiers"
    )]
    async fn collaborator_generate(
        &self,
        Parameters(input): Parameters<CollaboratorGenerateInput>,
    ) -> String {
        match self
            .client
            .generate_collaborator_payloads(GenerateCollaboratorPayloadsRequest {
                count: input.count.unwrap_or(1),
            })
            .await
        {
            Ok(response) => serde_json::json!({"payloads": response.payloads}).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_websocket_create",
        description = "Create a WebSocket connection through Burp"
    )]
    async fn websocket_create(
        &self,
        Parameters(input): Parameters<WebSocketCreateInput>,
    ) -> String {
        let https = input.https.unwrap_or(true);
        let port = input.port.unwrap_or(if https { 443 } else { 80 });
        match self
            .client
            .create_websocket(CreateWebSocketRequest {
                host: input.host,
                port,
                https,
                path: input.path.unwrap_or_else(|| "/".to_owned()),
            })
            .await
        {
            Ok(response) => {
                serde_json::json!({"id": response.id, "status": response.status}).to_string()
            }
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_websocket_send_text",
        description = "Send text on a managed WebSocket"
    )]
    async fn websocket_send_text(
        &self,
        Parameters(input): Parameters<WebSocketTextInput>,
    ) -> String {
        action_json(
            self.client
                .send_websocket_text(SendWebSocketTextRequest {
                    id: input.id,
                    text: input.text,
                })
                .await,
        )
    }

    #[tool(
        name = "burp_websocket_send_binary",
        description = "Send binary data encoded as base64 on a managed WebSocket"
    )]
    async fn websocket_send_binary(
        &self,
        Parameters(input): Parameters<WebSocketBinaryInput>,
    ) -> String {
        let data =
            match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, input.data) {
                Ok(data) => data,
                Err(error) => {
                    return serde_json::json!({"error": format!("invalid base64: {error}")})
                        .to_string();
                }
            };
        action_json(
            self.client
                .send_websocket_binary(SendWebSocketBinaryRequest { id: input.id, data })
                .await,
        )
    }

    #[tool(
        name = "burp_websocket_close",
        description = "Close a managed WebSocket"
    )]
    async fn websocket_close(&self, Parameters(input): Parameters<WebSocketIdInput>) -> String {
        action_json(
            self.client
                .close_websocket(CloseWebSocketRequest { id: input.id })
                .await,
        )
    }

    #[tool(
        name = "burp_websocket_list",
        description = "List active managed WebSockets"
    )]
    async fn websocket_list(&self) -> String {
        match self.client.list_websockets(ListWebSocketsRequest {}).await {
            Ok(response) => serde_json::json!({"websockets": response.ids.into_iter().map(|id| serde_json::json!({"id": id})).collect::<Vec<_>>()}).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_collaborator_poll",
        description = "Get a bounded page of Collaborator interactions"
    )]
    async fn collaborator_poll(
        &self,
        Parameters(input): Parameters<CollaboratorPollInput>,
    ) -> String {
        let limit = input.limit.unwrap_or(100);
        if limit > MAX_PAGE_SIZE {
            return serde_json::json!({"error": "limit must be at most 500"}).to_string();
        }
        match self
            .client
            .poll_collaborator_interactions(PollCollaboratorInteractionsRequest {
                page: Some(PageRequest {
                    limit,
                    cursor: input.cursor.unwrap_or_default(),
                }),
            })
            .await
        {
            Ok(response) => {
                let page = response.page.unwrap_or_default();
                serde_json::json!({
                    "interactions": response.items.into_iter().map(|item| serde_json::json!({
                        "id": item.id, "type": item.r#type, "client_ip": item.client_ip,
                        "client_port": item.client_port, "timestamp": item.timestamp,
                    })).collect::<Vec<_>>(),
                    "count": page.total, "truncated": page.truncated,
                    "next_cursor": (!page.next_cursor.is_empty()).then_some(page.next_cursor),
                })
                .to_string()
            }
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_job_status",
        description = "Get the current state of a Burp background job"
    )]
    async fn job_status(&self, Parameters(input): Parameters<JobInput>) -> String {
        job_status_json(
            self.client
                .get_job_status(GetJobStatusRequest { id: input.job_id })
                .await,
        )
    }

    #[tool(
        name = "burp_bambda_import",
        description = "Import a Bambda script into Burp without executing it"
    )]
    async fn bambda_import(&self, Parameters(input): Parameters<BambdaImportInput>) -> String {
        script_import_json(
            self.client
                .import_bambda(ImportBambdaRequest {
                    script: input.script,
                })
                .await,
        )
    }

    #[tool(
        name = "burp_bcheck_import",
        description = "Import a BCheck script into Burp without running it"
    )]
    async fn bcheck_import(&self, Parameters(input): Parameters<BCheckImportInput>) -> String {
        script_import_json(
            self.client
                .import_bcheck(ImportBCheckRequest {
                    script: input.script,
                    enabled: input.enabled,
                })
                .await,
        )
    }
    #[tool(name = "burp_job_cancel", description = "Cancel a Burp background job")]
    async fn job_cancel(&self, Parameters(input): Parameters<JobInput>) -> String {
        job_status_json(
            self.client
                .cancel_job(CancelJobRequest { id: input.job_id })
                .await,
        )
    }

    #[tool(
        name = "burp_job_result",
        description = "Get a bounded page of results from a Burp background job"
    )]
    async fn job_result(&self, Parameters(input): Parameters<JobResultInput>) -> String {
        let limit = input.limit.unwrap_or(100);
        if limit > MAX_PAGE_SIZE {
            return serde_json::json!({"error": "limit must be at most 500"}).to_string();
        }
        match self
            .client
            .get_job_result(GetJobResultRequest {
                id: input.job_id,
                page: Some(PageRequest {
                    limit,
                    cursor: input.cursor.unwrap_or_default(),
                }),
            })
            .await
        {
            Ok(result) => {
                let page = result.page.unwrap_or_default();
                serde_json::json!({
                    "job_id": result.id, "operation": result.operation, "state": result.state,
                    "items": result.items.into_iter().map(|item| serde_json::json!({"label": item.label, "status": item.status, "length": item.length, "error": item.error})).collect::<Vec<_>>(),
                    "total": page.total, "truncated": page.truncated,
                    "next_cursor": (!page.next_cursor.is_empty()).then_some(page.next_cursor),
                    "unique_lengths": result.unique_lengths, "verdict": result.verdict,
                    "request_count": result.request_count, "error_count": result.error_count, "error": result.error,
                }).to_string()
            }
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_convert_request",
        description = "Convert HTTP request method (e.g. GET to POST)"
    )]
    async fn convert_request(&self, Parameters(input): Parameters<ConvertRequestInput>) -> String {
        match convert_request_text(
            &input.request,
            input.convert_to.as_deref().unwrap_or("POST"),
        ) {
            Ok(request) => serde_json::json!({"request": request}).to_string(),
            Err(error) => serde_json::json!({"error": error}).to_string(),
        }
    }

    #[tool(
        name = "burp_export_request",
        description = "Export a request as curl or Python requests code"
    )]
    async fn export_request(&self, Parameters(input): Parameters<ExportRequestInput>) -> String {
        match export_request_text(input) {
            Ok(command) => serde_json::json!({"command": command}).to_string(),
            Err(error) => serde_json::json!({"error": error}).to_string(),
        }
    }

    #[tool(
        name = "burp_extract_from_response",
        description = "Extract data from a response using regex"
    )]
    async fn extract_from_response(
        &self,
        Parameters(input): Parameters<ExtractResponseInput>,
    ) -> String {
        let limit = input.limit.unwrap_or(100);
        if limit > 500 {
            return serde_json::json!({"error": "limit must be at most 500"}).to_string();
        }
        match self
            .client
            .proxy_detail(ProxyDetailRequest { index: input.index })
            .await
        {
            Ok(detail) if !detail.response.is_empty() => {
                let response = String::from_utf8_lossy(&detail.response);
                match regex::Regex::new(&input.regex) {
                    Ok(pattern) => serde_json::json!({"matches": pattern.find_iter(&response).take(limit).map(|found| found.as_str()).collect::<Vec<_>>()}).to_string(),
                    Err(error) => serde_json::json!({"error": format!("invalid regex: {error}")}).to_string(),
                }
            }
            Ok(_) => serde_json::json!({"matches": []}).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_scan_issues",
        description = "Get a bounded page of Burp Scanner issues"
    )]
    async fn scan_issues(&self, Parameters(input): Parameters<ScanIssuesInput>) -> String {
        let limit = input.limit.unwrap_or(100);
        if limit > MAX_PAGE_SIZE {
            return serde_json::json!({"error": "limit must be at most 500"}).to_string();
        }
        match self
            .client
            .scan_issues(ScanIssuesRequest {
                page: Some(PageRequest {
                    limit,
                    cursor: input.cursor.unwrap_or_default(),
                }),
            })
            .await
        {
            Ok(response) => {
                let page = response.page.unwrap_or_default();
                serde_json::to_string(&ScanIssuesOutput {
                    items: response
                        .items
                        .into_iter()
                        .map(|item| ScanIssueOutput {
                            index: item.index,
                            name: item.name,
                            severity: item.severity,
                            confidence: item.confidence,
                            url: item.url,
                            detail: item.detail,
                        })
                        .collect(),
                    total: page.total,
                    truncated: page.truncated,
                    next_cursor: (!page.next_cursor.is_empty()).then_some(page.next_cursor),
                })
                .expect("scan issues output must serialize")
            }
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_scan_issue_detail",
        description = "Get complete details for one Burp scanner issue index"
    )]
    async fn scan_issue_detail(
        &self,
        Parameters(input): Parameters<ScanIssueDetailInput>,
    ) -> String {
        match self
            .client
            .scan_issue_detail(ScanIssueDetailRequest { index: input.index })
            .await
        {
            Ok(item) => serde_json::to_string(&ScanIssueOutput {
                index: item.index,
                name: item.name,
                severity: item.severity,
                confidence: item.confidence,
                url: item.url,
                detail: item.detail,
            })
            .expect("scan issue detail must serialize"),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_add_issue",
        description = "Add one typed issue to the Burp site map"
    )]
    async fn add_issue(&self, Parameters(input): Parameters<AddIssueInput>) -> String {
        action_json(
            self.client
                .add_issue(AddIssueRequest {
                    name: input.name,
                    url: input.url,
                    detail: input.detail.unwrap_or_default(),
                    remediation: input.remediation.unwrap_or_default(),
                    severity: input.severity.unwrap_or_else(|| "INFORMATION".to_owned()),
                    confidence: input.confidence.unwrap_or_else(|| "TENTATIVE".to_owned()),
                })
                .await,
        )
    }

    #[tool(
        name = "decoder",
        description = "One bounded offline decoder tool with many operations. Supply operation for one transform, steps for a recipe, query to search the operation catalog, describe for one operation's metadata, or magic=true for deterministic decode suggestions."
    )]
    async fn decoder(&self, Parameters(input): Parameters<DecoderInput>) -> String {
        decoder_json(input)
    }
    #[tool(
        name = "sitegraph_sync",
        description = "Synchronize bounded Burp sitemap metadata into the local SQLite graph"
    )]
    async fn sitegraph_sync(&self, Parameters(input): Parameters<SiteGraphSyncInput>) -> String {
        let prefix = input.url_prefix.unwrap_or_default();
        let mut cursor = String::new();
        let mut sitemap = Vec::new();
        loop {
            let response = match self
                .client
                .sitemap_snapshot(burp_protocol::proto::SitemapSnapshotRequest {
                    url_prefix: prefix.clone(),
                    page: Some(PageRequest { limit: 500, cursor }),
                })
                .await
            {
                Ok(response) => response,
                Err(error) => return serde_json::json!({"error": error.to_string()}).to_string(),
            };
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
            if !page.truncated || page.next_cursor.is_empty() || sitemap.len() >= 10_000 {
                break;
            }
            cursor = page.next_cursor;
        }
        let issues = self
            .client
            .scan_issues(ScanIssuesRequest {
                page: Some(PageRequest {
                    limit: 500,
                    cursor: String::new(),
                }),
            })
            .await
            .map(|response| {
                response
                    .items
                    .into_iter()
                    .map(|issue| IssueObservation {
                        name: issue.name,
                        severity: issue.severity,
                        confidence: issue.confidence,
                        url: issue.url,
                    })
                    .collect()
            })
            .unwrap_or_default();
        match self
            .sitegraph
            .sync(&SyncBatch {
                sitemap,
                issues,
                ..SyncBatch::default()
            })
            .await
        {
            Ok(result) => serde_json::to_string(&result).expect("sync result serializes"),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "sitegraph_search",
        description = "Search normalized sitegraph endpoints with bounded pagination"
    )]
    async fn sitegraph_search(
        &self,
        Parameters(input): Parameters<SiteGraphSearchInput>,
    ) -> String {
        let limit = input.limit.unwrap_or(100);
        if limit == 0 || limit > 500 {
            return serde_json::json!({"error": "limit must be between 1 and 500"}).to_string();
        }
        match self
            .sitegraph
            .search(&input.query, input.cursor.unwrap_or(0) as u64, limit as u64)
            .await
        {
            Ok(result) => serde_json::to_string(&result).expect("graph page serializes"),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "sitegraph_endpoint_detail",
        description = "Get one normalized sitegraph endpoint"
    )]
    async fn sitegraph_endpoint_detail(
        &self,
        Parameters(input): Parameters<SiteGraphEndpointInput>,
    ) -> String {
        match self.sitegraph.endpoint(&input.id).await {
            Ok(Some(endpoint)) => serde_json::to_string(&endpoint).expect("endpoint serializes"),
            Ok(None) => serde_json::json!({"error": "endpoint not found"}).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "sitegraph_status",
        description = "Get local sitegraph synchronization and schema status"
    )]
    async fn sitegraph_status(&self) -> String {
        match self.sitegraph.status().await {
            Ok(status) => serde_json::to_string(&status).expect("graph status serializes"),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "sitegraph_stats",
        description = "Get bounded local sitegraph node and edge statistics"
    )]
    async fn sitegraph_stats(&self) -> String {
        match self.sitegraph.status().await {
            Ok(status) => serde_json::json!({
                "total_nodes": status.total_nodes,
                "total_edges": status.total_edges,
                "last_synced_at": status.last_synced_at,
                "schema_version": status.schema_version,
            })
            .to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "sitegraph_neighbors",
        description = "List a bounded deterministic page of adjacent graph nodes"
    )]
    async fn sitegraph_neighbors(
        &self,
        Parameters(input): Parameters<SiteGraphNeighborsInput>,
    ) -> String {
        match self
            .sitegraph
            .neighbors(
                &input.id,
                input.cursor.unwrap_or(0) as u64,
                input.limit.unwrap_or(100) as u64,
            )
            .await
        {
            Ok(page) => serde_json::to_string(&page).expect("neighbor page serializes"),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "sitegraph_trace",
        description = "Trace bounded graph relationships using recursive SQLite traversal"
    )]
    async fn sitegraph_trace(&self, Parameters(input): Parameters<SiteGraphTraceInput>) -> String {
        match self
            .sitegraph
            .trace(
                &input.id,
                input.max_depth.unwrap_or(4),
                input.limit.unwrap_or(100),
            )
            .await
        {
            Ok(page) => serde_json::to_string(&page).expect("trace page serializes"),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "sitegraph_diff",
        description = "List a bounded deterministic page of graph nodes changed since a timestamp"
    )]
    async fn sitegraph_diff(&self, Parameters(input): Parameters<SiteGraphDiffInput>) -> String {
        match self
            .sitegraph
            .diff(
                input.since,
                input.cursor.unwrap_or(0) as u64,
                input.limit.unwrap_or(100) as u64,
            )
            .await
        {
            Ok(diff) => serde_json::to_string(&diff).expect("graph diff serializes"),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "sitegraph_export",
        description = "Export a bounded metadata-only graph page as JSON or CSV"
    )]
    async fn sitegraph_export(
        &self,
        Parameters(input): Parameters<SiteGraphExportInput>,
    ) -> String {
        let cursor = input.cursor.unwrap_or(0) as u64;
        let limit = input.limit.unwrap_or(100) as u64;
        match input.format.as_deref().unwrap_or("json") {
            "json" => match self.sitegraph.export_json(cursor, limit).await {
                Ok(export) => serde_json::to_string(&export).expect("JSON graph export serializes"),
                Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
            },
            "csv" => match self.sitegraph.export_csv(cursor, limit).await {
                Ok(export) => serde_json::to_string(&export).expect("CSV graph export serializes"),
                Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
            },
            _ => serde_json::json!({"error": "format must be json or csv"}).to_string(),
        }
    }

    #[tool(
        name = "burp_cookie_jar",
        description = "List cookies in Burp cookie jar with name, value, domain, path, and expiration (optional domain filter)"
    )]
    async fn cookie_jar(&self, Parameters(input): Parameters<CookieJarInput>) -> String {
        let limit = input.limit.unwrap_or(100);
        if limit > MAX_PAGE_SIZE {
            return serde_json::json!({"error": "limit must be at most 500"}).to_string();
        }
        match self
            .client
            .cookie_jar(CookieJarRequest {
                domain: input.domain.unwrap_or_default(),
                limit,
            })
            .await
        {
            Ok(response) => serde_json::to_string(
                &response
                    .items
                    .into_iter()
                    .map(|cookie| CookieOutput {
                        name: cookie.name,
                        value: cookie.value,
                        domain: (!cookie.domain.is_empty()).then_some(cookie.domain),
                        path: (!cookie.path.is_empty()).then_some(cookie.path),
                        expiration: (!cookie.expiration.is_empty()).then_some(cookie.expiration),
                    })
                    .collect::<Vec<_>>(),
            )
            .expect("cookie output must serialize"),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }
    #[tool(
        name = "burp_burp_version",
        description = "Get Burp Suite version information"
    )]
    async fn burp_version(&self) -> String {
        match self
            .client
            .server_info(burp_protocol::proto::ServerInfoRequest {})
            .await
        {
            Ok(info) => serde_json::to_string(&ServerInfoOutput {
                extension: info.extension,
                version: info.version,
                burp_name: info.burp_name,
                burp_version: info.burp_version,
                burp_edition: info.burp_edition,
                burp_build_number: info.burp_build_number,
                capabilities: info.capabilities,
                max_message_bytes: info.max_message_bytes,
                max_page_size: info.max_page_size,
                max_concurrent_calls_per_connection: info.max_concurrent_calls_per_connection,
                max_rpc_timeout_seconds: info.max_rpc_timeout_seconds,
                max_response_bytes: info.max_response_bytes,
            })
            .expect("Burp version output must serialize"),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_cookie_jar_set",
        description = "Set one cookie in Burp's cookie jar"
    )]
    async fn set_cookie(&self, Parameters(input): Parameters<SetCookieInput>) -> String {
        action_json(
            self.client
                .set_cookie(SetCookieRequest {
                    name: input.name,
                    value: input.value,
                    domain: input.domain,
                    path: input.path.unwrap_or_else(|| "/".to_owned()),
                    expiration: input.expiration.unwrap_or_default(),
                })
                .await,
        )
    }

    #[tool(
        name = "burp_intercept_state",
        description = "Read or set Burp Proxy interception state"
    )]
    async fn intercept_state(&self, Parameters(input): Parameters<InterceptStateInput>) -> String {
        match self
            .client
            .intercept_state(InterceptStateRequest {
                enabled: input.enabled,
            })
            .await
        {
            Ok(response) => serde_json::json!({"enabled": response.enabled}).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_proxy_websocket_history",
        description = "Get a bounded page of Burp Proxy WebSocket history"
    )]
    async fn proxy_websocket_history(
        &self,
        Parameters(input): Parameters<ProxyWebSocketHistoryInput>,
    ) -> String {
        let limit = input.limit.unwrap_or(50).min(MAX_PAGE_SIZE);
        match self.client.proxy_websocket_history(ProxyWebSocketHistoryRequest {
            page: Some(PageRequest { limit, cursor: input.cursor.unwrap_or_default() }),
        }).await {
            Ok(response) => serde_json::json!({
                "items": response.items.into_iter().map(|item| serde_json::json!({
                    "index": item.index,
                    "id": item.id,
                    "websocket_id": item.web_socket_id,
                    "direction": item.direction,
                    "payload_base64": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, item.payload),
                    "edited_payload_base64": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, item.edited_payload),
                    "time": item.time,
                    "listener_port": item.listener_port,
                    "upgrade_url": item.upgrade_url,
                })).collect::<Vec<_>>(),
                "page": response.page.map(|page| serde_json::json!({"total": page.total, "truncated": page.truncated, "next_cursor": page.next_cursor})),
            }).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_send_to_intruder",
        description = "Open one request in Burp Intruder"
    )]
    async fn send_to_intruder(&self, Parameters(input): Parameters<SendToIntruderInput>) -> String {
        action_json(
            self.client
                .send_to_intruder(SendToIntruderRequest {
                    request: input.request.into_bytes(),
                    host: input.host,
                    port: input.port.unwrap_or(80),
                    https: input.https.unwrap_or(false),
                    tab_name: input.tab_name.unwrap_or_default(),
                })
                .await,
        )
    }

    #[tool(
        name = "burp_extension_info",
        description = "Get current Burp extension and process configuration metadata"
    )]
    async fn extension_info(&self) -> String {
        match self.client.extension_info(ExtensionInfoRequest {}).await {
            Ok(response) => serde_json::json!({
                "filename": response.filename,
                "is_bapp": response.is_bapp,
                "command_line_arguments": response.command_line_arguments,
            })
            .to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ScanIssuesInput {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

fn action_json(
    result: Result<burp_protocol::proto::ActionResponse, burp_protocol::ClientError>,
) -> String {
    match result {
        Ok(response) => {
            serde_json::json!({"success": response.success, "message": response.message})
                .to_string()
        }
        Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
    }
}
fn script_import_json(
    result: Result<burp_protocol::proto::ScriptImportResponse, burp_protocol::ClientError>,
) -> String {
    match result {
        Ok(response) => serde_json::json!({
            "success": response.success,
            "status": response.status,
            "errors": response.errors,
        })
        .to_string(),
        Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
    }
}
fn utility_value(input: UtilityValueInput) -> Result<DataValue, String> {
    match input {
        UtilityValueInput::Text { value } => Ok(DataValue::Text(value)),
        UtilityValueInput::Bytes { base64 } => {
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, base64)
                .map(DataValue::Bytes)
                .map_err(|error| format!("invalid base64 input: {error}"))
        }
        UtilityValueInput::Json { value } => Ok(DataValue::Json(value)),
    }
}

fn decoder_json(input: DecoderInput) -> String {
    if let Some(query) = input.query.as_deref() {
        return serde_json::to_string(&utility::search(query))
            .expect("decoder operation registry must serialize");
    }
    if let Some(operation) = input.describe.as_deref() {
        return match utility::describe(operation) {
            Some(operation) => serde_json::to_string(&operation)
                .expect("decoder operation metadata must serialize"),
            None => serde_json::json!({"error": "operation not found"}).to_string(),
        };
    }
    let value = match utility_value(input.input) {
        Ok(value) => value,
        Err(error) => return serde_json::json!({"error": error}).to_string(),
    };
    if input.magic {
        return serde_json::json!({"suggestions": utility::magic(&value)}).to_string();
    }
    let result = match (input.operation, input.steps.is_empty()) {
        (Some(operation), true) => utility::run(&operation, value, &input.args),
        (None, false) => {
            let steps = input
                .steps
                .into_iter()
                .map(|step| (step.op, step.args))
                .collect::<Vec<_>>();
            utility::run_recipe(value, &steps)
        }
        (Some(_), false) => Err("provide either operation or steps, not both".to_owned()),
        (None, true) => {
            Err("provide an operation, steps, query, describe, or magic mode".to_owned())
        }
    };
    decoder_result_json(result)
}

fn decoder_result_json(result: Result<DataValue, String>) -> String {
    match result {
        Ok(value) => utility_value_json(value).to_string(),
        Err(error) => serde_json::json!({"error": error}).to_string(),
    }
}

fn utility_value_json(value: DataValue) -> serde_json::Value {
    match value {
        DataValue::Text(value) => serde_json::json!({"kind": "text", "value": value}),
        DataValue::Bytes(value) => serde_json::json!({
            "kind": "bytes",
            "base64": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, value),
        }),
        DataValue::Json(value) => serde_json::json!({"kind": "json", "value": value}),
    }
}

fn to_proto_request(input: SendRequestInput) -> SendRequestRequest {
    SendRequestRequest {
        method: input.method.unwrap_or_else(|| "GET".to_owned()),
        url: input.url,
        body: input.body.unwrap_or_default().into_bytes(),
        headers: input
            .headers
            .unwrap_or_default()
            .into_iter()
            .map(|(name, value)| HttpHeaderEntry { name, value })
            .collect(),
    }
}

fn job_status_json(
    result: Result<burp_protocol::proto::JobStatusResponse, burp_protocol::ClientError>,
) -> String {
    match result {
        Ok(status) => serde_json::json!({
            "job_id": status.id,
            "operation": status.operation,
            "state": status.state,
            "error": (!status.error.is_empty()).then_some(status.error),
        })
        .to_string(),
        Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
    }
}
fn to_send_output(response: burp_protocol::proto::SendRequestResponse) -> SendResponseOutput {
    SendResponseOutput {
        request: String::from_utf8_lossy(&response.request).into_owned(),
        response: response
            .has_response
            .then(|| String::from_utf8_lossy(&response.response).into_owned()),
        status: response.has_response.then_some(response.status),
    }
}

fn convert_request_text(request: &str, target_method: &str) -> Result<String, String> {
    let newline = if request.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let (head, body) = request
        .split_once(&format!("{newline}{newline}"))
        .unwrap_or((request, ""));
    let mut lines = head.split(newline);
    let first = lines.next().ok_or_else(|| "request is empty".to_owned())?;
    let mut parts = first.split_whitespace();
    parts
        .next()
        .ok_or_else(|| "request line has no method".to_owned())?;
    let target = parts
        .next()
        .ok_or_else(|| "request line has no target".to_owned())?;
    let version = parts.next().unwrap_or("HTTP/1.1");
    let target_method = target_method.to_ascii_uppercase();
    let (converted_target, converted_body) = if target_method == "POST" && body.is_empty() {
        target.split_once('?').map_or_else(
            || (target.to_owned(), String::new()),
            |(path, query)| (path.to_owned(), query.to_owned()),
        )
    } else {
        (target.to_owned(), body.to_owned())
    };
    let mut output = format!("{target_method} {converted_target} {version}");
    for line in lines.filter(|line| !line.to_ascii_lowercase().starts_with("content-length:")) {
        output.push_str(newline);
        output.push_str(line);
    }
    if !converted_body.is_empty() {
        output.push_str(newline);
        output.push_str(&format!("Content-Length: {}", converted_body.len()));
    }
    output.push_str(newline);
    output.push_str(newline);
    output.push_str(&converted_body);
    Ok(output)
}

fn export_request_text(input: ExportRequestInput) -> Result<String, String> {
    let request = input.request.replace("\r\n", "\n");
    let (head, body) = request.split_once("\n\n").unwrap_or((&request, ""));
    let mut lines = head.lines();
    let first = lines.next().ok_or_else(|| "request is empty".to_owned())?;
    let mut parts = first.split_whitespace();
    let method = parts
        .next()
        .ok_or_else(|| "request line has no method".to_owned())?;
    let target = parts
        .next()
        .ok_or_else(|| "request line has no target".to_owned())?;
    let host = input
        .host
        .or_else(|| {
            lines
                .clone()
                .find_map(|line| line.strip_prefix("Host: ").map(str::to_owned))
        })
        .ok_or_else(|| "host is required when the request has no Host header".to_owned())?;
    let scheme = if input.https.unwrap_or(true) {
        "https"
    } else {
        "http"
    };
    let url = if target.starts_with("http://") || target.starts_with("https://") {
        target.to_owned()
    } else {
        format!("{scheme}://{host}{target}")
    };
    let format = input
        .format
        .unwrap_or_else(|| "curl".to_owned())
        .to_ascii_lowercase();
    if format == "python" {
        return Ok(format!(
            "requests.request({method:?}, {url:?}, data={body:?})"
        ));
    }
    if format != "curl" {
        return Err("format must be curl or python".to_owned());
    }
    let mut command = format!("curl -X {} {}", shell_quote(method), shell_quote(&url));
    for line in lines.filter(|line| !line.to_ascii_lowercase().starts_with("content-length:")) {
        if !line.is_empty() {
            command.push_str(" -H ");
            command.push_str(&shell_quote(line));
        }
    }
    if !body.is_empty() {
        command.push_str(" --data-raw ");
        command.push_str(&shell_quote(body));
    }
    Ok(command)
}
fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
#[cfg(test)]
mod contract_tests {
    use super::BurpTools;
    use serde::Deserialize;
    use serde_json::Value;
    use std::collections::BTreeSet;

    #[derive(Deserialize)]
    struct ParityManifest {
        statuses: BTreeSet<String>,
        tools: Vec<ParityEntry>,
    }

    #[derive(Deserialize)]
    struct ParityEntry {
        tool: String,
        status: String,
        local: Option<String>,
        reason: Option<String>,
    }

    #[test]
    fn native_burp_tools_match_the_v3_ported_contract() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../test-fixtures/contracts/burp-tools-v2.json"
        ))
        .expect("Burp contract fixture must be valid JSON");
        let legacy: BTreeSet<&str> = fixture["tools"]
            .as_array()
            .expect("Burp contract must contain tools")
            .iter()
            .filter_map(|tool| tool["name"].as_str())
            .collect();
        let actual = actual_tool_names();
        let parity: ParityManifest = serde_json::from_str(include_str!(
            "../../../test-fixtures/contracts/reference-tool-parity.json"
        ))
        .expect("reference parity manifest must be valid JSON");
        let classified_local = parity
            .tools
            .iter()
            .filter_map(|entry| entry.local.as_deref())
            .collect::<BTreeSet<_>>();
        assert!(actual.iter().all(|name| {
            legacy.contains(name.as_str())
                || classified_local.contains(name.as_str())
                || matches!(
                    name.as_str(),
                    "burp_job_cancel"
                        | "burp_job_result"
                        | "burp_job_status"
                        | "burp_scan_issues"
                        | "decoder"
                )
                || name.starts_with("sitegraph_")
        }));
    }

    #[test]
    fn every_reference_tool_is_classified_and_every_claimed_local_tool_exists() {
        let manifest: ParityManifest = serde_json::from_str(include_str!(
            "../../../test-fixtures/contracts/reference-tool-parity.json"
        ))
        .expect("reference parity manifest must be valid JSON");
        let expected_statuses = BTreeSet::from([
            "implemented".to_owned(),
            "intentionally_removed".to_owned(),
            "replaced_by".to_owned(),
        ]);
        assert_eq!(manifest.statuses, expected_statuses);

        let actual = actual_tool_names();
        let mut reference_names = BTreeSet::new();
        for entry in manifest.tools {
            assert!(
                reference_names.insert(entry.tool.clone()),
                "duplicate reference tool {}",
                entry.tool
            );
            assert!(
                expected_statuses.contains(&entry.status),
                "unclassified reference tool {}",
                entry.tool
            );
            match entry.status.as_str() {
                "implemented" | "replaced_by" => {
                    let local = entry
                        .local
                        .expect("implemented/replaced tool requires local");
                    assert!(
                        actual.contains(&local),
                        "{} claims missing local tool {local}",
                        entry.tool
                    );
                    assert!(
                        entry.reason.is_none(),
                        "{} must not carry removal reason",
                        entry.tool
                    );
                }
                "intentionally_removed" => {
                    assert!(
                        entry.local.is_none(),
                        "removed tool {} must not claim a local tool",
                        entry.tool
                    );
                    assert!(
                        entry
                            .reason
                            .as_deref()
                            .is_some_and(|reason| !reason.trim().is_empty()),
                        "removed tool {} requires a reason",
                        entry.tool
                    );
                }
                _ => unreachable!(),
            }
        }
        assert_eq!(
            reference_names.len(),
            78,
            "reference advertised tool count changed; update the manifest explicitly"
        );
    }

    fn actual_tool_names() -> BTreeSet<String> {
        BurpTools::tool_router()
            .map
            .keys()
            .map(ToString::to_string)
            .collect()
    }
}
