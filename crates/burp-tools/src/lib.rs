pub mod body_filter;
pub mod diff_engine;
mod sitegraph;
pub mod suite;
mod utility;
pub mod workflows;
use crate::sitegraph::SitegraphIndexer;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use burp_protocol::BurpClient;
use burp_protocol::protocol::{
    AddIssueRequest, CancelJobRequest, ClearHttpHandlerRequest, ClearLoggerRequest,
    ClearProxyRulesRequest, CloseWebSocketRequest, ConfigResponse,
    ControlInterceptedMessageRequest, ControlInterceptedWebSocketMessageRequest, CookieJarRequest,
    CreateMacroRequest, CreatePayloadListRequest, CreateWebSocketRequest, DeletePayloadListRequest,
    DeleteScanConfigurationRequest, DeleteScanResourcePoolRequest, DeleteSessionRuleRequest,
    EditorGetRequest, EditorPatchRequest, EditorRenewLeaseRequest, ExportConfigRequest,
    ExtensionInfoRequest, GenerateCollaboratorPayloadsRequest, GenerateScannerReportRequest,
    GetJobResultRequest, GetJobStatusRequest, GetPayloadListRequest, GetScanConfigurationRequest,
    GetScanResourcePoolRequest, GetSessionRuleRequest, HeaderPatch, HttpHeaderEntry,
    ImportBCheckRequest, ImportBambdaRequest, ImportConfigRequest, ImportPayloadListRequest,
    InterceptAction, InterceptControllerConfigRequest, InterceptStateRequest,
    InterceptedMessagesRequest, InterceptedWebSocketMessagesRequest, JsonPatch, ListMacrosRequest,
    ListPayloadGeneratorsRequest, ListPayloadListsRequest, ListPayloadProcessorsRequest,
    ListProxyRulesRequest, ListScanConfigurationsRequest, ListScanResourcePoolsRequest,
    ListSessionRulesRequest, ListWebSocketsRequest, LoggerDetailRequest, LoggerHistoryRequest,
    MacroDefinition, MacroItem, MacroParameter, ManagedWebSocketHistoryRequest, MarkerPayloadSet,
    MutateScopeRequest, OrganizerListRequest, PageRequest, ParamPatch,
    PollCollaboratorInteractionsRequest, ProxyDetailRequest, ProxyHistoryRequest,
    ProxyInterceptConfigRequest, ProxyInterceptConfigResponse, ProxyInterceptRule,
    ProxyInterceptRuleDelete, ProxyInterceptRuleMutation, ProxyInterceptToggle, ProxyListener,
    ProxyScriptFilter, ProxySettingsRequest, ProxySettingsResponse, ProxySettingsUpdateRequest,
    ProxyWebSocketHistoryRequest, RegexPatch, RegisterHttpHandlerRequest,
    RegisterPayloadGeneratorRequest, RegisterPayloadProcessorRequest, RegisterProxyRuleRequest,
    RemoveMacroRequest, RemovePayloadGeneratorRequest, RemovePayloadProcessorRequest,
    RunMacroRequest, ScanIssueDetailRequest, ScanIssuesRequest, ScopeCheckRequest,
    SendRequestRequest, SendRequestsRequest, SendToComparerRequest, SendToIntruderRequest,
    SendToOrganizerRequest, SendToRepeaterRequest, SendWebSocketBinaryRequest,
    SendWebSocketTextRequest, SetCookieRequest, SetHighlightRequest, SetNoteRequest,
    SitemapSnapshotRequest, StartAuditRequest, StartBoundedInputMatrixRequest,
    StartConcurrentRequestCheckRequest, StartCrawlRequest, TargetInfoRequest, TestBCheckRequest,
    UpdatePayloadListRequest, UpdateScanIssueStatusRequest, UpsertScanConfigurationRequest,
    UpsertScanResourcePoolRequest, UpsertSessionRuleRequest,
    WebSocketInterceptControllerConfigRequest,
};
use prost::Message;
use rmcp::handler::server::tool::ToolCallContext;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock};
use rmcp::service::RequestContext;
use rmcp::{RoleServer, schemars, tool, tool_handler, tool_router};
use serde::{Deserialize, Serialize};
use sitegraph_daemon::{GraphBackend, connect_or_spawn};
use std::path::Path;
use std::sync::Arc;
use utility_engine::{self as utility_engine_api, DataValue};

pub const DEFAULT_MAX_BODY_LENGTH: usize = 4096;
const MAX_PAGE_SIZE: u32 = 500;
const MAX_KOTLIN_INDEX: u32 = i32::MAX as u32;
const MAX_TRAVERSAL_DEPTH: u32 = 8;

