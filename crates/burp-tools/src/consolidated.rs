use serde::Deserialize;
use std::collections::HashMap;

// ==========================================
// 1. burp_proxy
// ==========================================
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ConsolidatedProxyInput {
    pub action: String, // "history", "detail", "annotate", "highlight", "extract"
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
pub struct ConsolidatedHttpInput {
    pub action: String, // "send", "send_batch", "convert", "export", "send_to_repeater"
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
pub struct ConsolidatedTargetInput {
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
pub struct ConsolidatedScannerInput {
    pub action: String, // "start_audit", "start_crawl", "stop", "list_issues", "issue_detail", "update_issue", "report"
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
}

// ==========================================
// 5. burp_fuzzer
// ==========================================
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ConsolidatedFuzzerInput {
    pub action: String, // "fuzz", "race", "send_to_intruder", "list_payloads", "upsert_payloads"
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
// 6. burp_collaborator
// ==========================================
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ConsolidatedCollaboratorInput {
    pub action: String, // "generate", "poll", "correlate"
    pub count: Option<u32>,
    pub target_url: Option<String>,
    pub injection_point: Option<String>,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

// ==========================================
// 7. burp_diff
// ==========================================
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ConsolidatedDiffInput {
    pub action: String, // "compare_exchanges", "diff_responses"
    pub response_a: Option<String>,
    pub response_b: Option<String>,
    pub index_a: Option<u32>,
    pub index_b: Option<u32>,
    pub first: Option<String>,
    pub second: Option<String>,
}
