use super::*;

#[test]
fn embedded_rules_keep_exact_binary_offsets_and_captures() {
    let pack = RulePack::default_exact().unwrap();
    let input = b"\xff token=exact-value eyJ12345678.abcdefgh.ijklmnop AKIA1234567890ABCDEF";
    let findings = pack.matches("response_body", input);

    let secret = findings
        .iter()
        .find(|finding| finding.rule_id == "secret_assignment")
        .unwrap();
    assert_eq!(secret.capture, b"exact-value");
    assert_eq!(&input[secret.byte_start..secret.byte_end], secret.capture);
    assert!(findings.iter().any(|finding| finding.rule_id == "jwt"));
    assert!(
        findings
            .iter()
            .any(|finding| finding.rule_id == "cloud_access_key")
    );
    assert!(findings.len() <= 256);
    assert_eq!(pack.id(), "burp-mcp-sitegraph");
    assert_eq!(pack.version(), "2026.08.25");
}

#[test]
fn rules_only_run_on_declared_surfaces() {
    let pack = RulePack::default_exact().unwrap();
    assert!(pack.matches("unsupported", b"token=exact-value").is_empty());
    assert_eq!(
        pack.matches("websocket_payload", b"token=exact-value")[0].capture,
        b"exact-value"
    );
}

#[test]
fn malformed_or_ambiguous_rule_packs_are_rejected() {
    let duplicate = br#"{
      "id":"duplicate","version":"1","max_matches":1,
      "rules":[
        {"id":"same","pattern":"a","capture_group":0,"severity":"low","surfaces":["response_body"]},
        {"id":"same","pattern":"b","capture_group":0,"severity":"low","surfaces":["response_body"]}
      ]
    }"#;
    assert!(
        RulePack::from_json(duplicate)
            .unwrap_err()
            .contains("duplicate rule id")
    );

    let missing_capture = br#"{
      "id":"bad-capture","version":"1","max_matches":1,
      "rules":[
        {"id":"bad","pattern":"a","capture_group":1,"severity":"low","surfaces":["response_body"]}
      ]
    }"#;
    assert!(
        RulePack::from_json(missing_capture)
            .unwrap_err()
            .contains("capture_group 1 does not exist")
    );
}
