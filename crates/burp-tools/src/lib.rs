mod sitegraph;
mod utility;

use crate::sitegraph::SitegraphIndexer;
use ::sitegraph::SiteGraph;
use burp_protocol::BurpClient;
use burp_protocol::protocol::{
    AddIssueRequest, CancelJobRequest, ClearHttpHandlerRequest, ClearProxyRulesRequest,
    CloseWebSocketRequest, ConfigResponse, CookieJarRequest, CreateMacroRequest,
    CreatePayloadListRequest, CreateWebSocketRequest, DeletePayloadListRequest,
    DeleteScanConfigurationRequest, DeleteSessionRuleRequest,
    DeleteScanResourcePoolRequest, ExportConfigRequest, ExtensionInfoRequest,
    GenerateCollaboratorPayloadsRequest, GenerateScannerReportRequest, GetJobResultRequest,
    GetJobStatusRequest, GetPayloadListRequest, GetScanConfigurationRequest,
    GetScanResourcePoolRequest, GetSessionRuleRequest, HttpHeaderEntry,
    ImportBCheckRequest, ImportBambdaRequest, ImportConfigRequest, ImportPayloadListRequest,
    InterceptStateRequest, ListMacrosRequest, ListPayloadGeneratorsRequest,
    ListPayloadListsRequest, ListPayloadProcessorsRequest, ListProxyRulesRequest,
    ListScanConfigurationsRequest, ListScanResourcePoolsRequest, ListSessionRulesRequest,
    ListWebSocketsRequest, MacroDefinition, MacroItem, MacroParameter,
    ManagedWebSocketHistoryRequest, MutateScopeRequest, PageRequest,
    PollCollaboratorInteractionsRequest, ProxyDetailRequest, ProxyHistoryRequest,
    ProxyInterceptConfigRequest, ProxyInterceptRule, ProxyWebSocketHistoryRequest,
    RegisterHttpHandlerRequest, RegisterPayloadGeneratorRequest, RegisterPayloadProcessorRequest,
    RegisterProxyRuleRequest, RemoveMacroRequest, RemovePayloadGeneratorRequest,
    RemovePayloadProcessorRequest, RunMacroRequest, ScanIssueDetailRequest,
    ScanIssuesRequest, ScopeCheckRequest, SendRequestRequest, SendRequestsRequest,
    SendToIntruderRequest, SendToRepeaterRequest, SendWebSocketBinaryRequest,
    SendWebSocketTextRequest, SetCookieRequest, SetHighlightRequest, SetNoteRequest,
    SitemapSnapshotRequest, StartAuditRequest, StartBoundedInputMatrixRequest,
    StartConcurrentRequestCheckRequest, StartCrawlRequest, TargetInfoRequest,
    UpdatePayloadListRequest, UpsertScanConfigurationRequest, UpsertScanResourcePoolRequest,
    UpsertSessionRuleRequest,
};
use rmcp::handler::server::tool::ToolCallContext;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock};
use rmcp::service::RequestContext;
use rmcp::{RoleServer, schemars, tool, tool_handler, tool_router};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use utility_engine::{self as utility_engine_api, DataValue};

const MAX_PAGE_SIZE: u32 = 500;
const MAX_KOTLIN_INDEX: u32 = i32::MAX as u32;
const MAX_TRAVERSAL_DEPTH: u32 = 8;

fn validated_index(index: u32) -> Result<u32, &'static str> {
    if index > MAX_KOTLIN_INDEX {
        Err("index must be at most 2147483647")
    } else {
        Ok(index)
    }
}

