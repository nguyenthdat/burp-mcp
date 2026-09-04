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
    assert_eq!(pack.rules.len(), 28);
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
fn duplicate_rule_ids_are_rejected() {
    let duplicate = r#"
    pack {
        id = "duplicate-test"
        version = "1.0.0"
        max_matches = 10
    }
    rule "same" {
        pattern = r"a"
        capture_group = 0
        severity = "low"
        surfaces = ["response_body"]
    }
    rule "same" {
        pattern = r"b"
        capture_group = 0
        severity = "low"
        surfaces = ["response_body"]
    }
    "#;
    let err = RulePack::from_dsl(duplicate).unwrap_err();
    assert!(
        err.contains("duplicate rule id: same"),
        "actual error: {err}"
    );
}

#[test]
fn missing_capture_group_is_rejected() {
    let missing_capture = r#"
    pack {
        id = "bad-capture"
        version = "1.0.0"
        max_matches = 10
    }
    rule "bad" {
        pattern = r"a"
        capture_group = 1
        severity = "low"
        surfaces = ["response_body"]
    }
    "#;
    let err = RulePack::from_dsl(missing_capture).unwrap_err();
    assert!(
        err.contains("capture_group 1 does not exist"),
        "actual error: {err}"
    );
}

#[test]
fn malformed_dsl_is_rejected() {
    let malformed = "this is not valid dsl at all { [ ] }";
    assert!(RulePack::from_dsl(malformed).is_err());

    let missing_pack = r#"
    rule "lonely" {
        pattern = r"test"
        capture_group = 0
        severity = "low"
        surfaces = ["response_body"]
    }
    "#;
    let err = RulePack::from_dsl(missing_pack).unwrap_err();
    assert!(err.contains("missing pack declaration"), "actual: {err}");

    let missing_field = r#"
    pack {
        id = "no-version"
        max_matches = 10
    }
    rule "test" {
        pattern = r"test"
        capture_group = 0
        severity = "low"
        surfaces = ["response_body"]
    }
    "#;
    let err = RulePack::from_dsl(missing_field).unwrap_err();
    assert!(
        err.contains("missing required field 'version'"),
        "actual: {err}"
    );
}

#[test]
fn raw_and_escaped_pattern_strings_work() {
    let dsl = r##"
    pack {
        id = "pattern-test"
        version = "1.0.0"
        max_matches = 50
    }
    rule "raw_quotes" {
        pattern = r#"needle="([a-z]+)""#
        capture_group = 1
        severity = "high"
        surfaces = ["response_body"]
    }
    rule "escaped_quotes" {
        pattern = "pin=\\\"([0-9]+)\\\""
        capture_group = 1
        severity = "medium"
        surfaces = ["response_body"]
    }
    "##;
    let pack = RulePack::from_dsl(dsl).unwrap();
    let input = b"\x80 needle=\"secretvalue\" \xfe pin=\"1234\" \xff";
    let matches = pack.matches("response_body", input);

    let raw_m = matches.iter().find(|m| m.rule_id == "raw_quotes").unwrap();
    assert_eq!(raw_m.capture, b"secretvalue");
    assert_eq!(&input[raw_m.byte_start..raw_m.byte_end], b"secretvalue");

    let esc_m = matches
        .iter()
        .find(|m| m.rule_id == "escaped_quotes")
        .unwrap();
    assert_eq!(esc_m.capture, b"1234");
    assert_eq!(&input[esc_m.byte_start..esc_m.byte_end], b"1234");
}
#[test]
fn line_comments_are_accepted_outside_literals() {
    let dsl = r#"
    # pack comment
    pack {
        id = "comments"
        version = "1"
        max_matches = 2
    }
    // rule comment
    rule "commented" {
        pattern = r"token#[0-9]+//literal"
        capture_group = 0
        severity = "medium"
        surfaces = ["response_body"]
    }
    "#;
    let pack = RulePack::from_dsl(dsl).unwrap();
    let matches = pack.matches("response_body", b"token#42//literal");
    assert_eq!(matches.len(), 1);
}

#[test]
fn json_format_is_rejected() {
    let json_doc = r#"{
      "id": "burp-mcp-sitegraph",
      "version": "2026.08.25",
      "max_matches": 256,
      "rules": []
    }"#;
    let err = RulePack::from_dsl(json_doc).unwrap_err();
    assert!(
        err.contains("failed to parse rule pack DSL"),
        "actual error: {err}"
    );
}

#[test]
fn bounds_and_enums_are_enforced() {
    let bad_severity = r#"
    pack {
        id = "test"
        version = "1.0.0"
        max_matches = 10
    }
    rule "test" {
        pattern = r"test"
        capture_group = 0
        severity = "urgent"
        surfaces = ["response_body"]
    }
    "#;
    assert!(
        RulePack::from_dsl(bad_severity)
            .unwrap_err()
            .contains("invalid severity")
    );

    let bad_surface = r#"
    pack {
        id = "test"
        version = "1.0.0"
        max_matches = 10
    }
    rule "test" {
        pattern = r"test"
        capture_group = 0
        severity = "low"
        surfaces = ["invalid_surface_name"]
    }
    "#;
    assert!(
        RulePack::from_dsl(bad_surface)
            .unwrap_err()
            .contains("invalid surface")
    );

    let zero_max_matches = r#"
    pack {
        id = "test"
        version = "1.0.0"
        max_matches = 0
    }
    rule "test" {
        pattern = r"test"
        capture_group = 0
        severity = "low"
        surfaces = ["response_body"]
    }
    "#;
    assert!(
        RulePack::from_dsl(zero_max_matches)
            .unwrap_err()
            .contains("between 1 and 4096")
    );
}
