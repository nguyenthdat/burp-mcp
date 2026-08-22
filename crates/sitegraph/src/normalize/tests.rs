use super::url::{metadata_url, normalize};

#[test]
fn normalization_is_deterministic_and_redacts_values() {
    let first = normalize("HTTPS://Example.test/a//b?token=secret&x=1").unwrap();
    let second = normalize("https://example.test/a/b?x=2&token=other").unwrap();
    assert_eq!(first.origin, second.origin);
    assert_eq!(first.path, second.path);
    assert_eq!(first.parameter_names, vec!["token", "x"]);
    assert!(!format!("{first:?}").contains("secret"));
    assert_eq!(
        metadata_url("/next?query=secret", "https://example.test/start"),
        Some("https://example.test/next".to_owned())
    );
}