fn validated_graph_limit(limit: Option<u32>) -> Result<u32, &'static str> {
    let limit = limit.unwrap_or(100);
    if (1..=MAX_PAGE_SIZE).contains(&limit) {
        Ok(limit)
    } else {
        Err("limit must be between 1 and 500")
    }
}

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
    pub url_filter: Option<String>,
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
pub struct ProxyInterceptRuleInput {
    pub enabled: Option<bool>,
    pub boolean_operator: Option<String>,
    pub match_type: String,
    pub match_relationship: String,
    pub match_condition: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ProxyInterceptConfigInput {
    pub master_intercept_enabled: Option<bool>,
    pub request_do_intercept: Option<bool>,
    pub request_auto_content_length: Option<bool>,
    pub request_fix_missing_new_lines: Option<bool>,
    pub response_do_intercept: Option<bool>,
    pub response_auto_content_length: Option<bool>,
    pub websocket_client_to_server: Option<bool>,
    pub websocket_server_to_client: Option<bool>,
    pub websocket_in_scope_only: Option<bool>,
    pub request_rules: Option<Vec<ProxyInterceptRuleInput>>,
    pub response_rules: Option<Vec<ProxyInterceptRuleInput>>,
    pub replace_request_rules: Option<bool>,
    pub replace_response_rules: Option<bool>,
    pub response_unhide_hidden_fields: Option<bool>,
    pub response_enable_disabled_fields: Option<bool>,
    pub response_remove_input_length_limits: Option<bool>,
    pub response_remove_javascript_validation: Option<bool>,
    pub response_remove_all_javascript: Option<bool>,
}

impl ProxyInterceptRuleInput {
    fn into_proto(self) -> ProxyInterceptRule {
        ProxyInterceptRule {
            enabled: self.enabled.unwrap_or(true),
            boolean_operator: self.boolean_operator.unwrap_or_else(|| "and".to_owned()),
            match_type: self.match_type,
            match_relationship: self.match_relationship,
            match_condition: self.match_condition.unwrap_or_default(),
        }
    }
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
    burp_build_number: String,
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
pub struct GenerateScannerReportInput {
    pub format: String,
    pub path: String,
    pub issue_indexes: Option<Vec<u32>>,
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
pub struct InspectConfigInput {
    pub paths: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RegisterProxyRuleInput {
    pub id: Option<String>,
    pub url_contains: String,
    pub phase: Option<String>,
    pub action: Option<String>,
    #[serde(rename = "match")]
    pub match_text: Option<String>,
    pub replace: Option<String>,
    pub header_name: Option<String>,
    pub header_value: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RemoveProxyRuleInput {
    pub id: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RegisterPayloadProcessorInput {
    pub id: String,
    pub display_name: String,
    pub operation: String,
    pub argument: Option<String>,
    pub replacement: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PayloadRegistrationInput {
    pub id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RegisterPayloadGeneratorInput {
    pub id: String,
    pub display_name: String,
    #[serde(default)]
    pub payloads: Vec<String>,
    pub max_output_count: Option<u32>,
    pub payload_list_id: Option<String>,
    pub payload_offset: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreatePayloadListInput {
    pub id: String,
    pub display_name: String,
    pub payloads: Vec<String>,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ImportPayloadListInput {
    pub id: String,
    pub display_name: String,
    pub content: String,
    pub format: Option<String>,
    pub keep_empty: Option<bool>,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PayloadListIdInput {
    pub id: String,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetPayloadListInput {
    pub id: String,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdatePayloadListInput {
    pub id: String,
    pub operation: String,
    pub payloads: Option<Vec<String>>,
    pub index: Option<u32>,
    pub indexes: Option<Vec<u32>>,
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SessionRuleUpsertInput {
    pub id: Option<String>,
    pub description: Option<String>,
    pub action_type: Option<String>,
    pub find: Option<String>,
    pub replace: Option<String>,
    pub header_name: Option<String>,
    pub parameter_name: Option<String>,
    pub macro_description: Option<String>,
    pub url_contains: Option<String>,
    pub tools: Option<Vec<String>>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SessionRuleIdInput { pub id: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MacroParameterInput {
    pub name: String,
    pub original_value: Option<String>,
    pub parameter_handling: Option<String>,
    pub preset_value: Option<String>,
    pub r#type: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MacroItemInput {
    pub request: String,
    pub method: Option<String>,
    pub url: String,
    pub response: Option<String>,
    pub status_code: Option<u32>,
    pub cookies_received: Option<String>,
    pub request_parameters: Option<Vec<MacroParameterInput>>,
    pub custom_parameters: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateMacroInput {
    pub description: String,
    pub serial_number: Option<u64>,
    pub items: Vec<MacroItemInput>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MacroDescriptionInput {
    pub description: String,
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
    pub payload_list_id: Option<String>,
    pub payload_offset: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CrawlInput {
    pub seed_urls: Vec<String>,
    pub scan_configuration_id: Option<String>,
    pub resource_pool_id: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub stable_seconds: Option<u64>,
    pub include_out_of_scope: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AuditInput {
    pub url: String,
    pub audit_type: Option<String>,
    pub scan_configuration_id: Option<String>,
    pub resource_pool_id: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub stable_seconds: Option<u64>,
    pub include_out_of_scope: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ScanConfigurationIdInput { pub id: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ScanConfigurationUpsertInput {
    pub id: Option<String>,
    pub name: String,
    pub scan_type: String,
    pub audit_type: Option<String>,
    pub include_out_of_scope: Option<bool>,
    pub timeout_seconds: Option<u64>,
    pub stable_seconds: Option<u64>,
    pub resource_pool_id: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ScanResourcePoolIdInput { pub id: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ScanResourcePoolUpsertInput {
    pub id: Option<String>,
    pub name: String,
    pub kind: String,
    pub existing_pool_name: Option<String>,
    pub concurrent_request_limit: Option<u32>,
    pub throttle_millis: Option<u64>,
    pub max_retries: Option<u32>,
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
pub struct ManagedWebSocketHistoryInput {
    pub id: Option<String>,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
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
    Text {
        value: String,
    },
    Bytes {
        base64: String,
    },
    Json {
        #[schemars(schema_with = "json_value_schema")]
        value: serde_json::Value,
    },
}

fn json_value_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    schemars::json_schema!({
        "type": ["null", "boolean", "number", "string", "array", "object"]
    })
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
pub struct SiteGraphConfigInput {
    pub mode: Option<String>,
    pub interval_seconds: Option<u64>,
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
pub struct SiteGraphShortestPathInput {
    pub from_id: String,
    pub to_id: String,
    pub max_depth: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SiteGraphClustersInput {
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SiteGraphImpactInput {
    pub id: String,
    pub max_depth: Option<u32>,
    pub limit: Option<u32>,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SiteGraphExportInput {
    pub profile: Option<String>,
    pub format: Option<String>,
    pub snapshot_id: Option<String>,
    pub cursor: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Clone)]
pub struct BurpTools {
    client: BurpClient,
    sitegraph: Arc<SiteGraph>,
    sitegraph_indexer: SitegraphIndexer,
    auto_index_shutdown: Arc<tokio::sync::watch::Sender<bool>>,
    auto_index_task: Arc<tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

#[tool_router(router = burp_router)]
impl BurpTools {
    pub async fn new(client: BurpClient, graph_path: &Path) -> Result<Self, String> {
        let identity = client
            .server_info(burp_protocol::protocol::ServerInfoRequest {})
            .await
            .ok();
        let (resolved_path, graph_id) = match identity {
            Some(info) if !info.graph_id.is_empty() => {
                let root = if graph_path.extension().is_some() {
                    graph_path.parent().unwrap_or_else(|| Path::new("."))
                } else {
                    graph_path
                };
                let file_name = if info.project_temporary {
                    format!("temp-{}.sqlite", info.graph_id)
                } else {
                    format!("{}.sqlite", info.graph_id)
                };
                (root.join("projects").join(file_name), info.graph_id)
            }
            _ if graph_path.extension().is_some() => (
                graph_path.to_path_buf(),
                graph_path
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .unwrap_or("offline")
                    .to_owned(),
            ),
            _ => {
                return Err(
                    "project identity unavailable; refusing to open a shared fallback graph"
                        .to_owned(),
                );
            }
        };
        let sitegraph = Arc::new(
            SiteGraph::open_with_id(&resolved_path, graph_id)
                .await
                .map_err(|error| error.to_string())?,
        );
        let sitegraph_indexer = SitegraphIndexer::spawn(client.clone(), Arc::clone(&sitegraph));
        let (auto_index_shutdown, _) = tokio::sync::watch::channel(false);
        Ok(Self {
            client,
            sitegraph,
            sitegraph_indexer,
            auto_index_shutdown: Arc::new(auto_index_shutdown),
            auto_index_task: Arc::new(tokio::sync::Mutex::new(None)),
        })
    }

    pub async fn start_auto_index(
        &self,
        mode: &str,
        interval: std::time::Duration,
    ) -> Result<(), String> {
        match mode {
            "off" => Ok(()),
            "startup" => {
                self.sitegraph_indexer.sync(String::new()).await?;
                Ok(())
            }
            "watch" => {
                let indexer = self.sitegraph_indexer.clone();
                let mut shutdown = self.auto_index_shutdown.subscribe();
                let task = tokio::spawn(async move {
                    loop {
                        tokio::select! {
                            _ = shutdown.changed() => break,
                            result = indexer.sync(String::new()) => {
                                let _ = result;
                            }
                        }
                        tokio::select! {
                            _ = shutdown.changed() => break,
                            _ = tokio::time::sleep(interval) => {}
                        }
                    }
                });
                *self.auto_index_task.lock().await = Some(task);
                Ok(())
            }
            _ => Err(format!("unsupported sitegraph mode: {mode}")),
        }
    }

    pub async fn shutdown(&self) {
        let _ = self.auto_index_shutdown.send(true);
        if let Some(task) = self.auto_index_task.lock().await.take() {
            let _ = tokio::time::timeout(std::time::Duration::from_secs(5), task).await;
        }
        self.sitegraph_indexer.shutdown().await;
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
        match self.client.proxy_history(burp_protocol::protocol::ProxyHistoryRequest {
            page: Some(burp_protocol::protocol::PageRequest {
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
        description = "Filter proxy history by URL, annotation color, or notes"
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
            .proxy_history(to_filtered_proxy_history_request(input, limit))
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
        let index = match validated_index(input.index) {
            Ok(index) => index,
            Err(error) => return serde_json::json!({"error": error}).to_string(),
        };
        match self.client.proxy_detail(ProxyDetailRequest { index }).await {
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
        let index = match validated_index(input.index) {
            Ok(index) => index,
            Err(error) => return serde_json::json!({"error": error}).to_string(),
        };
        match self
            .client
            .set_highlight(SetHighlightRequest {
                index,
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
        let index = match validated_index(input.index) {
            Ok(index) => index,
            Err(error) => return serde_json::json!({"error": error}).to_string(),
        };
        match self
            .client
            .set_note(SetNoteRequest {
                index,
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
        description = "Import validated, size-bounded Burp project configuration JSON"
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
        name = "burp_inspect_config",
        description = "Export scoped Burp project options and return discovered leaf paths and UTF-8 size"
    )]
    async fn inspect_config(&self, Parameters(input): Parameters<InspectConfigInput>) -> String {
        match self
            .client
            .inspect_config(ExportConfigRequest {
                paths: input.paths.unwrap_or_default(),
            })
            .await
        {
            Ok(response) => serde_json::json!({
                "config": response.config,
                "paths": response.paths,
                "size_bytes": response.size_bytes,
            })
            .to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_register_http_handler",
        description = "Register a bounded HTTP request handler rule"
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
        description = "Register a request or response Proxy rule: forward, intercept, drop, or edit"
    )]
    async fn register_proxy_rule(
        &self,
        Parameters(input): Parameters<RegisterProxyRuleInput>,
    ) -> String {
        let action = input.action.unwrap_or_else(|| "forward".to_owned());
        action_json(
            self.client
                .register_proxy_rule(RegisterProxyRuleRequest {
                    id: input.id.unwrap_or_else(|| "default".to_owned()),
                    url_contains: input.url_contains,
                    phase: input.phase.unwrap_or_else(|| "request".to_owned()),
                    action,
                    r#match: input.match_text.unwrap_or_default(),
                    replacement: input.replace.unwrap_or_default(),
                    header_name: input.header_name.unwrap_or_default(),
                    header_value: input.header_value.unwrap_or_default(),
                    enabled: input.enabled.unwrap_or(true),
                })
                .await,
        )
    }

    #[tool(
        name = "burp_list_proxy_rules",
        description = "List configured Proxy request and response rules"
    )]
    async fn list_proxy_rules(&self) -> String {
        match self.client.list_proxy_rules(ListProxyRulesRequest {}).await {
            Ok(response) => serde_json::json!({"rules": response.items.into_iter().map(|rule| serde_json::json!({
                "id": rule.id,
                "url_contains": rule.url_contains,
                "phase": rule.phase,
                "action": rule.action,
                "match": rule.r#match,
                "replace": rule.replacement,
                "header_name": rule.header_name,
                "header_value": rule.header_value,
                "enabled": rule.enabled,
            })).collect::<Vec<_>>()}).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_remove_proxy_rule",
        description = "Remove one Proxy rule or clear all Proxy rules"
    )]
    async fn remove_proxy_rule(
        &self,
        Parameters(input): Parameters<RemoveProxyRuleInput>,
    ) -> String {
        action_json(
            self.client
                .clear_proxy_rules(ClearProxyRulesRequest {
                    id: input.id.unwrap_or_default(),
                })
                .await,
        )
    }

    #[tool(
        name = "burp_session_create_rule",
        description = "Create a scoped MCP session rule and return its stable ID"
    )]
    async fn session_create_rule(
        &self,
        Parameters(input): Parameters<SessionRuleUpsertInput>,
    ) -> String {
        match self.client.create_session_rule(session_rule_request(input)).await {
            Ok(rule) => session_rule_json(rule).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(name = "burp_session_get_rule", description = "Get one MCP session rule by ID")]
    async fn session_get_rule(&self, Parameters(input): Parameters<SessionRuleIdInput>) -> String {
        match self.client.get_session_rule(GetSessionRuleRequest { id: input.id }).await {
            Ok(rule) => session_rule_json(rule).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(name = "burp_session_update_rule", description = "Replace one MCP session rule by ID")]
    async fn session_update_rule(
        &self,
        Parameters(input): Parameters<SessionRuleUpsertInput>,
    ) -> String {
        if input.id.as_deref().unwrap_or_default().is_empty() {
            return serde_json::json!({"error": "id is required"}).to_string();
        }
        match self.client.update_session_rule(session_rule_request(input)).await {
            Ok(rule) => session_rule_json(rule).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(name = "burp_session_list_rules", description = "List registered MCP session rules and scope")]
    async fn session_list_rules(&self) -> String {
        match self.client.list_session_rules(ListSessionRulesRequest {}).await {
            Ok(response) => serde_json::json!({"rules": response.items.into_iter().map(session_rule_json).collect::<Vec<_>>() }).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(name = "burp_session_delete_rule", description = "Delete one MCP session rule by ID")]
    async fn session_delete_rule(&self, Parameters(input): Parameters<SessionRuleIdInput>) -> String {
        action_json(self.client.delete_session_rule(DeleteSessionRuleRequest { id: input.id }).await)
    }

    #[tool(
        name = "burp_macro_create",
        description = "Create or replace a Burp Settings > Sessions > Macros definition"
    )]
    async fn macro_create(&self, Parameters(input): Parameters<CreateMacroInput>) -> String {
        action_json(
            self.client
                .create_macro(CreateMacroRequest {
                    r#macro: Some(MacroDefinition {
                        description: input.description,
                        serial_number: input.serial_number.unwrap_or_default(),
                        items: input
                            .items
                            .into_iter()
                            .map(|item| MacroItem {
                                request: item.request,
                                method: item.method.unwrap_or_default(),
                                url: item.url,
                                response: item.response.unwrap_or_default(),
                                status_code: item.status_code.unwrap_or_default(),
                                cookies_received: item.cookies_received.unwrap_or_default(),
                                request_parameters: item
                                    .request_parameters
                                    .unwrap_or_default()
                                    .into_iter()
                                    .map(|parameter| MacroParameter {
                                        name: parameter.name,
                                        original_value: parameter
                                            .original_value
                                            .unwrap_or_default(),
                                        parameter_handling: parameter
                                            .parameter_handling
                                            .unwrap_or_else(|| "preset_value".to_owned()),
                                        preset_value: parameter.preset_value.unwrap_or_default(),
                                        r#type: parameter.r#type.unwrap_or_default(),
                                    })
                                    .collect(),
                                custom_parameters: item.custom_parameters.unwrap_or_default(),
                            })
                            .collect(),
                    }),
                })
                .await,
        )
    }

    #[tool(
        name = "burp_macro_list",
        description = "List Burp Settings > Sessions > Macros definitions"
    )]
    async fn macro_list(&self) -> String {
        match self.client.list_macros(ListMacrosRequest {}).await {
            Ok(response) => serde_json::json!({"macros": response.macros.into_iter().map(macro_json).collect::<Vec<_>>()}).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_macro_run",
        description = "Execute requests from a Burp session macro definition"
    )]
    async fn macro_run(&self, Parameters(input): Parameters<MacroDescriptionInput>) -> String {
        match self.client.run_macro(RunMacroRequest { description: input.description }).await {
            Ok(response) => serde_json::json!({"items": response.items.into_iter().map(|item| serde_json::json!({"request": item.request, "response": item.response, "status_code": item.status_code, "has_response": item.has_response})).collect::<Vec<_>>()}).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_macro_remove",
        description = "Remove a Burp Settings > Sessions > Macros definition"
    )]
    async fn macro_remove(&self, Parameters(input): Parameters<MacroDescriptionInput>) -> String {
        action_json(
            self.client
                .remove_macro(RemoveMacroRequest {
                    description: input.description,
                })
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
        description = "Start a bounded raw HTTP request matrix; template must be a complete raw request containing the nonblank marker"
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
                    payload_list_id: input.payload_list_id.unwrap_or_default(),
                    payload_offset: input.payload_offset.unwrap_or(0),
                })
                .await,
        )
    }
    #[tool(
        name = "burp_intruder_payload_processor_register",
        description = "Register one bounded declarative Intruder payload processor"
    )]
    async fn intruder_payload_processor_register(
        &self,
        Parameters(input): Parameters<RegisterPayloadProcessorInput>,
    ) -> String {
        action_json(
            self.client
                .register_payload_processor(RegisterPayloadProcessorRequest {
                    id: input.id,
                    display_name: input.display_name,
                    operation: input.operation,
                    argument: input.argument.unwrap_or_default(),
                    replacement: input.replacement.unwrap_or_default(),
                })
                .await,
        )
    }

    #[tool(
        name = "burp_intruder_payload_processor_list",
        description = "List registered declarative Intruder payload processors"
    )]
    async fn intruder_payload_processor_list(&self) -> String {
        match self.client.list_payload_processors(ListPayloadProcessorsRequest {}).await {
            Ok(response) => serde_json::json!({"items": response.items.iter().map(|item| serde_json::json!({"id": item.id, "display_name": item.display_name, "operation": item.operation, "registered": item.registered})).collect::<Vec<_>>()}).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_intruder_payload_processor_remove",
        description = "Deregister one declarative Intruder payload processor"
    )]
    async fn intruder_payload_processor_remove(
        &self,
        Parameters(input): Parameters<PayloadRegistrationInput>,
    ) -> String {
        action_json(
            self.client
                .remove_payload_processor(RemovePayloadProcessorRequest { id: input.id })
                .await,
        )
    }

    #[tool(
        name = "burp_intruder_payload_generator_register",
        description = "Register one bounded declarative Intruder payload generator"
    )]
    async fn intruder_payload_generator_register(
        &self,
        Parameters(input): Parameters<RegisterPayloadGeneratorInput>,
    ) -> String {
        action_json(
            self.client
                .register_payload_generator(RegisterPayloadGeneratorRequest {
                    id: input.id,
                    display_name: input.display_name,
                    payloads: input.payloads,
                    max_output_count: input.max_output_count.unwrap_or(0),
                    payload_list_id: input.payload_list_id.unwrap_or_default(),
                    payload_offset: input.payload_offset.unwrap_or(0),
                })
                .await,
        )
    }

    #[tool(
        name = "burp_intruder_payload_generator_list",
        description = "List registered declarative Intruder payload generators"
    )]
    async fn intruder_payload_generator_list(&self) -> String {
        match self.client.list_payload_generators(ListPayloadGeneratorsRequest {}).await {
            Ok(response) => serde_json::json!({"items": response.items.iter().map(|item| serde_json::json!({"id": item.id, "display_name": item.display_name, "payload_count": item.payload_count, "max_output_count": item.max_output_count, "registered": item.registered})).collect::<Vec<_>>()}).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_intruder_payload_generator_remove",
        description = "Deregister one declarative Intruder payload generator"
    )]
    async fn intruder_payload_generator_remove(
        &self,
        Parameters(input): Parameters<PayloadRegistrationInput>,
    ) -> String {
        action_json(
            self.client
                .remove_payload_generator(RemovePayloadGeneratorRequest { id: input.id })
                .await,
        )
    }
    #[tool(
        name = "burp_payload_list_create",
        description = "Create one bounded in-memory payload list"
    )]
    async fn payload_list_create(
        &self,
        Parameters(input): Parameters<CreatePayloadListInput>,
    ) -> String {
        payload_list_entry_json(
            self.client
                .create_payload_list(CreatePayloadListRequest {
                    id: input.id,
                    display_name: input.display_name,
                    payloads: input.payloads,
                })
                .await,
        )
    }

    #[tool(
        name = "burp_payload_list_import",
        description = "Import a bounded payload list from newline text or a JSON string array"
    )]
    async fn payload_list_import(
        &self,
        Parameters(input): Parameters<ImportPayloadListInput>,
    ) -> String {
        payload_list_entry_json(
            self.client
                .import_payload_list(ImportPayloadListRequest {
                    id: input.id,
                    display_name: input.display_name,
                    content: input.content,
                    format: input.format.unwrap_or_else(|| "lines".to_owned()),
                    keep_empty: input.keep_empty.unwrap_or(false),
                })
                .await,
        )
    }
    #[tool(
        name = "burp_payload_list_list",
        description = "List bounded in-memory payload lists"
    )]
    async fn payload_list_list(&self) -> String {
        match self.client.list_payload_lists(ListPayloadListsRequest {}).await {
            Ok(response) => serde_json::json!({"items": response.items.iter().map(payload_list_proto_json).collect::<Vec<_>>()}).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_payload_list_get",
        description = "Read one bounded page from a payload list"
    )]
    async fn payload_list_get(&self, Parameters(input): Parameters<GetPayloadListInput>) -> String {
        match self.client.get_payload_list(GetPayloadListRequest {
            id: input.id,
            page: Some(PageRequest { limit: input.limit.unwrap_or(100).min(MAX_PAGE_SIZE), cursor: input.offset.unwrap_or(0).to_string() }),
        }).await {
            Ok(response) => serde_json::json!({"list": response.list.map(|item| payload_list_proto_json(&item)), "payloads": response.payloads, "page": response.page.map(|page| serde_json::json!({"total": page.total, "truncated": page.truncated, "next_cursor": page.next_cursor}))}).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_payload_list_update",
        description = "Append, prepend, insert, replace, remove, clear, or rename a payload list"
    )]
    async fn payload_list_update(
        &self,
        Parameters(input): Parameters<UpdatePayloadListInput>,
    ) -> String {
        payload_list_entry_json(
            self.client
                .update_payload_list(UpdatePayloadListRequest {
                    id: input.id,
                    operation: input.operation,
                    payloads: input.payloads.unwrap_or_default(),
                    index: input.index.unwrap_or(0),
                    indexes: input.indexes.unwrap_or_default(),
                    display_name: input.display_name,
                })
                .await,
        )
    }

    #[tool(
        name = "burp_payload_list_delete",
        description = "Delete one payload list"
    )]
    async fn payload_list_delete(
        &self,
        Parameters(input): Parameters<PayloadListIdInput>,
    ) -> String {
        action_json(
            self.client
                .delete_payload_list(DeletePayloadListRequest { id: input.id })
                .await,
        )
    }

    #[tool(
        name = "burp_scan_start",
        description = "Start passive stateless audit or active audit with explicit bounded scan options"
    )]
    async fn scan_start(&self, Parameters(input): Parameters<AuditInput>) -> String {
        let audit_type = input.audit_type.as_deref().unwrap_or("passive").to_lowercase();
        if !matches!(audit_type.as_str(), "passive" | "active") {
            return serde_json::json!({"error": "audit_type must be passive or active"}).to_string();
        }
        job_status_json(self.client.start_audit(StartAuditRequest {
            url: input.url,
            audit_type,
            scan_configuration_id: input.scan_configuration_id.unwrap_or_default(),
            resource_pool_id: input.resource_pool_id.unwrap_or_default(),
            timeout_seconds: input.timeout_seconds.unwrap_or_default(),
            stable_seconds: input.stable_seconds.unwrap_or_default(),
            include_out_of_scope: input.include_out_of_scope.unwrap_or(false),
        }).await)
    }
    #[tool(name = "burp_scan_stop", description = "Stop a running active Burp audit by job ID")]
    async fn scan_stop(&self, Parameters(input): Parameters<JobInput>) -> String {
        job_status_json(self.client.stop_audit(CancelJobRequest { id: input.job_id }).await)
    }

    #[tool(name = "burp_scan_config_list", description = "List built-in and project-persisted scan configurations")]
    async fn scan_config_list(&self) -> String {
        match self.client.list_scan_configurations(ListScanConfigurationsRequest {}).await {
            Ok(response) => serde_json::json!({"items": response.items.into_iter().map(scan_configuration_json).collect::<Vec<_>>() }).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(name = "burp_scan_config_get", description = "Get one scan configuration by ID")]
    async fn scan_config_get(&self, Parameters(input): Parameters<ScanConfigurationIdInput>) -> String {
        match self.client.get_scan_configuration(GetScanConfigurationRequest { id: input.id }).await {
            Ok(value) => scan_configuration_json(value).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(name = "burp_scan_config_create", description = "Create a bounded persisted scan configuration")]
    async fn scan_config_create(&self, Parameters(input): Parameters<ScanConfigurationUpsertInput>) -> String {
        match self.client.create_scan_configuration(scan_configuration_request(input)).await {
            Ok(value) => scan_configuration_json(value).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(name = "burp_scan_config_update", description = "Update a persisted scan configuration by ID")]
    async fn scan_config_update(&self, Parameters(input): Parameters<ScanConfigurationUpsertInput>) -> String {
        if input.id.as_deref().unwrap_or_default().is_empty() {
            return serde_json::json!({"error": "id is required"}).to_string();
        }
        match self.client.update_scan_configuration(scan_configuration_request(input)).await {
            Ok(value) => scan_configuration_json(value).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(name = "burp_scan_config_delete", description = "Delete a persisted scan configuration by ID")]
    async fn scan_config_delete(&self, Parameters(input): Parameters<ScanConfigurationIdInput>) -> String {
        action_json(self.client.delete_scan_configuration(DeleteScanConfigurationRequest { id: input.id }).await)
    }

    #[tool(name = "burp_scan_pool_list", description = "List scanner resource pool definitions and runtime support")]
    async fn scan_pool_list(&self) -> String {
        match self.client.list_scan_resource_pools(ListScanResourcePoolsRequest {}).await {
            Ok(response) => serde_json::json!({"items": response.items.into_iter().map(scan_pool_json).collect::<Vec<_>>(), "scanner_supported": response.scanner_supported, "support_message": response.support_message}).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(name = "burp_scan_pool_get", description = "Get one scanner resource pool definition by ID")]
    async fn scan_pool_get(&self, Parameters(input): Parameters<ScanResourcePoolIdInput>) -> String {
        match self.client.get_scan_resource_pool(GetScanResourcePoolRequest { id: input.id }).await {
            Ok(value) => scan_pool_json(value).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(name = "burp_scan_pool_create", description = "Create a persisted scanner resource pool definition")]
    async fn scan_pool_create(&self, Parameters(input): Parameters<ScanResourcePoolUpsertInput>) -> String {
        match self.client.create_scan_resource_pool(scan_pool_request(input)).await {
            Ok(value) => scan_pool_json(value).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(name = "burp_scan_pool_update", description = "Update a persisted scanner resource pool definition")]
    async fn scan_pool_update(&self, Parameters(input): Parameters<ScanResourcePoolUpsertInput>) -> String {
        if input.id.as_deref().unwrap_or_default().is_empty() {
            return serde_json::json!({"error": "id is required"}).to_string();
        }
        match self.client.update_scan_resource_pool(scan_pool_request(input)).await {
            Ok(value) => scan_pool_json(value).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(name = "burp_scan_pool_delete", description = "Delete a persisted scanner resource pool definition")]
    async fn scan_pool_delete(&self, Parameters(input): Parameters<ScanResourcePoolIdInput>) -> String {
        action_json(self.client.delete_scan_resource_pool(DeleteScanResourcePoolRequest { id: input.id }).await)
    }

    #[tool(
        name = "burp_scan_remove",
        description = "Remove a stopped or completed Burp audit by job ID"
    )]
    async fn scan_remove(&self, Parameters(input): Parameters<JobInput>) -> String {
        action_json(
            self.client
                .remove_audit(CancelJobRequest { id: input.job_id })
                .await,
        )
    }

    #[tool(name = "burp_crawl", description = "Start a bounded Burp crawl with explicit seeds, configuration, scope, and timing")]
    async fn crawl(&self, Parameters(input): Parameters<CrawlInput>) -> String {
        job_status_json(self.client.start_crawl(StartCrawlRequest {
            seed_urls: input.seed_urls,
            scan_configuration_id: input.scan_configuration_id.unwrap_or_default(),
            resource_pool_id: input.resource_pool_id.unwrap_or_default(),
            timeout_seconds: input.timeout_seconds.unwrap_or_default(),
            stable_seconds: input.stable_seconds.unwrap_or_default(),
            include_out_of_scope: input.include_out_of_scope.unwrap_or(false),
        }).await)
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
        name = "burp_websocket_history",
        description = "Read messages sent to or received from managed WebSocket connections"
    )]
    async fn websocket_history(
        &self,
        Parameters(input): Parameters<ManagedWebSocketHistoryInput>,
    ) -> String {
        let limit = input.limit.unwrap_or(100);
        if limit > MAX_PAGE_SIZE {
            return serde_json::json!({"error": "limit must be at most 500"}).to_string();
        }
        match self
            .client
            .managed_websocket_history(ManagedWebSocketHistoryRequest {
                id: input.id.unwrap_or_default(),
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
                    "items": response.items.into_iter().map(|item| serde_json::json!({
                        "index": item.index,
                        "websocket_id": item.websocket_id,
                        "direction": item.direction,
                        "type": item.r#type,
                        "payload": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, item.payload),
                    })).collect::<Vec<_>>(),
                    "total": page.total,
                    "truncated": page.truncated,
                    "next_cursor": (!page.next_cursor.is_empty()).then_some(page.next_cursor),
                }).to_string()
            }
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
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
                let page = result.page.clone().unwrap_or_default();
                serde_json::json!({
                    "job_id": result.id,
                    "operation": result.operation,
                    "state": result.state,
                    "scan_type": result.scan_type,
                    "stateless": result.stateless,
                    "status_message": result.status_message,
                    "items": result.items.into_iter().map(|item| serde_json::json!({"label": item.label, "status": item.status, "length": item.length, "error": item.error})).collect::<Vec<_>>(),
                    "total": page.total, "truncated": page.truncated,
                    "next_cursor": (!page.next_cursor.is_empty()).then_some(page.next_cursor),
                    "unique_lengths": result.unique_lengths, "verdict": result.verdict,
                    "request_count": result.request_count, "error_count": result.error_count, "issue_count": result.issue_count, "error": result.error,
                    "substitution_count": result.substitution_count,
                    "request_fingerprint": result.request_fingerprint,
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
        let index = match validated_index(input.index) {
            Ok(index) => index,
            Err(error) => return serde_json::json!({"error": error}).to_string(),
        };
        match self.client.proxy_detail(ProxyDetailRequest { index }).await {
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
        let index = match validated_index(input.index) {
            Ok(index) => index,
            Err(error) => return serde_json::json!({"error": error}).to_string(),
        };
        match self
            .client
            .scan_issue_detail(ScanIssueDetailRequest { index })
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
        name = "burp_scanner_generate_report",
        description = "Generate an HTML or XML Burp Scanner report for selected issue indexes, or all issues when omitted"
    )]
    async fn scanner_generate_report(
        &self,
        Parameters(input): Parameters<GenerateScannerReportInput>,
    ) -> String {
        match self
            .client
            .generate_scanner_report(GenerateScannerReportRequest {
                format: input.format,
                path: input.path,
                issue_indexes: input.issue_indexes.unwrap_or_default(),
            })
            .await
        {
            Ok(report) => serde_json::json!({
                "path": report.path,
                "format": report.format,
                "issue_count": report.issue_count,
                "size_bytes": report.size_bytes,
            })
            .to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "sitegraph_sync",
        description = "Synchronize bounded Burp sitemap metadata into the local SQLite graph"
    )]
    async fn sitegraph_sync(&self, Parameters(input): Parameters<SiteGraphSyncInput>) -> String {
        match self
            .sitegraph_indexer
            .sync(input.url_prefix.unwrap_or_default())
            .await
        {
            Ok(summary) => serde_json::to_string(&summary).expect("sync summary serializes"),
            Err(error) => serde_json::json!({"error": error}).to_string(),
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
        match self.sitegraph_indexer.status().await {
            Ok(status) => serde_json::to_string(&status).expect("graph status serializes"),
            Err(error) => serde_json::json!({"error": error}).to_string(),
        }
    }

    #[tool(
        name = "sitegraph_config",
        description = "Read or validate sitegraph auto-index configuration; mode is off, startup, or watch"
    )]
    async fn sitegraph_config(
        &self,
        Parameters(input): Parameters<SiteGraphConfigInput>,
    ) -> String {
        let mode = input.mode.unwrap_or_else(|| "off".to_owned());
        if !matches!(mode.as_str(), "off" | "startup" | "watch") {
            return serde_json::json!({"error": "mode must be off, startup, or watch"}).to_string();
        }
        let interval_seconds = input.interval_seconds.unwrap_or(30).max(1);
        serde_json::json!({
            "mode": mode,
            "interval_seconds": interval_seconds,
            "page_size": 500,
            "queue_capacity": 32,
            "max_items": null,
            "note": "configuration changes apply on the next process start"
        })
        .to_string()
    }

    #[tool(
        name = "sitegraph_projects",
        description = "List the active project-scoped graph identity"
    )]
    async fn sitegraph_projects(&self) -> String {
        match self.sitegraph.status().await {
            Ok(status) => serde_json::json!({
                "active_graph_id": status.graph_id,
                "items": [{
                    "graph_id": status.graph_id,
                    "state": status.state,
                    "last_success_at": status.last_success_at
                }],
                "total": 1,
                "truncated": false,
                "next_cursor": null
            })
            .to_string(),
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
        let limit = match validated_graph_limit(input.limit) {
            Ok(limit) => limit,
            Err(error) => return serde_json::json!({"error": error}).to_string(),
        };
        match self
            .sitegraph
            .neighbors(&input.id, input.cursor.unwrap_or(0) as u64, limit as u64)
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
        let limit = match validated_graph_limit(input.limit) {
            Ok(limit) => limit,
            Err(error) => return serde_json::json!({"error": error}).to_string(),
        };
        let max_depth = input.max_depth.unwrap_or(4);
        if max_depth == 0 || max_depth > MAX_TRAVERSAL_DEPTH {
            return serde_json::json!({"error": "max_depth must be between 1 and 8"}).to_string();
        }
        match self.sitegraph.trace(&input.id, max_depth, limit).await {
            Ok(page) => serde_json::to_string(&page).expect("trace page serializes"),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "sitegraph_shortest_path",
        description = "Find one bounded directed shortest path in the active project graph"
    )]
    async fn sitegraph_shortest_path(
        &self,
        Parameters(input): Parameters<SiteGraphShortestPathInput>,
    ) -> String {
        let max_depth = input.max_depth.unwrap_or(8);
        if max_depth == 0 || max_depth > 16 {
            return serde_json::json!({"error": "max_depth must be between 1 and 16"}).to_string();
        }
        match self
            .sitegraph
            .shortest_path(&input.from_id, &input.to_id, max_depth as usize)
            .await
        {
            Ok(result) => serde_json::to_string(&result).expect("shortest path serializes"),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "sitegraph_clusters",
        description = "Cluster active project endpoints by origin and first path segment"
    )]
    async fn sitegraph_clusters(
        &self,
        Parameters(input): Parameters<SiteGraphClustersInput>,
    ) -> String {
        let limit = match validated_graph_limit(input.limit) {
            Ok(limit) => limit,
            Err(error) => return serde_json::json!({"error": error}).to_string(),
        };
        match self.sitegraph.endpoint_clusters(limit as usize).await {
            Ok(items) => serde_json::json!({
                "items": items,
                "total": items.len(),
                "truncated": items.len() == limit as usize,
                "next_cursor": null,
            })
            .to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "sitegraph_impact",
        description = "List bounded downstream impact from one active project graph node"
    )]
    async fn sitegraph_impact(
        &self,
        Parameters(input): Parameters<SiteGraphImpactInput>,
    ) -> String {
        let limit = match validated_graph_limit(input.limit) {
            Ok(limit) => limit,
            Err(error) => return serde_json::json!({"error": error}).to_string(),
        };
        let max_depth = input.max_depth.unwrap_or(8);
        if max_depth == 0 || max_depth > 16 {
            return serde_json::json!({"error": "max_depth must be between 1 and 16"}).to_string();
        }
        match self
            .sitegraph
            .impact(&input.id, max_depth as usize, limit as usize)
            .await
        {
            Ok(items) => serde_json::json!({
                "items": items,
                "total": items.len(),
                "truncated": items.len() == limit as usize,
                "next_cursor": null,
            })
            .to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "sitegraph_diff",
        description = "List a bounded deterministic page of graph nodes changed since a timestamp"
    )]
    async fn sitegraph_diff(&self, Parameters(input): Parameters<SiteGraphDiffInput>) -> String {
        let limit = match validated_graph_limit(input.limit) {
            Ok(limit) => limit,
            Err(error) => return serde_json::json!({"error": error}).to_string(),
        };
        match self
            .sitegraph
            .diff(input.since, input.cursor.unwrap_or(0) as u64, limit as u64)
            .await
        {
            Ok(diff) => serde_json::to_string(&diff).expect("graph diff serializes"),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "sitegraph_export",
        description = "Export a bounded sitegraph page using explicit metadata or exact profile"
    )]
    async fn sitegraph_export(
        &self,
        Parameters(input): Parameters<SiteGraphExportInput>,
    ) -> String {
        let profile = input.profile.as_deref().unwrap_or("metadata");
        if !matches!(profile, "metadata" | "exact") {
            return serde_json::json!({"error": "profile must be metadata or exact"}).to_string();
        }
        let format = input.format.as_deref().unwrap_or("json");
        if !matches!(format, "json" | "csv") {
            return serde_json::json!({"error": "format must be json or csv"}).to_string();
        }
        if input.snapshot_id.is_some() && profile == "exact" {
            return serde_json::json!({"error": "exact export snapshot_id must be requested from the active project DB"}).to_string();
        }
        let cursor = input.cursor.unwrap_or(0) as u64;
        let limit = match validated_graph_limit(input.limit) {
            Ok(limit) => limit as u64,
            Err(error) => return serde_json::json!({"error": error}).to_string(),
        };
        match (profile, format) {
            ("metadata", "json") => match self.sitegraph.export_json(cursor, limit).await {
                Ok(export) => serde_json::to_string(&export).expect("JSON graph export serializes"),
                Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
            },
            ("exact", "json") => match self.sitegraph.export_exact_json(cursor, limit).await {
                Ok(export) => {
                    serde_json::to_string(&export).expect("exact graph export serializes")
                }
                Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
            },
            (_, "csv") => match self.sitegraph.export_csv(cursor, limit).await {
                Ok(export) => serde_json::to_string(
                    &serde_json::json!({"profile": profile, "export": export}),
                )
                .unwrap(),
                Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
            },
            _ => unreachable!(),
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
            .server_info(burp_protocol::protocol::ServerInfoRequest {})
            .await
        {
            Ok(info) => serde_json::to_string(&ServerInfoOutput {
                extension: info.extension,
                version: info.version,
                burp_name: info.burp_name,
                burp_version: info.burp_version,
                burp_edition: info.burp_edition,
                burp_build_number: info.burp_build_number.to_string(),
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
        name = "burp_proxy_intercept_config",
        description = "Read or patch Burp Proxy request, response, WebSocket interception filters and response modification settings"
    )]
    async fn proxy_intercept_config(
        &self,
        Parameters(input): Parameters<ProxyInterceptConfigInput>,
    ) -> String {
        let request_rules = input.request_rules.unwrap_or_default();
        let response_rules = input.response_rules.unwrap_or_default();
        let replace_request_rules = input.replace_request_rules.unwrap_or(false);
        let replace_response_rules = input.replace_response_rules.unwrap_or(false);
        match self
            .client
            .proxy_intercept_config(ProxyInterceptConfigRequest {
                master_intercept_enabled: input.master_intercept_enabled,
                request_do_intercept: input.request_do_intercept,
                request_auto_content_length: input.request_auto_content_length,
                response_do_intercept: input.response_do_intercept,
                response_auto_content_length: input.response_auto_content_length,
                websocket_client_to_server: input.websocket_client_to_server,
                websocket_server_to_client: input.websocket_server_to_client,
                websocket_in_scope_only: input.websocket_in_scope_only,
                request_rules: request_rules.into_iter().map(ProxyInterceptRuleInput::into_proto).collect(),
                response_rules: response_rules.into_iter().map(ProxyInterceptRuleInput::into_proto).collect(),
                replace_request_rules,
                replace_response_rules,
                response_unhide_hidden_fields: input.response_unhide_hidden_fields,
                response_enable_disabled_fields: input.response_enable_disabled_fields,
                response_remove_input_length_limits: input.response_remove_input_length_limits,
                response_remove_javascript_validation: input.response_remove_javascript_validation,
                response_remove_all_javascript: input.response_remove_all_javascript,
                request_fix_missing_new_lines: input.request_fix_missing_new_lines,
            })
            .await
        {
            Ok(response) => serde_json::json!({
                "master_intercept_enabled": response.master_intercept_enabled,
                "request": {
                    "do_intercept": response.request_do_intercept,
                    "auto_content_length": response.request_auto_content_length,
                    "fix_missing_new_lines": response.request_fix_missing_new_lines,
                    "rules": response.request_rules.into_iter().map(proxy_intercept_rule_json).collect::<Vec<_>>(),
                },
                "response": {
                    "do_intercept": response.response_do_intercept,
                    "auto_content_length": response.response_auto_content_length,
                    "rules": response.response_rules.into_iter().map(proxy_intercept_rule_json).collect::<Vec<_>>(),
                    "modification": {
                        "unhide_hidden_fields": response.response_unhide_hidden_fields,
                        "enable_disabled_fields": response.response_enable_disabled_fields,
                        "remove_input_length_limits": response.response_remove_input_length_limits,
                        "remove_javascript_validation": response.response_remove_javascript_validation,
                        "remove_all_javascript": response.response_remove_all_javascript,
                    },
                },
                "websocket": {
                    "client_to_server": response.websocket_client_to_server,
                    "server_to_client": response.websocket_server_to_client,
                    "in_scope_only": response.websocket_in_scope_only,
                },
            }).to_string(),
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

fn macro_json(macro_definition: MacroDefinition) -> serde_json::Value {
    serde_json::json!({
        "description": macro_definition.description,
        "serial_number": macro_definition.serial_number,
        "items": macro_definition.items.into_iter().map(|item| serde_json::json!({
            "request": item.request,
            "method": item.method,
            "url": item.url,
            "response": item.response,
            "status_code": item.status_code,
            "cookies_received": item.cookies_received,
            "request_parameters": item.request_parameters.into_iter().map(|parameter| serde_json::json!({
                "name": parameter.name,
                "original_value": parameter.original_value,
                "parameter_handling": parameter.parameter_handling,
                "preset_value": parameter.preset_value,
                "type": parameter.r#type,
            })).collect::<Vec<_>>(),
            "custom_parameters": item.custom_parameters,
        })).collect::<Vec<_>>(),
    })
}

#[tool_handler(router = Self::burp_router(), name = "burp-mcp", version = "3.0.0-alpha.1")]
impl rmcp::ServerHandler for BurpTools {
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, rmcp::ErrorData> {
        let tool_context = ToolCallContext::new(self, request, context);
        let router = Self::burp_router() + Self::utility_router();
        let response = router.call(tool_context).await?;
        Ok(mark_embedded_error(response))
    }

    async fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<rmcp::model::ListToolsResult, rmcp::ErrorData> {
        let router = Self::burp_router() + Self::utility_router();
        Ok(rmcp::model::ListToolsResult::with_all_items(
            router.list_all(),
        ))
    }
}

fn mark_embedded_error(response: CallToolResponse) -> CallToolResponse {
    let CallToolResponse::Complete(result) = response else {
        return response;
    };
    if result.is_error == Some(true) {
        return CallToolResponse::Complete(result);
    }
    let Some(error) = result.content.iter().find_map(|content| {
        let ContentBlock::Text(text) = content else {
            return None;
        };
        let value: serde_json::Value = serde_json::from_str(&text.text).ok()?;
        value
            .get("error")
            .filter(|error| match error {
                serde_json::Value::Null => false,
                serde_json::Value::String(message) => !message.is_empty(),
                _ => true,
            })
            .cloned()
    }) else {
        return CallToolResponse::Complete(result);
    };
    let value = serde_json::json!({"error": error});
    CallToolResponse::Complete(CallToolResult::structured_error(value))
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ScanIssuesInput {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

fn action_json(
    result: Result<burp_protocol::protocol::ActionResponse, burp_protocol::ClientError>,
) -> String {
    match result {
        Ok(response) => {
            serde_json::json!({"success": response.success, "message": response.message})
                .to_string()
        }
        Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
    }
}

fn proxy_intercept_rule_json(rule: ProxyInterceptRule) -> serde_json::Value {
    serde_json::json!({
        "enabled": rule.enabled,
        "boolean_operator": rule.boolean_operator,
        "match_type": rule.match_type,
        "match_relationship": rule.match_relationship,
        "match_condition": rule.match_condition,
    })
}

fn session_rule_request(input: SessionRuleUpsertInput) -> UpsertSessionRuleRequest {
    UpsertSessionRuleRequest {
        id: input.id.unwrap_or_default(),
        find: input.find.unwrap_or_default(),
        replacement: input.replace.unwrap_or_default(),
        description: input.description.unwrap_or_else(|| "Burp MCP session rule".to_owned()),
        action_type: input.action_type.unwrap_or_else(|| "replace_text".to_owned()),
        header_name: input.header_name.unwrap_or_default(),
        parameter_name: input.parameter_name.unwrap_or_default(),
        macro_description: input.macro_description.unwrap_or_default(),
        url_contains: input.url_contains.unwrap_or_default(),
        tools: input.tools.unwrap_or_default(),
        enabled: input.enabled.unwrap_or(true),
    }
}

fn session_rule_json(rule: burp_protocol::protocol::SessionRuleEntry) -> serde_json::Value {
    serde_json::json!({
        "id": rule.id,
        "description": rule.description,
        "action_type": rule.action_type,
        "find": rule.find,
        "replace": rule.replacement,
        "header_name": rule.header_name,
        "parameter_name": rule.parameter_name,
        "macro_description": rule.macro_description,
        "url_contains": rule.url_contains,
        "tools": rule.tools,
        "enabled": rule.enabled,
    })
}
fn script_import_json(
    result: Result<burp_protocol::protocol::ScriptImportResponse, burp_protocol::ClientError>,
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
fn utility_value(input: UtilityValueInput) -> utility_engine_api::UtilityResult<DataValue> {
    match input {
        UtilityValueInput::Text { value } => Ok(DataValue::Text(value)),
        UtilityValueInput::Bytes { base64 } => {
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, base64)
                .map(DataValue::Bytes)
                .map_err(|error| {
                    utility_engine_api::UtilityError::message(format!(
                        "invalid base64 input: {error}"
                    ))
                })
        }
        UtilityValueInput::Json { value } => Ok(DataValue::Json(value)),
    }
}

fn decoder_json(input: DecoderInput) -> String {
    if let Some(query) = input.query.as_deref() {
        return serde_json::to_string(&utility_engine_api::search(query))
            .expect("decoder operation registry must serialize");
    }
    if let Some(operation) = input.describe.as_deref() {
        return match utility_engine_api::describe(operation) {
            Some(operation) => serde_json::to_string(&operation)
                .expect("decoder operation metadata must serialize"),
            None => serde_json::json!({"error": "operation not found"}).to_string(),
        };
    }
    let value = match utility_value(input.input) {
        Ok(value) => value,
        Err(error) => return serde_json::json!({"error": error.to_string()}).to_string(),
    };
    if input.magic {
        return serde_json::json!({"suggestions": utility_engine_api::magic(&value)}).to_string();
    }
    let result = match (input.operation, input.steps.is_empty()) {
        (Some(operation), true) => utility_engine_api::run(&operation, value, &input.args),
        (None, false) => {
            let steps = input
                .steps
                .iter()
                .map(|step| utility_engine_api::RecipeStep {
                    operation: step.op.clone(),
                    args: step.args.clone(),
                })
                .collect::<Vec<_>>();
            utility_engine_api::run_recipe(value, &steps, utility_engine_api::run)
        }
        (Some(_), false) => Err(utility_engine_api::UtilityError::message(
            "provide either operation or steps, not both",
        )),
        (None, true) => Err(utility_engine_api::UtilityError::message(
            "provide an operation, steps, query, describe, or magic mode",
        )),
    };
    decoder_result_json(result)
}

fn decoder_result_json(result: utility_engine_api::UtilityResult<DataValue>) -> String {
    match result {
        Ok(value) => utility_value_json(value).to_string(),
        Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
    }
}
fn scan_configuration_request(input: ScanConfigurationUpsertInput) -> UpsertScanConfigurationRequest {
    UpsertScanConfigurationRequest {
        id: input.id.unwrap_or_default(),
        name: input.name,
        scan_type: input.scan_type,
        audit_type: input.audit_type.unwrap_or_default(),
        include_out_of_scope: input.include_out_of_scope.unwrap_or(false),
        timeout_seconds: input.timeout_seconds.unwrap_or(900),
        stable_seconds: input.stable_seconds.unwrap_or(2),
        resource_pool_id: input.resource_pool_id.unwrap_or_default(),
    }
}

fn scan_configuration_json(value: burp_protocol::protocol::ScanConfigurationEntry) -> serde_json::Value {
    serde_json::json!({"id": value.id, "name": value.name, "scan_type": value.scan_type, "audit_type": value.audit_type, "include_out_of_scope": value.include_out_of_scope, "timeout_seconds": value.timeout_seconds, "stable_seconds": value.stable_seconds, "resource_pool_id": value.resource_pool_id, "source": value.source})
}

fn scan_pool_json(value: burp_protocol::protocol::ScanResourcePoolEntry) -> serde_json::Value {
    serde_json::json!({"id": value.id, "name": value.name, "kind": value.kind, "existing_pool_name": value.existing_pool_name, "concurrent_request_limit": value.concurrent_request_limit, "throttle_millis": value.throttle_millis, "max_retries": value.max_retries, "source": value.source})
}

fn scan_pool_request(input: ScanResourcePoolUpsertInput) -> UpsertScanResourcePoolRequest {
    UpsertScanResourcePoolRequest {
        id: input.id.unwrap_or_default(),
        name: input.name,
        kind: input.kind,
        existing_pool_name: input.existing_pool_name.unwrap_or_default(),
        concurrent_request_limit: input.concurrent_request_limit.unwrap_or(10),
        throttle_millis: input.throttle_millis.unwrap_or(0),
        max_retries: input.max_retries.unwrap_or(0),
    }
}

fn payload_list_proto_json(item: &burp_protocol::protocol::PayloadListEntry) -> serde_json::Value {
    serde_json::json!({
        "id": item.id,
        "display_name": item.display_name,
        "payload_count": item.payload_count,
        "size_bytes": item.size_bytes,
        "fingerprint": item.fingerprint,
    })
}

fn payload_list_entry_json(
    result: Result<burp_protocol::protocol::PayloadListEntry, burp_protocol::ClientError>,
) -> String {
    match result {
        Ok(item) => payload_list_proto_json(&item).to_string(),
        Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
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

fn to_filtered_proxy_history_request(
    input: ProxyHistoryFilteredInput,
    limit: u32,
) -> ProxyHistoryRequest {
    ProxyHistoryRequest {
        page: Some(PageRequest {
            limit,
            cursor: input
                .cursor
                .unwrap_or_else(|| input.offset.unwrap_or_default().to_string()),
        }),
        url_filter: input.url_filter.unwrap_or_default(),
        method_filter: String::new(),
        status_filter: None,
        has_notes: input.has_notes.unwrap_or(false),
        color: input.color.unwrap_or_default(),
    }
}

fn job_status_json(
    result: Result<burp_protocol::protocol::JobStatusResponse, burp_protocol::ClientError>,
) -> String {
    match result {
        Ok(status) => serde_json::json!({
            "job_id": status.id,
            "operation": status.operation,
            "state": status.state,
            "error": (!status.error.is_empty()).then_some(status.error),
            "scan_type": (!status.scan_type.is_empty()).then_some(status.scan_type),
            "stateless": status.stateless,
            "status_message": (!status.status_message.is_empty()).then_some(status.status_message),
            "request_count": status.request_count,
            "error_count": status.error_count,
            "issue_count": status.issue_count,
        })
        .to_string(),
        Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
    }
}
fn to_send_output(response: burp_protocol::protocol::SendRequestResponse) -> SendResponseOutput {
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
    use super::{
        BurpTools, DecoderInput, ProxyHistoryFilteredInput, RegisterProxyRuleInput,
        to_filtered_proxy_history_request,
    };
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
                        | "burp_macro_create"
                        | "burp_macro_list"
                        | "burp_macro_remove"
                        | "burp_macro_run"
                        | "burp_proxy_intercept_config"
                        | "burp_list_proxy_rules"
                        | "burp_inspect_config"
                        | "burp_scanner_generate_report"
                        | "burp_scan_issues"
                        | "burp_websocket_history"
                        | "burp_scan_start"
                        | "burp_scan_stop"
                        | "burp_scan_remove"
                        | "burp_scan_config_list"
                        | "burp_scan_config_get"
                        | "burp_scan_config_create"
                        | "burp_scan_config_update"
                        | "burp_scan_config_delete"
                        | "burp_scan_pool_list"
                        | "burp_scan_pool_get"
                        | "burp_scan_pool_create"
                        | "burp_scan_pool_update"
                        | "burp_scan_pool_delete"
                        | "decoder"
                        | "burp_intruder_payload_processor_register"
                        | "burp_intruder_payload_processor_list"
                        | "burp_intruder_payload_processor_remove"
                        | "burp_intruder_payload_generator_register"
                        | "burp_intruder_payload_generator_list"
                        | "burp_intruder_payload_generator_remove"
                        | "burp_payload_list_create"
                        | "burp_payload_list_import"
                        | "burp_payload_list_list"
                        | "burp_payload_list_get"
                        | "burp_payload_list_update"
                        | "burp_payload_list_delete"
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

    #[test]
    fn filtered_proxy_history_exposes_and_forwards_url_filter() {
        let schema = serde_json::to_value(schemars::schema_for!(ProxyHistoryFilteredInput))
            .expect("filtered proxy history input schema must serialize");
        assert!(
            schema["properties"].get("url_filter").is_some(),
            "filtered proxy history must expose url_filter"
        );

        let request = to_filtered_proxy_history_request(
            ProxyHistoryFilteredInput {
                url_filter: Some("https://mcl-staging.opswat.com/".to_owned()),
                has_notes: None,
                color: None,
                limit: Some(5),
                offset: None,
                cursor: None,
            },
            5,
        );
        assert_eq!(request.url_filter, "https://mcl-staging.opswat.com/");
        assert_eq!(request.page.expect("page is required").limit, 5);
    }

    #[test]
    fn decoder_schema_avoids_true_subschemas() {
        let schema = serde_json::to_value(schemars::schema_for!(DecoderInput))
            .expect("decoder input schema must serialize");
        let true_paths = schema_true_paths(&schema, "$".to_owned());
        assert!(
            true_paths.is_empty(),
            "true subschemas are unsupported by LM Studio: {true_paths:?}"
        );
    }

    #[test]
    fn embedded_error_json_becomes_mcp_error_result() {
        let response =
            rmcp::model::CallToolResponse::Complete(rmcp::model::CallToolResult::success(vec![
                rmcp::model::ContentBlock::text(serde_json::json!({"error": "boom"}).to_string()),
            ]));

        let rmcp::model::CallToolResponse::Complete(result) = super::mark_embedded_error(response)
        else {
            panic!("expected complete result");
        };

        assert_eq!(Some(true), result.is_error);
        assert_eq!(
            Some(&serde_json::json!({"error": "boom"})),
            result.structured_content.as_ref()
        );
    }

    #[test]
    fn nullable_error_field_remains_successful() {
        let response =
            rmcp::model::CallToolResponse::Complete(rmcp::model::CallToolResult::success(vec![
                rmcp::model::ContentBlock::text(
                    serde_json::json!({"error": null, "state": "queued"}).to_string(),
                ),
            ]));

        let rmcp::model::CallToolResponse::Complete(result) = super::mark_embedded_error(response)
        else {
            panic!("expected complete result");
        };
        assert_eq!(Some(false), result.is_error);
    }

    #[test]
    fn empty_error_field_remains_successful() {
        let response =
            rmcp::model::CallToolResponse::Complete(rmcp::model::CallToolResult::success(vec![
                rmcp::model::ContentBlock::text(
                    serde_json::json!({"error": "", "state": "running"}).to_string(),
                ),
            ]));

        let rmcp::model::CallToolResponse::Complete(result) = super::mark_embedded_error(response)
        else {
            panic!("expected complete result");
        };
        assert_eq!(Some(false), result.is_error);
    }

    #[test]
    fn kotlin_indices_reject_unsigned_overflow() {
        assert_eq!(Ok(i32::MAX as u32), super::validated_index(i32::MAX as u32));
        assert_eq!(
            Err("index must be at most 2147483647"),
            super::validated_index(u32::MAX)
        );
    }
    #[test]
    fn proxy_rule_schema_has_no_deprecated_intercept_field() {
        let schema = serde_json::to_value(schemars::schema_for!(RegisterProxyRuleInput))
            .expect("proxy rule schema must serialize");
        assert!(schema.pointer("/properties/intercept").is_none());
        assert!(schema.pointer("/properties/action").is_some());
    }

    #[test]
    fn intruder_payload_lifecycle_tools_are_mounted() {
        let tools = actual_tool_names();
        for name in [
            "burp_intruder_payload_processor_register",
            "burp_intruder_payload_processor_list",
            "burp_intruder_payload_processor_remove",
            "burp_intruder_payload_generator_register",
            "burp_intruder_payload_generator_list",
            "burp_intruder_payload_generator_remove",
        ] {
            assert!(tools.contains(name), "missing {name}");
        }
    }

    #[test]
    fn graph_limits_are_validated_before_storage_clamping() {
        assert_eq!(Ok(100), super::validated_graph_limit(None));
        assert_eq!(Ok(500), super::validated_graph_limit(Some(500)));
        assert_eq!(
            Err("limit must be between 1 and 500"),
            super::validated_graph_limit(Some(0))
        );
        assert_eq!(
            Err("limit must be between 1 and 500"),
            super::validated_graph_limit(Some(501))
        );
    }

    fn schema_true_paths(value: &Value, path: String) -> Vec<String> {
        match value {
            Value::Bool(true) => vec![path],
            Value::Array(values) => values
                .iter()
                .enumerate()
                .flat_map(|(index, value)| schema_true_paths(value, format!("{path}[{index}]")))
                .collect(),
            Value::Object(values) => values
                .iter()
                .flat_map(|(key, value)| schema_true_paths(value, format!("{path}.{key}")))
                .collect(),
            _ => Vec::new(),
        }
    }
    fn actual_tool_names() -> BTreeSet<String> {
        (BurpTools::burp_router() + BurpTools::utility_router())
            .map
            .keys()
            .map(ToString::to_string)
            .collect()
    }
}
