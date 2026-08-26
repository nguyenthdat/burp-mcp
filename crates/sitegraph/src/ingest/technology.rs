use crate::model::{SitemapObservation, TechnologyObservation};
use std::collections::BTreeSet;

const MAX_TECHNOLOGIES: usize = 64;
const MAX_BODY_BYTES: usize = 512 * 1024;

pub fn detect(observation: &SitemapObservation) -> Vec<TechnologyObservation> {
    let body = &observation.response_body[..observation.response_body.len().min(MAX_BODY_BYTES)];
    let body = String::from_utf8_lossy(body);
    let lower_body = body.to_ascii_lowercase();
    let content_type = observation.content_type.to_ascii_lowercase();
    let lower_url = observation.url.to_ascii_lowercase();
    let mut names = BTreeSet::new();

    if lower_url.contains("/graphql")
        || lower_body.contains("graphql")
        || lower_body.contains("__typename")
        || lower_body.contains("query {")
        || lower_body.contains("mutation {")
    {
        names.insert("GraphQL");
    }
    if lower_url.contains("/swagger")
        || lower_url.contains("/openapi")
        || lower_body.contains("swagger-ui")
        || lower_body.contains("\"openapi\"")
        || lower_body.contains("\"swagger\"")
    {
        names.insert("OpenAPI/Swagger");
    }
    if lower_body.contains("application/vnd.oai.openapi")
        || lower_url.ends_with(".yaml")
        || lower_url.ends_with(".yml")
    {
        names.insert("OpenAPI");
    }
    if lower_body.contains("apollo-client") || lower_body.contains("apollo cache") {
        names.insert("Apollo");
    }
    if lower_body.contains("relayenvironment") || lower_body.contains("relay-runtime") {
        names.insert("Relay");
    }
    if lower_body.contains("next/static") || lower_body.contains("__next_data__") {
        names.insert("Next.js");
    }
    if lower_body.contains("_nuxt/") || lower_body.contains("__nuxt") {
        names.insert("Nuxt");
    }
    if lower_body.contains("ng-version") || lower_body.contains("angular") {
        names.insert("Angular");
    }
    if lower_body.contains("data-reactroot") || lower_body.contains("react-dom") {
        names.insert("React");
    }
    if lower_body.contains("vue.js") || lower_body.contains("__vue__") {
        names.insert("Vue.js");
    }
    if lower_body.contains("svelte") || lower_body.contains("svelte-") {
        names.insert("Svelte");
    }
    if lower_body.contains("wp-content/") || lower_body.contains("wordpress") {
        names.insert("WordPress");
    }
    if lower_body.contains("drupal-settings-json") || lower_body.contains("drupal.settings") {
        names.insert("Drupal");
    }
    if lower_body.contains("laravel_session") || lower_body.contains("laravel") {
        names.insert("Laravel");
    }
    if lower_body.contains("django") || lower_body.contains("csrftoken") {
        names.insert("Django");
    }
    if lower_body.contains("spring") || lower_body.contains("whitelabel error page") {
        names.insert("Spring");
    }
    if lower_body.contains("asp.net") || lower_body.contains("__viewstate") {
        names.insert("ASP.NET");
    }
    if lower_body.contains("php") || lower_body.contains("phpsessid") {
        names.insert("PHP");
    }
    if lower_body.contains("express") || lower_body.contains("connect.sid") {
        names.insert("Express");
    }
    if lower_body.contains("fastapi") || lower_body.contains("starlette") {
        names.insert("FastAPI/Starlette");
    }
    if lower_body.contains("rails") || lower_body.contains("actioncontroller") {
        names.insert("Ruby on Rails");
    }
    if content_type.contains("application/graphql") {
        names.insert("GraphQL");
    }
    if content_type.contains("application/json")
        && (lower_url.contains("/api/") || lower_url.contains("/rest/"))
    {
        names.insert("JSON API");
    }

    names
        .into_iter()
        .take(MAX_TECHNOLOGIES)
        .map(|name| TechnologyObservation {
            name: name.to_owned(),
            endpoint_url: observation.url.clone(),
            method: observation.method.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::detect;
    use crate::model::SitemapObservation;

    fn observation(url: &str, content_type: &str, body: &[u8]) -> SitemapObservation {
        SitemapObservation {
            url: url.to_owned(),
            content_type: content_type.to_owned(),
            response_body: body.to_vec(),
            ..SitemapObservation::default()
        }
    }

    #[test]
    fn detects_graphql_and_common_frontend_frameworks() {
        let found = detect(&observation(
            "https://example.test/graphql",
            "application/json",
            br#"{"data":{"__typename":"User"},"build":"/_next/static/chunk.js"}"#,
        ));
        let names = found.into_iter().map(|item| item.name).collect::<Vec<_>>();
        assert!(names.iter().any(|name| name == "GraphQL"));
        assert!(names.iter().any(|name| name == "Next.js"));
    }

    #[test]
    fn detects_backend_and_api_markers_without_unbounded_body_scans() {
        let found = detect(&observation(
            "https://example.test/api/users",
            "application/json; charset=utf-8",
            b"Spring Whitelabel Error Page and connect.sid",
        ));
        let names = found.into_iter().map(|item| item.name).collect::<Vec<_>>();
        assert!(names.iter().any(|name| name == "Spring"));
        assert!(names.iter().any(|name| name == "Express"));
        assert!(names.iter().any(|name| name == "JSON API"));
    }
}
