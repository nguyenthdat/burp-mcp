pub fn content_type(value: &str) -> String {
    value
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .chars()
        .take(256)
        .collect()
}

pub fn technology_names(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut names = values
        .into_iter()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty() && value.len() <= 256)
        .collect::<Vec<_>>();
    names.sort_unstable();
    names.dedup();
    names
}
