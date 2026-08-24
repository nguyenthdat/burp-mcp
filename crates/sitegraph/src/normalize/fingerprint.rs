pub fn stable_id(kind: &str, parts: &[&str]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(kind.as_bytes());
    for part in parts {
        hasher.update(&[0]);
        hasher.update(part.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

pub fn response(body: &[u8]) -> Option<String> {
    (!body.is_empty()).then(|| blake3::hash(body).to_hex().to_string())
}
