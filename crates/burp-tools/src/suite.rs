use serde::Deserialize;
use std::collections::HashMap;

fn require_object_schema(schema: &mut schemars::Schema) {
    schema.insert("type".to_owned(), "object".into());
}

// ==========================================
// Action Enums
// ==========================================

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HttpAction {
    Send,
    SendBatch,
    Convert,
    Export,
    SendToRepeater,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProxyAction {
    History,
    Detail,
    Annotate,
    Highlight,
    Extract,
    WebsocketHistory,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TargetAction {
    GetScope,
    AddScope,
    RemoveScope,
    Info,
    Sitemap,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ScannerAction {
    StartAudit,
    StartCrawl,
    Stop,
    ListIssues,
    IssueDetail,
    UpdateIssue,
    Report,
    TestBcheck,
    Remove,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ScanConfigAction {
    ListConfigs,
    GetConfig,
    UpsertConfig,
    DeleteConfig,
    ListPools,
    GetPool,
    UpsertPool,
    DeletePool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FuzzerAction {
    Fuzz,
    Race,
    SendToIntruder,
    ListPayloads,
    GetPayloadList,
    CreatePayloadList,
    ImportPayloadList,
    UpsertPayloads,
    DeletePayloadList,
    RegisterPayloadProcessor,
    ListPayloadProcessors,
    RemovePayloadProcessor,
    RegisterPayloadGenerator,
    ListPayloadGenerators,
    RemovePayloadGenerator,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CollaboratorAction {
    Generate,
    Poll,
    Correlate,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WebSocketAction {
    Create,
    SendText,
    SendBinary,
    History,
    Close,
    List,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SessionAction {
    ListRules,
    GetRule,
    UpsertRule,
    DeleteRule,
    RunMacro,
    UpsertMacro,
    ListMacros,
    DeleteMacro,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LoggerAction {
    Query,
    Detail,
    Clear,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OrganizerAction {
    Add,
    List,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DiffAction {
    CompareExchanges,
    DiffResponses,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SiteGraphAction {
    Status,
    Stats,
    Sync,
    Search,
    SecurityView,
    ImportSpec,
    Neighbors,
    Trace,
    ShortestPath,
    Clusters,
    Impact,
    Diff,
    Export,
    HistorySearch,
    EndpointDetail,
    Projects,
    Config,
}

// ==========================================
// 1. burp_proxy
// ==========================================
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ProxyActionInput {
    pub action: ProxyAction,
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
    pub index: Option<u32>,
    pub notes: Option<String>,
    pub regex: Option<String>,
}

// ==========================================
// 2. burp_http
// ==========================================
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct HttpActionInput {
    pub action: HttpAction,
    pub method: Option<String>,
    pub url: Option<String>,
    pub body: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub headers_only: Option<bool>,
    pub extract_css: Option<String>,
    pub extract_json: Option<String>,
    pub max_body_length: Option<usize>,
    pub requests: Option<Vec<crate::SendRequestInput>>,
    pub request: Option<String>,
    pub convert_to: Option<String>,
    pub host: Option<String>,
    pub port: Option<u32>,
    pub https: Option<bool>,
    pub format: Option<String>,
    pub tab_name: Option<String>,
}

// ==========================================
// 3. burp_target
// ==========================================
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TargetActionInput {
    pub action: TargetAction,
    pub url: Option<String>,
    pub url_prefix: Option<String>,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

// ==========================================
// 4. burp_scanner
// ==========================================
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ScannerActionInput {
    pub action: ScannerAction,
    pub url: Option<String>,
    pub audit_type: Option<String>,
    pub seed_urls: Option<Vec<String>>,
    pub scan_configuration_id: Option<String>,
    pub resource_pool_id: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub stable_seconds: Option<u64>,
    pub include_out_of_scope: Option<bool>,
    pub job_id: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub cursor: Option<String>,
    pub index: Option<u32>,
    pub status: Option<String>,
    pub severity: Option<String>,
    pub confidence: Option<String>,
    pub notes: Option<String>,
    pub format: Option<String>,
    pub path: Option<String>,
    pub issue_indexes: Option<Vec<u32>>,
    pub script: Option<String>,
    pub request: Option<String>,
    pub response: Option<String>,
    pub host: Option<String>,
    pub port: Option<u32>,
    pub https: Option<bool>,
}

// ==========================================
// 5. burp_scan_config
// ==========================================
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ScanConfigActionInput {
    pub action: ScanConfigAction,
    pub id: Option<String>,
    pub name: Option<String>,
    pub scan_type: Option<String>,
    pub audit_type: Option<String>,
    pub include_out_of_scope: Option<bool>,
    pub timeout_seconds: Option<u64>,
    pub stable_seconds: Option<u64>,
    pub resource_pool_id: Option<String>,
    pub kind: Option<String>,
    pub existing_pool_name: Option<String>,
    pub concurrent_request_limit: Option<u32>,
    pub throttle_millis: Option<u64>,
    pub max_retries: Option<u32>,
}

// ==========================================
// 6. burp_fuzzer
// ==========================================
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FuzzerActionInput {
    pub action: FuzzerAction,
    pub template: Option<String>,
    pub host: Option<String>,
    pub port: Option<u32>,
    pub https: Option<bool>,
    pub marker: Option<String>,
    pub wordlist: Option<Vec<String>>,
    pub payload_list_id: Option<String>,
    pub payload_offset: Option<u32>,
    pub attack_mode: Option<String>,
    pub markers: Option<HashMap<String, Vec<String>>>,
    pub request: Option<String>,
    pub count: Option<u32>,
    pub single_packet_attack: Option<bool>,
    pub tab_name: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub payloads: Option<Vec<String>>,
}

// ==========================================
// 7. burp_collaborator
// ==========================================
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CollaboratorActionInput {
    pub action: CollaboratorAction,
    pub count: Option<u32>,
    pub target_url: Option<String>,
    pub injection_point: Option<String>,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

// ==========================================
// 8. burp_websocket
// ==========================================
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WebSocketActionInput {
    pub action: WebSocketAction,
    pub host: Option<String>,
    pub port: Option<u32>,
    pub https: Option<bool>,
    pub path: Option<String>,
    pub id: Option<String>,
    pub text: Option<String>,
    pub data: Option<String>, // base64 binary payload
    pub limit: Option<u32>,
    pub cursor: Option<String>,
    pub include_bodies: Option<bool>,
    pub max_body_length: Option<usize>,
}

// ==========================================
// 9. burp_session
// ==========================================
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SessionActionInput {
    pub action: SessionAction,
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
    pub serial_number: Option<u64>,
    pub items: Option<Vec<crate::MacroItemInput>>,
}

// ==========================================
// 10. burp_settings
// ==========================================
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(transform = require_object_schema)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum SettingsActionInput {
    GetProxySettings,
    UpdateProxySettings {
        update: crate::ProxySettingsUpdateInput,
    },
    ExportConfig,
    InspectConfig {
        paths: Option<Vec<String>>,
    },
    ImportConfig {
        config: Option<String>,
    },
    InterceptState,
    SetInterceptState {
        enabled: Option<bool>,
    },
    ProxyInterceptConfig,
    UpdateProxyInterceptConfig {
        master_enabled: Option<bool>,
        request_enabled: Option<bool>,
        response_enabled: Option<bool>,
    },
    RegisterHttpHandler {
        header_name: Option<String>,
        header_value: Option<String>,
        #[serde(rename = "match")]
        match_text: Option<String>,
        replace: Option<String>,
    },
    RemoveHttpHandler,
    RegisterProxyRule {
        id: Option<String>,
        url_contains: String,
        phase: Option<crate::ProxyRulePhaseInput>,
        rule_action: Option<crate::ProxyRuleActionInput>,
        #[serde(rename = "match")]
        match_text: Option<String>,
        replace: Option<String>,
        header_name: Option<String>,
        header_value: Option<String>,
        enabled: Option<bool>,
    },
    ListProxyRules,
    RemoveProxyRule {
        id: Option<String>,
    },
}

// ==========================================
// 11. burp_logger
// ==========================================
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LoggerActionInput {
    pub action: LoggerAction,
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
    pub index: Option<u32>,
}

// ==========================================
// 12. burp_organizer
// ==========================================
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct OrganizerActionInput {
    pub action: OrganizerAction,
    pub request: Option<String>,
    pub response: Option<String>,
    pub host: Option<String>,
    pub port: Option<u32>,
    pub https: Option<bool>,
    pub notes: Option<String>,
    pub highlight: Option<String>,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
    pub status_filter: Option<String>,
    pub url_filter: Option<String>,
}

// ==========================================
// 13. burp_diff
// ==========================================
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DiffActionInput {
    pub action: DiffAction,
    pub response_a: Option<String>,
    pub response_b: Option<String>,
    pub index_a: Option<u32>,
    pub index_b: Option<u32>,
    pub first: Option<String>,
    pub second: Option<String>,
}

// ==========================================
// 14. burp_editor
// ==========================================
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EditorGetInput {
    pub target_hint: Option<String>,
    pub ttl_seconds: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EditorPatchInput {
    pub token: String,
    pub expected_sha256: String,
    pub mode: Option<String>, // "replace_selection", "regex", "set_header", "json_patch", "set_param", "replace_all"
    pub text: Option<String>,
    pub payload_base64: Option<String>,
    pub selection_replacement: Option<String>,
    pub header_name: Option<String>,
    pub header_value: Option<String>,
    pub header_remove: Option<bool>,
    pub regex_pattern: Option<String>,
    pub regex_replacement: Option<String>,
    pub regex_replace_all: Option<bool>,
    pub regex_case_insensitive: Option<bool>,
    pub json_path: Option<String>,
    pub json_value: Option<String>,
    pub param_name: Option<String>,
    pub param_value: Option<String>,
    pub param_remove: Option<bool>,
    pub param_type: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EditorRenewInput {
    pub token: String,
    pub extend_seconds: Option<u32>,
}

// ==========================================
// 15. sitegraph
// ==========================================
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SiteGraphActionInput {
    pub action: SiteGraphAction,
    pub url_prefix: Option<String>,
    pub query: Option<String>,
    pub id: Option<String>,
    pub from_id: Option<String>,
    pub to_id: Option<String>,
    pub limit: Option<u32>,
    pub cursor: Option<u32>,
    pub max_depth: Option<u32>,
    pub since: Option<i64>,
    pub profile: Option<String>,
    pub format: Option<String>, // "json", "mermaid", "ascii_tree", "csv"
    pub snapshot_id: Option<String>,
    pub spec_content: Option<String>,
    pub view_name: Option<String>,
}
