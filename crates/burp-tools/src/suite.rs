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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EditorParamType {
    Query,
    Body,
}

impl EditorParamType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Query => "query",
            Self::Body => "body",
        }
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EditorGetInput {
    /// Optional editor target hint (e.g. 'request' / 'req', 'response' / 'resp', 'websocket' / 'ws', or tab name substring). If omitted, resolves to currently focused or last-active editor.
    pub target_hint: Option<String>,
    /// Optional lease time-to-live in seconds (defaults to extension configuration if omitted).
    pub ttl_seconds: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum EditorPatchOperation {
    ReplaceSelection {
        text: String,
    },
    SetHeader {
        name: String,
        #[serde(default)]
        value: String,
        #[serde(default)]
        remove: bool,
    },
    JsonPatch {
        json_path: String,
        value_json: String,
    },
    SetParam {
        name: String,
        #[serde(default)]
        value: String,
        #[serde(default)]
        remove: bool,
        param_type: Option<EditorParamType>,
    },
    Regex {
        pattern: String,
        replacement: String,
        #[serde(default)]
        replace_all: bool,
        #[serde(default)]
        case_insensitive: bool,
    },
    ReplaceAll {
        text: Option<String>,
        payload_base64: Option<String>,
    },
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EditorPatchInput {
    pub token: String,
    pub expected_sha256: String,
    #[serde(flatten)]
    pub operation: EditorPatchOperation,
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_editor_patch_input_deserialization_all_modes() {
        // 1. replace_selection
        let json_data = json!({
            "token": "lease-123",
            "expected_sha256": "abc123hash",
            "mode": "replace_selection",
            "text": "new selected text"
        });
        let patch: EditorPatchInput =
            serde_json::from_value(json_data).expect("replace_selection deserializes");
        assert_eq!(patch.token, "lease-123");
        assert_eq!(patch.expected_sha256, "abc123hash");
        assert_eq!(
            patch.operation,
            EditorPatchOperation::ReplaceSelection {
                text: "new selected text".to_string()
            }
        );

        // 2. set_header
        let json_data = json!({
            "token": "lease-123",
            "expected_sha256": "abc123hash",
            "mode": "set_header",
            "name": "Authorization",
            "value": "Bearer token123"
        });
        let patch: EditorPatchInput =
            serde_json::from_value(json_data).expect("set_header deserializes");
        assert_eq!(
            patch.operation,
            EditorPatchOperation::SetHeader {
                name: "Authorization".to_string(),
                value: "Bearer token123".to_string(),
                remove: false,
            }
        );

        // set_header with remove
        let json_data = json!({
            "token": "lease-123",
            "expected_sha256": "abc123hash",
            "mode": "set_header",
            "name": "X-Old-Header",
            "remove": true
        });
        let patch: EditorPatchInput =
            serde_json::from_value(json_data).expect("set_header remove deserializes");
        assert_eq!(
            patch.operation,
            EditorPatchOperation::SetHeader {
                name: "X-Old-Header".to_string(),
                value: "".to_string(),
                remove: true,
            }
        );

        // 3. json_patch
        let json_data = json!({
            "token": "lease-123",
            "expected_sha256": "abc123hash",
            "mode": "json_patch",
            "json_path": "user.roles[0]",
            "value_json": "\"admin\""
        });
        let patch: EditorPatchInput =
            serde_json::from_value(json_data).expect("json_patch deserializes");
        assert_eq!(
            patch.operation,
            EditorPatchOperation::JsonPatch {
                json_path: "user.roles[0]".to_string(),
                value_json: "\"admin\"".to_string(),
            }
        );

        // 4. set_param (query)
        let json_data = json!({
            "token": "lease-123",
            "expected_sha256": "abc123hash",
            "mode": "set_param",
            "name": "search",
            "value": "term",
            "param_type": "query"
        });
        let patch: EditorPatchInput =
            serde_json::from_value(json_data).expect("set_param query deserializes");
        assert_eq!(
            patch.operation,
            EditorPatchOperation::SetParam {
                name: "search".to_string(),
                value: "term".to_string(),
                remove: false,
                param_type: Some(EditorParamType::Query),
            }
        );

        // 4. set_param (body)
        let json_data = json!({
            "token": "lease-123",
            "expected_sha256": "abc123hash",
            "mode": "set_param",
            "name": "csrf",
            "value": "xyz",
            "param_type": "body"
        });
        let patch: EditorPatchInput =
            serde_json::from_value(json_data).expect("set_param body deserializes");
        assert_eq!(
            patch.operation,
            EditorPatchOperation::SetParam {
                name: "csrf".to_string(),
                value: "xyz".to_string(),
                remove: false,
                param_type: Some(EditorParamType::Body),
            }
        );

        // 5. regex
        let json_data = json!({
            "token": "lease-123",
            "expected_sha256": "abc123hash",
            "mode": "regex",
            "pattern": "User: \\w+",
            "replacement": "User: admin",
            "replace_all": true,
            "case_insensitive": false
        });
        let patch: EditorPatchInput =
            serde_json::from_value(json_data).expect("regex deserializes");
        assert_eq!(
            patch.operation,
            EditorPatchOperation::Regex {
                pattern: "User: \\w+".to_string(),
                replacement: "User: admin".to_string(),
                replace_all: true,
                case_insensitive: false,
            }
        );

        // 6. replace_all text
        let json_data = json!({
            "token": "lease-123",
            "expected_sha256": "abc123hash",
            "mode": "replace_all",
            "text": "full content"
        });
        let patch: EditorPatchInput =
            serde_json::from_value(json_data).expect("replace_all text deserializes");
        assert_eq!(
            patch.operation,
            EditorPatchOperation::ReplaceAll {
                text: Some("full content".to_string()),
                payload_base64: None,
            }
        );

        // 6. replace_all payload_base64
        let json_data = json!({
            "token": "lease-123",
            "expected_sha256": "abc123hash",
            "mode": "replace_all",
            "payload_base64": "SGVsbG8="
        });
        let patch: EditorPatchInput =
            serde_json::from_value(json_data).expect("replace_all payload_base64 deserializes");
        assert_eq!(
            patch.operation,
            EditorPatchOperation::ReplaceAll {
                text: None,
                payload_base64: Some("SGVsbG8=".to_string()),
            }
        );
    }

    #[test]
    fn test_editor_patch_input_rejects_invalid_inputs_and_removed_aliases() {
        // 1. regex_replace alias must be rejected
        let alias_data = json!({
            "token": "lease-123",
            "expected_sha256": "abc123hash",
            "mode": "regex_replace",
            "pattern": "foo",
            "replacement": "bar"
        });
        let err = serde_json::from_value::<EditorPatchInput>(alias_data).unwrap_err();
        assert!(
            err.to_string().contains("unknown variant"),
            "error was: {err}"
        );

        // 2. Unknown mode must be rejected
        let unknown_mode = json!({
            "token": "lease-123",
            "expected_sha256": "abc123hash",
            "mode": "unknown_action"
        });
        let err = serde_json::from_value::<EditorPatchInput>(unknown_mode).unwrap_err();
        assert!(
            err.to_string().contains("unknown variant"),
            "error was: {err}"
        );

        // 3. Missing required field in replace_selection (missing text)
        let missing_text = json!({
            "token": "lease-123",
            "expected_sha256": "abc123hash",
            "mode": "replace_selection"
        });
        let err = serde_json::from_value::<EditorPatchInput>(missing_text).unwrap_err();
        assert!(
            err.to_string().contains("missing field"),
            "error was: {err}"
        );

        // 4. Missing required field in set_header (missing name)
        let missing_name = json!({
            "token": "lease-123",
            "expected_sha256": "abc123hash",
            "mode": "set_header",
            "value": "bar"
        });
        let err = serde_json::from_value::<EditorPatchInput>(missing_name).unwrap_err();
        assert!(
            err.to_string().contains("missing field"),
            "error was: {err}"
        );

        // 5. Missing required field in json_patch (missing value_json)
        let missing_val = json!({
            "token": "lease-123",
            "expected_sha256": "abc123hash",
            "mode": "json_patch",
            "json_path": "path"
        });
        let err = serde_json::from_value::<EditorPatchInput>(missing_val).unwrap_err();
        assert!(
            err.to_string().contains("missing field"),
            "error was: {err}"
        );

        // 6. Invalid param_type
        let invalid_param_type = json!({
            "token": "lease-123",
            "expected_sha256": "abc123hash",
            "mode": "set_param",
            "name": "id",
            "param_type": "cookie"
        });
        let err = serde_json::from_value::<EditorPatchInput>(invalid_param_type).unwrap_err();
        assert!(
            err.to_string().contains("unknown variant"),
            "error was: {err}"
        );
    }

    #[test]
    fn test_editor_patch_schema_discriminated_by_mode() {
        let schema = serde_json::to_value(schemars::schema_for!(EditorPatchInput))
            .expect("EditorPatchInput schema must serialize");
        let schema_text = schema.to_string();

        // Must contain mode as property or discriminator
        assert!(schema_text.contains("\"mode\""), "schema must contain mode");
        // Must contain all valid modes
        for mode in [
            "replace_selection",
            "set_header",
            "json_patch",
            "set_param",
            "regex",
            "replace_all",
        ] {
            assert!(
                schema_text.contains(&format!("\"{mode}\"")),
                "missing mode '{mode}' in schema"
            );
        }
        // Must NOT contain old alias regex_replace
        assert!(
            !schema_text.contains("\"regex_replace\""),
            "schema must not contain removed alias 'regex_replace'"
        );

        // Verify EditorGetInput target_hint description
        let get_schema = serde_json::to_value(schemars::schema_for!(EditorGetInput))
            .expect("EditorGetInput schema must serialize");
        let get_schema_text = get_schema.to_string();
        assert!(
            get_schema_text.contains("target_hint"),
            "EditorGetInput schema must contain target_hint"
        );
        assert!(
            get_schema_text.contains("hint"),
            "EditorGetInput schema must contain description for target_hint"
        );
    }
}
