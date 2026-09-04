use super::css_selector::extract_css_selector;
use super::json_path::extract_json_path;

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

    if let Some(mime) = content_type {
        if is_binary_mime_type(mime) {
            let len = bytes.len();
            return (format!("<binary data: {len} bytes [{mime}]>"), false);
        }
    } else if is_binary_payload(bytes) {
        let len = bytes.len();
        return (format!("<binary data: {len} bytes>"), false);
    }

    let raw_text = String::from_utf8_lossy(bytes);
    if headers_only {
        return truncate_text(&extract_headers_only(&raw_text), max_length);
    }

    let newline = if raw_text.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let (_head, body) = raw_text
        .split_once(&format!("{newline}{newline}"))
        .unwrap_or(("", &raw_text));

    if let Some((css_sel, matches)) =
        extract_css.and_then(|sel| extract_css_selector(body, sel).ok().map(|m| (sel, m)))
    {
        let formatted = format!(
            "--- CSS Matches for `{css_sel}` ({}) ---\n{}",
            matches.len(),
            matches.join("\n")
        );
        return truncate_text(&formatted, max_length);
    }

    if let Some((json_path, matches)) =
        extract_json.and_then(|path| extract_json_path(body, path).ok().map(|m| (path, m)))
    {
        let json_out = serde_json::to_string_pretty(&matches).unwrap_or_else(|_| "[]".to_string());
        let formatted = format!(
            "--- JSONPath Matches for `{json_path}` ({}) ---\n{}",
            matches.len(),
            json_out
        );
        return truncate_text(&formatted, max_length);
    }

    truncate_text(&raw_text, max_length)
}

fn truncate_text(text: &str, max_length: Option<usize>) -> (String, bool) {
    let Some(max_len) = max_length.filter(|&len| text.len() > len) else {
        return (text.to_owned(), false);
    };

    let mut cut = max_len.min(text.len());
    while !text.is_char_boundary(cut) {
        cut -= 1;
    }
    (
        format!(
            "{}\n\n... [truncated {} bytes]",
            &text[..cut],
            text.len().saturating_sub(cut)
        ),
        true,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_headers_only() {
        let raw = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello World";
        assert_eq!(
            extract_headers_only(raw),
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain"
        );
    }

    #[test]
    fn test_is_binary_mime_type() {
        assert!(is_binary_mime_type("image/png"));
        assert!(is_binary_mime_type("application/octet-stream"));
        assert!(!is_binary_mime_type("application/json"));
        assert!(!is_binary_mime_type("text/html"));
    }

    #[test]
    fn test_filter_and_truncate_payload() {
        let body = b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<div>Hello</div>";
        let (out, trunc) =
            filter_and_truncate_payload(body, Some("text/html"), false, Some("div"), None, None);
        assert!(!trunc);
        assert!(out.contains("<div>Hello</div>"));
    }

    #[test]
    fn test_filter_and_truncate_utf8_boundary() {
        // "🦀" is 4 bytes: [0xF0, 0x9F, 0x96, 0x80]
        let raw = "Hello 🦀 World".as_bytes();
        // Truncate at 8 bytes (middle of crab emoji)
        let (out, trunc) = filter_and_truncate_payload(raw, None, false, None, None, Some(8));
        assert!(trunc);
        assert!(out.starts_with("Hello "));
        assert!(!out.starts_with("Hello 🦀"));
    }

    #[test]
    fn projections_respect_output_limit() {
        let json = format!("{{\"items\":[\"{}\"]}}", "x".repeat(1024));
        let (json_output, json_truncated) = filter_and_truncate_payload(
            json.as_bytes(),
            None,
            false,
            None,
            Some("$.items[*]"),
            Some(64),
        );
        assert!(json_truncated);
        assert!(json_output.starts_with("--- JSONPath Matches"));

        let html = format!("<div>{}</div>", "y".repeat(1024));
        let (css_output, css_truncated) =
            filter_and_truncate_payload(html.as_bytes(), None, false, Some("div"), None, Some(64));
        assert!(css_truncated);
        assert!(css_output.starts_with("--- CSS Matches"));
    }
}
