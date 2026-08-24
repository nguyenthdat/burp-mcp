use regex::Regex;
use std::sync::LazyLock;

const MAX_LINKS: usize = 1_024;
const MAX_URL_BYTES: usize = 8 * 1_024;

static REFERENCE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)<(?P<tag>a|area|link|form|script)\b[^>]*?\b(?:href|action|src)\s*=\s*[\"'](?P<value>[^\"']+)[\"']"#)
        .expect("static HTML reference regex must compile")
});

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    pub kind: &'static str,
    pub value: String,
}

pub fn references(body: &[u8]) -> Vec<Reference> {
    let text = String::from_utf8_lossy(body);
    REFERENCE
        .captures_iter(&text)
        .filter_map(|capture| {
            let value = capture.name("value")?.as_str();
            if value.is_empty() || value.len() > MAX_URL_BYTES {
                return None;
            }
            let kind = match capture.name("tag")?.as_str().to_ascii_lowercase().as_str() {
                "form" => "form",
                "script" => "script",
                _ => "link",
            };
            Some(Reference {
                kind,
                value: value.to_owned(),
            })
        })
        .take(MAX_LINKS)
        .collect()
}
