use crate::diff_engine;
use burp_protocol::BurpClient;
use burp_protocol::protocol::{HttpHeaderEntry, SendRequestRequest};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct VerifyIdorInput {
    pub url: String,
    pub method: Option<String>,
    pub body: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub original_auth_header: String,
    pub victim_auth_header: String,
    pub auth_header_name: Option<String>,
    pub match_pattern: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct VerifyIdorOutput {
    pub vulnerable: bool,
    pub verdict: String,
    pub user_a_status: Option<u32>,
    pub user_b_status: Option<u32>,
    pub similarity_score: f64,
    pub pattern_matched_in_victim: bool,
    pub header_diffs: Vec<diff_engine::HeaderDiffEntry>,
    pub response_diff_summary: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CheckCorsInput {
    pub url: String,
    pub method: Option<String>,
    pub test_origins: Option<Vec<String>>,
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct CorsFinding {
    pub origin: String,
    pub allowed_origin: Option<String>,
    pub allow_credentials: Option<String>,
    pub vulnerable: bool,
    pub severity: String,
    pub description: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct CheckCorsOutput {
    pub url: String,
    pub findings: Vec<CorsFinding>,
    pub overall_vulnerable: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AuthMatrixInput {
    pub endpoints: Vec<String>,
    pub method: Option<String>,
    pub body: Option<String>,
    pub roles: HashMap<String, HashMap<String, String>>, // role_name -> headers
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AuthMatrixCell {
    pub endpoint: String,
    pub role: String,
    pub status: Option<u32>,
    pub length: usize,
    pub accessible: bool,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AuthMatrixOutput {
    pub matrix: Vec<AuthMatrixCell>,
    pub potential_access_control_violations: Vec<String>,
}

pub async fn run_verify_idor(
    client: &BurpClient,
    input: VerifyIdorInput,
) -> Result<VerifyIdorOutput, String> {
    let method = input.method.unwrap_or_else(|| "GET".to_string());
    let auth_header = input
        .auth_header_name
        .unwrap_or_else(|| "Authorization".to_string());
    let base_headers = input.headers.unwrap_or_default();

    // 1. Send Request as User A (Original)
    let mut headers_a = base_headers.clone();
    headers_a.insert(auth_header.clone(), input.original_auth_header);
    let proto_headers_a = headers_a
        .into_iter()
        .map(|(name, value)| HttpHeaderEntry { name, value })
        .collect();

    let resp_a = client
        .send_request(SendRequestRequest {
            method: method.clone(),
            url: input.url.clone(),
            body: input.body.clone().unwrap_or_default().into_bytes(),
            headers: proto_headers_a,
        })
        .await
        .map_err(|e| format!("Failed to send request for User A: {e}"))?;

    // 2. Send Request as User B (Victim)
    let mut headers_b = base_headers;
    headers_b.insert(auth_header, input.victim_auth_header);
    let proto_headers_b = headers_b
        .into_iter()
        .map(|(name, value)| HttpHeaderEntry { name, value })
        .collect();

    let resp_b = client
        .send_request(SendRequestRequest {
            method,
            url: input.url,
            body: input.body.unwrap_or_default().into_bytes(),
            headers: proto_headers_b,
        })
        .await
        .map_err(|e| format!("Failed to send request for User B: {e}"))?;

    let text_a = String::from_utf8_lossy(&resp_a.response).into_owned();
    let text_b = String::from_utf8_lossy(&resp_b.response).into_owned();

    let diff = diff_engine::compare_http_messages(&text_a, &text_b);

    let pattern_matched = if let Some(ref pat) = input.match_pattern {
        text_b.contains(pat)
    } else {
        false
    };

    let is_vulnerable =
        (resp_b.has_response && resp_b.status == 200 && diff.similarity_score > 0.85)
            || pattern_matched;

    let verdict = if is_vulnerable {
        "POTENTIAL_IDOR_CONFIRMED: Victim token accessed resource with identical response or pattern match".to_string()
    } else if resp_b.status == 401 || resp_b.status == 403 {
        "PROTECTED: Victim request returned Access Denied".to_string()
    } else {
        "INCONCLUSIVE_OR_DIFFERENT: Response difference detected between roles".to_string()
    };

    Ok(VerifyIdorOutput {
        vulnerable: is_vulnerable,
        verdict,
        user_a_status: resp_a.has_response.then_some(resp_a.status),
        user_b_status: resp_b.has_response.then_some(resp_b.status),
        similarity_score: diff.similarity_score,
        pattern_matched_in_victim: pattern_matched,
        header_diffs: diff.headers_diff,
        response_diff_summary: diff
            .body_diff
            .lines()
            .take(20)
            .collect::<Vec<_>>()
            .join("\n"),
    })
}

pub async fn run_check_cors(
    client: &BurpClient,
    input: CheckCorsInput,
) -> Result<CheckCorsOutput, String> {
    let method = input.method.unwrap_or_else(|| "GET".to_string());
    let default_origins = vec![
        "https://evil.com".to_string(),
        "null".to_string(),
        "https://target.com.evil.com".to_string(),
    ];
    let origins = input.test_origins.unwrap_or(default_origins);
    let mut findings = Vec::new();
    let mut overall_vulnerable = false;

    for origin in origins {
        let mut headers = input.headers.clone().unwrap_or_default();
        headers.insert("Origin".to_string(), origin.clone());

        let proto_headers = headers
            .into_iter()
            .map(|(name, value)| HttpHeaderEntry { name, value })
            .collect();

        if let Ok(resp) = client
            .send_request(SendRequestRequest {
                method: method.clone(),
                url: input.url.clone(),
                body: vec![],
                headers: proto_headers,
            })
            .await
        {
            let text = String::from_utf8_lossy(&resp.response).to_lowercase();
            let mut acao = None;
            let mut acac = None;

            for line in text.lines() {
                if line.starts_with("access-control-allow-origin:") {
                    acao = Some(line.split_once(':').unwrap().1.trim().to_string());
                } else if line.starts_with("access-control-allow-credentials:") {
                    acac = Some(line.split_once(':').unwrap().1.trim().to_string());
                }
            }

            let mut vuln = false;
            let mut severity = "INFO".to_string();
            let mut desc = "Origin rejected or restricted.".to_string();

            if let Some(ref allow_orig) = acao {
                if allow_orig == "*" {
                    if acac.as_deref() == Some("true") {
                        vuln = true;
                        severity = "HIGH".to_string();
                        desc = "Wildcard '*' origin with credentials enabled!".to_string();
                    } else {
                        vuln = true;
                        severity = "LOW".to_string();
                        desc = "Wildcard '*' origin allows public embedding.".to_string();
                    }
                } else if allow_orig.eq_ignore_ascii_case(&origin) {
                    if acac.as_deref() == Some("true") {
                        vuln = true;
                        severity = "CRITICAL".to_string();
                        desc = format!(
                            "Arbitrary origin reflection for '{origin}' with credentials enabled!"
                        );
                    } else {
                        vuln = true;
                        severity = "MEDIUM".to_string();
                        desc = format!("Origin reflection for '{origin}' without credentials.");
                    }
                }
            }

            if vuln && (severity == "HIGH" || severity == "CRITICAL" || severity == "MEDIUM") {
                overall_vulnerable = true;
            }

            findings.push(CorsFinding {
                origin,
                allowed_origin: acao,
                allow_credentials: acac,
                vulnerable: vuln,
                severity,
                description: desc,
            });
        }
    }

    Ok(CheckCorsOutput {
        url: input.url,
        findings,
        overall_vulnerable,
    })
}

pub async fn run_auth_matrix(
    client: &BurpClient,
    input: AuthMatrixInput,
) -> Result<AuthMatrixOutput, String> {
    let method = input.method.unwrap_or_else(|| "GET".to_string());
    let mut matrix = Vec::new();
    let mut violations = Vec::new();

    for endpoint in &input.endpoints {
        for (role_name, role_headers) in &input.roles {
            let proto_headers = role_headers
                .iter()
                .map(|(k, v)| HttpHeaderEntry {
                    name: k.clone(),
                    value: v.clone(),
                })
                .collect();

            let resp = client
                .send_request(SendRequestRequest {
                    method: method.clone(),
                    url: endpoint.clone(),
                    body: input.body.clone().unwrap_or_default().into_bytes(),
                    headers: proto_headers,
                })
                .await;

            let (status, length, accessible) = match resp {
                Ok(r) if r.has_response => {
                    let acc = r.status < 400;
                    (Some(r.status), r.response.len(), acc)
                }
                _ => (None, 0, false),
            };

            if accessible
                && (role_name.eq_ignore_ascii_case("anonymous")
                    || role_name.eq_ignore_ascii_case("guest")
                    || role_name.eq_ignore_ascii_case("unauthenticated"))
            {
                violations.push(format!("Endpoint `{endpoint}` is accessible by unauthenticated role: {role_name} (HTTP {:?})", status));
            }

            matrix.push(AuthMatrixCell {
                endpoint: endpoint.clone(),
                role: role_name.clone(),
                status,
                length,
                accessible,
            });
        }
    }

    Ok(AuthMatrixOutput {
        matrix,
        potential_access_control_violations: violations,
    })
}
