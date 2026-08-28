use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct HeaderDiffEntry {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_a: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_b: Option<String>,
    pub change_type: String, // "added", "removed", "modified", "unchanged"
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct DiffResult {
    pub similarity_score: f64,
    pub status_a: Option<u32>,
    pub status_b: Option<u32>,
    pub length_a: usize,
    pub length_b: usize,
    pub length_delta: isize,
    pub headers_diff: Vec<HeaderDiffEntry>,
    pub body_diff: String,
}

pub fn compare_http_messages(text_a: &str, text_b: &str) -> DiffResult {
    let (head_a, body_a, status_a) = parse_http_message(text_a);
    let (head_b, body_b, status_b) = parse_http_message(text_b);

    let headers_diff = diff_headers(&head_a, &head_b);
    let body_diff = generate_line_diff(&body_a, &body_b);
    let similarity_score = calculate_similarity(&body_a, &body_b);

    let length_a = body_a.len();
    let length_b = body_b.len();
    let length_delta = (length_b as isize) - (length_a as isize);

    DiffResult {
        similarity_score,
        status_a,
        status_b,
        length_a,
        length_b,
        length_delta,
        headers_diff,
        body_diff,
    }
}

fn parse_http_message(raw: &str) -> (BTreeMap<String, String>, String, Option<u32>) {
    let newline = if raw.contains("\r\n") { "\r\n" } else { "\n" };
    let delimiter = format!("{newline}{newline}");
    let (head, body) = raw.split_once(&delimiter).unwrap_or((raw, ""));

    let mut headers = BTreeMap::new();
    let mut status = None;
    let mut lines = head.lines();

    if let Some(first_line) = lines.next() {
        if first_line.starts_with("HTTP/") {
            let parts: Vec<&str> = first_line.split_whitespace().collect();
            if parts.len() >= 2 {
                status = parts[1].parse::<u32>().ok();
            }
        }
    }

    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            headers.insert(k.trim().to_lowercase(), v.trim().to_string());
        }
    }

    (headers, body.to_string(), status)
}

fn diff_headers(
    headers_a: &BTreeMap<String, String>,
    headers_b: &BTreeMap<String, String>,
) -> Vec<HeaderDiffEntry> {
    let all_keys: BTreeSet<String> = headers_a.keys().chain(headers_b.keys()).cloned().collect();
    let mut diffs = Vec::new();

    for key in all_keys {
        let val_a = headers_a.get(&key);
        let val_b = headers_b.get(&key);

        match (val_a, val_b) {
            (Some(a), Some(b)) => {
                if a != b {
                    diffs.push(HeaderDiffEntry {
                        name: key,
                        value_a: Some(a.clone()),
                        value_b: Some(b.clone()),
                        change_type: "modified".to_string(),
                    });
                }
            }
            (Some(a), None) => {
                diffs.push(HeaderDiffEntry {
                    name: key,
                    value_a: Some(a.clone()),
                    value_b: None,
                    change_type: "removed".to_string(),
                });
            }
            (None, Some(b)) => {
                diffs.push(HeaderDiffEntry {
                    name: key,
                    value_a: None,
                    value_b: Some(b.clone()),
                    change_type: "added".to_string(),
                });
            }
            (None, None) => {}
        }
    }

    diffs
}

pub fn generate_line_diff(a: &str, b: &str) -> String {
    let lines_a: Vec<&str> = a.lines().collect();
    let lines_b: Vec<&str> = b.lines().collect();

    let mut output = String::new();
    let max_len = lines_a.len().max(lines_b.len());

    let mut diff_count = 0;
    for i in 0..max_len {
        let line_a = lines_a.get(i);
        let line_b = lines_b.get(i);

        match (line_a, line_b) {
            (Some(la), Some(lb)) => {
                if la == lb {
                    if diff_count > 0 && diff_count < 5 {
                        output.push_str(&format!("  {la}\n"));
                    }
                } else {
                    output.push_str(&format!("- {la}\n"));
                    output.push_str(&format!("+ {lb}\n"));
                    diff_count += 1;
                }
            }
            (Some(la), None) => {
                output.push_str(&format!("- {la}\n"));
                diff_count += 1;
            }
            (None, Some(lb)) => {
                output.push_str(&format!("+ {lb}\n"));
                diff_count += 1;
            }
            (None, None) => {}
        }
    }

    if output.is_empty() {
        "--- Responses are identical ---".to_string()
    } else {
        output
    }
}

pub fn calculate_similarity(a: &str, b: &str) -> f64 {
    if a == b {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    let bigrams_a = extract_bigrams(a);
    let bigrams_b = extract_bigrams(b);

    if bigrams_a.is_empty() || bigrams_b.is_empty() {
        return 0.0;
    }

    let mut intersection = 0;
    for item in &bigrams_a {
        if bigrams_b.contains(item) {
            intersection += 1;
        }
    }

    let dice = (2.0 * intersection as f64) / (bigrams_a.len() + bigrams_b.len()) as f64;
    (dice * 1000.0).round() / 1000.0
}

fn extract_bigrams(s: &str) -> BTreeSet<(char, char)> {
    let mut set = BTreeSet::new();
    let chars: Vec<char> = s.chars().collect();
    for i in 0..chars.len().saturating_sub(1) {
        set.insert((chars[i], chars[i + 1]));
    }
    set
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_http_messages() {
        let msg_a =
            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 13\r\n\r\nHello World A";
        let msg_b =
            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 13\r\n\r\nHello World B";

        let diff = compare_http_messages(msg_a, msg_b);
        assert_eq!(diff.status_a, Some(200));
        assert_eq!(diff.status_b, Some(200));
        assert!(diff.similarity_score > 0.8);
        assert!(diff.body_diff.contains("- Hello World A"));
        assert!(diff.body_diff.contains("+ Hello World B"));
    }

    #[test]
    fn test_similarity_calculation() {
        assert_eq!(calculate_similarity("identical", "identical"), 1.0);
        assert!(calculate_similarity("abcde", "abcdf") >= 0.75);
    }
}
