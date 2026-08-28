use regex::Regex;
use serde_json::Value;

mod jsonpath_parser {
    use pest_derive::Parser;

    #[derive(Parser)]
    #[grammar = "jsonpath.pest"]
    pub struct JsonPathParser;
}

mod css_parser {
    use pest_derive::Parser;

    #[derive(Parser)]
    #[grammar = "css.pest"]
    pub struct CssParser;
}

use css_parser::Rule as CssRule;
use jsonpath_parser::Rule as JsonRule;
use pest::Parser;

#[derive(Debug, PartialEq, Eq)]
pub enum JsonSegment {
    Field(String),
    Index(isize),
    Wildcard,
}

pub fn parse_json_path(path: &str) -> Result<Vec<JsonSegment>, String> {
    let pairs = jsonpath_parser::JsonPathParser::parse(JsonRule::json_path, path)
        .map_err(|e| format!("JSONPath parse error: {e}"))?;

    let mut segments = Vec::new();
    for pair in pairs {
        for inner in pair.into_inner() {
            match inner.as_rule() {
                JsonRule::root_ident => {
                    segments.push(JsonSegment::Field(inner.as_str().to_string()));
                }
                JsonRule::dot_segment => {
                    for part in inner.into_inner() {
                        match part.as_rule() {
                            JsonRule::wildcard => segments.push(JsonSegment::Wildcard),
                            JsonRule::ident => {
                                segments.push(JsonSegment::Field(part.as_str().to_string()))
                            }
                            _ => {}
                        }
                    }
                }
                JsonRule::bracket_segment => {
                    for part in inner.into_inner() {
                        match part.as_rule() {
                            JsonRule::wildcard => segments.push(JsonSegment::Wildcard),
                            JsonRule::number => {
                                if let Ok(n) = part.as_str().parse::<isize>() {
                                    segments.push(JsonSegment::Index(n));
                                }
                            }
                            JsonRule::string_lit => {
                                let s = part.as_str();
                                let unquoted = if s.len() >= 2 { &s[1..s.len() - 1] } else { s };
                                segments.push(JsonSegment::Field(unquoted.to_string()));
                            }
                            JsonRule::ident => {
                                segments.push(JsonSegment::Field(part.as_str().to_string()))
                            }
                            _ => {}
                        }
                    }
                }
                JsonRule::EOI => {}
                _ => {}
            }
        }
    }
    Ok(segments)
}

pub fn extract_json_path(json_text: &str, path: &str) -> Result<Vec<Value>, String> {
    let parsed: Value =
        serde_json::from_str(json_text).map_err(|e| format!("Invalid JSON for extraction: {e}"))?;

    let segments = parse_json_path(path)?;
    let mut current = vec![parsed];

    for segment in &segments {
        let mut next = Vec::new();
        for val in &current {
            match segment {
                JsonSegment::Field(field) => {
                    if let Value::Object(map) = val {
                        if let Some(v) = map.get(field) {
                            next.push(v.clone());
                        }
                    }
                }
                JsonSegment::Index(idx) => {
                    if let Value::Array(arr) = val {
                        let actual_idx = if *idx < 0 {
                            arr.len() as isize + *idx
                        } else {
                            *idx
                        };
                        if actual_idx >= 0 && (actual_idx as usize) < arr.len() {
                            next.push(arr[actual_idx as usize].clone());
                        }
                    }
                }
                JsonSegment::Wildcard => {
                    if let Value::Array(arr) = val {
                        next.extend(arr.iter().cloned());
                    } else if let Value::Object(map) = val {
                        next.extend(map.values().cloned());
                    }
                }
            }
        }
        current = next;
    }

    Ok(current)
}

#[derive(Debug, Default, Clone)]
pub struct ParsedCssStep {
    pub tag: Option<String>,
    pub id: Option<String>,
    pub class: Option<String>,
    pub attr_name: Option<String>,
    pub attr_val: Option<String>,
}