pub fn encode_bounded_base64(bytes: &[u8], max_length: Option<usize>) -> (String, bool, usize) {
    let total_len = bytes.len();
    let cap = max_length.unwrap_or(DEFAULT_MAX_BODY_LENGTH);
    if bytes.len() > cap {
        (STANDARD.encode(&bytes[..cap]), true, total_len)
    } else {
        (STANDARD.encode(bytes), false, total_len)
    }
}
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
    pub has_notes: Option<bool>,
    pub color: Option<String>,
    pub include_bodies: Option<bool>,
    pub headers_only: Option<bool>,
    pub extract_css: Option<String>,
    pub extract_json: Option<String>,
    pub max_body_length: Option<usize>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct ProxyHistoryItemOutput {
    index: u32,
    method: String,
    url: String,
    status: u32,
    length: u64,
    has_response: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_base64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_base64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    highlight: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extracted_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    body_truncated: Option<bool>,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
struct ProxyHistoryOutput {
    items: Vec<ProxyHistoryItemOutput>,
    total: u32,
    truncated: bool,
    next_cursor: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ProxyDetailInput {
    pub index: u32,
    pub include_bodies: Option<bool>,
    pub headers_only: Option<bool>,
    pub extract_css: Option<String>,
    pub extract_json: Option<String>,
    pub max_body_length: Option<usize>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct ProxyDetailOutput {
    index: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_base64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_base64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extracted_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
pub struct SetInterceptStateInput {
    pub enabled: bool,
}

#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
pub struct InterceptControllerInput {
    pub enabled: Option<bool>,
    pub timeout_seconds: Option<u32>,
    #[schemars(
        description = "Case-insensitive URL substring. Set to an empty string to clear; enabling requires this or in_scope_only=true"
    )]
    pub url_filter: Option<String>,
    #[schemars(description = "Pause only messages that Burp Target currently marks in scope")]
    pub in_scope_only: Option<bool>,
}

#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
pub struct InterceptedMessagesInput {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
    pub include_bodies: Option<bool>,
    pub max_body_length: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum InterceptActionInput {
    Forward,
    Drop,
    Intercept,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ControlInterceptedMessageInput {
    pub id: u64,
    pub action: InterceptActionInput,
    #[schemars(
        description = "Optional complete replacement HTTP message encoded as standard Base64"
    )]
    pub message_base64: Option<String>,
    pub max_body_length: Option<usize>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct InterceptedMessageOutput {
    id: u64,
    direction: String,
    phase: String,
    url: String,
    method: String,
    status: u32,
    is_in_scope: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_base64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_base64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_truncated: Option<bool>,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProxyInterceptRuleBooleanOperatorInput {
    And,
    Or,
}

impl ProxyInterceptRuleBooleanOperatorInput {
    fn into_proto(self) -> String {
        match self {
            Self::And => "and",
            Self::Or => "or",
        }
        .to_owned()
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProxyInterceptRuleMatchTypeInput {
    FileExtension,
    Request,
    HttpMethod,
    Url,
    ContentTypeHeader,
    StatusCode,
}

impl ProxyInterceptRuleMatchTypeInput {
    fn into_proto(self) -> String {
        match self {
            Self::FileExtension => "file_extension",
            Self::Request => "request",
            Self::HttpMethod => "http_method",
            Self::Url => "url",
            Self::ContentTypeHeader => "content_type_header",
            Self::StatusCode => "status_code",
        }
        .to_owned()
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProxyInterceptRuleRelationshipInput {
    Matches,
    DoesNotMatch,
    ContainsParameters,
    IsInTargetScope,
    WasModified,
    WasIntercepted,
}

impl ProxyInterceptRuleRelationshipInput {
    fn into_proto(self) -> String {
        match self {
            Self::Matches => "matches",
            Self::DoesNotMatch => "does_not_match",
            Self::ContainsParameters => "contains_parameters",
            Self::IsInTargetScope => "is_in_target_scope",
            Self::WasModified => "was_modified",
            Self::WasIntercepted => "was_intercepted",
        }
        .to_owned()
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ProxyInterceptRuleInput {
    pub enabled: Option<bool>,
    pub boolean_operator: Option<ProxyInterceptRuleBooleanOperatorInput>,
    pub match_type: ProxyInterceptRuleMatchTypeInput,
    pub match_relationship: ProxyInterceptRuleRelationshipInput,
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
            boolean_operator: self
                .boolean_operator
                .map(ProxyInterceptRuleBooleanOperatorInput::into_proto)
                .unwrap_or_else(|| "and".to_owned()),
            match_type: self.match_type.into_proto(),
            match_relationship: self.match_relationship.into_proto(),
            match_condition: self.match_condition.unwrap_or_default(),
        }
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ProxyWebSocketHistoryInput {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
    pub include_bodies: Option<bool>,
    pub max_body_length: Option<usize>,
}

#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
pub struct WebSocketInterceptControllerInput {
    pub enabled: Option<bool>,
    pub timeout_seconds: Option<u32>,
}

#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
pub struct InterceptedWebSocketMessagesInput {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
    pub include_bodies: Option<bool>,
    pub max_body_length: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ControlInterceptedWebSocketMessageInput {
    pub id: u64,
    pub action: InterceptActionInput,
    #[schemars(
        description = "Optional replacement payload encoded as standard Base64; empty Base64 replaces with an empty payload"
    )]
    pub payload_base64: Option<String>,
    pub max_body_length: Option<usize>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct InterceptedWebSocketMessageOutput {
    id: u64,
    web_socket_id: u32,
    upgrade_url: String,
    direction: String,
    message_type: String,
    phase: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload_base64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload_truncated: Option<bool>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct ProxyWebSocketHistoryItemOutput {
    index: u32,
    id: u32,
    websocket_id: u32,
    direction: String,
    payload_length: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload_base64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload_truncated: Option<bool>,
    edited_payload_length: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    edited_payload_base64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    edited_payload_truncated: Option<bool>,
    time: String,
    listener_port: u32,
    upgrade_url: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct ProxyWebSocketHistoryOutput {
    items: Vec<ProxyWebSocketHistoryItemOutput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page: Option<serde_json::Value>,
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
    pub headers_only: Option<bool>,
    pub extract_css: Option<String>,
    pub extract_json: Option<String>,
    pub max_body_length: Option<usize>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    extracted: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    truncated: Option<bool>,
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
pub struct LoggerHistoryInput {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub cursor: Option<String>,
    pub source_filter: Option<String>,
    pub url_filter: Option<String>,
    pub method_filter: Option<String>,
    pub status_filter: Option<u32>,
    pub has_notes: Option<bool>,
    pub color: Option<String>,
    pub include_bodies: Option<bool>,
    pub headers_only: Option<bool>,
    pub extract_css: Option<String>,
    pub extract_json: Option<String>,
    pub max_body_length: Option<usize>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct LoggerHistoryItemOutput {
    index: u32,
    id: u32,
    source: String,
    method: String,
    url: String,
    status: u32,
    length: u64,
    has_response: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_base64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_base64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    highlight: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extracted_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    body_truncated: Option<bool>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct LoggerHistoryOutput {
    items: Vec<LoggerHistoryItemOutput>,
    total: u32,
    truncated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_cursor: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LoggerDetailInput {
    pub index: u32,
    pub include_bodies: Option<bool>,
    pub headers_only: Option<bool>,
    pub extract_css: Option<String>,
    pub extract_json: Option<String>,
    pub max_body_length: Option<usize>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct LoggerDetailOutput {
    index: u32,
    source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_base64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_base64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    highlight: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LoggerClearInput {}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct OrganizerSendInput {
    pub request: String,
    pub response: Option<String>,
    pub host: String,
    pub port: Option<u32>,
    pub https: Option<bool>,
    pub notes: Option<String>,
    pub highlight: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct OrganizerListInput {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
    pub status_filter: Option<String>,
    pub url_filter: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct OrganizerItemOutput {
    id: u32,
    index: u32,
    url: String,
    method: String,
    status_code: u32,
    status: String,
    notes: String,
    highlight: String,
    has_response: bool,
    content_type: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct OrganizerListOutput {
    items: Vec<OrganizerItemOutput>,
    total: u32,
    truncated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_cursor: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct BCheckTestInput {
    pub script: String,
    pub request: String,
    pub response: Option<String>,
    pub host: Option<String>,
    pub port: Option<u32>,
    pub https: Option<bool>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct BCheckTestOutput {
    valid: bool,
    matched: bool,
    status: String,
    errors: Vec<String>,
    findings: Vec<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ScanIssueUpdateInput {
    pub index: u32,
    pub status: String,
    pub severity: Option<String>,
    pub confidence: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct ScanIssueUpdateOutput {
    success: bool,
    message: String,
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
pub struct ProxySettingsUpdateInput {
    /// listener_upsert, listener_delete, script_filter_upsert, script_filter_delete,
    /// intercept_rule_upsert, intercept_rule_delete, or intercept_toggle.
    pub operation: String,
    pub port: Option<u32>,
    pub running: Option<bool>,
    pub listen_mode: Option<String>,
    pub listen_specific_address: Option<String>,
    pub certificate_mode: Option<String>,
    pub enable_http2: Option<bool>,
    pub support_invisible_proxying: Option<bool>,
    pub target: Option<String>,
    pub mode: Option<String>,
    pub script: Option<String>,
    pub script_id: Option<String>,
    pub script_name: Option<String>,
    /// request or response for interception-rule operations.
    pub kind: Option<String>,
    /// Omit to append for intercept_rule_upsert; required for update and delete.
    pub index: Option<u32>,
    pub rule: Option<ProxyInterceptRuleInput>,
    pub master_enabled: Option<bool>,
    pub request_enabled: Option<bool>,
    pub response_enabled: Option<bool>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProxyRulePhaseInput {
    Request,
    Response,
}

impl ProxyRulePhaseInput {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Request => "request",
            Self::Response => "response",
        }
    }
}

impl From<ProxyRulePhaseInput> for String {
    fn from(phase: ProxyRulePhaseInput) -> Self {
        phase.as_str().to_owned()
    }
}

impl std::fmt::Display for ProxyRulePhaseInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProxyRuleActionInput {
    Forward,
    Intercept,
    Drop,
    Edit,
}

impl ProxyRuleActionInput {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Forward => "forward",
            Self::Intercept => "intercept",
            Self::Drop => "drop",
            Self::Edit => "edit",
        }
    }
}

impl From<ProxyRuleActionInput> for String {
    fn from(action: ProxyRuleActionInput) -> Self {
        action.as_str().to_owned()
    }
}

impl std::fmt::Display for ProxyRuleActionInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RegisterProxyRuleInput {
    pub id: Option<String>,
    pub url_contains: String,
    pub phase: Option<ProxyRulePhaseInput>,
    pub rule_action: Option<ProxyRuleActionInput>,
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
pub struct SessionRuleIdInput {
    pub id: String,
}

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
    pub single_packet_attack: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct BoundedInputMatrixInput {
    pub template: String,
    pub host: String,
    pub port: Option<u32>,
    pub https: Option<bool>,
    pub marker: Option<String>,
    pub wordlist: Option<Vec<String>>,
    pub payload_list_id: Option<String>,
    pub payload_offset: Option<u32>,
    pub attack_mode: Option<String>,
    pub markers: Option<std::collections::HashMap<String, Vec<String>>>,
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
pub struct ScanConfigurationIdInput {
    pub id: String,
}

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
pub struct ScanResourcePoolIdInput {
    pub id: String,
}

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
    pub target_url: Option<String>,
    pub injection_point: Option<String>,
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
    pub include_bodies: Option<bool>,
    pub max_body_length: Option<usize>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct ManagedWebSocketHistoryItemOutput {
    index: u64,
    websocket_id: String,
    direction: String,
    r#type: String,
    length: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload_truncated: Option<bool>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct ManagedWebSocketHistoryOutput {
    items: Vec<ManagedWebSocketHistoryItemOutput>,
    total: u32,
    truncated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_cursor: Option<String>,
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
pub struct SiteGraphSearchInput {
    pub query: String,
    pub limit: Option<u32>,
    pub cursor: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SiteGraphHistorySearchInput {
    pub query: String,
    #[schemars(description = "History source: all, http, or websocket")]
    pub source: Option<String>,
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

const SITEGRAPH_TOOL_PREFIX: &str = "sitegraph_";

#[derive(Clone)]
struct SitegraphRuntime {
    graph: GraphBackend,
    indexer: SitegraphIndexer,
    mode: Arc<str>,
    interval_seconds: u64,
    auto_index_shutdown: Arc<tokio::sync::watch::Sender<bool>>,
    auto_index_task: Arc<tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DiffResponsesInput {
    pub response_a: Option<String>,
    pub response_b: Option<String>,
    pub index_a: Option<u32>,
    pub index_b: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SendToComparerInput {
    pub first: String,
    pub second: String,
}

#[derive(Clone)]
pub struct BurpTools {
    client: BurpClient,
    sitegraph: Option<SitegraphRuntime>,
}

#[tool_router(router = burp_router)]
impl BurpTools {
    pub async fn new(
        client: BurpClient,
        project_root: Option<&Path>,
        daemon_endpoint: Option<&Path>,
        rules_path: &Path,
    ) -> Result<Self, String> {
        let Some(project_root) = project_root else {
            return Ok(Self {
                client,
                sitegraph: None,
            });
        };
        let identity = client
            .server_info(burp_protocol::protocol::ServerInfoRequest {})
            .await
            .ok();
        let (resolved_path, graph_id) = match identity {
            Some(info) if !info.graph_id.is_empty() => {
                let file_name = if info.project_temporary {
                    format!("temp-{}.sqlite", info.graph_id)
                } else {
                    format!("{}.sqlite", info.graph_id)
                };
                (project_root.join(file_name), info.graph_id)
            }
            _ => {
                return Err(
                    "project identity unavailable; refusing to open a shared fallback graph"
                        .to_owned(),
                );
            }
        };
        let daemon = match daemon_endpoint {
            Some(endpoint) => sitegraph_daemon::Client::new(endpoint),
            None => connect_or_spawn(&resolved_path, &graph_id, rules_path)
                .await
                .map_err(|error| error.to_string())?,
        };
        let graph = GraphBackend::Remote(daemon);
        let indexer = SitegraphIndexer::spawn(client.clone(), graph.clone());
        let (auto_index_shutdown, _) = tokio::sync::watch::channel(false);
        Ok(Self {
            client,
            sitegraph: Some(SitegraphRuntime {
                graph,
                indexer,
                mode: Arc::from("off"),
                interval_seconds: 30,
                auto_index_shutdown: Arc::new(auto_index_shutdown),
                auto_index_task: Arc::new(tokio::sync::Mutex::new(None)),
            }),
        })
    }
    fn sitegraph_runtime(&self) -> &SitegraphRuntime {
        self.sitegraph
            .as_ref()
            .expect("sitegraph tools must not be routed when sitegraph is disabled")
    }

    fn tool_router(&self) -> rmcp::handler::server::tool::ToolRouter<Self> {
        Self::tool_router_for(self.sitegraph.is_some())
    }

    fn tool_router_for(sitegraph_enabled: bool) -> rmcp::handler::server::tool::ToolRouter<Self> {
        let mut router = Self::burp_router() + Self::utility_router();
        if !sitegraph_enabled {
            router
                .map
                .retain(|name, _| *name != "sitegraph" && !name.starts_with(SITEGRAPH_TOOL_PREFIX));
        }
        router
    }

    fn validate_sitegraph_mode(sitegraph_enabled: bool, mode: &str) -> Result<(), String> {
        if sitegraph_enabled || mode == "off" {
            Ok(())
        } else {
            Err("sitegraph must be enabled before selecting an indexing mode".to_owned())
        }
    }

    pub async fn start_auto_index(
        &mut self,
        mode: &str,
        interval: std::time::Duration,
    ) -> Result<(), String> {
        Self::validate_sitegraph_mode(self.sitegraph.is_some(), mode)?;
        let Some(sitegraph) = &mut self.sitegraph else {
            return Ok(());
        };
        sitegraph.mode = Arc::from(mode);
        sitegraph.interval_seconds = interval.as_secs();
        match mode {
            "off" => Ok(()),
            "startup" => {
                sitegraph.indexer.sync(String::new()).await?;
                Ok(())
            }
            "watch" => {
                let indexer = sitegraph.indexer.clone();
                let mut shutdown = sitegraph.auto_index_shutdown.subscribe();
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
                *sitegraph.auto_index_task.lock().await = Some(task);
                Ok(())
            }
            _ => Err(format!("unsupported sitegraph mode: {mode}")),
        }
    }

    pub async fn shutdown(&self) {
        let Some(sitegraph) = &self.sitegraph else {
            return;
        };
        let _ = sitegraph.auto_index_shutdown.send(true);
        if let Some(task) = sitegraph.auto_index_task.lock().await.take() {
            let _ = tokio::time::timeout(std::time::Duration::from_secs(5), task).await;
        }
        sitegraph.indexer.shutdown().await;
    }

    async fn proxy_history(&self, Parameters(input): Parameters<ProxyHistoryInput>) -> String {
        let limit = input.limit.unwrap_or(100);
        if limit > MAX_PAGE_SIZE {
            return serde_json::json!({"error": "limit must be at most 500"}).to_string();
        }
        let include_bodies = input.include_bodies.unwrap_or(false);
        let headers_only = input.headers_only.unwrap_or(false);
        let extract_css = input.extract_css.clone();
        let extract_json = input.extract_json.clone();
        let effective_max_length = Some(input.max_body_length.unwrap_or(DEFAULT_MAX_BODY_LENGTH));
        match self
            .client
            .proxy_history(to_proxy_history_request(input, limit))
            .await
        {
            Ok(response) => {
                let page = response.page.unwrap_or_default();
                serde_json::to_string(&ProxyHistoryOutput {
                    items: response
                        .items
                        .into_iter()
                        .map(|item| {
                            let req_len = item.request.len();
                            let (req_b64, req_trunc) = if include_bodies && !item.request.is_empty() {
                                let (b64, trunc, _) = encode_bounded_base64(&item.request, effective_max_length);
                                (Some(b64), trunc.then_some(true))
                            } else {
                                (None, None)
                            };

                            let resp_len = item.has_response.then_some(item.response.len());
                            let (resp_b64, resp_text, is_trunc, resp_trunc) = if include_bodies && item.has_response {
                                let (filtered, trunc) = body_filter::filter_and_truncate_payload(
                                    &item.response,
                                    (!item.content_type.is_empty()).then_some(&item.content_type),
                                    headers_only,
                                    extract_css.as_deref(),
                                    extract_json.as_deref(),
                                    effective_max_length,
                                );
                                let (b64, b64_trunc, _) = encode_bounded_base64(&item.response, effective_max_length);
                                (Some(b64), Some(filtered), trunc || b64_trunc, b64_trunc.then_some(true))
                            } else {
                                (None, None, false, None)
                            };

                            ProxyHistoryItemOutput {
                                index: item.index,
                                method: item.method,
                                url: item.url,
                                status: item.status,
                                length: item.length,
                                has_response: item.has_response,
                                request_base64: req_b64,
                                request_length: Some(req_len),
                                request_truncated: req_trunc,
                                response_base64: resp_b64,
                                response_length: resp_len,
                                response_truncated: resp_trunc,
                                notes: (!item.notes.is_empty()).then_some(item.notes),
                                highlight: (!item.highlight.is_empty()).then_some(item.highlight),
                                time: (!item.time.is_empty()).then_some(item.time),
                                content_type: (!item.content_type.is_empty()).then_some(item.content_type),
                                extracted_text: resp_text,
                                body_truncated: is_trunc.then_some(true),
                            }
                        })
                        .collect(),
                    total: page.total,
                    truncated: page.truncated,
                    next_cursor: (!page.next_cursor.is_empty()).then_some(page.next_cursor),
                }).expect("proxy output must serialize")
            }
            Err(error) => serde_json::json!({"error": error.to_string(), "connected": false, "action": "Start Burp with the Burp MCP extension and retry"}).to_string(),
        }
    }

    async fn proxy_detail(&self, Parameters(input): Parameters<ProxyDetailInput>) -> String {
        let index = match validated_index(input.index) {
            Ok(index) => index,
            Err(error) => return serde_json::json!({"error": error}).to_string(),
        };
        let headers_only = input.headers_only.unwrap_or(false);
        let extract_css = input.extract_css.clone();
        let extract_json = input.extract_json.clone();
        let effective_max_length = Some(input.max_body_length.unwrap_or(DEFAULT_MAX_BODY_LENGTH));
        match self.client.proxy_detail(ProxyDetailRequest { index }).await {
            Ok(detail) => {
                let (req_b64, req_trunc, req_len) = if !detail.request.is_empty() {
                    let (b64, trunc, len) =
                        encode_bounded_base64(&detail.request, effective_max_length);
                    (Some(b64), trunc, Some(len))
                } else {
                    (None, false, Some(0))
                };

                let (resp_b64, resp_trunc, resp_len) = if !detail.response.is_empty() {
                    let (b64, trunc, len) =
                        encode_bounded_base64(&detail.response, effective_max_length);
                    (Some(b64), trunc, Some(len))
                } else {
                    (None, false, None)
                };

                let req_text = String::from_utf8_lossy(&detail.request).into_owned();
                let req_display = if headers_only {
                    body_filter::extract_headers_only(&req_text)
                } else {
                    let (filtered, _) = body_filter::filter_and_truncate_payload(
                        &detail.request,
                        None,
                        headers_only,
                        None,
                        None,
                        effective_max_length,
                    );
                    filtered
                };

                let (resp_text, is_trunc) = if !detail.response.is_empty() {
                    let (filtered, trunc) = body_filter::filter_and_truncate_payload(
                        &detail.response,
                        None,
                        headers_only,
                        extract_css.as_deref(),
                        extract_json.as_deref(),
                        effective_max_length,
                    );
                    (Some(filtered), trunc)
                } else {
                    (None, false)
                };

                serde_json::to_string(&ProxyDetailOutput {
                    index: detail.index,
                    request_base64: req_b64,
                    request_length: req_len,
                    request_truncated: req_trunc.then_some(true),
                    response_base64: resp_b64,
                    response_length: resp_len,
                    response_truncated: resp_trunc.then_some(true),
                    request_text: Some(req_display),
                    response_text: resp_text,
                    extracted_text: None,
                    truncated: (is_trunc || req_trunc || resp_trunc).then_some(true),
                    notes: (!detail.notes.is_empty()).then_some(detail.notes),
                    highlight: (!detail.highlight.is_empty()).then_some(detail.highlight),
                })
                .expect("proxy detail output must serialize")
            }
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    async fn logger_history(&self, Parameters(input): Parameters<LoggerHistoryInput>) -> String {
        let limit = input.limit.unwrap_or(100);
        if limit > MAX_PAGE_SIZE {
            return serde_json::json!({"error": "limit must be at most 500"}).to_string();
        }
        let include_bodies = input.include_bodies.unwrap_or(false);
        let headers_only = input.headers_only.unwrap_or(false);
        let extract_css = input.extract_css.clone();
        let extract_json = input.extract_json.clone();
        let effective_max_length = Some(input.max_body_length.unwrap_or(DEFAULT_MAX_BODY_LENGTH));
        match self
            .client
            .logger_history(LoggerHistoryRequest {
                page: Some(PageRequest {
                    limit,
                    cursor: input
                        .cursor
                        .unwrap_or_else(|| input.offset.unwrap_or_default().to_string()),
                }),
                source_filter: input.source_filter.unwrap_or_default(),
                url_filter: input.url_filter.unwrap_or_default(),
                method_filter: input.method_filter.unwrap_or_default(),
                status_filter: input.status_filter,
                has_notes: input.has_notes.unwrap_or(false),
                color: input.color.unwrap_or_default(),
                after_id: None,
            })
            .await
        {
            Ok(response) => {
                let page = response.page.unwrap_or_default();
                serde_json::to_string(&LoggerHistoryOutput {
                    items: response
                        .items
                        .into_iter()
                        .map(|item| {
                            let req_len = item.request.len();
                            let (req_b64, req_trunc) = if include_bodies && !item.request.is_empty()
                            {
                                let (b64, trunc, _) =
                                    encode_bounded_base64(&item.request, effective_max_length);
                                (Some(b64), trunc.then_some(true))
                            } else {
                                (None, None)
                            };

                            let resp_len = item.has_response.then_some(item.response.len());
                            let (resp_b64, resp_text, is_trunc, resp_trunc) = if include_bodies
                                && item.has_response
                            {
                                let (filtered, trunc) = body_filter::filter_and_truncate_payload(
                                    &item.response,
                                    (!item.content_type.is_empty()).then_some(&item.content_type),
                                    headers_only,
                                    extract_css.as_deref(),
                                    extract_json.as_deref(),
                                    effective_max_length,
                                );
                                let (b64, b64_trunc, _) =
                                    encode_bounded_base64(&item.response, effective_max_length);
                                (
                                    Some(b64),
                                    Some(filtered),
                                    trunc || b64_trunc,
                                    b64_trunc.then_some(true),
                                )
                            } else {
                                (None, None, false, None)
                            };

                            LoggerHistoryItemOutput {
                                index: item.index,
                                id: item.id,
                                source: item.source,
                                method: item.method,
                                url: item.url,
                                status: item.status,
                                length: item.length,
                                has_response: item.has_response,
                                request_base64: req_b64,
                                request_length: Some(req_len),
                                request_truncated: req_trunc,
                                response_base64: resp_b64,
                                response_length: resp_len,
                                response_truncated: resp_trunc,
                                notes: (!item.notes.is_empty()).then_some(item.notes),
                                highlight: (!item.highlight.is_empty()).then_some(item.highlight),
                                time: (!item.time.is_empty()).then_some(item.time),
                                content_type: (!item.content_type.is_empty())
                                    .then_some(item.content_type),
                                extracted_text: resp_text,
                                body_truncated: is_trunc.then_some(true),
                            }
                        })
                        .collect(),
                    total: page.total,
                    truncated: page.truncated,
                    next_cursor: (!page.next_cursor.is_empty()).then_some(page.next_cursor),
                })
                .expect("logger history output must serialize")
            }
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    async fn logger_detail(&self, Parameters(input): Parameters<LoggerDetailInput>) -> String {
        let index = match validated_index(input.index) {
            Ok(index) => index,
            Err(error) => return serde_json::json!({"error": error}).to_string(),
        };
        let headers_only = input.headers_only.unwrap_or(false);
        let extract_css = input.extract_css.clone();
        let extract_json = input.extract_json.clone();
        let effective_max_length = Some(input.max_body_length.unwrap_or(DEFAULT_MAX_BODY_LENGTH));
        match self
            .client
            .logger_detail(LoggerDetailRequest { index })
            .await
        {
            Ok(detail) => {
                let (req_b64, req_trunc, req_len) = if !detail.request.is_empty() {
                    let (b64, trunc, len) =
                        encode_bounded_base64(&detail.request, effective_max_length);
                    (Some(b64), trunc, Some(len))
                } else {
                    (None, false, Some(0))
                };

                let (resp_b64, resp_trunc, resp_len) = if !detail.response.is_empty() {
                    let (b64, trunc, len) =
                        encode_bounded_base64(&detail.response, effective_max_length);
                    (Some(b64), trunc, Some(len))
                } else {
                    (None, false, None)
                };

                let req_text = String::from_utf8_lossy(&detail.request).into_owned();
                let req_display = if headers_only {
                    body_filter::extract_headers_only(&req_text)
                } else {
                    let (filtered, _) = body_filter::filter_and_truncate_payload(
                        &detail.request,
                        None,
                        headers_only,
                        None,
                        None,
                        effective_max_length,
                    );
                    filtered
                };

                let (resp_text, is_trunc) = if !detail.response.is_empty() {
                    let (filtered, trunc) = body_filter::filter_and_truncate_payload(
                        &detail.response,
                        None,
                        headers_only,
                        extract_css.as_deref(),
                        extract_json.as_deref(),
                        effective_max_length,
                    );
                    (Some(filtered), trunc)
                } else {
                    (None, false)
                };

                serde_json::to_string(&LoggerDetailOutput {
                    index: detail.index,
                    source: detail.source,
                    request_base64: req_b64,
                    request_length: req_len,
                    request_truncated: req_trunc.then_some(true),
                    response_base64: resp_b64,
                    response_length: resp_len,
                    response_truncated: resp_trunc.then_some(true),
                    request_text: Some(req_display),
                    response_text: resp_text,
                    truncated: (is_trunc || req_trunc || resp_trunc).then_some(true),
                    notes: (!detail.notes.is_empty()).then_some(detail.notes),
                    highlight: (!detail.highlight.is_empty()).then_some(detail.highlight),
                })
                .expect("logger detail output must serialize")
            }
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    async fn clear_logger(&self, Parameters(_input): Parameters<LoggerClearInput>) -> String {
        match self.client.clear_logger(ClearLoggerRequest {}).await {
            Ok(res) => {
                serde_json::json!({"success": res.success, "message": res.message}).to_string()
            }
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    async fn organizer_send(&self, Parameters(input): Parameters<OrganizerSendInput>) -> String {
        match self
            .client
            .send_to_organizer(SendToOrganizerRequest {
                request: input.request.into_bytes(),
                response: input.response.map(|r| r.into_bytes()).unwrap_or_default(),
                host: input.host,
                port: input
                    .port
                    .unwrap_or(if input.https.unwrap_or(true) { 443 } else { 80 }),
                https: input.https.unwrap_or(true),
                notes: input.notes.unwrap_or_default(),
                highlight: input.highlight.unwrap_or_default(),
            })
            .await
        {
            Ok(res) => {
                serde_json::json!({"success": res.success, "message": res.message}).to_string()
            }
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    async fn organizer_list(&self, Parameters(input): Parameters<OrganizerListInput>) -> String {
        let limit = input.limit.unwrap_or(100);
        if limit > MAX_PAGE_SIZE {
            return serde_json::json!({"error": "limit must be at most 500"}).to_string();
        }
        match self
            .client
            .organizer_list(OrganizerListRequest {
                page: Some(PageRequest {
                    limit,
                    cursor: input.cursor.unwrap_or_default(),
                }),
                status_filter: input.status_filter.unwrap_or_default(),
                url_filter: input.url_filter.unwrap_or_default(),
            })
            .await
        {
            Ok(response) => {
                let page = response.page.unwrap_or_default();
                serde_json::to_string(&OrganizerListOutput {
                    items: response
                        .items
                        .into_iter()
                        .map(|item| OrganizerItemOutput {
                            id: item.id,
                            index: item.index,
                            url: item.url,
                            method: item.method,
                            status_code: item.status_code,
                            status: item.status,
                            notes: item.notes,
                            highlight: item.highlight,
                            has_response: item.has_response,
                            content_type: item.content_type,
                        })
                        .collect(),
                    total: page.total,
                    truncated: page.truncated,
                    next_cursor: (!page.next_cursor.is_empty()).then_some(page.next_cursor),
                })
                .expect("organizer list output must serialize")
            }
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    async fn test_bcheck(&self, Parameters(input): Parameters<BCheckTestInput>) -> String {
        match self
            .client
            .test_bcheck(TestBCheckRequest {
                script: input.script,
                request: input.request.into_bytes(),
                response: input.response.map(|r| r.into_bytes()).unwrap_or_default(),
                host: input.host.unwrap_or_default(),
                port: input
                    .port
                    .unwrap_or(if input.https.unwrap_or(true) { 443 } else { 80 }),
                https: input.https.unwrap_or(true),
            })
            .await
        {
            Ok(res) => serde_json::to_string(&BCheckTestOutput {
                valid: res.valid,
                matched: res.matched,
                status: res.status,
                errors: res.errors,
                findings: res.findings,
            })
            .expect("bcheck test output must serialize"),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    async fn update_scan_issue_status(
        &self,
        Parameters(input): Parameters<ScanIssueUpdateInput>,
    ) -> String {
        match self
            .client
            .update_scan_issue_status(UpdateScanIssueStatusRequest {
                index: input.index,
                status: input.status,
                severity: input.severity,
                confidence: input.confidence,
                notes: input.notes,
            })
            .await
        {
            Ok(res) => serde_json::to_string(&ScanIssueUpdateOutput {
                success: res.success,
                message: res.message,
            })
            .expect("update scan issue status output must serialize"),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

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

    async fn send_request(&self, Parameters(input): Parameters<SendRequestInput>) -> String {
        let headers_only = input.headers_only.unwrap_or(false);
        let extract_css = input.extract_css.clone();
        let extract_json = input.extract_json.clone();
        let max_body_length = input.max_body_length;
        match self.client.send_request(to_proto_request(&input)).await {
            Ok(response) => serde_json::to_string(&to_send_output_with_options(
                response,
                headers_only,
                extract_css.as_deref(),
                extract_json.as_deref(),
                max_body_length,
            ))
            .expect("send output must serialize"),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    async fn send_request_parallel(
        &self,
        Parameters(input): Parameters<SendRequestsInput>,
    ) -> String {
        if input.requests.len() > 32 {
            return serde_json::json!({"error": "at most 32 requests may be sent in one batch"})
                .to_string();
        }
        let proto_requests = input.requests.iter().map(to_proto_request).collect();
        match self
            .client
            .send_requests(SendRequestsRequest {
                requests: proto_requests,
            })
            .await
        {
            Ok(response) => serde_json::to_string(
                &response
                    .responses
                    .into_iter()
                    .enumerate()
                    .map(|(i, resp)| {
                        let req_input = input.requests.get(i);
                        to_send_output_with_options(
                            resp,
                            req_input.and_then(|r| r.headers_only).unwrap_or(false),
                            req_input.and_then(|r| r.extract_css.as_deref()),
                            req_input.and_then(|r| r.extract_json.as_deref()),
                            req_input.and_then(|r| r.max_body_length),
                        )
                    })
                    .collect::<Vec<_>>(),
            )
            .expect("parallel send output must serialize"),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    async fn send_to_repeater(&self, Parameters(input): Parameters<SendToRepeaterInput>) -> String {
        let request = match normalize_repeater_input(input) {
            Ok(request) => request,
            Err(message) => return tool_input_error("burp_http", "send_to_repeater", message),
        };
        match self.client.send_to_repeater(request).await {
            Ok(response) => {
                serde_json::json!({"success": response.success, "message": response.message})
                    .to_string()
            }
            Err(error) => rpc_error_json(error),
        }
    }
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
    async fn add_to_scope(&self, Parameters(input): Parameters<ScopeMutationInput>) -> String {
        self.mutate_scope(input.url, true).await
    }

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

    async fn remove_http_handler(&self) -> String {
        action_json(
            self.client
                .clear_http_handler(ClearHttpHandlerRequest {})
                .await,
        )
    }

    async fn register_proxy_rule(
        &self,
        Parameters(input): Parameters<RegisterProxyRuleInput>,
    ) -> String {
        if input.url_contains.trim().is_empty() {
            return tool_input_error(
                "burp_settings",
                "register_proxy_rule",
                "`url_contains` is required and must not be empty",
            );
        }
        let action = input
            .rule_action
            .map_or("forward", |a| a.as_str())
            .to_owned();
        let phase = input.phase.map_or("request", |p| p.as_str()).to_owned();
        action_json(
            self.client
                .register_proxy_rule(RegisterProxyRuleRequest {
                    id: input.id.unwrap_or_else(|| "default".to_owned()),
                    url_contains: input.url_contains,
                    phase,
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

    async fn session_create_rule(
        &self,
        Parameters(input): Parameters<SessionRuleUpsertInput>,
    ) -> String {
        match self
            .client
            .create_session_rule(session_rule_request(input))
            .await
        {
            Ok(rule) => session_rule_json(rule).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    async fn session_get_rule(&self, Parameters(input): Parameters<SessionRuleIdInput>) -> String {
        match self
            .client
            .get_session_rule(GetSessionRuleRequest { id: input.id })
            .await
        {
            Ok(rule) => session_rule_json(rule).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    async fn session_update_rule(
        &self,
        Parameters(input): Parameters<SessionRuleUpsertInput>,
    ) -> String {
        if input.id.as_deref().unwrap_or_default().is_empty() {
            return serde_json::json!({"error": "id is required"}).to_string();
        }
        match self
            .client
            .update_session_rule(session_rule_request(input))
            .await
        {
            Ok(rule) => session_rule_json(rule).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    async fn session_list_rules(&self) -> String {
        match self.client.list_session_rules(ListSessionRulesRequest {}).await {
            Ok(response) => serde_json::json!({"rules": response.items.into_iter().map(session_rule_json).collect::<Vec<_>>() }).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    async fn session_delete_rule(
        &self,
        Parameters(input): Parameters<SessionRuleIdInput>,
    ) -> String {
        action_json(
            self.client
                .delete_session_rule(DeleteSessionRuleRequest { id: input.id })
                .await,
        )
    }

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

    async fn macro_list(&self) -> String {
        match self.client.list_macros(ListMacrosRequest {}).await {
            Ok(response) => serde_json::json!({"macros": response.macros.into_iter().map(macro_json).collect::<Vec<_>>()}).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    async fn macro_run(&self, Parameters(input): Parameters<MacroDescriptionInput>) -> String {
        match self.client.run_macro(RunMacroRequest { description: input.description }).await {
            Ok(response) => serde_json::json!({"items": response.items.into_iter().map(|item| serde_json::json!({"request": item.request, "response": item.response, "status_code": item.status_code, "has_response": item.has_response})).collect::<Vec<_>>()}).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    async fn macro_remove(&self, Parameters(input): Parameters<MacroDescriptionInput>) -> String {
        action_json(
            self.client
                .remove_macro(RemoveMacroRequest {
                    description: input.description,
                })
                .await,
        )
    }

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
                    single_packet_attack: input.single_packet_attack.unwrap_or(false),
                    warmup_connections: 0,
                })
                .await,
        )
    }

    async fn inline_fuzzer(
        &self,
        Parameters(input): Parameters<BoundedInputMatrixInput>,
    ) -> String {
        let https = input.https.unwrap_or(true);
        let port = input.port.unwrap_or(if https { 443 } else { 80 });
        let marker_payloads = input
            .markers
            .unwrap_or_default()
            .into_iter()
            .map(|(m, p)| MarkerPayloadSet {
                marker: m,
                payloads: p,
            })
            .collect();
        job_status_json(
            self.client
                .start_bounded_input_matrix(StartBoundedInputMatrixRequest {
                    template: input.template.into_bytes(),
                    host: input.host,
                    port,
                    https,
                    marker: input.marker.unwrap_or_else(|| "FUZZ".to_owned()),
                    inputs: input.wordlist.unwrap_or_default(),
                    payload_list_id: input.payload_list_id.unwrap_or_default(),
                    payload_offset: input.payload_offset.unwrap_or(0),
                    attack_mode: input.attack_mode.unwrap_or_else(|| "pitchfork".to_owned()),
                    marker_payloads,
                })
                .await,
        )
    }
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

    async fn intruder_payload_processor_list(&self) -> String {
        match self.client.list_payload_processors(ListPayloadProcessorsRequest {}).await {
            Ok(response) => serde_json::json!({"items": response.items.iter().map(|item| serde_json::json!({"id": item.id, "display_name": item.display_name, "operation": item.operation, "registered": item.registered})).collect::<Vec<_>>()}).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

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

    async fn intruder_payload_generator_list(&self) -> String {
        match self.client.list_payload_generators(ListPayloadGeneratorsRequest {}).await {
            Ok(response) => serde_json::json!({"items": response.items.iter().map(|item| serde_json::json!({"id": item.id, "display_name": item.display_name, "payload_count": item.payload_count, "max_output_count": item.max_output_count, "registered": item.registered})).collect::<Vec<_>>()}).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

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
    async fn payload_list_list(&self) -> String {
        match self.client.list_payload_lists(ListPayloadListsRequest {}).await {
            Ok(response) => serde_json::json!({"items": response.items.iter().map(payload_list_proto_json).collect::<Vec<_>>()}).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    async fn payload_list_get(&self, Parameters(input): Parameters<GetPayloadListInput>) -> String {
        match self.client.get_payload_list(GetPayloadListRequest {
            id: input.id,
            page: Some(PageRequest { limit: input.limit.unwrap_or(100).min(MAX_PAGE_SIZE), cursor: input.offset.unwrap_or(0).to_string() }),
        }).await {
            Ok(response) => serde_json::json!({"list": response.list.map(|item| payload_list_proto_json(&item)), "payloads": response.payloads, "page": response.page.map(|page| serde_json::json!({"total": page.total, "truncated": page.truncated, "next_cursor": page.next_cursor}))}).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

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

    async fn scan_start(&self, Parameters(input): Parameters<AuditInput>) -> String {
        let audit_type = input
            .audit_type
            .as_deref()
            .unwrap_or("passive")
            .to_lowercase();
        if !matches!(audit_type.as_str(), "passive" | "active") {
            return serde_json::json!({"error": "audit_type must be passive or active"})
                .to_string();
        }
        job_status_json(
            self.client
                .start_audit(StartAuditRequest {
                    url: input.url,
                    audit_type,
                    scan_configuration_id: input.scan_configuration_id.unwrap_or_default(),
                    resource_pool_id: input.resource_pool_id.unwrap_or_default(),
                    timeout_seconds: input.timeout_seconds.unwrap_or_default(),
                    stable_seconds: input.stable_seconds.unwrap_or_default(),
                    include_out_of_scope: input.include_out_of_scope.unwrap_or(false),
                })
                .await,
        )
    }
    async fn scan_stop(&self, Parameters(input): Parameters<JobInput>) -> String {
        job_status_json(
            self.client
                .stop_audit(CancelJobRequest { id: input.job_id })
                .await,
        )
    }

    async fn scan_config_list(&self) -> String {
        match self.client.list_scan_configurations(ListScanConfigurationsRequest {}).await {
            Ok(response) => serde_json::json!({"items": response.items.into_iter().map(scan_configuration_json).collect::<Vec<_>>() }).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    async fn scan_config_get(
        &self,
        Parameters(input): Parameters<ScanConfigurationIdInput>,
    ) -> String {
        match self
            .client
            .get_scan_configuration(GetScanConfigurationRequest { id: input.id })
            .await
        {
            Ok(value) => scan_configuration_json(value).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    async fn scan_config_create(
        &self,
        Parameters(input): Parameters<ScanConfigurationUpsertInput>,
    ) -> String {
        match self
            .client
            .create_scan_configuration(scan_configuration_request(input))
            .await
        {
            Ok(value) => scan_configuration_json(value).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    async fn scan_config_update(
        &self,
        Parameters(input): Parameters<ScanConfigurationUpsertInput>,
    ) -> String {
        if input.id.as_deref().unwrap_or_default().is_empty() {
            return serde_json::json!({"error": "id is required"}).to_string();
        }
        match self
            .client
            .update_scan_configuration(scan_configuration_request(input))
            .await
        {
            Ok(value) => scan_configuration_json(value).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    async fn scan_config_delete(
        &self,
        Parameters(input): Parameters<ScanConfigurationIdInput>,
    ) -> String {
        action_json(
            self.client
                .delete_scan_configuration(DeleteScanConfigurationRequest { id: input.id })
                .await,
        )
    }

    async fn scan_pool_list(&self) -> String {
        match self.client.list_scan_resource_pools(ListScanResourcePoolsRequest {}).await {
            Ok(response) => serde_json::json!({"items": response.items.into_iter().map(scan_pool_json).collect::<Vec<_>>(), "scanner_supported": response.scanner_supported, "support_message": response.support_message}).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    async fn scan_pool_get(
        &self,
        Parameters(input): Parameters<ScanResourcePoolIdInput>,
    ) -> String {
        match self
            .client
            .get_scan_resource_pool(GetScanResourcePoolRequest { id: input.id })
            .await
        {
            Ok(value) => scan_pool_json(value).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    async fn scan_pool_create(
        &self,
        Parameters(input): Parameters<ScanResourcePoolUpsertInput>,
    ) -> String {
        match self
            .client
            .create_scan_resource_pool(scan_pool_request(input))
            .await
        {
            Ok(value) => scan_pool_json(value).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    async fn scan_pool_update(
        &self,
        Parameters(input): Parameters<ScanResourcePoolUpsertInput>,
    ) -> String {
        if input.id.as_deref().unwrap_or_default().is_empty() {
            return serde_json::json!({"error": "id is required"}).to_string();
        }
        match self
            .client
            .update_scan_resource_pool(scan_pool_request(input))
            .await
        {
            Ok(value) => scan_pool_json(value).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    async fn scan_pool_delete(
        &self,
        Parameters(input): Parameters<ScanResourcePoolIdInput>,
    ) -> String {
        action_json(
            self.client
                .delete_scan_resource_pool(DeleteScanResourcePoolRequest { id: input.id })
                .await,
        )
    }

    async fn scan_remove(&self, Parameters(input): Parameters<JobInput>) -> String {
        action_json(
            self.client
                .remove_audit(CancelJobRequest { id: input.job_id })
                .await,
        )
    }

    async fn crawl(&self, Parameters(input): Parameters<CrawlInput>) -> String {
        job_status_json(
            self.client
                .start_crawl(StartCrawlRequest {
                    seed_urls: input.seed_urls,
                    scan_configuration_id: input.scan_configuration_id.unwrap_or_default(),
                    resource_pool_id: input.resource_pool_id.unwrap_or_default(),
                    timeout_seconds: input.timeout_seconds.unwrap_or_default(),
                    stable_seconds: input.stable_seconds.unwrap_or_default(),
                    include_out_of_scope: input.include_out_of_scope.unwrap_or(false),
                })
                .await,
        )
    }

    async fn collaborator_generate(
        &self,
        Parameters(input): Parameters<CollaboratorGenerateInput>,
    ) -> String {
        match self
            .client
            .generate_collaborator_payloads(GenerateCollaboratorPayloadsRequest {
                count: input.count.unwrap_or(1),
                target_url: input.target_url.unwrap_or_default(),
                injection_point: input.injection_point.unwrap_or_default(),
            })
            .await
        {
            Ok(response) => serde_json::json!({"payloads": response.payloads}).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

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

    async fn websocket_history(
        &self,
        Parameters(input): Parameters<ManagedWebSocketHistoryInput>,
    ) -> String {
        let limit = input.limit.unwrap_or(100);
        if limit > MAX_PAGE_SIZE {
            return serde_json::json!({"error": "limit must be at most 500"}).to_string();
        }
        let include_bodies = input.include_bodies.unwrap_or(false);
        let effective_max_length = Some(input.max_body_length.unwrap_or(DEFAULT_MAX_BODY_LENGTH));
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
                serde_json::to_string(&ManagedWebSocketHistoryOutput {
                    items: response
                        .items
                        .into_iter()
                        .map(|item| {
                            let payload_len = item.payload.len();
                            let (payload_b64, payload_trunc) =
                                if include_bodies && !item.payload.is_empty() {
                                    let (b64, trunc, _) =
                                        encode_bounded_base64(&item.payload, effective_max_length);
                                    (Some(b64), trunc.then_some(true))
                                } else {
                                    (None, None)
                                };
                            ManagedWebSocketHistoryItemOutput {
                                index: item.index,
                                websocket_id: item.websocket_id,
                                direction: item.direction,
                                r#type: item.r#type,
                                length: payload_len,
                                payload: payload_b64,
                                payload_truncated: payload_trunc,
                            }
                        })
                        .collect(),
                    total: page.total,
                    truncated: page.truncated,
                    next_cursor: (!page.next_cursor.is_empty()).then_some(page.next_cursor),
                })
                .expect("managed websocket history must serialize")
            }
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

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

    async fn websocket_close(&self, Parameters(input): Parameters<WebSocketIdInput>) -> String {
        action_json(
            self.client
                .close_websocket(CloseWebSocketRequest { id: input.id })
                .await,
        )
    }

    async fn websocket_list(&self) -> String {
        match self.client.list_websockets(ListWebSocketsRequest {}).await {
            Ok(response) => serde_json::json!({"websockets": response.ids.into_iter().map(|id| serde_json::json!({"id": id})).collect::<Vec<_>>()}).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

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
                        "id": item.id,
                        "type": item.r#type,
                        "client_ip": item.client_ip,
                        "client_port": item.client_port,
                        "timestamp": item.timestamp,
                        "target_url": (!item.target_url.is_empty()).then_some(item.target_url),
                        "injection_point": (!item.injection_point.is_empty()).then_some(item.injection_point),
                        "payload": (!item.payload.is_empty()).then_some(item.payload),
                    })).collect::<Vec<_>>(),
                    "count": page.total,
                    "truncated": page.truncated,
                    "next_cursor": (!page.next_cursor.is_empty()).then_some(page.next_cursor),
                })
                .to_string()
            }
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    async fn diff_responses(&self, Parameters(input): Parameters<DiffResponsesInput>) -> String {
        let text_a = if let Some(a) = input.response_a {
            a
        } else if let Some(idx) = input.index_a {
            match self
                .client
                .proxy_detail(ProxyDetailRequest { index: idx })
                .await
            {
                Ok(detail) => String::from_utf8_lossy(&detail.response).into_owned(),
                Err(err) => {
                    return serde_json::json!({"error": format!("failed to fetch index_a: {err}")})
                        .to_string();
                }
            }
        } else {
            return serde_json::json!({"error": "either response_a or index_a must be provided"})
                .to_string();
        };

        let text_b = if let Some(b) = input.response_b {
            b
        } else if let Some(idx) = input.index_b {
            match self
                .client
                .proxy_detail(ProxyDetailRequest { index: idx })
                .await
            {
                Ok(detail) => String::from_utf8_lossy(&detail.response).into_owned(),
                Err(err) => {
                    return serde_json::json!({"error": format!("failed to fetch index_b: {err}")})
                        .to_string();
                }
            }
        } else {
            return serde_json::json!({"error": "either response_b or index_b must be provided"})
                .to_string();
        };

        let result = diff_engine::compare_http_messages(&text_a, &text_b);
        serde_json::to_string(&result).expect("diff result must serialize")
    }

    async fn send_to_comparer(&self, Parameters(input): Parameters<SendToComparerInput>) -> String {
        match self
            .client
            .send_to_comparer(SendToComparerRequest {
                first: input.first.into_bytes(),
                second: input.second.into_bytes(),
            })
            .await
        {
            Ok(res) => {
                serde_json::json!({"success": res.success, "message": res.message}).to_string()
            }
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_job_status",
        description = "Get the current state of a Burp background job",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
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
        description = "Import a complete Bambda YAML document with id, name, function, location, and source; does not execute it. Bambda is JVM-compiled: do not embed large payloads or string literals near the 65,535-byte CONSTANT_Utf8 limit; use a proxy rule or external streaming proxy instead",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
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
        name = "burp_proxy",
        description = "Burp Proxy tool (actions: history, detail, annotate, highlight, extract)",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn proxy(&self, Parameters(input): Parameters<suite::ProxyActionInput>) -> String {
        match input.action {
            suite::ProxyAction::History => {
                self.proxy_history(Parameters(ProxyHistoryInput {
                    limit: input.limit,
                    offset: input.offset,
                    cursor: input.cursor,
                    url_filter: input.url_filter,
                    method_filter: input.method_filter,
                    status_filter: input.status_filter,
                    has_notes: input.has_notes,
                    color: input.color,
                    include_bodies: input.include_bodies,
                    headers_only: input.headers_only,
                    extract_css: input.extract_css,
                    extract_json: input.extract_json,
                    max_body_length: input.max_body_length,
                }))
                .await
            }
            suite::ProxyAction::Detail => {
                let index = input.index.unwrap_or(0);
                self.proxy_detail(Parameters(ProxyDetailInput {
                    index,
                    include_bodies: input.include_bodies,
                    headers_only: input.headers_only,
                    extract_css: input.extract_css,
                    extract_json: input.extract_json,
                    max_body_length: input.max_body_length,
                }))
                .await
            }
            suite::ProxyAction::Annotate => {
                let index = input.index.unwrap_or(0);
                let note = input.notes.unwrap_or_default();
                self.annotate(Parameters(AnnotateInput { index, note }))
                    .await
            }
            suite::ProxyAction::Highlight => {
                let index = input.index.unwrap_or(0);
                self.highlight(Parameters(HighlightInput {
                    index,
                    color: input.color,
                }))
                .await
            }
            suite::ProxyAction::Extract => {
                let index = input.index.unwrap_or(0);
                let regex = input.regex.unwrap_or_default();
                self.extract_from_response(Parameters(ExtractResponseInput {
                    index,
                    regex,
                    limit: input.limit.map(|l| l as usize),
                }))
                .await
            }
            suite::ProxyAction::WebsocketHistory => {
                self.proxy_websocket_history(Parameters(ProxyWebSocketHistoryInput {
                    limit: input.limit,
                    cursor: input.cursor,
                    include_bodies: input.include_bodies,
                    max_body_length: input.max_body_length,
                }))
                .await
            }
        }
    }

    #[tool(
        name = "burp_http",
        description = "Burp HTTP client (actions: send, send_batch, convert, export, send_to_repeater)",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn http(&self, Parameters(input): Parameters<suite::HttpActionInput>) -> String {
        match input.action {
            suite::HttpAction::Send => {
                let url = input.url.unwrap_or_default();
                self.send_request(Parameters(SendRequestInput {
                    method: input.method,
                    url,
                    body: input.body,
                    headers: input.headers,
                    headers_only: input.headers_only,
                    extract_css: input.extract_css,
                    extract_json: input.extract_json,
                    max_body_length: input.max_body_length,
                }))
                .await
            }
            suite::HttpAction::SendBatch => {
                let requests = input.requests.unwrap_or_default();
                self.send_request_parallel(Parameters(SendRequestsInput { requests }))
                    .await
            }
            suite::HttpAction::Convert => {
                let request = input.request.unwrap_or_default();
                self.convert_request(Parameters(ConvertRequestInput {
                    request,
                    convert_to: input.convert_to,
                }))
                .await
            }
            suite::HttpAction::Export => {
                let request = input.request.unwrap_or_default();
                self.export_request(Parameters(ExportRequestInput {
                    request,
                    host: input.host,
                    format: input.format,
                    https: input.https,
                }))
                .await
            }
            suite::HttpAction::SendToRepeater => {
                let url = input.url.as_deref();
                let method = input.method.as_deref();
                let body = input.body.as_deref();
                let headers = input.headers.as_ref();
                let repeater_input = SendToRepeaterInput {
                    request: input.request.unwrap_or_default(),
                    host: input.host.unwrap_or_default(),
                    port: input.port,
                    https: input.https,
                    tab_name: input.tab_name,
                };
                match repeater_input_from_http_action(repeater_input, url, method, body, headers) {
                    Ok(request) => self.send_to_repeater(Parameters(request)).await,
                    Err(message) => tool_input_error("burp_http", "send_to_repeater", message),
                }
            }
        }
    }

    #[tool(
        name = "burp_target",
        description = "Burp Target tool (actions: get_scope, add_scope, remove_scope, info, sitemap)",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn target(&self, Parameters(input): Parameters<suite::TargetActionInput>) -> String {
        match input.action {
            suite::TargetAction::GetScope => {
                let url = input.url.unwrap_or_default();
                self.scope_check(Parameters(ScopeCheckInput { url })).await
            }
            suite::TargetAction::AddScope => {
                let url = input.url.unwrap_or_default();
                self.add_to_scope(Parameters(ScopeMutationInput { url }))
                    .await
            }
            suite::TargetAction::RemoveScope => {
                let url = input.url.unwrap_or_default();
                self.remove_from_scope(Parameters(ScopeMutationInput { url }))
                    .await
            }
            suite::TargetAction::Info => {
                self.target_info(Parameters(TargetInfoInput {
                    url: input.url_prefix,
                    limit: input.limit,
                }))
                .await
            }
            suite::TargetAction::Sitemap => {
                self.sitemap(Parameters(SitemapInput {
                    url_prefix: input.url_prefix,
                    limit: input.limit,
                    cursor: input.cursor,
                }))
                .await
            }
        }
    }

    #[tool(
        name = "burp_scanner",
        description = "Burp Scanner (actions: start_audit, start_crawl, stop, list_issues, issue_detail, update_issue, report)",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn scanner(&self, Parameters(input): Parameters<suite::ScannerActionInput>) -> String {
        match input.action {
            suite::ScannerAction::StartAudit => {
                let url = input.url.unwrap_or_default();
                self.scan_start(Parameters(AuditInput {
                    url,
                    audit_type: input.audit_type,
                    scan_configuration_id: input.scan_configuration_id,
                    resource_pool_id: input.resource_pool_id,
                    timeout_seconds: input.timeout_seconds,
                    stable_seconds: input.stable_seconds,
                    include_out_of_scope: input.include_out_of_scope,
                }))
                .await
            }
            suite::ScannerAction::StartCrawl => {
                let seed_urls = input.seed_urls.unwrap_or_default();
                self.crawl(Parameters(CrawlInput {
                    seed_urls,
                    scan_configuration_id: input.scan_configuration_id,
                    resource_pool_id: input.resource_pool_id,
                    timeout_seconds: input.timeout_seconds,
                    stable_seconds: input.stable_seconds,
                    include_out_of_scope: input.include_out_of_scope,
                }))
                .await
            }
            suite::ScannerAction::Stop => {
                let job_id = input.job_id.unwrap_or_default();
                self.scan_stop(Parameters(JobInput { job_id })).await
            }
            suite::ScannerAction::ListIssues => {
                self.scan_issues(Parameters(ScanIssuesInput {
                    limit: input.limit,
                    cursor: input.cursor,
                }))
                .await
            }
            suite::ScannerAction::IssueDetail => {
                let index = input.index.unwrap_or(0);
                self.scan_issue_detail(Parameters(ScanIssueDetailInput { index }))
                    .await
            }
            suite::ScannerAction::UpdateIssue => {
                let index = input.index.unwrap_or(0);
                let status = input.status.unwrap_or_else(|| "confirmed".to_string());
                self.update_scan_issue_status(Parameters(ScanIssueUpdateInput {
                    index,
                    status,
                    severity: input.severity,
                    confidence: input.confidence,
                    notes: input.notes,
                }))
                .await
            }
            suite::ScannerAction::Report => {
                let format = input.format.unwrap_or_else(|| "html".to_string());
                let path = input.path.unwrap_or_default();
                self.scanner_generate_report(Parameters(GenerateScannerReportInput {
                    format,
                    path,
                    issue_indexes: input.issue_indexes,
                }))
                .await
            }
            suite::ScannerAction::Remove => {
                let job_id = input.job_id.unwrap_or_default();
                self.scan_remove(Parameters(JobInput { job_id })).await
            }
            suite::ScannerAction::TestBcheck => {
                self.test_bcheck(Parameters(BCheckTestInput {
                    script: input.script.unwrap_or_default(),
                    request: input.request.unwrap_or_default(),
                    response: input.response,
                    host: input.host,
                    port: input.port,
                    https: input.https,
                }))
                .await
            }
        }
    }

    #[tool(
        name = "burp_fuzzer",
        description = "Burp Fuzzer & Intruder tool (actions: fuzz, race, send_to_intruder, list_payloads, upsert_payloads)",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn fuzzer(&self, Parameters(input): Parameters<suite::FuzzerActionInput>) -> String {
        match input.action {
            suite::FuzzerAction::Fuzz => {
                let template = input.template.unwrap_or_default();
                let host = input.host.unwrap_or_default();
                self.inline_fuzzer(Parameters(BoundedInputMatrixInput {
                    template,
                    host,
                    port: input.port,
                    https: input.https,
                    marker: input.marker,
                    wordlist: input.wordlist,
                    payload_list_id: input.payload_list_id,
                    payload_offset: input.payload_offset,
                    attack_mode: input.attack_mode,
                    markers: input.markers,
                }))
                .await
            }
            suite::FuzzerAction::Race => {
                let request = input.request.unwrap_or_default();
                let host = input.host.unwrap_or_default();
                self.race_condition(Parameters(ConcurrentRequestCheckInput {
                    request,
                    host,
                    port: input.port,
                    https: input.https,
                    count: input.count,
                    single_packet_attack: input.single_packet_attack,
                }))
                .await
            }
            suite::FuzzerAction::SendToIntruder => {
                let request = input.request.unwrap_or_default();
                let host = input.host.unwrap_or_default();
                self.send_to_intruder(Parameters(SendToIntruderInput {
                    request,
                    host,
                    port: input.port,
                    https: input.https,
                    tab_name: input.tab_name,
                }))
                .await
            }
            suite::FuzzerAction::ListPayloads => self.payload_list_list().await,
            suite::FuzzerAction::GetPayloadList => {
                let id = input.id.unwrap_or_default();
                self.payload_list_get(Parameters(GetPayloadListInput {
                    id,
                    limit: input.count,
                    offset: input.payload_offset,
                }))
                .await
            }
            suite::FuzzerAction::CreatePayloadList => {
                let id = input.id.unwrap_or_else(|| "default".to_string());
                let display_name = input.name.unwrap_or_else(|| id.clone());
                let payloads = input.payloads.unwrap_or_default();
                self.payload_list_create(Parameters(CreatePayloadListInput {
                    id,
                    display_name,
                    payloads,
                }))
                .await
            }
            suite::FuzzerAction::ImportPayloadList => {
                let id = input.id.unwrap_or_else(|| "imported".to_string());
                let display_name = input.name.unwrap_or_else(|| id.clone());
                let content = input.template.unwrap_or_default();
                self.payload_list_import(Parameters(ImportPayloadListInput {
                    id,
                    display_name,
                    content,
                    format: input.attack_mode,
                    keep_empty: None,
                }))
                .await
            }
            suite::FuzzerAction::DeletePayloadList => {
                let id = input.id.unwrap_or_default();
                self.payload_list_delete(Parameters(PayloadListIdInput { id }))
                    .await
            }
            suite::FuzzerAction::UpsertPayloads => {
                let id = input.id.unwrap_or_default();
                let name = input.name.unwrap_or_else(|| id.clone());
                let payloads = input.payloads.unwrap_or_default();
                self.payload_list_update(Parameters(UpdatePayloadListInput {
                    id,
                    operation: "replace_all".to_string(),
                    payloads: Some(payloads),
                    index: None,
                    indexes: None,
                    display_name: Some(name),
                }))
                .await
            }
            suite::FuzzerAction::RegisterPayloadProcessor => {
                let id = input.id.unwrap_or_default();
                let display_name = input.name.unwrap_or_else(|| id.clone());
                let operation = input.attack_mode.unwrap_or_else(|| "prefix".to_string());
                self.intruder_payload_processor_register(Parameters(
                    RegisterPayloadProcessorInput {
                        id,
                        display_name,
                        operation,
                        argument: input.marker,
                        replacement: input.template,
                    },
                ))
                .await
            }
            suite::FuzzerAction::ListPayloadProcessors => {
                self.intruder_payload_processor_list().await
            }
            suite::FuzzerAction::RemovePayloadProcessor => {
                let id = input.id.unwrap_or_default();
                self.intruder_payload_processor_remove(Parameters(PayloadRegistrationInput { id }))
                    .await
            }
            suite::FuzzerAction::RegisterPayloadGenerator => {
                let id = input.id.unwrap_or_default();
                let display_name = input.name.unwrap_or_else(|| id.clone());
                let payloads = input.payloads.unwrap_or_default();
                self.intruder_payload_generator_register(Parameters(
                    RegisterPayloadGeneratorInput {
                        id,
                        display_name,
                        payloads,
                        payload_list_id: input.payload_list_id,
                        payload_offset: input.payload_offset,
                        max_output_count: input.count,
                    },
                ))
                .await
            }
            suite::FuzzerAction::ListPayloadGenerators => {
                self.intruder_payload_generator_list().await
            }
            suite::FuzzerAction::RemovePayloadGenerator => {
                let id = input.id.unwrap_or_default();
                self.intruder_payload_generator_remove(Parameters(PayloadRegistrationInput { id }))
                    .await
            }
        }
    }
    #[tool(
        name = "burp_collaborator",
        description = "Burp Collaborator tool (actions: generate, poll, correlate)",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn collaborator(
        &self,
        Parameters(input): Parameters<suite::CollaboratorActionInput>,
    ) -> String {
        match input.action {
            suite::CollaboratorAction::Generate | suite::CollaboratorAction::Correlate => {
                self.collaborator_generate(Parameters(CollaboratorGenerateInput {
                    count: input.count,
                    target_url: input.target_url,
                    injection_point: input.injection_point,
                }))
                .await
            }
            suite::CollaboratorAction::Poll => {
                self.collaborator_poll(Parameters(CollaboratorPollInput {
                    limit: input.limit,
                    cursor: input.cursor,
                }))
                .await
            }
        }
    }

    #[tool(
        name = "burp_diff",
        description = "Response Comparer & Diff engine (actions: compare_exchanges, diff_responses)",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn diff(&self, Parameters(input): Parameters<suite::DiffActionInput>) -> String {
        match input.action {
            suite::DiffAction::DiffResponses => {
                self.diff_responses(Parameters(DiffResponsesInput {
                    response_a: input.response_a,
                    response_b: input.response_b,
                    index_a: input.index_a,
                    index_b: input.index_b,
                }))
                .await
            }
            suite::DiffAction::CompareExchanges => {
                let first = input.first.or(input.response_a).unwrap_or_default();
                let second = input.second.or(input.response_b).unwrap_or_default();
                self.send_to_comparer(Parameters(SendToComparerInput { first, second }))
                    .await
            }
        }
    }

    #[tool(
        name = "burp_scan_config",
        description = "Burp Scanner configurations and resource pools manager",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn scan_config(
        &self,
        Parameters(input): Parameters<suite::ScanConfigActionInput>,
    ) -> String {
        match input.action {
            suite::ScanConfigAction::ListConfigs => self.scan_config_list().await,
            suite::ScanConfigAction::GetConfig => {
                let id = input.id.unwrap_or_default();
                self.scan_config_get(Parameters(ScanConfigurationIdInput { id }))
                    .await
            }
            suite::ScanConfigAction::UpsertConfig => {
                let upsert = ScanConfigurationUpsertInput {
                    id: input.id,
                    name: input.name.unwrap_or_default(),
                    scan_type: input.scan_type.unwrap_or_else(|| "audit".to_string()),
                    audit_type: input.audit_type,
                    include_out_of_scope: input.include_out_of_scope,
                    timeout_seconds: input.timeout_seconds,
                    stable_seconds: input.stable_seconds,
                    resource_pool_id: input.resource_pool_id,
                };
                if upsert.id.is_some() {
                    self.scan_config_update(Parameters(upsert)).await
                } else {
                    self.scan_config_create(Parameters(upsert)).await
                }
            }
            suite::ScanConfigAction::DeleteConfig => {
                let id = input.id.unwrap_or_default();
                self.scan_config_delete(Parameters(ScanConfigurationIdInput { id }))
                    .await
            }
            suite::ScanConfigAction::ListPools => self.scan_pool_list().await,
            suite::ScanConfigAction::GetPool => {
                let id = input.id.unwrap_or_default();
                self.scan_pool_get(Parameters(ScanResourcePoolIdInput { id }))
                    .await
            }
            suite::ScanConfigAction::UpsertPool => {
                let upsert = ScanResourcePoolUpsertInput {
                    id: input.id,
                    name: input.name.unwrap_or_default(),
                    kind: input.kind.unwrap_or_else(|| "custom".to_string()),
                    existing_pool_name: input.existing_pool_name,
                    concurrent_request_limit: input.concurrent_request_limit,
                    throttle_millis: input.throttle_millis,
                    max_retries: input.max_retries,
                };
                if upsert.id.is_some() {
                    self.scan_pool_update(Parameters(upsert)).await
                } else {
                    self.scan_pool_create(Parameters(upsert)).await
                }
            }
            suite::ScanConfigAction::DeletePool => {
                let id = input.id.unwrap_or_default();
                self.scan_pool_delete(Parameters(ScanResourcePoolIdInput { id }))
                    .await
            }
        }
    }

    #[tool(
        name = "burp_websocket",
        description = "Burp WebSocket management tool (actions: create, send_text, send_binary, history, close, list)",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn websocket(
        &self,
        Parameters(input): Parameters<suite::WebSocketActionInput>,
    ) -> String {
        match input.action {
            suite::WebSocketAction::Create => {
                let host = input.host.unwrap_or_default();
                self.websocket_create(Parameters(WebSocketCreateInput {
                    host,
                    port: input.port,
                    https: input.https,
                    path: input.path,
                }))
                .await
            }
            suite::WebSocketAction::SendText => {
                let id = input.id.unwrap_or_default();
                let text = input.text.unwrap_or_default();
                self.websocket_send_text(Parameters(WebSocketTextInput { id, text }))
                    .await
            }
            suite::WebSocketAction::SendBinary => {
                let id = input.id.unwrap_or_default();
                let data = input.data.unwrap_or_default();
                self.websocket_send_binary(Parameters(WebSocketBinaryInput { id, data }))
                    .await
            }
            suite::WebSocketAction::History => {
                self.websocket_history(Parameters(ManagedWebSocketHistoryInput {
                    id: input.id,
                    limit: input.limit,
                    cursor: input.cursor,
                    include_bodies: input.include_bodies,
                    max_body_length: input.max_body_length,
                }))
                .await
            }
            suite::WebSocketAction::Close => {
                let id = input.id.unwrap_or_default();
                self.websocket_close(Parameters(WebSocketIdInput { id }))
                    .await
            }
            suite::WebSocketAction::List => self.websocket_list().await,
        }
    }

    #[tool(
        name = "burp_session",
        description = "Burp Session Rules & Macros tool (actions: list_rules, get_rule, upsert_rule, delete_rule, run_macro, upsert_macro, list_macros, delete_macro)",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn session(&self, Parameters(input): Parameters<suite::SessionActionInput>) -> String {
        match input.action {
            suite::SessionAction::ListRules => self.session_list_rules().await,
            suite::SessionAction::GetRule => {
                let id = input.id.unwrap_or_default();
                self.session_get_rule(Parameters(SessionRuleIdInput { id }))
                    .await
            }
            suite::SessionAction::UpsertRule => {
                let upsert = SessionRuleUpsertInput {
                    id: input.id,
                    description: input.description,
                    action_type: input.action_type,
                    find: input.find,
                    replace: input.replace,
                    header_name: input.header_name,
                    parameter_name: input.parameter_name,
                    macro_description: input.macro_description,
                    url_contains: input.url_contains,
                    tools: input.tools,
                    enabled: input.enabled,
                };
                if upsert.id.is_some() {
                    self.session_update_rule(Parameters(upsert)).await
                } else {
                    self.session_create_rule(Parameters(upsert)).await
                }
            }
            suite::SessionAction::DeleteRule => {
                let id = input.id.unwrap_or_default();
                self.session_delete_rule(Parameters(SessionRuleIdInput { id }))
                    .await
            }
            suite::SessionAction::RunMacro => {
                let description = input.description.unwrap_or_default();
                self.macro_run(Parameters(MacroDescriptionInput { description }))
                    .await
            }
            suite::SessionAction::UpsertMacro => {
                let description = input.description.unwrap_or_default();
                let items = input.items.unwrap_or_default();
                self.macro_create(Parameters(CreateMacroInput {
                    description,
                    serial_number: input.serial_number,
                    items,
                }))
                .await
            }
            suite::SessionAction::ListMacros => self.macro_list().await,
            suite::SessionAction::DeleteMacro => {
                let description = input.description.unwrap_or_default();
                self.macro_remove(Parameters(MacroDescriptionInput { description }))
                    .await
            }
        }
    }

    #[tool(
        name = "burp_settings",
        description = "Burp Proxy Settings & Configuration tool",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn settings(&self, Parameters(input): Parameters<suite::SettingsActionInput>) -> String {
        match input {
            suite::SettingsActionInput::GetProxySettings => self.proxy_settings().await,
            suite::SettingsActionInput::UpdateProxySettings {
                operation,
                port,
                running,
                listen_mode,
                listen_specific_address,
                certificate_mode,
                enable_http2,
                support_invisible_proxying,
                target,
                mode,
                script,
                script_id,
                script_name,
                kind,
                index,
                rule,
                master_enabled,
                request_enabled,
                response_enabled,
            } => {
                let op = operation.unwrap_or_else(|| "intercept_toggle".to_string());
                self.update_proxy_settings(Parameters(ProxySettingsUpdateInput {
                    operation: op,
                    port,
                    running,
                    listen_mode,
                    listen_specific_address,
                    certificate_mode,
                    enable_http2,
                    support_invisible_proxying,
                    target,
                    mode,
                    script,
                    script_id,
                    script_name,
                    kind,
                    index,
                    rule,
                    master_enabled,
                    request_enabled,
                    response_enabled,
                }))
                .await
            }
            suite::SettingsActionInput::ExportConfig => self.export_config().await,
            suite::SettingsActionInput::InspectConfig { paths } => {
                self.inspect_config(Parameters(InspectConfigInput { paths }))
                    .await
            }
            suite::SettingsActionInput::ImportConfig { config } => {
                let config = config.unwrap_or_default();
                self.import_config(Parameters(ImportConfigInput { config }))
                    .await
            }
            suite::SettingsActionInput::InterceptState => self.intercept_state().await,
            suite::SettingsActionInput::SetInterceptState { enabled } => {
                let enabled = enabled.unwrap_or(false);
                self.set_intercept_state(Parameters(SetInterceptStateInput { enabled }))
                    .await
            }
            suite::SettingsActionInput::ProxyInterceptConfig => self.proxy_intercept_config().await,
            suite::SettingsActionInput::UpdateProxyInterceptConfig {
                master_enabled,
                request_enabled,
                response_enabled,
            } => {
                self.update_proxy_intercept_config(Parameters(ProxyInterceptConfigInput {
                    master_intercept_enabled: master_enabled,
                    request_do_intercept: request_enabled,
                    request_auto_content_length: None,
                    request_fix_missing_new_lines: None,
                    request_rules: None,
                    replace_request_rules: None,
                    response_do_intercept: response_enabled,
                    response_auto_content_length: None,
                    response_rules: None,
                    replace_response_rules: None,
                    response_unhide_hidden_fields: None,
                    response_enable_disabled_fields: None,
                    response_remove_input_length_limits: None,
                    response_remove_javascript_validation: None,
                    response_remove_all_javascript: None,
                    websocket_client_to_server: None,
                    websocket_server_to_client: None,
                    websocket_in_scope_only: None,
                }))
                .await
            }
            suite::SettingsActionInput::RegisterHttpHandler {
                header_name,
                header_value,
                match_text,
                replace,
            } => {
                self.register_http_handler(Parameters(RegisterHttpHandlerInput {
                    header_name,
                    header_value,
                    match_text,
                    replace,
                }))
                .await
            }
            suite::SettingsActionInput::RemoveHttpHandler => self.remove_http_handler().await,
            suite::SettingsActionInput::RegisterProxyRule {
                id,
                url_contains,
                phase,
                rule_action,
                match_text,
                replace,
                header_name,
                header_value,
                enabled,
            } => {
                if url_contains.trim().is_empty() {
                    return tool_input_error(
                        "burp_settings",
                        "register_proxy_rule",
                        "`url_contains` is required and must not be empty",
                    );
                }
                self.register_proxy_rule(Parameters(RegisterProxyRuleInput {
                    id,
                    url_contains,
                    phase,
                    rule_action,
                    match_text,
                    replace,
                    header_name,
                    header_value,
                    enabled,
                }))
                .await
            }
            suite::SettingsActionInput::ListProxyRules => self.list_proxy_rules().await,
            suite::SettingsActionInput::RemoveProxyRule { id } => {
                self.remove_proxy_rule(Parameters(RemoveProxyRuleInput { id }))
                    .await
            }
        }
    }

    #[tool(
        name = "burp_logger",
        description = "Burp Logger tool across all tools (actions: query, detail, clear)",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn logger(&self, Parameters(input): Parameters<suite::LoggerActionInput>) -> String {
        match input.action {
            suite::LoggerAction::Query => {
                self.logger_history(Parameters(LoggerHistoryInput {
                    limit: input.limit,
                    offset: input.offset,
                    cursor: input.cursor,
                    source_filter: input.source_filter,
                    url_filter: input.url_filter,
                    method_filter: input.method_filter,
                    status_filter: input.status_filter,
                    has_notes: input.has_notes,
                    color: input.color,
                    include_bodies: input.include_bodies,
                    headers_only: input.headers_only,
                    extract_css: input.extract_css,
                    extract_json: input.extract_json,
                    max_body_length: input.max_body_length,
                }))
                .await
            }
            suite::LoggerAction::Detail => {
                let index = input.index.unwrap_or(0);
                self.logger_detail(Parameters(LoggerDetailInput {
                    index,
                    include_bodies: input.include_bodies,
                    headers_only: input.headers_only,
                    extract_css: input.extract_css,
                    extract_json: input.extract_json,
                    max_body_length: input.max_body_length,
                }))
                .await
            }
            suite::LoggerAction::Clear => self.clear_logger(Parameters(LoggerClearInput {})).await,
        }
    }

    #[tool(
        name = "burp_organizer",
        description = "Burp Organizer tool (actions: add, list)",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn organizer(
        &self,
        Parameters(input): Parameters<suite::OrganizerActionInput>,
    ) -> String {
        match input.action {
            suite::OrganizerAction::Add => {
                let request = input.request.unwrap_or_default();
                let host = input.host.unwrap_or_default();
                self.organizer_send(Parameters(OrganizerSendInput {
                    request,
                    response: input.response,
                    host,
                    port: input.port,
                    https: input.https,
                    notes: input.notes,
                    highlight: input.highlight,
                }))
                .await
            }
            suite::OrganizerAction::List => {
                self.organizer_list(Parameters(OrganizerListInput {
                    limit: input.limit,
                    cursor: input.cursor,
                    status_filter: input.status_filter,
                    url_filter: input.url_filter,
                }))
                .await
            }
        }
    }

    #[tool(
        name = "sitegraph",
        description = "SiteGraph attack surface analysis tool (actions: status, sync, search, neighbors, trace, shortest_path, clusters, impact, diff, export)",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn sitegraph(
        &self,
        Parameters(input): Parameters<suite::SiteGraphActionInput>,
    ) -> String {
        match input.action {
            suite::SiteGraphAction::Status => self.sitegraph_status().await,
            suite::SiteGraphAction::Stats => self.sitegraph_stats().await,
            suite::SiteGraphAction::Sync => {
                self.sitegraph_sync(Parameters(SiteGraphSyncInput {
                    url_prefix: input.url_prefix,
                }))
                .await
            }
            suite::SiteGraphAction::Search => {
                let query = input.query.unwrap_or_default();
                self.sitegraph_search(Parameters(SiteGraphSearchInput {
                    query,
                    limit: input.limit,
                    cursor: input.cursor,
                }))
                .await
            }
            suite::SiteGraphAction::SecurityView => {
                let Some(sitegraph) = &self.sitegraph else {
                    return sitegraph_disabled_json();
                };
                let view = input
                    .view_name
                    .unwrap_or_else(|| "unauthenticated".to_string());
                let limit = input.limit.unwrap_or(50) as usize;
                match sitegraph.graph.security_view(&view, limit).await {
                    Ok(res) => serde_json::to_string(&res).expect("security view serializes"),
                    Err(err) => serde_json::json!({"error": err.to_string()}).to_string(),
                }
            }
            suite::SiteGraphAction::ImportSpec => {
                let Some(sitegraph) = &self.sitegraph else {
                    return sitegraph_disabled_json();
                };
                let spec = input.spec_content.unwrap_or_default();
                let base_url = input
                    .url_prefix
                    .unwrap_or_else(|| "https://localhost".to_string());
                match sitegraph.graph.import_openapi(&spec, &base_url).await {
                    Ok(summary) => serde_json::to_string(&summary).expect("summary serializes"),
                    Err(err) => serde_json::json!({"error": err.to_string()}).to_string(),
                }
            }
            suite::SiteGraphAction::Neighbors => {
                let id = input.id.unwrap_or_default();
                self.sitegraph_neighbors(Parameters(SiteGraphNeighborsInput {
                    id,
                    limit: input.limit,
                    cursor: input.cursor,
                }))
                .await
            }
            suite::SiteGraphAction::Trace => {
                let id = input.id.unwrap_or_default();
                self.sitegraph_trace(Parameters(SiteGraphTraceInput {
                    id,
                    max_depth: input.max_depth,
                    limit: input.limit,
                }))
                .await
            }
            suite::SiteGraphAction::ShortestPath => {
                let from_id = input.from_id.unwrap_or_default();
                let to_id = input.to_id.unwrap_or_default();
                self.sitegraph_shortest_path(Parameters(SiteGraphShortestPathInput {
                    from_id,
                    to_id,
                    max_depth: input.max_depth,
                }))
                .await
            }
            suite::SiteGraphAction::Clusters => {
                self.sitegraph_clusters(Parameters(SiteGraphClustersInput { limit: input.limit }))
                    .await
            }
            suite::SiteGraphAction::Impact => {
                let id = input.id.unwrap_or_default();
                self.sitegraph_impact(Parameters(SiteGraphImpactInput {
                    id,
                    max_depth: input.max_depth,
                    limit: input.limit,
                }))
                .await
            }
            suite::SiteGraphAction::Diff => {
                let since = input.since.unwrap_or(0);
                self.sitegraph_diff(Parameters(SiteGraphDiffInput {
                    since,
                    limit: input.limit,
                    cursor: input.cursor,
                }))
                .await
            }
            suite::SiteGraphAction::Export => {
                self.sitegraph_export(Parameters(SiteGraphExportInput {
                    profile: input.profile,
                    format: input.format,
                    snapshot_id: input.snapshot_id,
                    cursor: input.cursor,
                    limit: input.limit,
                }))
                .await
            }
            suite::SiteGraphAction::HistorySearch => {
                let query = input.query.unwrap_or_default();
                self.sitegraph_history_search(Parameters(SiteGraphHistorySearchInput {
                    query,
                    source: input.profile,
                    limit: input.limit,
                    cursor: input.cursor,
                }))
                .await
            }
            suite::SiteGraphAction::EndpointDetail => {
                let id = input.id.unwrap_or_default();
                self.sitegraph_endpoint_detail(Parameters(SiteGraphEndpointInput { id }))
                    .await
            }
            suite::SiteGraphAction::Projects => self.sitegraph_projects().await,
            suite::SiteGraphAction::Config => self.sitegraph_config().await,
        }
    }

    #[tool(
        name = "burp_verify_idor",
        description = "High-level Compound Workflow to verify Insecure Direct Object Reference (IDOR) with two roles",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn verify_idor(
        &self,
        Parameters(input): Parameters<workflows::VerifyIdorInput>,
    ) -> String {
        match workflows::run_verify_idor(&self.client, input).await {
            Ok(output) => serde_json::to_string(&output).expect("idor output must serialize"),
            Err(error) => serde_json::json!({"error": error}).to_string(),
        }
    }

    #[tool(
        name = "burp_check_cors",
        description = "High-level Compound Workflow to audit CORS misconfigurations on a target URL",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn check_cors(&self, Parameters(input): Parameters<workflows::CheckCorsInput>) -> String {
        match workflows::run_check_cors(&self.client, input).await {
            Ok(output) => serde_json::to_string(&output).expect("cors output must serialize"),
            Err(error) => serde_json::json!({"error": error}).to_string(),
        }
    }

    #[tool(
        name = "burp_auth_matrix",
        description = "High-level Compound Workflow to run an Access Control Matrix across multiple endpoints and roles",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn auth_matrix(
        &self,
        Parameters(input): Parameters<workflows::AuthMatrixInput>,
    ) -> String {
        match workflows::run_auth_matrix(&self.client, input).await {
            Ok(output) => {
                serde_json::to_string(&output).expect("auth matrix output must serialize")
            }
            Err(error) => serde_json::json!({"error": error}).to_string(),
        }
    }

    #[tool(
        name = "burp_audit_jwt",
        description = "High-level Compound Workflow to audit JWT security (None algorithm, Key Confusion HS256, and Claim Tampering)",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn audit_jwt(&self, Parameters(input): Parameters<workflows::AuditJwtInput>) -> String {
        match workflows::run_audit_jwt(&self.client, input).await {
            Ok(output) => serde_json::to_string(&output).expect("jwt audit output must serialize"),
            Err(error) => serde_json::json!({"error": error}).to_string(),
        }
    }

    #[tool(
        name = "burp_verify_ssrf",
        description = "High-level Compound Workflow to verify Server-Side Request Forgery (SSRF) with Burp Collaborator callbacks",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn verify_ssrf(
        &self,
        Parameters(input): Parameters<workflows::VerifySsrfInput>,
    ) -> String {
        match workflows::run_verify_ssrf(&self.client, input).await {
            Ok(output) => serde_json::to_string(&output).expect("ssrf output must serialize"),
            Err(error) => serde_json::json!({"error": error}).to_string(),
        }
    }

    #[tool(
        name = "burp_verify_sqli_blind",
        description = "High-level Compound Workflow to verify Blind SQL Injection via differential response similarity and time delays",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn verify_sqli_blind(
        &self,
        Parameters(input): Parameters<workflows::VerifySqliBlindInput>,
    ) -> String {
        match workflows::run_verify_sqli_blind(&self.client, input).await {
            Ok(output) => serde_json::to_string(&output).expect("sqli output must serialize"),
            Err(error) => serde_json::json!({"error": error}).to_string(),
        }
    }

    #[tool(
        name = "burp_audit_graphql",
        description = "High-level Compound Workflow to audit GraphQL security (Introspection, Field Suggestions, and Query Batching)",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn audit_graphql(
        &self,
        Parameters(input): Parameters<workflows::AuditGraphqlInput>,
    ) -> String {
        match workflows::run_audit_graphql(&self.client, input).await {
            Ok(output) => {
                serde_json::to_string(&output).expect("graphql audit output must serialize")
            }
            Err(error) => serde_json::json!({"error": error}).to_string(),
        }
    }

    #[tool(
        name = "burp_verify_csrf_samesite",
        description = "High-level Compound Workflow to test CSRF vulnerability, check SameSite cookie flags, and generate an HTML exploit PoC",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn verify_csrf_samesite(
        &self,
        Parameters(input): Parameters<workflows::VerifyCsrfInput>,
    ) -> String {
        match workflows::run_verify_csrf_samesite(&self.client, input).await {
            Ok(output) => serde_json::to_string(&output).expect("csrf output must serialize"),
            Err(error) => serde_json::json!({"error": error}).to_string(),
        }
    }

    #[tool(
        name = "burp_api_fuzz_orchestrator",
        description = "High-level Compound Workflow to automatically fuzz an entire API attack surface from an OpenAPI/Swagger spec",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn api_fuzz_orchestrator(
        &self,
        Parameters(input): Parameters<workflows::ApiFuzzOrchestratorInput>,
    ) -> String {
        match workflows::run_api_fuzz_orchestrator(&self.client, input).await {
            Ok(output) => serde_json::to_string(&output).expect("api fuzz output must serialize"),
            Err(error) => serde_json::json!({"error": error}).to_string(),
        }
    }

    #[tool(
        name = "burp_bcheck_import",
        description = "Import a complete Burp BCheck definition with metadata and a given block; does not run it",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
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
    #[tool(
        name = "burp_job_cancel",
        description = "Cancel a Burp background job",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn job_cancel(&self, Parameters(input): Parameters<JobInput>) -> String {
        job_status_json(
            self.client
                .cancel_job(CancelJobRequest { id: input.job_id })
                .await,
        )
    }

    #[tool(
        name = "burp_job_result",
        description = "Get a bounded page of results from a Burp background job",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
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

    async fn convert_request(&self, Parameters(input): Parameters<ConvertRequestInput>) -> String {
        match convert_request_text(
            &input.request,
            input.convert_to.as_deref().unwrap_or("POST"),
        ) {
            Ok(request) => serde_json::json!({"request": request}).to_string(),
            Err(error) => serde_json::json!({"error": error}).to_string(),
        }
    }

    async fn export_request(&self, Parameters(input): Parameters<ExportRequestInput>) -> String {
        match export_request_text(input) {
            Ok(command) => serde_json::json!({"command": command}).to_string(),
            Err(error) => serde_json::json!({"error": error}).to_string(),
        }
    }

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
        description = "Add one typed issue to the Burp site map",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
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

    async fn sitegraph_sync(&self, Parameters(input): Parameters<SiteGraphSyncInput>) -> String {
        let Some(sitegraph) = &self.sitegraph else {
            return sitegraph_disabled_json();
        };
        match sitegraph
            .indexer
            .sync(input.url_prefix.unwrap_or_default())
            .await
        {
            Ok(summary) => serde_json::to_string(&summary).expect("sync summary serializes"),
            Err(error) => serde_json::json!({"error": error}).to_string(),
        }
    }
    async fn sitegraph_search(
        &self,
        Parameters(input): Parameters<SiteGraphSearchInput>,
    ) -> String {
        let limit = input.limit.unwrap_or(100);
        if limit == 0 || limit > 500 {
            return serde_json::json!({"error": "limit must be between 1 and 500"}).to_string();
        }
        match self
            .sitegraph_runtime()
            .graph
            .search(&input.query, input.cursor.unwrap_or(0) as u64, limit as u64)
            .await
        {
            Ok(result) => serde_json::to_string(&result).expect("graph page serializes"),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }
    async fn sitegraph_history_search(
        &self,
        Parameters(input): Parameters<SiteGraphHistorySearchInput>,
    ) -> String {
        let limit = input.limit.unwrap_or(100);
        if limit == 0 || limit > 500 {
            return serde_json::json!({"error": "limit must be between 1 and 500"}).to_string();
        }
        match self
            .sitegraph_runtime()
            .graph
            .search_history(
                &input.query,
                input.source.as_deref(),
                input.cursor.unwrap_or(0) as u64,
                limit as u64,
            )
            .await
        {
            Ok(result) => serde_json::to_string(&result).expect("history search page serializes"),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    async fn sitegraph_endpoint_detail(
        &self,
        Parameters(input): Parameters<SiteGraphEndpointInput>,
    ) -> String {
        match self.sitegraph_runtime().graph.endpoint(&input.id).await {
            Ok(Some(endpoint)) => serde_json::to_string(&endpoint).expect("endpoint serializes"),
            Ok(None) => serde_json::json!({"error": "endpoint not found"}).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    async fn sitegraph_status(&self) -> String {
        let Some(sitegraph) = &self.sitegraph else {
            return sitegraph_disabled_json();
        };
        match sitegraph.indexer.status().await {
            Ok(status) => serde_json::to_string(&status).expect("graph status serializes"),
            Err(error) => serde_json::json!({"error": error}).to_string(),
        }
    }

    async fn sitegraph_config(&self) -> String {
        let Some(sitegraph) = &self.sitegraph else {
            return sitegraph_disabled_json();
        };
        serde_json::json!({
            "mode": sitegraph.mode,
            "interval_seconds": sitegraph.interval_seconds,
            "page_size": 500,
            "queue_capacity": 32,
            "max_items": null,
            "note": "edit ~/.config/burp-mcp/config.toml or the selected config file and restart burp-mcp"
        })
        .to_string()
    }

    async fn sitegraph_projects(&self) -> String {
        match self.sitegraph_runtime().graph.status().await {
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

    async fn sitegraph_stats(&self) -> String {
        match self.sitegraph_runtime().graph.status().await {
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

    async fn sitegraph_neighbors(
        &self,
        Parameters(input): Parameters<SiteGraphNeighborsInput>,
    ) -> String {
        let limit = match validated_graph_limit(input.limit) {
            Ok(limit) => limit,
            Err(error) => return serde_json::json!({"error": error}).to_string(),
        };
        match self
            .sitegraph_runtime()
            .graph
            .neighbors(&input.id, input.cursor.unwrap_or(0) as u64, limit as u64)
            .await
        {
            Ok(page) => serde_json::to_string(&page).expect("neighbor page serializes"),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    async fn sitegraph_trace(&self, Parameters(input): Parameters<SiteGraphTraceInput>) -> String {
        let limit = match validated_graph_limit(input.limit) {
            Ok(limit) => limit,
            Err(error) => return serde_json::json!({"error": error}).to_string(),
        };
        let max_depth = input.max_depth.unwrap_or(4);
        if max_depth == 0 || max_depth > MAX_TRAVERSAL_DEPTH {
            return serde_json::json!({"error": "max_depth must be between 1 and 8"}).to_string();
        }
        match self
            .sitegraph_runtime()
            .graph
            .trace(&input.id, max_depth, limit)
            .await
        {
            Ok(page) => serde_json::to_string(&page).expect("trace page serializes"),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    async fn sitegraph_shortest_path(
        &self,
        Parameters(input): Parameters<SiteGraphShortestPathInput>,
    ) -> String {
        let max_depth = input.max_depth.unwrap_or(8);
        if max_depth == 0 || max_depth > 16 {
            return serde_json::json!({"error": "max_depth must be between 1 and 16"}).to_string();
        }
        match self
            .sitegraph_runtime()
            .graph
            .shortest_path(&input.from_id, &input.to_id, max_depth as usize)
            .await
        {
            Ok(result) => serde_json::to_string(&result).expect("shortest path serializes"),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    async fn sitegraph_clusters(
        &self,
        Parameters(input): Parameters<SiteGraphClustersInput>,
    ) -> String {
        let limit = match validated_graph_limit(input.limit) {
            Ok(limit) => limit,
            Err(error) => return serde_json::json!({"error": error}).to_string(),
        };
        match self
            .sitegraph_runtime()
            .graph
            .endpoint_clusters(limit as usize)
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
            .sitegraph_runtime()
            .graph
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

    async fn sitegraph_diff(&self, Parameters(input): Parameters<SiteGraphDiffInput>) -> String {
        let limit = match validated_graph_limit(input.limit) {
            Ok(limit) => limit,
            Err(error) => return serde_json::json!({"error": error}).to_string(),
        };
        match self
            .sitegraph_runtime()
            .graph
            .diff(input.since, input.cursor.unwrap_or(0) as u64, limit as u64)
            .await
        {
            Ok(diff) => serde_json::to_string(&diff).expect("graph diff serializes"),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

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
        let graph = &self.sitegraph_runtime().graph;
        match (profile, format) {
            ("metadata", "json") => match graph.export_json(cursor, limit).await {
                Ok(export) => serde_json::to_string(&export).expect("JSON graph export serializes"),
                Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
            },
            ("exact", "json") => match graph.export_exact_json(cursor, limit).await {
                Ok(export) => {
                    serde_json::to_string(&export).expect("exact graph export serializes")
                }
                Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
            },
            (_, "csv") => match graph.export_csv(cursor, limit).await {
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
        description = "List cookies in Burp cookie jar with name, value, domain, path, and expiration (optional domain filter)",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
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
        description = "Get Burp Suite version information",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
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
        name = "burp_editor_get",
        description = "Capture the active or last-focused Burp editor tab (HTTP Request/Response or WebSocket) with rich metadata, selection offsets, and UTF-8 decoded text",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn editor_get(&self, Parameters(input): Parameters<suite::EditorGetInput>) -> String {
        match self
            .client
            .editor_get(EditorGetRequest {
                target_hint: input.target_hint,
                ttl_seconds: input.ttl_seconds,
            })
            .await
        {
            Ok(s) => serde_json::to_string(&serde_json::json!({
                "token": s.token,
                "kind": s.kind,
                "tool_source": s.tool_source,
                "tab_name": s.tab_name,
                "host": s.host,
                "port": if s.port > 0 { Some(s.port) } else { None },
                "https": if s.port > 0 { Some(s.https) } else { None },
                "text": s.text,
                "payload_base64": STANDARD.encode(&s.payload),
                "is_json": s.is_json,
                "editable": s.editable,
                "sha256": s.sha256,
                "caret_position": s.caret_position,
                "selection_start": s.selection_start,
                "selection_end": s.selection_end,
                "selected_text": s.selected_text,
                "expires_at_millis": s.expires_at_millis,
            }))
            .expect("editor snapshot serializes"),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_editor_patch",
        description = "Surgically modify the active Burp editor contents (replace selection, update header, patch JSON, or regex replace) without transmitting full text payloads",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn editor_patch(&self, Parameters(input): Parameters<suite::EditorPatchInput>) -> String {
        let mode = input.mode.as_deref().unwrap_or("replace_all");
        let patch_operation = match mode {
            "replace_selection" => {
                let text = input
                    .selection_replacement
                    .or(input.text)
                    .unwrap_or_default();
                burp_protocol::protocol::editor_patch_request::PatchOperation::ReplaceSelection(
                    text,
                )
            }
            "set_header" => {
                let name = input.header_name.unwrap_or_default();
                let value = input.header_value.unwrap_or_default();
                let remove = input.header_remove.unwrap_or(false);
                burp_protocol::protocol::editor_patch_request::PatchOperation::HeaderPatch(
                    HeaderPatch {
                        name,
                        value,
                        remove,
                    },
                )
            }
            "json_patch" => {
                let json_path = input.json_path.unwrap_or_default();
                let value_json = input.json_value.or(input.text).unwrap_or_default();
                burp_protocol::protocol::editor_patch_request::PatchOperation::JsonPatch(
                    JsonPatch {
                        json_path,
                        value_json,
                    },
                )
            }
            "set_param" => {
                let name = input.param_name.unwrap_or_default();
                let value = input.param_value.unwrap_or_default();
                let remove = input.param_remove.unwrap_or(false);
                burp_protocol::protocol::editor_patch_request::PatchOperation::ParamPatch(
                    ParamPatch {
                        name,
                        value,
                        remove,
                        param_type: input.param_type,
                    },
                )
            }
            "regex" | "regex_replace" => {
                let pattern = input.regex_pattern.unwrap_or_default();
                let replacement = input.regex_replacement.unwrap_or_default();
                let replace_all = input.regex_replace_all.unwrap_or(false);
                let case_insensitive = input.regex_case_insensitive.unwrap_or(false);
                burp_protocol::protocol::editor_patch_request::PatchOperation::RegexPatch(
                    RegexPatch {
                        pattern,
                        replacement,
                        replace_all,
                        case_insensitive,
                    },
                )
            }
            _ => {
                if let Some(b64) = input.payload_base64 {
                    let bytes = match STANDARD.decode(&b64) {
                        Ok(b) => b,
                        Err(e) => {
                            return serde_json::json!({"error": format!("invalid base64: {e}")})
                                .to_string();
                        }
                    };
                    burp_protocol::protocol::editor_patch_request::PatchOperation::ReplaceAllPayload(
                        bytes,
                    )
                } else {
                    let text = input.text.unwrap_or_default();
                    burp_protocol::protocol::editor_patch_request::PatchOperation::ReplaceAllText(
                        text,
                    )
                }
            }
        };

        match self
            .client
            .editor_patch(EditorPatchRequest {
                token: input.token,
                expected_sha256: input.expected_sha256,
                patch_operation: Some(patch_operation),
            })
            .await
        {
            Ok(s) => serde_json::to_string(&serde_json::json!({
                "token": s.token,
                "kind": s.kind,
                "text": s.text,
                "payload_base64": STANDARD.encode(&s.payload),
                "sha256": s.sha256,
                "expires_at_millis": s.expires_at_millis,
            }))
            .expect("patch result serializes"),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_editor_renew_lease",
        description = "Extend the lifetime of an active Burp editor lease token",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn editor_renew_lease(
        &self,
        Parameters(input): Parameters<suite::EditorRenewInput>,
    ) -> String {
        match self
            .client
            .editor_renew_lease(EditorRenewLeaseRequest {
                token: input.token,
                extend_seconds: input.extend_seconds.unwrap_or(60),
            })
            .await
        {
            Ok(res) => serde_json::json!({"success": res.success, "new_expires_at_millis": res.new_expires_at_millis}).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }
    #[tool(
        name = "burp_cookie_jar_set",
        description = "Set one cookie in Burp's cookie jar; Montoya API does not expose cookie deletion",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
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

    async fn intercept_state(&self) -> String {
        intercept_state_json(
            self.client
                .intercept_state(InterceptStateRequest { enabled: None })
                .await,
        )
    }

    async fn set_intercept_state(
        &self,
        Parameters(input): Parameters<SetInterceptStateInput>,
    ) -> String {
        intercept_state_json(
            self.client
                .intercept_state(InterceptStateRequest {
                    enabled: Some(input.enabled),
                })
                .await,
        )
    }

    #[tool(
        name = "burp_intercept_controller",
        description = "Read or configure MCP-controlled Burp Proxy request/response interception. Enabling requires url_filter or in_scope_only=true; non-matching traffic continues without entering the queue; pending messages forward when timeout expires.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn intercept_controller(
        &self,
        Parameters(input): Parameters<InterceptControllerInput>,
    ) -> String {
        match self
            .client
            .intercept_controller_config(InterceptControllerConfigRequest {
                enabled: input.enabled,
                timeout_seconds: input.timeout_seconds,
                url_filter: input.url_filter,
                in_scope_only: input.in_scope_only,
            })
            .await
        {
            Ok(state) => serde_json::json!({
                "enabled": state.enabled,
                "timeout_seconds": state.timeout_seconds,
                "pending": state.pending,
                "url_filter": state.url_filter,
                "in_scope_only": state.in_scope_only,
            })
            .to_string(),
            Err(error) => rpc_error_json(error),
        }
    }

    #[tool(
        name = "burp_intercepted_messages",
        description = "List HTTP requests and responses currently paused by the MCP intercept controller, including lossless Base64 raw messages",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn intercepted_messages(
        &self,
        Parameters(input): Parameters<InterceptedMessagesInput>,
    ) -> String {
        let limit = input.limit.unwrap_or(100);
        if limit > MAX_PAGE_SIZE {
            return serde_json::json!({"error": "limit must be at most 500"}).to_string();
        }
        let include_bodies = input.include_bodies.unwrap_or(false);
        let max_body_length = input.max_body_length;
        match self
            .client
            .intercepted_messages(InterceptedMessagesRequest {
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
                    "items": response.items.into_iter().map(|item| intercepted_message_output(item, include_bodies, max_body_length)).collect::<Vec<_>>(),
                    "total": page.total,
                    "truncated": page.truncated,
                    "next_cursor": (!page.next_cursor.is_empty()).then_some(page.next_cursor),
                }).to_string()
            }
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_control_intercepted_message",
        description = "Forward, drop, or send an MCP-paused HTTP message to Burp's manual Intercept tab; optionally replace the complete raw request/response from standard Base64 before acting",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn control_intercepted_message(
        &self,
        Parameters(input): Parameters<ControlInterceptedMessageInput>,
    ) -> String {
        let action = match input.action {
            InterceptActionInput::Forward => InterceptAction::Forward,
            InterceptActionInput::Drop => InterceptAction::Drop,
            InterceptActionInput::Intercept => InterceptAction::Intercept,
        } as i32;
        let message = match input.message_base64 {
            Some(value) => match STANDARD.decode(value) {
                Ok(bytes) => bytes,
                Err(error) => {
                    return serde_json::json!({"error": format!("invalid message_base64: {error}")})
                        .to_string();
                }
            },
            None => Vec::new(),
        };
        let max_body_length = input.max_body_length;
        match self
            .client
            .control_intercepted_message(ControlInterceptedMessageRequest {
                id: input.id,
                action,
                message,
            })
            .await
        {
            Ok(response) => response
                .message
                .map(|msg| intercepted_message_output(msg, true, max_body_length))
                .map(|item| serde_json::to_string(&item).expect("intercept output must serialize"))
                .unwrap_or_else(|| {
                    serde_json::json!({"error": "empty intercept response"}).to_string()
                }),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }
    #[tool(
        name = "burp_websocket_intercept_controller",
        description = "Read or configure MCP-controlled Burp Proxy WebSocket interception for text and binary messages",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn websocket_intercept_controller(
        &self,
        Parameters(input): Parameters<WebSocketInterceptControllerInput>,
    ) -> String {
        match self.client.websocket_intercept_controller_config(WebSocketInterceptControllerConfigRequest {
            enabled: input.enabled,
            timeout_seconds: input.timeout_seconds,
        }).await {
            Ok(state) => serde_json::json!({"enabled": state.enabled, "timeout_seconds": state.timeout_seconds, "pending": state.pending}).to_string(),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_intercepted_websocket_messages",
        description = "List text and binary WebSocket messages currently paused by the MCP intercept controller",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn intercepted_websocket_messages(
        &self,
        Parameters(input): Parameters<InterceptedWebSocketMessagesInput>,
    ) -> String {
        let limit = input.limit.unwrap_or(100);
        if limit > MAX_PAGE_SIZE {
            return serde_json::json!({"error": "limit must be at most 500"}).to_string();
        }
        let include_bodies = input.include_bodies.unwrap_or(false);
        let max_body_length = input.max_body_length;
        match self
            .client
            .intercepted_websocket_messages(InterceptedWebSocketMessagesRequest {
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
                    "items": response.items.into_iter().map(|item| intercepted_websocket_message_output(item, include_bodies, max_body_length)).collect::<Vec<_>>(),
                    "total": page.total,
                    "truncated": page.truncated,
                    "next_cursor": (!page.next_cursor.is_empty()).then_some(page.next_cursor),
                }).to_string()
            }
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    #[tool(
        name = "burp_control_intercepted_websocket_message",
        description = "Forward, drop, or send a paused text/binary WebSocket message to Burp's manual Intercept tab; optionally replace its raw payload from standard Base64",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn control_intercepted_websocket_message(
        &self,
        Parameters(input): Parameters<ControlInterceptedWebSocketMessageInput>,
    ) -> String {
        let action = match input.action {
            InterceptActionInput::Forward => InterceptAction::Forward,
            InterceptActionInput::Drop => InterceptAction::Drop,
            InterceptActionInput::Intercept => InterceptAction::Intercept,
        } as i32;
        let replace_payload = input.payload_base64.is_some();
        let payload = match input.payload_base64 {
            Some(value) => match STANDARD.decode(value) {
                Ok(payload) => payload,
                Err(error) => {
                    return serde_json::json!({"error": format!("invalid payload_base64: {error}")})
                        .to_string();
                }
            },
            None => Vec::new(),
        };
        let max_body_length = input.max_body_length;
        match self
            .client
            .control_intercepted_websocket_message(ControlInterceptedWebSocketMessageRequest {
                id: input.id,
                action,
                payload,
                replace_payload,
            })
            .await
        {
            Ok(response) => response
                .message
                .map(|item| intercepted_websocket_message_output(item, true, max_body_length))
                .map(|item| {
                    serde_json::to_string(&item).expect("WebSocket intercept output must serialize")
                })
                .unwrap_or_else(|| {
                    serde_json::json!({"error": "empty WebSocket intercept response"}).to_string()
                }),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

    async fn proxy_intercept_config(&self) -> String {
        proxy_intercept_config_json(
            self.client
                .proxy_intercept_config(empty_proxy_intercept_config_request())
                .await,
        )
    }

    async fn update_proxy_intercept_config(
        &self,
        Parameters(input): Parameters<ProxyInterceptConfigInput>,
    ) -> String {
        let request_rules = input.request_rules.unwrap_or_default();
        let response_rules = input.response_rules.unwrap_or_default();
        let replace_request_rules = input.replace_request_rules.unwrap_or(false);
        let replace_response_rules = input.replace_response_rules.unwrap_or(false);
        proxy_intercept_config_json(
            self.client
                .proxy_intercept_config(ProxyInterceptConfigRequest {
                    master_intercept_enabled: input.master_intercept_enabled,
                    request_do_intercept: input.request_do_intercept,
                    request_auto_content_length: input.request_auto_content_length,
                    response_do_intercept: input.response_do_intercept,
                    response_auto_content_length: input.response_auto_content_length,
                    websocket_client_to_server: input.websocket_client_to_server,
                    websocket_server_to_client: input.websocket_server_to_client,
                    websocket_in_scope_only: input.websocket_in_scope_only,
                    request_rules: request_rules
                        .into_iter()
                        .map(ProxyInterceptRuleInput::into_proto)
                        .collect(),
                    response_rules: response_rules
                        .into_iter()
                        .map(ProxyInterceptRuleInput::into_proto)
                        .collect(),
                    replace_request_rules,
                    replace_response_rules,
                    response_unhide_hidden_fields: input.response_unhide_hidden_fields,
                    response_enable_disabled_fields: input.response_enable_disabled_fields,
                    response_remove_input_length_limits: input.response_remove_input_length_limits,
                    response_remove_javascript_validation: input
                        .response_remove_javascript_validation,
                    response_remove_all_javascript: input.response_remove_all_javascript,
                    request_fix_missing_new_lines: input.request_fix_missing_new_lines,
                })
                .await,
        )
    }
    async fn proxy_settings(&self) -> String {
        proxy_settings_json(self.client.proxy_settings(ProxySettingsRequest {}).await)
    }

    async fn update_proxy_settings(
        &self,
        Parameters(input): Parameters<ProxySettingsUpdateInput>,
    ) -> String {
        let operation = match proxy_settings_operation(input) {
            Ok(operation) => operation,
            Err(error) => return serde_json::json!({"error": error}).to_string(),
        };
        proxy_settings_json(
            self.client
                .proxy_settings_update(ProxySettingsUpdateRequest {
                    operation: Some(operation),
                })
                .await,
        )
    }
    async fn proxy_websocket_history(
        &self,
        Parameters(input): Parameters<ProxyWebSocketHistoryInput>,
    ) -> String {
        let limit = input.limit.unwrap_or(50).min(MAX_PAGE_SIZE);
        let include_bodies = input.include_bodies.unwrap_or(false);
        let effective_max_length = Some(input.max_body_length.unwrap_or(DEFAULT_MAX_BODY_LENGTH));
        match self.client.proxy_websocket_history(ProxyWebSocketHistoryRequest {
            page: Some(PageRequest { limit, cursor: input.cursor.unwrap_or_default() }),
            after_id: None,
        }).await {
            Ok(response) => serde_json::to_string(&ProxyWebSocketHistoryOutput {
                items: response.items.into_iter().map(|item| {
                    let payload_len = item.payload.len();
                    let (payload_b64, payload_trunc) = if include_bodies && !item.payload.is_empty() {
                        let (b64, trunc, _) = encode_bounded_base64(&item.payload, effective_max_length);
                        (Some(b64), trunc.then_some(true))
                    } else {
                        (None, None)
                    };

                    let edited_len = item.edited_payload.len();
                    let (edited_b64, edited_trunc) = if include_bodies && !item.edited_payload.is_empty() {
                        let (b64, trunc, _) = encode_bounded_base64(&item.edited_payload, effective_max_length);
                        (Some(b64), trunc.then_some(true))
                    } else {
                        (None, None)
                    };

                    ProxyWebSocketHistoryItemOutput {
                        index: item.index,
                        id: item.id,
                        websocket_id: item.web_socket_id,
                        direction: item.direction,
                        payload_length: payload_len,
                        payload_base64: payload_b64,
                        payload_truncated: payload_trunc,
                        edited_payload_length: edited_len,
                        edited_payload_base64: edited_b64,
                        edited_payload_truncated: edited_trunc,
                        time: item.time,
                        listener_port: item.listener_port,
                        upgrade_url: item.upgrade_url,
                    }
                }).collect::<Vec<_>>(),
                page: response.page.map(|page| serde_json::json!({"total": page.total, "truncated": page.truncated, "next_cursor": page.next_cursor})),
            }).expect("proxy websocket history must serialize"),
            Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
        }
    }

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
        description = "Get current Burp extension and process configuration metadata",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
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

#[tool_handler(router = Self::burp_router(), name = "burp-mcp", version = "3.2.0")]
impl rmcp::ServerHandler for BurpTools {
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, rmcp::ErrorData> {
        let router = self.tool_router();
        let tool_name = request.name.to_string();
        let arguments = request.arguments.clone().unwrap_or_default();
        let tool_context = ToolCallContext::new(self, request, context);
        match router.call(tool_context).await {
            Ok(response) => Ok(finalize_tool_response(
                &router, &tool_name, &arguments, response,
            )),
            Err(error) => Ok(actionable_call_error(
                &router, &tool_name, &arguments, error,
            )),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<rmcp::model::ListToolsResult, rmcp::ErrorData> {
        Ok(rmcp::model::ListToolsResult::with_all_items(
            self.tool_router().list_all(),
        ))
    }

    fn get_info(&self) -> rmcp::model::ServerInfo {
        mcp_server_info()
    }
}

fn mcp_server_info() -> rmcp::model::ServerInfo {
    rmcp::model::ServerInfo::new(
        rmcp::model::ServerCapabilities::builder()
            .enable_tools()
            .build(),
    )
    .with_server_info(rmcp::model::Implementation::new("burp-mcp", "3.2.0"))
    .with_instructions(BURP_MCP_USAGE_INSTRUCTIONS)
}

fn actionable_call_error(
    router: &rmcp::handler::server::tool::ToolRouter<BurpTools>,
    tool_name: &str,
    arguments: &serde_json::Map<String, serde_json::Value>,
    error: rmcp::ErrorData,
) -> CallToolResponse {
    let Some(tool) = router.get(tool_name) else {
        let available_tools = router
            .list_all()
            .into_iter()
            .map(|tool| tool.name.into_owned())
            .collect::<Vec<_>>();
        return CallToolResult::structured_error(serde_json::json!({
            "error": "unknown_tool",
            "message": format!("tool `{tool_name}` is not available on this server"),
            "received_tool": tool_name,
            "available_tools": available_tools,
            "correction": "Use the exact server-local name returned by tools/list. In eval kernels, call the host-provided qualified binding (for example tool.mcp__burp_mcp_burp_http), not an assumed alias.",
        }))
        .into();
    };
    invalid_arguments_result(tool, tool_name, arguments, error.message.into_owned()).into()
}

fn finalize_tool_response(
    router: &rmcp::handler::server::tool::ToolRouter<BurpTools>,
    tool_name: &str,
    arguments: &serde_json::Map<String, serde_json::Value>,
    response: CallToolResponse,
) -> CallToolResponse {
    let CallToolResponse::Complete(result) = response else {
        return response;
    };
    let Some(tool) = router.get(tool_name) else {
        return CallToolResponse::Complete(result);
    };

    if result.is_error == Some(true) {
        if result.structured_content.is_some() {
            return CallToolResponse::Complete(result);
        }
        let message = result
            .content
            .iter()
            .find_map(ContentBlock::as_text)
            .map(|text| text.text.clone())
            .unwrap_or_else(|| "tool call failed".to_owned());
        return invalid_arguments_result(tool, tool_name, arguments, message).into();
    }

    let Some(mut value) = result.content.iter().find_map(|content| {
        let ContentBlock::Text(text) = content else {
            return None;
        };
        let value: serde_json::Value = serde_json::from_str(&text.text).ok()?;
        has_nonempty_error(&value).then_some(value)
    }) else {
        return CallToolResponse::Complete(result);
    };
    let object = value
        .as_object_mut()
        .expect("has_nonempty_error only accepts JSON objects");
    object
        .entry("tool")
        .or_insert_with(|| serde_json::Value::String(tool_name.to_owned()));
    if !object.contains_key("message")
        && let Some(error) = object.get("error").and_then(serde_json::Value::as_str)
    {
        object.insert(
            "message".to_owned(),
            serde_json::Value::String(error.to_owned()),
        );
    }
    let (required_fields, accepted_fields, valid_actions) = schema_hints(tool);
    object
        .entry("required_fields")
        .or_insert_with(|| serde_json::json!(required_fields));
    object
        .entry("accepted_fields")
        .or_insert_with(|| serde_json::json!(accepted_fields));
    if !valid_actions.is_empty() {
        object
            .entry("valid_actions")
            .or_insert_with(|| serde_json::json!(valid_actions));
    }
    object.entry("correction").or_insert_with(|| {
        serde_json::Value::String(
            "Use message plus the accepted fields and valid actions, correct the call, then retry the same tool once."
                .to_owned(),
        )
    });
    CallToolResult::structured_error(value).into()
}

fn invalid_arguments_result(
    tool: &rmcp::model::Tool,
    tool_name: &str,
    arguments: &serde_json::Map<String, serde_json::Value>,
    message: String,
) -> CallToolResult {
    let (required_fields, accepted_fields, valid_actions) = schema_hints(tool);
    let received_fields = arguments.keys().cloned().collect::<Vec<_>>();
    CallToolResult::structured_error(serde_json::json!({
        "error": "invalid_tool_arguments",
        "message": message,
        "tool": tool_name,
        "received_fields": received_fields,
        "required_fields": required_fields,
        "accepted_fields": accepted_fields,
        "valid_actions": valid_actions,
        "correction": "Retry the same tool using only accepted_fields, include every required_field, and choose action from valid_actions when present.",
    }))
}

fn resolve_schema_ref<'a>(
    schema: &'a serde_json::Value,
    root: &'a serde_json::Value,
) -> &'a serde_json::Value {
    if let Some(ref_str) = schema.get("$ref").and_then(serde_json::Value::as_str) {
        if let Some(def_name) = ref_str.strip_prefix("#/$defs/") {
            if let Some(target) = root.get("$defs").and_then(|d| d.get(def_name)) {
                return resolve_schema_ref(target, root);
            }
        } else if let Some(def_name) = ref_str.strip_prefix("#/definitions/") {
            if let Some(target) = root.get("definitions").and_then(|d| d.get(def_name)) {
                return resolve_schema_ref(target, root);
            }
        }
    }
    schema
}

fn schema_hints(tool: &rmcp::model::Tool) -> (Vec<String>, Vec<String>, Vec<String>) {
    let root = serde_json::Value::Object((*tool.input_schema).clone());
    let resolved_root = resolve_schema_ref(&root, &root);
    let one_of = resolved_root
        .get("oneOf")
        .or_else(|| resolved_root.get("anyOf"))
        .and_then(serde_json::Value::as_array);

    if let Some(variants) = one_of {
        let mut accepted_fields = Vec::new();
        let mut valid_actions = Vec::new();
        let mut common_required: Option<std::collections::BTreeSet<String>> = None;

        for variant in variants {
            let resolved_variant = resolve_schema_ref(variant, &root);
            if let Some(props) = resolved_variant
                .get("properties")
                .and_then(serde_json::Value::as_object)
            {
                for key in props.keys() {
                    if !accepted_fields.contains(key) {
                        accepted_fields.push(key.clone());
                    }
                }
                if let Some(action_prop) = props.get("action") {
                    let resolved_action = resolve_schema_ref(action_prop, &root);
                    if let Some(enums) = resolved_action
                        .get("enum")
                        .and_then(serde_json::Value::as_array)
                    {
                        for val in enums.iter().filter_map(serde_json::Value::as_str) {
                            if !valid_actions.iter().any(|a| a == val) {
                                valid_actions.push(val.to_owned());
                            }
                        }
                    } else if let Some(const_val) = resolved_action
                        .get("const")
                        .and_then(serde_json::Value::as_str)
                    {
                        if !valid_actions.iter().any(|a| a == const_val) {
                            valid_actions.push(const_val.to_owned());
                        }
                    }
                }
            }

            let variant_req: std::collections::BTreeSet<String> = resolved_variant
                .get("required")
                .and_then(serde_json::Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(serde_json::Value::as_str)
                .map(str::to_owned)
                .collect();

            common_required = match common_required {
                None => Some(variant_req),
                Some(current) => Some(current.intersection(&variant_req).cloned().collect()),
            };
        }

        let required_fields = common_required
            .map(|s| s.into_iter().collect())
            .unwrap_or_default();
        (required_fields, accepted_fields, valid_actions)
    } else {
        let required_fields = resolved_root
            .get("required")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(serde_json::Value::as_str)
            .map(str::to_owned)
            .collect();
        let accepted_fields = resolved_root
            .get("properties")
            .and_then(serde_json::Value::as_object)
            .into_iter()
            .flat_map(|properties| properties.keys().cloned())
            .collect();
        let action_schema = resolved_root
            .get("properties")
            .and_then(serde_json::Value::as_object)
            .and_then(|properties| properties.get("action"));
        let mut valid_actions = Vec::new();
        if let Some(act) = action_schema {
            let resolved_action = resolve_schema_ref(act, &root);
            if let Some(enums) = resolved_action
                .get("enum")
                .and_then(serde_json::Value::as_array)
            {
                for val in enums.iter().filter_map(serde_json::Value::as_str) {
                    valid_actions.push(val.to_owned());
                }
            } else if let Some(const_val) = resolved_action
                .get("const")
                .and_then(serde_json::Value::as_str)
            {
                valid_actions.push(const_val.to_owned());
            }
        }
        (required_fields, accepted_fields, valid_actions)
    }
}

fn has_nonempty_error(value: &serde_json::Value) -> bool {
    value
        .as_object()
        .and_then(|object| object.get("error"))
        .is_some_and(|error| match error {
            serde_json::Value::Null => false,
            serde_json::Value::String(message) => !message.is_empty(),
            _ => true,
        })
}

fn tool_input_error(tool: &str, action: &str, message: impl Into<String>) -> String {
    serde_json::json!({
        "error": "invalid_tool_arguments",
        "message": message.into(),
        "tool": tool,
        "action": action,
        "correction": "Correct the arguments described by message and retry the same tool call.",
    })
    .to_string()
}

fn rpc_error_json(error: burp_protocol::ClientError) -> String {
    match error {
        burp_protocol::ClientError::Rpc(status) => {
            let code = status.code();
            let detail = decode_rpc_error(&status);
            let message = detail
                .as_ref()
                .map_or_else(|| status.message().to_owned(), |detail| detail.message.clone());
            let correction = match code {
                tonic::Code::InvalidArgument => {
                    "Correct the named argument or action fields and retry the same tool call."
                }
                tonic::Code::Unavailable => {
                    "Start Burp Suite with the Burp MCP extension, verify the configured endpoint, then retry."
                }
                tonic::Code::DeadlineExceeded => {
                    "Retry once; if the operation is long-running, use its background-job action and poll the returned job id."
                }
                tonic::Code::NotFound => {
                    "Refresh the relevant list, use a current id or index, then retry."
                }
                _ => "Use the message and gRPC code to correct the call before retrying.",
            };
            serde_json::json!({
                "error": "burp_rpc_error",
                "code": format!("{code:?}").to_ascii_lowercase(),
                "message": message,
                "details": detail.as_ref().map(|detail| detail.details.as_str()).filter(|details| !details.is_empty()),
                "retryable": detail.as_ref().map_or_else(
                    || matches!(code, tonic::Code::Unavailable | tonic::Code::DeadlineExceeded | tonic::Code::ResourceExhausted),
                    |detail| detail.retryable,
                ),
                "correction": correction,
            })
            .to_string()
        }
        other => serde_json::json!({
            "error": "burp_client_error",
            "message": other.to_string(),
            "retryable": matches!(other, burp_protocol::ClientError::QueueFull),
            "correction": "Follow the message, correct the environment or wait for an in-flight call, then retry.",
        })
        .to_string(),
    }
}

#[derive(Clone, PartialEq, Message)]
struct GoogleRpcStatus {
    #[prost(int32, tag = "1")]
    code: i32,
    #[prost(string, tag = "2")]
    message: String,
    #[prost(message, repeated, tag = "3")]
    details: Vec<GoogleRpcAny>,
}

#[derive(Clone, PartialEq, Message)]
struct GoogleRpcAny {
    #[prost(string, tag = "1")]
    type_url: String,
    #[prost(bytes = "vec", tag = "2")]
    value: Vec<u8>,
}

fn decode_rpc_error(status: &tonic::Status) -> Option<burp_protocol::protocol::RpcError> {
    let google_status = GoogleRpcStatus::decode(status.details()).ok()?;
    google_status.details.into_iter().find_map(|detail| {
        (detail.type_url == "type.googleapis.com/burp.v1.RpcError")
            .then(|| burp_protocol::protocol::RpcError::decode(detail.value.as_slice()).ok())
            .flatten()
    })
}

const BURP_MCP_USAGE_INSTRUCTIONS: &str = r#"Use Burp MCP only on targets the operator is authorized to test.
On connection, call burp_burp_version first; its capabilities and runtime limits are authoritative. Use server-local tool names exactly as returned by tools/list. A host client may expose qualified bindings such as mcp__burp_mcp_burp_http or tool.mcp__burp_mcp_burp_http; never assume the server-local name is a valid eval-kernel binding.
Prefer compact reads before active traffic: burp_proxy history defaults to metadata-only, then fetch one detail or server-side projection. Treat success=true as acceptance and verify the observable effect.
For burp_http send_to_repeater, pass either {action:"send_to_repeater",url,method?,body?,headers?,tab_name?} or {action:"send_to_repeater",request,host?,port?,https?,tab_name?}; a raw request may derive host/port from Host.
Enable burp_intercept_controller only with url_filter or in_scope_only=true and a bounded timeout. Resolve queued messages, disable the controller, and restore temporary Burp state before completion.
For register_proxy_rule body edits use action:"register_proxy_rule", url_contains:url substring, phase:"request"|"response", rule_action:"edit", match, and replace. Use header_name/header_value only for header name/value edits.
Bambda source is JVM-compiled: do not embed large payloads or string literals near the 65,535-byte class-file UTF-8 limit. Use register_proxy_rule for bounded replacements or another external streaming mechanism.
Tool errors are structured with message, accepted_fields, valid_actions, and correction when available. Correct the call and retry instead of guessing aliases."#;

fn sitegraph_disabled_json() -> String {
    serde_json::json!({
        "error": "sitegraph is disabled; restart burp-mcp with --enable-sitegraph"
    })
    .to_string()
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
        Err(error) => rpc_error_json(error),
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
        description: input
            .description
            .unwrap_or_else(|| "Burp MCP session rule".to_owned()),
        action_type: input
            .action_type
            .unwrap_or_else(|| "replace_text".to_owned()),
        header_name: input.header_name.unwrap_or_default(),
        parameter_name: input.parameter_name.unwrap_or_default(),
        macro_description: input.macro_description.unwrap_or_default(),
        url_contains: input.url_contains.unwrap_or_default(),
        tools: input.tools.unwrap_or_default(),
        enabled: input.enabled.unwrap_or(true),
    }
}

fn intercepted_websocket_message_output(
    item: burp_protocol::protocol::InterceptedWebSocketMessage,
    include_bodies: bool,
    max_body_length: Option<usize>,
) -> InterceptedWebSocketMessageOutput {
    let payload_len = item.payload.len();
    let (payload_b64, payload_trunc) = if include_bodies && !item.payload.is_empty() {
        let (b64, trunc, _) = encode_bounded_base64(&item.payload, max_body_length);
        (Some(b64), trunc.then_some(true))
    } else {
        (None, None)
    };

    InterceptedWebSocketMessageOutput {
        id: item.id,
        web_socket_id: item.web_socket_id,
        upgrade_url: item.upgrade_url,
        direction: item.direction,
        message_type: item.message_type,
        phase: item.phase,
        payload_base64: payload_b64,
        payload_length: Some(payload_len),
        payload_truncated: payload_trunc,
    }
}

fn intercepted_message_output(
    item: burp_protocol::protocol::InterceptedMessage,
    include_bodies: bool,
    max_body_length: Option<usize>,
) -> InterceptedMessageOutput {
    let req_len = item.request.len();
    let (req_b64, req_trunc) = if include_bodies && !item.request.is_empty() {
        let (b64, trunc, _) = encode_bounded_base64(&item.request, max_body_length);
        (Some(b64), trunc.then_some(true))
    } else {
        (None, None)
    };

    let resp_len = (!item.response.is_empty()).then_some(item.response.len());
    let (resp_b64, resp_trunc) = if include_bodies && !item.response.is_empty() {
        let (b64, trunc, _) = encode_bounded_base64(&item.response, max_body_length);
        (Some(b64), trunc.then_some(true))
    } else {
        (None, None)
    };

    InterceptedMessageOutput {
        id: item.id,
        direction: item.direction,
        phase: item.phase,
        url: item.url,
        method: item.method,
        status: item.status,
        is_in_scope: item.is_in_scope,
        request_base64: req_b64,
        request_length: Some(req_len),
        request_truncated: req_trunc,
        response_base64: resp_b64,
        response_length: resp_len,
        response_truncated: resp_trunc,
    }
}

fn intercept_state_json(
    result: Result<burp_protocol::protocol::InterceptStateResponse, burp_protocol::ClientError>,
) -> String {
    match result {
        Ok(response) => serde_json::json!({"enabled": response.enabled}).to_string(),
        Err(error) => rpc_error_json(error),
    }
}

fn empty_proxy_intercept_config_request() -> ProxyInterceptConfigRequest {
    ProxyInterceptConfigRequest {
        master_intercept_enabled: None,
        request_do_intercept: None,
        request_auto_content_length: None,
        response_do_intercept: None,
        response_auto_content_length: None,
        websocket_client_to_server: None,
        websocket_server_to_client: None,
        websocket_in_scope_only: None,
        request_rules: Vec::new(),
        response_rules: Vec::new(),
        replace_request_rules: false,
        replace_response_rules: false,
        response_unhide_hidden_fields: None,
        response_enable_disabled_fields: None,
        response_remove_input_length_limits: None,
        response_remove_javascript_validation: None,
        response_remove_all_javascript: None,
        request_fix_missing_new_lines: None,
    }
}

fn proxy_intercept_config_json(
    result: Result<ProxyInterceptConfigResponse, burp_protocol::ClientError>,
) -> String {
    match result {
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
        Err(error) => rpc_error_json(error),
    }
}
fn proxy_settings_json(
    result: Result<ProxySettingsResponse, burp_protocol::ClientError>,
) -> String {
    match result {
        Ok(response) => serde_json::json!({
            "listeners": response.listeners.into_iter().map(|listener| serde_json::json!({
                "port": listener.port,
                "running": listener.running,
                "listen_mode": listener.listen_mode,
                "listen_specific_address": listener.listen_specific_address,
                "certificate_mode": listener.certificate_mode,
                "enable_http2": listener.enable_http2,
                "support_invisible_proxying": listener.support_invisible_proxying,
            })).collect::<Vec<_>>(),
            "script_filters": response.script_filters.into_iter().map(|filter| serde_json::json!({
                "target": filter.target,
                "mode": filter.mode,
                "script": filter.script,
                "script_id": filter.script_id,
                "script_name": filter.script_name,
            })).collect::<Vec<_>>(),
            "interception": response.interception.map(|config| serde_json::json!({
                "master_intercept_enabled": config.master_intercept_enabled,
                "request": {
                    "do_intercept": config.request_do_intercept,
                    "auto_content_length": config.request_auto_content_length,
                    "fix_missing_new_lines": config.request_fix_missing_new_lines,
                    "rules": config.request_rules.into_iter().map(proxy_intercept_rule_json).collect::<Vec<_>>(),
                },
                "response": {
                    "do_intercept": config.response_do_intercept,
                    "auto_content_length": config.response_auto_content_length,
                    "rules": config.response_rules.into_iter().map(proxy_intercept_rule_json).collect::<Vec<_>>(),
                    "modification": {
                        "unhide_hidden_fields": config.response_unhide_hidden_fields,
                        "enable_disabled_fields": config.response_enable_disabled_fields,
                        "remove_input_length_limits": config.response_remove_input_length_limits,
                        "remove_javascript_validation": config.response_remove_javascript_validation,
                        "remove_all_javascript": config.response_remove_all_javascript,
                    },
                },
                "websocket": {
                    "client_to_server": config.websocket_client_to_server,
                    "server_to_client": config.websocket_server_to_client,
                    "in_scope_only": config.websocket_in_scope_only,
                },
            })),
        }).to_string(),
        Err(error) => rpc_error_json(error),
    }
}

fn proxy_settings_operation(
    input: ProxySettingsUpdateInput,
) -> Result<burp_protocol::protocol::proxy_settings_update_request::Operation, &'static str> {
    use burp_protocol::protocol::proxy_settings_update_request::Operation;
    match input.operation.as_str() {
        "listener_upsert" => Ok(Operation::ListenerUpsert(ProxyListener {
            port: input.port.ok_or("port is required")?,
            running: input.running.unwrap_or(true),
            listen_mode: input
                .listen_mode
                .unwrap_or_else(|| "loopback_only".to_owned()),
            listen_specific_address: input.listen_specific_address.unwrap_or_default(),
            certificate_mode: input
                .certificate_mode
                .unwrap_or_else(|| "per_host".to_owned()),
            enable_http2: input.enable_http2.unwrap_or(true),
            support_invisible_proxying: input.support_invisible_proxying.unwrap_or(false),
        })),
        "listener_delete" => Ok(Operation::ListenerDeletePort(
            input.port.ok_or("port is required")?,
        )),
        "script_filter_upsert" => Ok(Operation::ScriptFilterUpsert(ProxyScriptFilter {
            target: input.target.ok_or("target is required")?,
            mode: input.mode.unwrap_or_else(|| "script".to_owned()),
            script: input.script.unwrap_or_default(),
            script_id: input.script_id.unwrap_or_default(),
            script_name: input.script_name.unwrap_or_default(),
        })),
        "script_filter_delete" => Ok(Operation::ScriptFilterDeleteTarget(
            input.target.ok_or("target is required")?,
        )),
        "intercept_rule_upsert" => Ok(Operation::InterceptRuleUpsert(ProxyInterceptRuleMutation {
            kind: input.kind.ok_or("kind is required")?,
            index: input.index,
            rule: Some(input.rule.ok_or("rule is required")?.into_proto()),
        })),
        "intercept_rule_delete" => Ok(Operation::InterceptRuleDelete(ProxyInterceptRuleDelete {
            kind: input.kind.ok_or("kind is required")?,
            index: input.index.ok_or("index is required")?,
        })),
        "intercept_toggle" => {
            if input.master_enabled.is_none()
                && input.request_enabled.is_none()
                && input.response_enabled.is_none()
            {
                return Err(
                    "intercept_toggle requires master_enabled, request_enabled, or response_enabled",
                );
            }
            Ok(Operation::InterceptToggle(ProxyInterceptToggle {
                master_enabled: input.master_enabled,
                request_enabled: input.request_enabled,
                response_enabled: input.response_enabled,
            }))
        }
        _ => Err(
            "operation must be listener_upsert, listener_delete, script_filter_upsert, script_filter_delete, intercept_rule_upsert, intercept_rule_delete, or intercept_toggle",
        ),
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
        Err(error) => rpc_error_json(error),
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
        (Some(operation), true) => {
            utility_engine_api::run(normalize_decoder_operation(&operation), value, &input.args)
        }
        (None, false) => {
            let steps = input
                .steps
                .iter()
                .map(|step| utility_engine_api::RecipeStep {
                    operation: normalize_decoder_operation(&step.op).to_owned(),
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

fn normalize_decoder_operation(operation: &str) -> &str {
    match operation {
        "base64_encode" => "base64.encode",
        "base64_decode" => "base64.decode",
        "base64url_encode" => "base64url.encode",
        "base64url_decode" => "base64url.decode",
        "hex_encode" => "hex.encode",
        "hex_decode" => "hex.decode",
        "url_encode" => "url.encode",
        "url_decode" => "url.decode",
        _ => operation,
    }
}

fn decoder_result_json(result: utility_engine_api::UtilityResult<DataValue>) -> String {
    match result {
        Ok(value) => utility_value_json(value).to_string(),
        Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
    }
}
fn scan_configuration_request(
    input: ScanConfigurationUpsertInput,
) -> UpsertScanConfigurationRequest {
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

fn scan_configuration_json(
    value: burp_protocol::protocol::ScanConfigurationEntry,
) -> serde_json::Value {
    serde_json::json!({
        "id": value.id,
        "name": value.name,
        "scan_type": value.scan_type,
        "audit_type": value.audit_type,
        "include_out_of_scope": value.include_out_of_scope,
        "timeout_seconds": value.timeout_seconds,
        "stable_seconds": value.stable_seconds,
        "resource_pool_id": value.resource_pool_id,
        "source": value.source,
    })
}

fn scan_pool_json(value: burp_protocol::protocol::ScanResourcePoolEntry) -> serde_json::Value {
    serde_json::json!({
        "id": value.id,
        "name": value.name,
        "kind": value.kind,
        "existing_pool_name": value.existing_pool_name,
        "concurrent_request_limit": value.concurrent_request_limit,
        "throttle_millis": value.throttle_millis,
        "max_retries": value.max_retries,
        "source": value.source,
    })
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

fn to_proto_request(input: &SendRequestInput) -> SendRequestRequest {
    SendRequestRequest {
        method: input.method.clone().unwrap_or_else(|| "GET".to_owned()),
        url: input.url.clone(),
        body: input.body.clone().unwrap_or_default().into_bytes(),
        headers: input
            .headers
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(|(name, value)| HttpHeaderEntry { name, value })
            .collect(),
    }
}

fn to_proxy_history_request(input: ProxyHistoryInput, limit: u32) -> ProxyHistoryRequest {
    ProxyHistoryRequest {
        page: Some(PageRequest {
            limit,
            cursor: input
                .cursor
                .unwrap_or_else(|| input.offset.unwrap_or_default().to_string()),
        }),
        url_filter: input.url_filter.unwrap_or_default(),
        method_filter: input.method_filter.unwrap_or_default(),
        status_filter: input.status_filter,
        has_notes: input.has_notes.unwrap_or(false),
        color: input.color.unwrap_or_default(),
        after_id: None,
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
            "error": status.error,
            "scan_type": status.scan_type,
            "stateless": status.stateless,
            "status_message": status.status_message,
            "request_count": status.request_count,
            "error_count": status.error_count,
            "issue_count": status.issue_count,
        })
        .to_string(),
        Err(error) => serde_json::json!({"error": error.to_string()}).to_string(),
    }
}
fn to_send_output_with_options(
    response: burp_protocol::protocol::SendRequestResponse,
    headers_only: bool,
    extract_css: Option<&str>,
    extract_json: Option<&str>,
    max_length: Option<usize>,
) -> SendResponseOutput {
    let effective_max = Some(max_length.unwrap_or(DEFAULT_MAX_BODY_LENGTH));
    let raw_req_str = String::from_utf8_lossy(&response.request).into_owned();
    let (req_str, req_trunc) = if headers_only {
        (body_filter::extract_headers_only(&raw_req_str), false)
    } else {
        let (filtered, trunc) = body_filter::filter_and_truncate_payload(
            &response.request,
            None,
            headers_only,
            None,
            None,
            effective_max,
        );
        (filtered, trunc)
    };

    let (resp_str, resp_trunc) = if response.has_response {
        let (filtered, trunc) = body_filter::filter_and_truncate_payload(
            &response.response,
            None,
            headers_only,
            extract_css,
            extract_json,
            effective_max,
        );
        (Some(filtered), trunc)
    } else {
        (None, false)
    };

    SendResponseOutput {
        request: req_str,
        response: resp_str,
        status: response.has_response.then_some(response.status),
        extracted: None,
        truncated: (req_trunc || resp_trunc).then_some(true),
    }
}

#[allow(dead_code)]
fn to_send_output(response: burp_protocol::protocol::SendRequestResponse) -> SendResponseOutput {
    to_send_output_with_options(response, false, None, None, None)
}

fn repeater_input_from_http_action(
    mut input: SendToRepeaterInput,
    url: Option<&str>,
    method: Option<&str>,
    body: Option<&str>,
    headers: Option<&std::collections::HashMap<String, String>>,
) -> Result<SendToRepeaterInput, String> {
    if input.request.is_empty() {
        let url = url.ok_or_else(|| {
            "provide either `url` or a complete raw HTTP `request`; accepted fields for this action: url, method?, body?, headers?, request?, host?, port?, https?, tab_name?".to_owned()
        })?;
        let parsed = url::Url::parse(url)
            .map_err(|error| format!("`url` must be an absolute http(s) URL: {error}"))?;
        if !matches!(parsed.scheme(), "http" | "https") {
            return Err(format!(
                "unsupported URL scheme `{}`; use `http` or `https`",
                parsed.scheme()
            ));
        }
        let host = parsed
            .host_str()
            .ok_or_else(|| "`url` must include a host".to_owned())?;
        input.host = host.to_owned();
        input.https = Some(parsed.scheme() == "https");
        input.port = parsed.port_or_known_default().map(u32::from);

        let mut target = parsed.path().to_owned();
        if let Some(query) = parsed.query() {
            target.push('?');
            target.push_str(query);
        }
        let method = method.unwrap_or("GET").to_ascii_uppercase();
        let body = body.unwrap_or_default();
        let newline = "\r\n";
        let mut request = format!("{method} {target} HTTP/1.1{newline}");
        let has_host = headers
            .is_some_and(|headers| headers.keys().any(|name| name.eq_ignore_ascii_case("host")));
        if !has_host {
            request.push_str("Host: ");
            request.push_str(host);
            if parsed.port().is_some() {
                request.push(':');
                request.push_str(&input.port.unwrap_or_default().to_string());
            }
            request.push_str(newline);
        }
        if let Some(headers) = headers {
            let mut headers = headers.iter().collect::<Vec<_>>();
            headers.sort_unstable_by_key(|(name, _)| *name);
            for (name, value) in headers {
                request.push_str(name);
                request.push_str(": ");
                request.push_str(value);
                request.push_str(newline);
            }
        }
        if !body.is_empty()
            && !headers.is_some_and(|headers| {
                headers
                    .keys()
                    .any(|name| name.eq_ignore_ascii_case("content-length"))
            })
        {
            request.push_str(&format!("Content-Length: {}{newline}", body.len()));
        }
        request.push_str(newline);
        request.push_str(body);
        input.request = request;
    }
    Ok(input)
}

fn normalize_repeater_input(
    mut input: SendToRepeaterInput,
) -> Result<SendToRepeaterRequest, String> {
    if input.request.trim().is_empty() {
        return Err("raw `request` must not be empty".to_owned());
    }
    let (request_host, request_port) = authority_from_raw_request(&input.request)?;
    if input.host.is_empty() {
        input.host = request_host.ok_or_else(|| {
            "raw `request` must contain a Host header when `host` is omitted".to_owned()
        })?;
    }
    let https = input.https.unwrap_or(false);
    let port = input
        .port
        .or(request_port)
        .unwrap_or(if https { 443 } else { 80 });
    if port == 0 || port > u16::MAX as u32 {
        return Err("`port` must be between 1 and 65535".to_owned());
    }
    Ok(SendToRepeaterRequest {
        request: input.request.into_bytes(),
        host: input.host,
        port,
        https,
        tab_name: input.tab_name.unwrap_or_else(|| "MCP".to_owned()),
    })
}

fn authority_from_raw_request(request: &str) -> Result<(Option<String>, Option<u32>), String> {
    let head = request
        .split_once("\r\n\r\n")
        .or_else(|| request.split_once("\n\n"))
        .map_or(request, |(head, _)| head);
    let mut lines = head.lines();
    let request_line = lines
        .next()
        .ok_or_else(|| "raw `request` must include an HTTP request line".to_owned())?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next();
    let target = parts.next();
    let version = parts.next();
    if method.is_none()
        || target.is_none()
        || version.is_none_or(|value| !value.starts_with("HTTP/"))
    {
        return Err(
            "raw `request` must start with `<METHOD> <request-target> HTTP/<version>`".to_owned(),
        );
    }
    let host = lines.find_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.eq_ignore_ascii_case("host").then_some(value.trim())
    });
    match host {
        Some("") => Err("Host header must not be empty".to_owned()),
        Some(authority) => parse_http_authority(authority).map(|(host, port)| (Some(host), port)),
        None => Ok((None, None)),
    }
}

fn parse_http_authority(authority: &str) -> Result<(String, Option<u32>), String> {
    let parsed = url::Url::parse(&format!("http://{authority}/"))
        .map_err(|error| format!("invalid Host header `{authority}`: {error}"))?;
    let host = parsed
        .host_str()
        .ok_or_else(|| format!("invalid Host header `{authority}`"))?;
    Ok((host.to_owned(), parsed.port().map(u32::from)))
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
    let host = input.host.or_else(|| {
        lines.clone().find_map(|line| {
            line.strip_prefix("Host: ")
                .or_else(|| line.strip_prefix("host: "))
                .map(str::to_owned)
        })
    });
    let format = input
        .format
        .clone()
        .unwrap_or_else(|| "curl".to_owned())
        .to_ascii_lowercase();
    if format == "raw" {
        return Ok(input.request);
    }
    let host =
        host.ok_or_else(|| "host is required when the request has no Host header".to_owned())?;
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
    if format == "python" {
        let headers = lines
            .filter_map(|line| line.split_once(':'))
            .filter(|(name, _)| !name.eq_ignore_ascii_case("content-length"))
            .map(|(name, value)| format!("{:?}: {:?}", name.trim(), value.trim()))
            .collect::<Vec<_>>()
            .join(", ");
        return Ok(format!(
            "requests.request({method:?}, {url:?}, headers={{{headers}}}, data={body:?})"
        ));
    }
    if format != "curl" {
        return Err("format must be raw, curl, or python".to_owned());
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
        BurpTools, ControlInterceptedMessageInput, ControlInterceptedWebSocketMessageInput,
        DEFAULT_MAX_BODY_LENGTH, DecoderInput, ExportRequestInput, InterceptControllerInput,
        InterceptedMessagesInput, InterceptedWebSocketMessagesInput, LoggerDetailInput,
        ManagedWebSocketHistoryInput, ProxyDetailInput, ProxyHistoryInput,
        ProxyInterceptRuleBooleanOperatorInput, ProxyInterceptRuleInput,
        ProxyInterceptRuleMatchTypeInput, ProxyInterceptRuleRelationshipInput,
        ProxyRuleActionInput, ProxyRulePhaseInput, ProxySettingsUpdateInput,
        ProxyWebSocketHistoryInput, RegisterProxyRuleInput, SITEGRAPH_TOOL_PREFIX,
        authority_from_raw_request, decode_rpc_error, encode_bounded_base64, export_request_text,
        has_nonempty_error, intercepted_message_output, intercepted_websocket_message_output,
        normalize_decoder_operation, normalize_repeater_input, proxy_settings_operation,
        repeater_input_from_http_action, resolve_schema_ref, schema_hints,
        to_proxy_history_request, to_send_output_with_options,
    };
    use crate::suite::{self, HttpActionInput, SettingsActionInput};
    use base64::Engine as _;
    use burp_protocol::protocol::proxy_settings_update_request::Operation;
    use prost::Message;
    use serde_json::Value;
    use std::collections::BTreeSet;

    #[test]
    fn every_tool_declares_complete_annotations() {
        let router = BurpTools::burp_router() + BurpTools::utility_router();
        for (name, route) in router.map {
            let annotations = route
                .attr
                .annotations
                .unwrap_or_else(|| panic!("{name} must declare MCP tool annotations"));
            assert!(
                annotations.read_only_hint.is_some()
                    && annotations.destructive_hint.is_some()
                    && annotations.idempotent_hint.is_some()
                    && annotations.open_world_hint.is_some(),
                "{name} must declare every MCP behavior hint"
            );
        }
    }

    #[test]
    fn initialization_instructions_publish_usage_contract() {
        let instructions = super::mcp_server_info().instructions.expect("instructions");
        assert!(instructions.contains("call burp_burp_version first"));
        assert!(instructions.contains("tool.mcp__burp_mcp_burp_http"));
        assert!(instructions.contains("url_filter or in_scope_only=true"));
    }

    #[test]
    fn proxy_history_exposes_and_forwards_all_filters() {
        let schema = serde_json::to_value(schemars::schema_for!(ProxyHistoryInput))
            .expect("proxy history input schema must serialize");
        for property in [
            "url_filter",
            "method_filter",
            "status_filter",
            "has_notes",
            "color",
            "include_bodies",
            "headers_only",
            "extract_css",
            "extract_json",
            "max_body_length",
        ] {
            assert!(
                schema["properties"].get(property).is_some(),
                "proxy history must expose {property}"
            );
        }

        let request = to_proxy_history_request(
            ProxyHistoryInput {
                url_filter: Some("https://mcl-staging.opswat.com/".to_owned()),
                method_filter: Some("POST".to_owned()),
                status_filter: Some(201),
                has_notes: Some(true),
                color: Some("red".to_owned()),
                limit: Some(5),
                offset: None,
                cursor: None,
                include_bodies: Some(false),
                headers_only: Some(false),
                extract_css: None,
                extract_json: None,
                max_body_length: None,
            },
            5,
        );
        assert_eq!(request.url_filter, "https://mcl-staging.opswat.com/");
        assert_eq!(request.method_filter, "POST");
        assert_eq!(request.status_filter, Some(201));
        assert!(request.has_notes);
        assert_eq!(request.color, "red");
        assert_eq!(request.page.expect("page is required").limit, 5);
    }

    #[test]
    fn action_schemas_expose_valid_enum_values_and_no_aliases() {
        let http = serde_json::to_value(schemars::schema_for!(HttpActionInput))
            .expect("HTTP action schema must serialize");
        let settings = serde_json::to_value(schemars::schema_for!(SettingsActionInput))
            .expect("settings action schema must serialize");
        let target = serde_json::to_value(schemars::schema_for!(suite::TargetActionInput))
            .expect("target action schema must serialize");
        let scanner = serde_json::to_value(schemars::schema_for!(suite::ScannerActionInput))
            .expect("scanner action schema must serialize");
        let fuzzer = serde_json::to_value(schemars::schema_for!(suite::FuzzerActionInput))
            .expect("fuzzer action schema must serialize");
        let logger = serde_json::to_value(schemars::schema_for!(suite::LoggerActionInput))
            .expect("logger action schema must serialize");
        let organizer = serde_json::to_value(schemars::schema_for!(suite::OrganizerActionInput))
            .expect("organizer action schema must serialize");
        let sitegraph = serde_json::to_value(schemars::schema_for!(suite::SiteGraphActionInput))
            .expect("sitegraph action schema must serialize");
        let websocket = serde_json::to_value(schemars::schema_for!(suite::WebSocketActionInput))
            .expect("websocket action schema must serialize");
        let scan_config = serde_json::to_value(schemars::schema_for!(suite::ScanConfigActionInput))
            .expect("scan_config action schema must serialize");
        let collaborator =
            serde_json::to_value(schemars::schema_for!(suite::CollaboratorActionInput))
                .expect("collaborator action schema must serialize");
        let diff = serde_json::to_value(schemars::schema_for!(suite::DiffActionInput))
            .expect("diff action schema must serialize");
        let session = serde_json::to_value(schemars::schema_for!(suite::SessionActionInput))
            .expect("session action schema must serialize");

        let http_text = http.to_string();
        for action in [
            "send",
            "send_batch",
            "convert",
            "export",
            "send_to_repeater",
        ] {
            assert!(
                http_text.contains(&format!("\"{action}\"")),
                "missing http action {action}"
            );
        }

        let target_text = target.to_string();
        for action in ["get_scope", "add_scope", "remove_scope", "info", "sitemap"] {
            assert!(
                target_text.contains(&format!("\"{action}\"")),
                "missing target action {action}"
            );
        }
        assert!(
            !target_text.contains("\"scope_check\""),
            "alias scope_check must not be in schema"
        );

        let scanner_text = scanner.to_string();
        for action in [
            "start_audit",
            "start_crawl",
            "stop",
            "list_issues",
            "issue_detail",
            "update_issue",
            "report",
            "test_bcheck",
            "remove",
        ] {
            assert!(
                scanner_text.contains(&format!("\"{action}\"")),
                "missing scanner action {action}"
            );
        }
        assert!(
            !scanner_text.contains("\"dry_run\""),
            "alias dry_run must not be in schema"
        );

        let fuzzer_text = fuzzer.to_string();
        for action in [
            "fuzz",
            "race",
            "send_to_intruder",
            "list_payloads",
            "get_payload_list",
            "create_payload_list",
            "import_payload_list",
            "upsert_payloads",
            "delete_payload_list",
            "register_payload_processor",
            "list_payload_processors",
            "remove_payload_processor",
            "register_payload_generator",
            "list_payload_generators",
            "remove_payload_generator",
        ] {
            assert!(
                fuzzer_text.contains(&format!("\"{action}\"")),
                "missing fuzzer action {action}"
            );
        }
        assert!(
            !fuzzer_text.contains("\"get_payloads\""),
            "alias get_payloads must not be in schema"
        );
        assert!(
            !fuzzer_text.contains("\"delete_payloads\""),
            "alias delete_payloads must not be in schema"
        );

        let logger_text = logger.to_string();
        for action in ["query", "detail", "clear"] {
            assert!(
                logger_text.contains(&format!("\"{action}\"")),
                "missing logger action {action}"
            );
        }

        let organizer_text = organizer.to_string();
        for action in ["add", "list"] {
            assert!(
                organizer_text.contains(&format!("\"{action}\"")),
                "missing organizer action {action}"
            );
        }

        let sitegraph_text = sitegraph.to_string();
        for action in [
            "status",
            "stats",
            "sync",
            "search",
            "security_view",
            "import_spec",
            "neighbors",
            "trace",
            "shortest_path",
            "clusters",
            "impact",
            "diff",
            "export",
            "history_search",
            "endpoint_detail",
            "projects",
            "config",
        ] {
            assert!(
                sitegraph_text.contains(&format!("\"{action}\"")),
                "missing sitegraph action {action}"
            );
        }
        assert!(
            !sitegraph_text.contains("\"import_openapi\""),
            "alias import_openapi must not be in schema"
        );

        let websocket_text = websocket.to_string();
        for action in [
            "create",
            "send_text",
            "send_binary",
            "history",
            "close",
            "list",
        ] {
            assert!(
                websocket_text.contains(&format!("\"{action}\"")),
                "missing websocket action {action}"
            );
        }

        let scan_config_text = scan_config.to_string();
        for action in [
            "list_configs",
            "get_config",
            "upsert_config",
            "delete_config",
            "list_pools",
            "get_pool",
            "upsert_pool",
            "delete_pool",
        ] {
            assert!(
                scan_config_text.contains(&format!("\"{action}\"")),
                "missing scan_config action {action}"
            );
        }

        let collaborator_text = collaborator.to_string();
        for action in ["generate", "poll", "correlate"] {
            assert!(
                collaborator_text.contains(&format!("\"{action}\"")),
                "missing collaborator action {action}"
            );
        }

        let diff_text = diff.to_string();
        for action in ["compare_exchanges", "diff_responses"] {
            assert!(
                diff_text.contains(&format!("\"{action}\"")),
                "missing diff action {action}"
            );
        }

        let session_text = session.to_string();
        for action in [
            "list_rules",
            "get_rule",
            "upsert_rule",
            "delete_rule",
            "run_macro",
            "upsert_macro",
            "list_macros",
            "delete_macro",
        ] {
            assert!(
                session_text.contains(&format!("\"{action}\"")),
                "missing session action {action}"
            );
        }

        let settings_text = settings.to_string();
        for action in [
            "get_proxy_settings",
            "update_proxy_settings",
            "export_config",
            "inspect_config",
            "import_config",
            "intercept_state",
            "set_intercept_state",
            "proxy_intercept_config",
            "update_proxy_intercept_config",
            "register_http_handler",
            "remove_http_handler",
            "register_proxy_rule",
            "list_proxy_rules",
            "remove_proxy_rule",
        ] {
            assert!(
                settings_text.contains(&format!("\"{action}\"")),
                "missing settings action {action}"
            );
        }
    }

    #[test]
    fn settings_schema_contains_semantic_proxy_rule_fields_and_no_overloaded_fields() {
        let schema = serde_json::to_value(schemars::schema_for!(SettingsActionInput))
            .expect("settings action schema must serialize");
        let root = &schema;
        let main = resolve_schema_ref(root, root);
        let variants = main
            .get("oneOf")
            .or_else(|| main.get("anyOf"))
            .and_then(Value::as_array)
            .expect("SettingsActionInput must be a oneOf/anyOf schema");

        let register_proxy_rule_variant = variants
            .iter()
            .map(|v| resolve_schema_ref(v, root))
            .find(|v| {
                v.get("properties")
                    .and_then(|p| p.get("action"))
                    .and_then(|a| {
                        let a_res = resolve_schema_ref(a, root);
                        a_res.get("const").and_then(Value::as_str).or_else(|| {
                            a_res
                                .get("enum")
                                .and_then(Value::as_array)
                                .and_then(|arr| arr.first())
                                .and_then(Value::as_str)
                        })
                    })
                    == Some("register_proxy_rule")
            })
            .expect("must find register_proxy_rule variant in SettingsActionInput");

        let props = register_proxy_rule_variant
            .get("properties")
            .and_then(Value::as_object)
            .expect("properties object");

        assert!(props.contains_key("id"));
        assert!(props.contains_key("url_contains"));
        assert!(props.contains_key("phase"));
        assert!(props.contains_key("rule_action"));
        assert!(props.contains_key("match"));
        assert!(props.contains_key("replace"));
        assert!(props.contains_key("header_name"));
        assert!(props.contains_key("header_value"));
        assert!(props.contains_key("enabled"));

        assert!(!props.contains_key("target"));
        assert!(!props.contains_key("mode"));
        assert!(!props.contains_key("kind"));
        assert!(!props.contains_key("script"));
        assert!(!props.contains_key("script_id"));
        assert!(!props.contains_key("script_name"));
    }

    #[test]
    fn settings_proxy_rule_preserves_match_and_replacement() {
        let input = SettingsActionInput::RegisterProxyRule {
            id: Some("replace-js".to_owned()),
            url_contains: "example.test/app.js".to_owned(),
            phase: Some(ProxyRulePhaseInput::Response),
            rule_action: Some(ProxyRuleActionInput::Edit),
            match_text: Some("old".to_owned()),
            replace: Some("new".to_owned()),
            header_name: None,
            header_value: None,
            enabled: Some(true),
        };

        match input {
            SettingsActionInput::RegisterProxyRule {
                id,
                url_contains,
                phase,
                rule_action,
                match_text,
                replace,
                header_name,
                header_value,
                enabled,
            } => {
                let rule_input = RegisterProxyRuleInput {
                    id,
                    url_contains,
                    phase,
                    rule_action,
                    match_text,
                    replace,
                    header_name,
                    header_value,
                    enabled,
                };
                assert_eq!(Some("old".to_owned()), rule_input.match_text);
                assert_eq!(Some("new".to_owned()), rule_input.replace);
                assert_eq!("example.test/app.js", rule_input.url_contains);
                assert_eq!(Some(ProxyRulePhaseInput::Response), rule_input.phase);
                assert_eq!(Some(ProxyRuleActionInput::Edit), rule_input.rule_action);
                assert_eq!(Some(true), rule_input.enabled);
            }
            _ => panic!("expected RegisterProxyRule variant"),
        }
    }
    #[test]
    fn repeater_url_builds_complete_request_and_service() {
        let input = repeater_input_from_http_action(
            super::SendToRepeaterInput {
                request: String::new(),
                host: String::new(),
                port: None,
                https: None,
                tab_name: Some("case".to_owned()),
            },
            Some("https://example.test:8443/a?q=1"),
            Some("post"),
            Some("{}"),
            None,
        )
        .expect("URL input should normalize");
        let request = normalize_repeater_input(input).expect("request should normalize");
        assert_eq!("example.test", request.host);
        assert_eq!(8443, request.port);
        assert!(request.https);
        assert_eq!("case", request.tab_name);
        assert_eq!(
            "POST /a?q=1 HTTP/1.1\r\nHost: example.test:8443\r\nContent-Length: 2\r\n\r\n{}",
            String::from_utf8(request.request).expect("request is UTF-8")
        );
    }

    #[test]
    fn rpc_status_details_preserve_backend_message_and_retryability() {
        let detail = burp_protocol::protocol::RpcError {
            code: burp_protocol::protocol::ErrorCode::InvalidArgument as i32,
            message: "url_filter is required".to_owned(),
            retryable: false,
            details: "set url_filter or in_scope_only=true".to_owned(),
        };
        let status = super::GoogleRpcStatus {
            code: tonic::Code::InvalidArgument as i32,
            message: detail.message.clone(),
            details: vec![super::GoogleRpcAny {
                type_url: "type.googleapis.com/burp.v1.RpcError".to_owned(),
                value: detail.encode_to_vec(),
            }],
        };
        let tonic_status = tonic::Status::with_details(
            tonic::Code::InvalidArgument,
            "Invalid data",
            status.encode_to_vec().into(),
        );

        let decoded = decode_rpc_error(&tonic_status).expect("RPC detail should decode");
        assert_eq!("url_filter is required", decoded.message);
        assert_eq!("set url_filter or in_scope_only=true", decoded.details);
        assert!(!decoded.retryable);
    }

    #[test]
    fn repeater_raw_request_derives_host_and_port() {
        let (host, port) =
            authority_from_raw_request("GET / HTTP/1.1\r\nHost: example.test:8081\r\n\r\n")
                .expect("authority should parse");
        assert_eq!(Some("example.test".to_owned()), host);
        assert_eq!(Some(8081), port);
    }

    #[test]
    fn intercept_controller_schema_exposes_scope_guards() {
        let schema = serde_json::to_value(schemars::schema_for!(InterceptControllerInput))
            .expect("intercept controller schema must serialize");
        assert!(schema.pointer("/properties/url_filter").is_some());
        assert!(schema.pointer("/properties/in_scope_only").is_some());
    }

    #[test]
    fn tool_schema_hints_resolve_action_enum_reference() {
        let router = BurpTools::burp_router();
        let tool = router.map.get("burp_settings").expect("settings tool");
        let (_, fields, actions) = schema_hints(&tool.attr);
        assert!(fields.contains(&"action".to_owned()));
        assert!(fields.contains(&"match".to_owned()));
        assert!(actions.contains(&"register_proxy_rule".to_owned()));
        assert!(has_nonempty_error(&serde_json::json!({"error": "boom"})));

        let result = super::invalid_arguments_result(
            &tool.attr,
            "burp_settings",
            &serde_json::Map::new(),
            "bad action".to_owned(),
        );
        assert_eq!(Some(true), result.is_error);
        let structured = result.structured_content.expect("structured error");
        assert!(structured["accepted_fields"].as_array().is_some());
        assert!(
            structured["valid_actions"]
                .as_array()
                .is_some_and(|actions| actions.contains(&serde_json::json!("register_proxy_rule")))
        );
        assert!(structured["correction"].as_str().is_some());
    }

    #[test]
    fn raw_request_export_preserves_input_without_host() {
        let request = "GET /health HTTP/1.1\r\n\r\n".to_owned();
        let exported = export_request_text(ExportRequestInput {
            request: request.clone(),
            host: None,
            format: Some("raw".to_owned()),
            https: None,
        })
        .expect("raw export must not require a host");

        assert_eq!(request, exported);
    }

    #[test]
    fn python_request_export_preserves_headers_and_body() {
        let exported = export_request_text(ExportRequestInput {
            request: "POST /submit HTTP/1.1\r\nHost: example.test\r\nAuthorization: Bearer token\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{}".to_owned(),
            host: None,
            format: Some("python".to_owned()),
            https: Some(true),
        })
        .expect("Python export must succeed");

        assert!(exported.contains("\"Host\": \"example.test\""));
        assert!(exported.contains("\"Authorization\": \"Bearer token\""));
        assert!(exported.contains("\"Content-Type\": \"application/json\""));
        assert!(!exported.contains("Content-Length"));
        assert!(exported.contains("data=\"{}\""));
    }

    #[test]
    fn decoder_accepts_legacy_base64_operation_name() {
        assert_eq!(
            "base64.encode",
            normalize_decoder_operation("base64_encode")
        );
        assert_eq!(
            "base64.decode",
            normalize_decoder_operation("base64_decode")
        );
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
        assert!(schema.pointer("/properties/action").is_none());
        assert!(schema.pointer("/properties/rule_action").is_some());
        assert!(schema.pointer("/properties/phase").is_some());
    }

    #[test]
    fn proxy_settings_schema_exposes_every_crud_selector() {
        let schema = serde_json::to_value(schemars::schema_for!(ProxySettingsUpdateInput))
            .expect("proxy settings input schema must serialize");
        for property in [
            "operation",
            "port",
            "target",
            "kind",
            "index",
            "rule",
            "master_enabled",
            "request_enabled",
            "response_enabled",
        ] {
            assert!(
                schema.pointer(&format!("/properties/{property}")).is_some(),
                "missing {property}"
            );
        }
    }

    #[test]
    fn proxy_interception_rule_schema_rejects_burp_ignored_relationships() {
        let rule_schema = serde_json::to_value(schemars::schema_for!(ProxyInterceptRuleInput))
            .expect("interception rule schema must serialize");
        let serialized = rule_schema.to_string();
        for value in [
            "matches",
            "does_not_match",
            "contains_parameters",
            "is_in_target_scope",
            "was_modified",
            "was_intercepted",
        ] {
            assert!(
                serialized.contains(&format!("\"{value}\"")),
                "missing {value}"
            );
        }
        assert!(!serialized.contains("\"contains\""));
        assert!(!serialized.contains("\"does_not_contain\""));
    }

    #[test]
    fn editor_tools_are_mounted_with_guarded_annotations() {
        let router = BurpTools::burp_router();
        for name in [
            "burp_editor_get",
            "burp_editor_patch",
            "burp_editor_renew_lease",
        ] {
            let route = router
                .map
                .get(name)
                .unwrap_or_else(|| panic!("missing {name}"));
            let annotations = route
                .attr
                .annotations
                .as_ref()
                .expect("editor annotations required");
            assert_eq!(Some(false), annotations.open_world_hint);
        }
    }

    #[test]
    fn proxy_settings_operations_build_typed_interception_mutations() {
        let upsert = proxy_settings_operation(ProxySettingsUpdateInput {
            operation: "intercept_rule_upsert".to_owned(),
            kind: Some("request".to_owned()),
            rule: Some(ProxyInterceptRuleInput {
                enabled: Some(true),
                boolean_operator: Some(ProxyInterceptRuleBooleanOperatorInput::And),
                match_type: ProxyInterceptRuleMatchTypeInput::Url,
                match_relationship: ProxyInterceptRuleRelationshipInput::Matches,
                match_condition: Some(".*/admin.*".to_owned()),
            }),
            ..empty_proxy_settings_input()
        })
        .expect("rule upsert must build");
        assert!(matches!(upsert, Operation::InterceptRuleUpsert(_)));

        let toggle = proxy_settings_operation(ProxySettingsUpdateInput {
            operation: "intercept_toggle".to_owned(),
            request_enabled: Some(true),
            response_enabled: Some(false),
            ..empty_proxy_settings_input()
        })
        .expect("toggle must build");
        assert!(matches!(toggle, Operation::InterceptToggle(_)));
    }
    #[test]
    fn editor_tool_schemas_expose_token_hash_and_payload_contracts() {
        let router = BurpTools::burp_router();
        let patch_schema = router
            .map
            .get("burp_editor_patch")
            .expect("burp_editor_patch tool missing")
            .attr
            .input_schema
            .clone();
        let patch_text =
            serde_json::to_string(&patch_schema).expect("Editor patch schema must serialize");
        for field in ["token", "expected_sha256"] {
            assert!(
                patch_text.contains(field),
                "Editor patch schema missing {field}"
            );
        }
    }

    fn empty_proxy_settings_input() -> ProxySettingsUpdateInput {
        ProxySettingsUpdateInput {
            operation: String::new(),
            port: None,
            running: None,
            listen_mode: None,
            listen_specific_address: None,
            certificate_mode: None,
            enable_http2: None,
            support_invisible_proxying: None,
            target: None,
            mode: None,
            script: None,
            script_id: None,
            script_name: None,
            kind: None,
            index: None,
            rule: None,
            master_enabled: None,
            request_enabled: None,
            response_enabled: None,
        }
    }

    #[test]
    fn sitegraph_tools_are_hidden_until_explicitly_enabled() {
        let disabled = BurpTools::tool_router_for(false);
        assert!(
            disabled
                .list_all()
                .iter()
                .all(|tool| tool.name != "sitegraph"
                    && !tool.name.starts_with(SITEGRAPH_TOOL_PREFIX))
        );

        let enabled = BurpTools::tool_router_for(true);
        assert!(
            enabled
                .list_all()
                .iter()
                .any(|tool| tool.name == "sitegraph")
        );
    }

    #[test]
    fn disabled_sitegraph_rejects_indexing_modes() {
        assert_eq!(Ok(()), BurpTools::validate_sitegraph_mode(false, "off"));
        assert_eq!(
            Err("sitegraph must be enabled before selecting an indexing mode".to_owned()),
            BurpTools::validate_sitegraph_mode(false, "startup")
        );
        assert_eq!(Ok(()), BurpTools::validate_sitegraph_mode(true, "watch"));
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

    #[test]
    fn action_and_core_tools_are_mounted() {
        let tools = actual_tool_names();
        for tool in [
            "burp_burp_version",
            "burp_extension_info",
            "burp_proxy",
            "burp_http",
            "burp_target",
            "burp_scanner",
            "burp_scan_config",
            "burp_fuzzer",
            "burp_collaborator",
            "burp_websocket",
            "burp_session",
            "burp_settings",
            "burp_logger",
            "burp_organizer",
            "burp_diff",
            "burp_verify_idor",
            "burp_check_cors",
            "burp_auth_matrix",
            "burp_audit_jwt",
            "burp_verify_ssrf",
            "burp_verify_sqli_blind",
            "burp_audit_graphql",
            "burp_verify_csrf_samesite",
            "burp_api_fuzz_orchestrator",
            "burp_editor_get",
            "burp_editor_patch",
            "burp_editor_renew_lease",
            "burp_cookie_jar_set",
            "burp_job_status",
            "burp_job_result",
            "burp_job_cancel",
            "burp_bambda_import",
            "burp_bcheck_import",
            "burp_add_issue",
            "burp_intercept_controller",
            "burp_intercepted_messages",
            "burp_control_intercepted_message",
            "burp_websocket_intercept_controller",
            "burp_intercepted_websocket_messages",
            "burp_control_intercepted_websocket_message",
            "decoder",
        ] {
            assert!(tools.contains(tool), "Tool must be mounted: {tool}");
        }
    }

    #[test]
    fn encode_bounded_base64_bounds_large_payload_by_default() {
        let large_payload = vec![b'X'; 1_700_000];
        let (encoded, truncated, length) = encode_bounded_base64(&large_payload, None);
        assert!(truncated, "1.7MB payload must be marked truncated");
        assert_eq!(length, 1_700_000);
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(&encoded)
            .expect("must be valid Base64");
        assert_eq!(decoded.len(), DEFAULT_MAX_BODY_LENGTH);
        assert_eq!(decoded.len(), 4096);
    }

    #[test]
    fn encode_bounded_base64_honors_explicit_max_body_length() {
        let large_payload = vec![b'Y'; 1_700_000];
        let (encoded, truncated, length) = encode_bounded_base64(&large_payload, Some(256));
        assert!(truncated, "payload must be marked truncated");
        assert_eq!(length, 1_700_000);
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(&encoded)
            .expect("must be valid Base64");
        assert_eq!(decoded.len(), 256);
    }

    #[test]
    fn encode_bounded_base64_preserves_small_payload_verbatim() {
        let small_payload = b"Hello, Burp MCP!".to_vec();
        let (encoded, truncated, length) = encode_bounded_base64(&small_payload, None);
        assert!(!truncated, "small payload must not be marked truncated");
        assert_eq!(length, small_payload.len());
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(&encoded)
            .expect("must be valid Base64");
        assert_eq!(decoded, small_payload);
    }

    #[test]
    fn intercepted_message_output_is_metadata_only_by_default_and_bounds_explicit_body() {
        let large_req = vec![b'R'; 1_700_000];
        let large_resp = vec![b'S'; 1_700_000];
        let proto_msg = burp_protocol::protocol::InterceptedMessage {
            id: 42,
            direction: "client_to_server".to_owned(),
            phase: "request".to_owned(),
            url: "https://example.test/upload".to_owned(),
            method: "POST".to_owned(),
            status: 200,
            is_in_scope: true,
            request: large_req.clone(),
            response: large_resp.clone(),
        };

        // 1. Default (include_bodies = false): metadata only, no large base64 emitted
        let meta_output = intercepted_message_output(proto_msg.clone(), false, None);
        assert_eq!(meta_output.request_base64, None);
        assert_eq!(meta_output.response_base64, None);
        assert_eq!(meta_output.request_length, Some(1_700_000));
        assert_eq!(meta_output.response_length, Some(1_700_000));

        // 2. Explicit include_bodies with default cap: bounded at 4096 bytes
        let bounded_output = intercepted_message_output(proto_msg.clone(), true, None);
        assert!(bounded_output.request_base64.is_some());
        assert_eq!(bounded_output.request_truncated, Some(true));
        assert_eq!(bounded_output.request_length, Some(1_700_000));
        let decoded_req = base64::engine::general_purpose::STANDARD
            .decode(bounded_output.request_base64.unwrap())
            .expect("decoded request base64");
        assert_eq!(decoded_req.len(), DEFAULT_MAX_BODY_LENGTH);

        // 3. Explicit cap: honors custom max_body_length
        let custom_output = intercepted_message_output(proto_msg, true, Some(128));
        let decoded_custom = base64::engine::general_purpose::STANDARD
            .decode(custom_output.request_base64.unwrap())
            .expect("decoded custom base64");
        assert_eq!(decoded_custom.len(), 128);
    }

    #[test]
    fn intercepted_websocket_message_output_is_metadata_only_by_default_and_bounds_explicit_body() {
        let large_payload = vec![b'W'; 1_700_000];
        let proto_ws = burp_protocol::protocol::InterceptedWebSocketMessage {
            id: 101,
            web_socket_id: 1,
            upgrade_url: "wss://example.test/ws".to_owned(),
            direction: "client_to_server".to_owned(),
            message_type: "text".to_owned(),
            phase: "message".to_owned(),
            payload: large_payload,
        };

        // 1. Default (include_bodies = false): metadata only
        let meta_ws = intercepted_websocket_message_output(proto_ws.clone(), false, None);
        assert_eq!(meta_ws.payload_base64, None);
        assert_eq!(meta_ws.payload_length, Some(1_700_000));

        // 2. Explicit include_bodies with default cap: bounded at 4096 bytes
        let bounded_ws = intercepted_websocket_message_output(proto_ws, true, None);
        assert!(bounded_ws.payload_base64.is_some());
        assert_eq!(bounded_ws.payload_truncated, Some(true));
        assert_eq!(bounded_ws.payload_length, Some(1_700_000));
        let decoded_ws = base64::engine::general_purpose::STANDARD
            .decode(bounded_ws.payload_base64.unwrap())
            .expect("decoded websocket base64");
        assert_eq!(decoded_ws.len(), DEFAULT_MAX_BODY_LENGTH);
    }

    #[test]
    fn to_send_output_with_options_truncates_large_payloads_with_default_cap() {
        let large_response = vec![b'A'; 1_700_000];
        let proto_resp = burp_protocol::protocol::SendRequestResponse {
            request: b"GET / HTTP/1.1\r\nHost: example.test\r\n\r\n".to_vec(),
            response: large_response,
            has_response: true,
            status: 200,
        };
        let send_output = to_send_output_with_options(proto_resp, false, None, None, None);
        assert_eq!(send_output.truncated, Some(true));
        assert!(send_output.response.is_some());
        let resp_text = send_output.response.unwrap();
        assert!(resp_text.contains("... [truncated"));
    }

    #[test]
    fn input_schemas_expose_include_bodies_and_max_body_length() {
        let ws_history_schema =
            serde_json::to_value(schemars::schema_for!(ProxyWebSocketHistoryInput))
                .expect("schema");
        assert!(
            ws_history_schema
                .pointer("/properties/include_bodies")
                .is_some()
        );
        assert!(
            ws_history_schema
                .pointer("/properties/max_body_length")
                .is_some()
        );

        let managed_ws_schema =
            serde_json::to_value(schemars::schema_for!(ManagedWebSocketHistoryInput))
                .expect("schema");
        assert!(
            managed_ws_schema
                .pointer("/properties/include_bodies")
                .is_some()
        );
        assert!(
            managed_ws_schema
                .pointer("/properties/max_body_length")
                .is_some()
        );

        let intercepted_msg_schema =
            serde_json::to_value(schemars::schema_for!(InterceptedMessagesInput)).expect("schema");
        assert!(
            intercepted_msg_schema
                .pointer("/properties/include_bodies")
                .is_some()
        );
        assert!(
            intercepted_msg_schema
                .pointer("/properties/max_body_length")
                .is_some()
        );

        let intercepted_ws_schema =
            serde_json::to_value(schemars::schema_for!(InterceptedWebSocketMessagesInput))
                .expect("schema");
        assert!(
            intercepted_ws_schema
                .pointer("/properties/include_bodies")
                .is_some()
        );
        assert!(
            intercepted_ws_schema
                .pointer("/properties/max_body_length")
                .is_some()
        );

        let ctrl_msg_schema =
            serde_json::to_value(schemars::schema_for!(ControlInterceptedMessageInput))
                .expect("schema");
        assert!(
            ctrl_msg_schema
                .pointer("/properties/max_body_length")
                .is_some()
        );

        let ctrl_ws_schema = serde_json::to_value(schemars::schema_for!(
            ControlInterceptedWebSocketMessageInput
        ))
        .expect("schema");
        assert!(
            ctrl_ws_schema
                .pointer("/properties/max_body_length")
                .is_some()
        );

        let proxy_detail_schema =
            serde_json::to_value(schemars::schema_for!(ProxyDetailInput)).expect("schema");
        assert!(
            proxy_detail_schema
                .pointer("/properties/include_bodies")
                .is_some()
        );
        assert!(
            proxy_detail_schema
                .pointer("/properties/max_body_length")
                .is_some()
        );

        let logger_detail_schema =
            serde_json::to_value(schemars::schema_for!(LoggerDetailInput)).expect("schema");
        assert!(
            logger_detail_schema
                .pointer("/properties/include_bodies")
                .is_some()
        );
        assert!(
            logger_detail_schema
                .pointer("/properties/max_body_length")
                .is_some()
        );

        let proxy_action_schema =
            serde_json::to_value(schemars::schema_for!(suite::ProxyActionInput)).expect("schema");
        assert!(
            proxy_action_schema
                .pointer("/properties/include_bodies")
                .is_some()
        );
        assert!(
            proxy_action_schema
                .pointer("/properties/max_body_length")
                .is_some()
        );

        let ws_action_schema =
            serde_json::to_value(schemars::schema_for!(suite::WebSocketActionInput))
                .expect("schema");
        assert!(
            ws_action_schema
                .pointer("/properties/include_bodies")
                .is_some()
        );
        assert!(
            ws_action_schema
                .pointer("/properties/max_body_length")
                .is_some()
        );

        let logger_action_schema =
            serde_json::to_value(schemars::schema_for!(suite::LoggerActionInput)).expect("schema");
        assert!(
            logger_action_schema
                .pointer("/properties/include_bodies")
                .is_some()
        );
        assert!(
            logger_action_schema
                .pointer("/properties/max_body_length")
                .is_some()
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
