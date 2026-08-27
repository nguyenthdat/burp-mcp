use crate::model::SitemapObservation;
use crate::normalize::url::metadata_url;
use serde_json::{Map, Value};

const MAX_DOCUMENT_BYTES: usize = 16 * 1024 * 1024;
const MAX_OPERATIONS: usize = 16_384;
const HTTP_METHODS: &[&str] = &[
    "get", "put", "post", "delete", "options", "head", "patch", "trace",
];

pub fn observations(
    document: &[u8],
    base_url: &str,
    limit: usize,
) -> Result<Vec<SitemapObservation>, String> {
    if limit == 0 {
        return Ok(Vec::new());
    }
    if document.len() > MAX_DOCUMENT_BYTES {
        return Err(format!(
            "OpenAPI document exceeds {MAX_DOCUMENT_BYTES} bytes"
        ));
    }
    let root: Value = serde_json::from_slice(document)
        .map_err(|error| format!("invalid OpenAPI JSON document: {error}"))?;
    let object = root
        .as_object()
        .ok_or_else(|| "OpenAPI document root must be an object".to_owned())?;
    validate_version(object)?;
    let paths = object
        .get("paths")
        .and_then(Value::as_object)
        .ok_or_else(|| "OpenAPI document requires an object-valued paths field".to_owned())?;
    let effective_base = resolve_base_url(object, base_url)?;
    let capacity = paths.len().min(limit).min(MAX_OPERATIONS);
    let mut output = Vec::with_capacity(capacity);

    'paths: for (path, path_item) in paths {
        if path.len() > 8 * 1024 || !path.starts_with('/') {
            continue;
        }
        let Some(path_item) = path_item.as_object() else {
            continue;
        };
        for method in HTTP_METHODS {
            if !path_item.contains_key(*method) {
                continue;
            }
            let Some(url) = endpoint_url(path, &effective_base) else {
                continue;
            };
            output.push(SitemapObservation {
                url,
                method: method.to_ascii_uppercase(),
                content_type: "application/json".to_owned(),
                ..SitemapObservation::default()
            });
            if output.len() >= limit.min(MAX_OPERATIONS) {
                break 'paths;
            }
        }
    }

    output.sort_unstable_by(|left, right| {
        (&left.url, &left.method).cmp(&(&right.url, &right.method))
    });
    output.dedup_by(|left, right| left.url == right.url && left.method == right.method);
    Ok(output)
}

fn validate_version(root: &Map<String, Value>) -> Result<(), String> {
    let openapi = root.get("openapi").and_then(Value::as_str);
    let swagger = root.get("swagger").and_then(Value::as_str);
    match (openapi, swagger) {
        (Some(version), _) if version.starts_with("3.") => Ok(()),
        (_, Some("2.0")) => Ok(()),
        (Some(version), _) => Err(format!("unsupported OpenAPI version {version}")),
        (_, Some(version)) => Err(format!("unsupported Swagger version {version}")),
        _ => Err("document is not OpenAPI or Swagger".to_owned()),
    }
}

fn resolve_base_url(root: &Map<String, Value>, supplied: &str) -> Result<String, String> {
    if let Some(server) = root
        .get("servers")
        .and_then(Value::as_array)
        .and_then(|servers| servers.first())
        .and_then(Value::as_object)
        .and_then(|server| server.get("url"))
        .and_then(Value::as_str)
        .and_then(|server| metadata_url(server, supplied))
    {
        return Ok(server);
    }

    if root.get("swagger").and_then(Value::as_str) == Some("2.0") {
        let scheme = root
            .get("schemes")
            .and_then(Value::as_array)
            .and_then(|schemes| schemes.first())
            .and_then(Value::as_str);
        let host = root.get("host").and_then(Value::as_str);
        let base_path = root.get("basePath").and_then(Value::as_str).unwrap_or("/");
        if let (Some(scheme), Some(host)) = (scheme, host) {
            let candidate = format!("{scheme}://{host}{base_path}");
            if let Some(url) = metadata_url(&candidate, supplied) {
                return Ok(url);
            }
        }
    }

    metadata_url(supplied, supplied).ok_or_else(|| "invalid OpenAPI base URL".to_owned())
}

fn endpoint_url(path: &str, base: &str) -> Option<String> {
    let mut base = url::Url::parse(base).ok()?;
    if !matches!(base.scheme(), "http" | "https") {
        return None;
    }
    base.set_query(None);
    base.set_fragment(None);
    let endpoint_segments = path.trim_start_matches('/').split('/');
    let mut segments = base.path_segments_mut().ok()?;
    segments.pop_if_empty().extend(endpoint_segments);
    drop(segments);
    Some(base.into())
}

#[cfg(test)]
mod tests {
    use super::observations;

    #[test]
    fn ingests_openapi_three_operations_against_server_url() {
        let document = br#"{
          "openapi":"3.1.0",
          "servers":[{"url":"/api/v1"}],
          "paths":{
            "/users":{"get":{},"post":{},"parameters":[]},
            "/users/{id}":{"delete":{}}
          }
        }"#;
        let found = observations(document, "https://example.test/spec/openapi.json", 10).unwrap();
        let values = found
            .into_iter()
            .map(|item| (item.method, item.url))
            .collect::<Vec<_>>();
        assert_eq!(
            values,
            vec![
                (
                    "GET".to_owned(),
                    "https://example.test/api/v1/users".to_owned()
                ),
                (
                    "POST".to_owned(),
                    "https://example.test/api/v1/users".to_owned()
                ),
                (
                    "DELETE".to_owned(),
                    "https://example.test/api/v1/users/%7Bid%7D".to_owned()
                ),
            ]
        );
    }

    #[test]
    fn preserves_absolute_server_path_prefix() {
        let document = br#"{
          "openapi":"3.0.3",
          "servers":[{"url":"https://example.test/api/v1/"}],
          "paths":{"/users/{id}":{"get":{}}}
        }"#;
        let found = observations(document, "https://fallback.test/openapi.json", 10).unwrap();
        assert_eq!(found[0].url, "https://example.test/api/v1/users/%7Bid%7D");
    }

    #[test]
    fn ingests_swagger_two_against_base_path_and_honors_limit() {
        let document = br#"{
          "swagger":"2.0","schemes":["https"],"host":"api.example.test","basePath":"/v2",
          "paths":{"/pets":{"get":{},"post":{}},"/owners":{"get":{}}}
        }"#;
        let found = observations(document, "https://fallback.test/swagger.json", 2).unwrap();
        let values = found
            .into_iter()
            .map(|item| (item.method, item.url))
            .collect::<Vec<_>>();
        assert_eq!(
            values,
            vec![
                (
                    "GET".to_owned(),
                    "https://api.example.test/v2/owners".to_owned()
                ),
                (
                    "GET".to_owned(),
                    "https://api.example.test/v2/pets".to_owned()
                ),
            ]
        );
    }

    #[test]
    fn rejects_non_openapi_json() {
        let error =
            observations(br#"{"paths":{}}"#, "https://example.test/openapi.json", 10).unwrap_err();
        assert_eq!("document is not OpenAPI or Swagger", error);
    }
}