pub fn parse_css_selector(selector: &str) -> Result<Vec<ParsedCssStep>, String> {
    let pairs = css_parser::CssParser::parse(CssRule::css_query, selector)
        .map_err(|e| format!("CSS selector parse error: {e}"))?;

    let mut steps = Vec::new();
    for pair in pairs {
        for inner in pair.into_inner() {
            if inner.as_rule() == CssRule::selector_step {
                let mut step = ParsedCssStep::default();
                for part in inner.into_inner() {
                    match part.as_rule() {
                        CssRule::tag_name => {
                            let t = part.as_str();
                            if t != "*" {
                                step.tag = Some(t.to_lowercase());
                            }
                        }
                        CssRule::id_selector => {
                            let mut id_parts = part.into_inner();
                            if let Some(id_ident) = id_parts.next() {
                                step.id = Some(id_ident.as_str().to_string());
                            }
                        }
                        CssRule::class_selector => {
                            let mut class_parts = part.into_inner();
                            if let Some(c_ident) = class_parts.next() {
                                step.class = Some(c_ident.as_str().to_string());
                            }
                        }
                        CssRule::attr_selector => {
                            let mut attr_parts = part.into_inner();
                            if let Some(attr_name_part) = attr_parts.next() {
                                step.attr_name = Some(attr_name_part.as_str().to_string());
                            }
                            if let Some(_op) = attr_parts.next() {
                                if let Some(val_part) = attr_parts.next() {
                                    let raw = val_part.as_str();
                                    let clean = if (raw.starts_with('"') && raw.ends_with('"'))
                                        || (raw.starts_with('\'') && raw.ends_with('\''))
                                    {
                                        if raw.len() >= 2 {
                                            &raw[1..raw.len() - 1]
                                        } else {
                                            raw
                                        }
                                    } else {
                                        raw
                                    };
                                    step.attr_val = Some(clean.to_string());
                                }
                            }
                        }
                        _ => {}
                    }
                }
                steps.push(step);
            }
        }
    }
    Ok(steps)
}

pub fn extract_css_selector(html: &str, selector: &str) -> Result<Vec<String>, String> {
    let steps = parse_css_selector(selector)?;
    if steps.is_empty() {
        return Ok(vec![]);
    }

    let mut current_blocks = vec![html.to_string()];
    for step in &steps {
        let mut next_matches = Vec::new();
        for block in &current_blocks {
            let matched = match_css_step(block, step)?;
            next_matches.extend(matched);
        }
        current_blocks = next_matches;
    }

    Ok(current_blocks)
}

fn match_css_step(html: &str, step: &ParsedCssStep) -> Result<Vec<String>, String> {
    let tag_pattern = step.tag.as_deref().unwrap_or(r"[a-zA-Z0-9_-]+");
    let open_tag_pattern = format!(r"(?is)<({tag_pattern})(\s+[^>]*)?(/)?>");
    let re_open =
        Regex::new(&open_tag_pattern).map_err(|e| format!("Invalid selector regex: {e}"))?;

    let mut results = Vec::new();
    for cap in re_open.captures_iter(html) {
        let whole_open = cap.get(0).map(|m| m.as_str()).unwrap_or_default();
        let matched_tag = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
        let attrs_str = cap.get(2).map(|m| m.as_str()).unwrap_or_default();
        let is_self_closing = cap.get(3).is_some() || whole_open.ends_with("/>");

        if let Some(required_id) = &step.id {
            if !has_attribute_value(attrs_str, "id", required_id) {
                continue;
            }
        }
        if let Some(required_class) = &step.class {
            if !has_class_value(attrs_str, required_class) {
                continue;
            }
        }
        if let Some(req_attr) = &step.attr_name {
            if let Some(req_val) = &step.attr_val {
                if !has_attribute_value(attrs_str, req_attr, req_val) {
                    continue;
                }
            } else if !has_attribute(attrs_str, req_attr) {
                continue;
            }
        }

        if is_self_closing {
            results.push(whole_open.trim().to_string());
        } else {
            // Find matching closing tag
            let start_pos = cap.get(0).map(|m| m.start()).unwrap_or(0);
            let after_open = &html[start_pos..];
            let close_tag = format!("</{matched_tag}>");
            if let Some(end_offset) = after_open.to_lowercase().find(&close_tag.to_lowercase()) {
                let full_element = &after_open[..end_offset + close_tag.len()];
                results.push(full_element.trim().to_string());
            } else {
                results.push(whole_open.trim().to_string());
            }
        }
    }

    Ok(results)
}

fn has_attribute_value(attrs: &str, name: &str, expected_val: &str) -> bool {
    let pattern = format!(
        r#"(?i)\b{}\s*=\s*["']?([^"'\s>]+)["']?"#,
        regex::escape(name)
    );
    if let Ok(re) = Regex::new(&pattern) {
        if let Some(caps) = re.captures(attrs) {
            if let Some(val) = caps.get(1) {
                return val.as_str().eq_ignore_ascii_case(expected_val);
            }
        }
    }
    false
}

fn has_class_value(attrs: &str, expected_class: &str) -> bool {
    let pattern = r#"(?i)\bclass\s*=\s*["']([^"']+)["']"#;
    if let Ok(re) = Regex::new(pattern) {
        if let Some(caps) = re.captures(attrs) {
            if let Some(val) = caps.get(1) {
                let classes: Vec<&str> = val.as_str().split_whitespace().collect();
                return classes
                    .iter()
                    .any(|c| c.eq_ignore_ascii_case(expected_class));
            }
        }
    }
    false
}

fn has_attribute(attrs: &str, name: &str) -> bool {
    let pattern = format!(r"(?i)\b{}\b", regex::escape(name));
    if let Ok(re) = Regex::new(&pattern) {
        return re.is_match(attrs);
    }
    false
}

