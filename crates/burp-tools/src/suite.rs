use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HttpAction {
    Send,
    SendBatch,
    Convert,
    Export,
    SendToRepeater,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SettingsAction {
    GetProxySettings,
    UpdateProxySettings,
    ExportConfig,
    InspectConfig,
    ImportConfig,
    InterceptState,
    SetInterceptState,
    ProxyInterceptConfig,
    UpdateProxyInterceptConfig,
    RegisterHttpHandler,
    RemoveHttpHandler,
    RegisterProxyRule,
    ListProxyRules,
    RemoveProxyRule,
}

// ==========================================
// 1. burp_proxy
// ==========================================
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ProxyActionInput {
    pub action: String, // "history", "detail", "annotate", "highlight", "extract", "websocket_history"
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
    pub action: String, // "get_scope", "add_scope", "remove_scope", "info", "sitemap"
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
    pub action: String, // "start_audit", "start_crawl", "stop", "list_issues", "issue_detail", "update_issue", "report", "test_bcheck", "remove"
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
    pub action: String, // "list_configs", "get_config", "upsert_config", "delete_config", "list_pools", "get_pool", "upsert_pool", "delete_pool"
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
    pub action: String, // "fuzz", "race", "send_to_intruder", "list_payloads", "get_payload_list", "create_payload_list", "import_payload_list", "upsert_payloads", "delete_payload_list", "register_payload_processor", "list_payload_processors", "remove_payload_processor", "register_payload_generator", "list_payload_generators", "remove_payload_generator"
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
    pub action: String, // "generate", "poll", "correlate"
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
    pub action: String, // "create", "send_text", "send_binary", "history", "close", "list"
    pub host: Option<String>,
    pub port: Option<u32>,
    pub https: Option<bool>,
    pub path: Option<String>,
    pub id: Option<String>,
    pub text: Option<String>,
    pub data: Option<String>, // base64 binary payload
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

// ==========================================
// 9. burp_session
// ==========================================
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SessionActionInput {
    pub action: String, // "list_rules", "get_rule", "upsert_rule", "delete_rule", "run_macro", "upsert_macro", "list_macros", "delete_macro"
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
pub struct SettingsActionInput {
    pub action: SettingsAction,
    pub config: Option<String>,
    pub paths: Option<Vec<String>>,
    pub enabled: Option<bool>,
    pub operation: Option<String>,
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
    pub kind: Option<String>,
    #[serde(rename = "match")]
    pub match_text: Option<String>,
    pub replace: Option<String>,
    pub index: Option<u32>,
    pub rule: Option<crate::ProxyInterceptRuleInput>,
    pub master_enabled: Option<bool>,
    pub request_enabled: Option<bool>,
    pub response_enabled: Option<bool>,
}

// ==========================================
// 11. burp_logger
// ==========================================
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LoggerActionInput {
    pub action: String, // "query", "detail", "clear"
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
    pub action: String, // "add", "list"
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
    pub action: String, // "compare_exchanges", "diff_responses"
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
    pub action: String, // "status", "sync", "search", "security_view", "neighbors", "trace", "shortest_path", "clusters", "impact", "diff", "export", "import_spec", "history_search", "endpoint_detail", "projects", "config"
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
