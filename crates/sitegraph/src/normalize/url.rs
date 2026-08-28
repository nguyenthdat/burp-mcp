use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedUrl {
    pub origin: String,
    pub path: String,
    pub parameter_names: Vec<String>,
}

pub fn normalize(value: &str) -> Result<NormalizedUrl, String> {
    let url = Url::parse(value).map_err(|error| format!("invalid URL: {error}"))?;
    let scheme = url.scheme().to_ascii_lowercase();
    if !matches!(scheme.as_str(), "http" | "https") {
        return Err("only HTTP and HTTPS URLs are supported".to_owned());
    }
    let host = url
        .host_str()
        .ok_or_else(|| "URL has no host".to_owned())?
        .to_ascii_lowercase();
    let default_port = if scheme == "https" { 443 } else { 80 };
    let port = url.port_or_known_default().unwrap_or(default_port);
    let origin = if port == default_port {
        format!("{scheme}://{host}")
    } else {
        format!("{scheme}://{host}:{port}")
    };
    let mut path = String::with_capacity(url.path().len().max(1));
    let mut slash = false;
    for character in url.path().chars() {
        if character == '/' {
            if !slash {
                path.push('/');
            }
            slash = true;
        } else {
            path.push(character);
            slash = false;
        }
    }
    if path.is_empty() {
        path.push('/');
    }
    let mut parameter_names = url
        .query_pairs()
        .map(|(name, _)| name.into_owned())
        .collect::<Vec<_>>();
    parameter_names.sort_unstable();
    parameter_names.dedup();
    Ok(NormalizedUrl {
        origin,
        path,
        parameter_names,
    })
}

pub fn metadata_url(value: &str, base: &str) -> Option<String> {
    if value.is_empty() || value.len() > 8 * 1024 {
        return None;
    }
    let mut url = Url::parse(base).ok()?.join(value).ok()?;
    if !matches!(url.scheme(), "http" | "https") {
        return None;
    }
    url.set_query(None);
    url.set_fragment(None);
    Some(url.into())
}

pub fn parameterize_path(raw_path: &str) -> (String, bool) {
    let mut parameterized = String::with_capacity(raw_path.len());
    let mut has_template = false;
    let segments: Vec<&str> = raw_path.split('/').collect();

    for (i, seg) in segments.iter().enumerate() {
        if i > 0 {
            parameterized.push('/');
        }
        if is_dynamic_segment(seg) {
            parameterized.push_str("{id}");
            has_template = true;
        } else {
            parameterized.push_str(seg);
        }
    }

    (parameterized, has_template)
}

fn is_dynamic_segment(seg: &str) -> bool {
    if seg.is_empty() {
        return false;
    }
    // Integer check: 12345
    if seg.chars().all(|c| c.is_ascii_digit()) && seg.len() <= 19 {
        return true;
    }
    // UUID check: 8-4-4-4-12
    if seg.len() == 36 && seg.chars().filter(|&c| c == '-').count() == 4 {
        let clean = seg.replace('-', "");
        if clean.len() == 32 && clean.chars().all(|c| c.is_ascii_hexdigit()) {
            return true;
        }
    }
    // Hex Hash check (e.g. 32-64 hex chars)
    let hex_part = seg.strip_prefix("0x").unwrap_or(seg);
    if (hex_part.len() == 32 || hex_part.len() == 40 || hex_part.len() == 64)
        && hex_part.chars().all(|c| c.is_ascii_hexdigit())
    {
        return true;
    }
    false
}