pub fn extract_headers_only(http_text: &str) -> String {
    let newline = if http_text.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let delimiter = format!("{newline}{newline}");
    if let Some((head, _)) = http_text.split_once(&delimiter) {
        head.to_string()
    } else {
        http_text.to_string()
    }
}

pub fn is_binary_mime_type(mime: &str) -> bool {
    let mime = mime.to_lowercase();
    mime.starts_with("image/")
        || mime.starts_with("video/")
        || mime.starts_with("audio/")
        || mime.starts_with("font/")
        || mime.contains("octet-stream")
        || mime.contains("pdf")
        || mime.contains("zip")
        || mime.contains("gzip")
        || mime.contains("tar")
        || mime.contains("protobuf")
        || mime.contains("wasm")
}

pub fn is_binary_payload(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }
    let sample = &bytes[..bytes.len().min(1024)];
    let null_count = sample.iter().filter(|&&b| b == 0).count();
    null_count > 0
        || sample
            .iter()
            .filter(|&&b| b < 9 || (b > 13 && b < 32))
            .count()
            > sample.len() / 10
}

pub fn filter_and_truncate_payload(
    bytes: &[u8],
    content_type: Option<&str>,
    headers_only: bool,
    extract_css: Option<&str>,
    extract_json: Option<&str>,
    max_length: Option<usize>,
) -> (String, bool) {
    if bytes.is_empty() {
        return (String::new(), false);
    }

    let raw_text = String::from_utf8_lossy(bytes).into_owned();

    if headers_only {
        return (extract_headers_only(&raw_text), false);
    }

    if let Some(mime) = content_type {
        if is_binary_mime_type(mime) {
            let len = bytes.len();
            return (format!("<binary data: {len} bytes, type: {mime}>"), false);
        }
    } else if is_binary_payload(bytes) {
        let len = bytes.len();
        return (format!("<binary data: {len} bytes>"), false);
    }

    let newline = if raw_text.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let (_head, body) = raw_text
        .split_once(&format!("{newline}{newline}"))
        .unwrap_or(("", &raw_text));

    if let Some(css_sel) = extract_css {
        if let Ok(matches) = extract_css_selector(body, css_sel) {
            let formatted = format!(
                "--- CSS Matches for `{css_sel}` ({}) ---\n{}",
                matches.len(),
                matches.join("\n")
            );
            return (formatted, false);
        }
    }

    if let Some(json_path) = extract_json {
        if let Ok(matches) = extract_json_path(body, json_path) {
            let json_out =
                serde_json::to_string_pretty(&matches).unwrap_or_else(|_| "[]".to_string());
            let formatted = format!(
                "--- JSONPath Matches for `{json_path}` ({}) ---\n{}",
                matches.len(),
                json_out
            );
            return (formatted, false);
        }
    }

    if let Some(max_len) = max_length {
        if raw_text.len() > max_len {
            let truncated = format!(
                "{}\n\n... [truncated {} bytes]",
                &raw_text[..max_len.min(raw_text.len())],
                raw_text.len().saturating_sub(max_len)
            );
            return (truncated, true);
        }
    }

    (raw_text, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pest_json_path() {
        let json = r#"{
            "data": {
                "users": [
                    {"id": 1, "name": "Alice"},
                    {"id": 2, "name": "Bob"}
                ],
                "meta": {"total": 2}
            }
        }"#;

        let res = extract_json_path(json, "$.data.users[*].name").unwrap();
        assert_eq!(res.len(), 2);
        assert_eq!(res[0], "Alice");
        assert_eq!(res[1], "Bob");

        let res_single = extract_json_path(json, "data.meta.total").unwrap();
        assert_eq!(res_single.len(), 1);
        assert_eq!(res_single[0], 2);

        let res_idx = extract_json_path(json, "$.data.users[0].id").unwrap();
        assert_eq!(res_idx.len(), 1);
        assert_eq!(res_idx[0], 1);
    }

    #[test]
    fn test_pest_css_selector() {
        let html = r#"
        <html>
            <body>
                <form id="login" action="/login" method="POST">
                    <input type="hidden" name="csrf" value="secret123" />
                    <input type="text" name="username" />
                </form>
                <div class="alert error">Invalid credentials</div>
            </body>
        </html>
        "#;

        let res = extract_css_selector(html, "form#login").unwrap();
        assert_eq!(res.len(), 1);
        assert!(res[0].contains("csrf"));

        let res_input = extract_css_selector(html, "input[name=csrf]").unwrap();
        assert_eq!(res_input.len(), 1);
        assert!(res_input[0].contains("secret123"));

        let res_class = extract_css_selector(html, ".alert").unwrap();
        assert_eq!(res_class.len(), 1);
        assert!(res_class[0].contains("Invalid credentials"));
    }
}
