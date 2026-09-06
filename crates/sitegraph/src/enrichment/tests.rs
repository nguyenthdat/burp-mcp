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
    assert_eq!(pack.version(), "2026.09.05");
    assert_eq!(pack.rules.len(), 105);
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

fn rule_toml(pack_id: &str, version: &str, max_matches: usize, rules: &str) -> String {
    format!(
        "[pack]\nid = \"{pack_id}\"\nversion = \"{version}\"\nmax_matches = {max_matches}\n\n{rules}"
    )
}

#[test]
fn duplicate_rule_ids_are_rejected() {
    let duplicate = rule_toml(
        "duplicate-test",
        "1.0.0",
        10,
        r#"[[rules]]
id = "same"
pattern = 'a'
capture_group = 0
severity = "low"
surfaces = ["response_body"]

[[rules]]
id = "same"
pattern = 'b'
capture_group = 0
severity = "low"
surfaces = ["response_body"]
"#,
    );
    let err = RulePack::from_toml(&duplicate).unwrap_err();
    assert!(
        err.contains("duplicate rule id: same"),
        "actual error: {err}"
    );
}

#[test]
fn missing_capture_group_is_rejected() {
    let invalid_capture = rule_toml(
        "bad-capture",
        "1.0.0",
        10,
        r#"[[rules]]
id = "bad"
pattern = 'a'
capture_group = 1
severity = "low"
surfaces = ["response_body"]
"#,
    );
    let err = RulePack::from_toml(&invalid_capture).unwrap_err();
    assert!(
        err.contains("capture_group 1 does not exist"),
        "actual error: {err}"
    );
}

#[test]
fn malformed_and_incomplete_toml_are_rejected() {
    assert!(RulePack::from_toml("this is not valid TOML { [ ] }").is_err());

    let missing_pack = r#"[[rules]]
id = "lonely"
pattern = 'test'
capture_group = 0
severity = "low"
surfaces = ["response_body"]
"#;
    let err = RulePack::from_toml(missing_pack).unwrap_err();
    assert!(err.contains("missing field `pack`"), "actual error: {err}");

    let missing_version = r#"[pack]
id = "no-version"
max_matches = 10

[[rules]]
id = "test"
pattern = 'test'
capture_group = 0
severity = "low"
surfaces = ["response_body"]
"#;
    let err = RulePack::from_toml(missing_version).unwrap_err();
    assert!(
        err.contains("missing field `version`"),
        "actual error: {err}"
    );
}

#[test]
fn toml_literal_and_basic_pattern_strings_work() {
    let toml = rule_toml(
        "pattern-test",
        "1.0.0",
        50,
        r#"[[rules]]
id = "literal_quotes"
pattern = '''needle="([a-z]+)"'''
capture_group = 1
severity = "high"
surfaces = ["response_body"]

[[rules]]
id = "basic_quotes"
pattern = "pin=\\\"([0-9]+)\\\""
capture_group = 1
severity = "medium"
surfaces = ["response_body"]
"#,
    );
    let pack = RulePack::from_toml(&toml).unwrap();
    let input = b"\x80 needle=\"secretvalue\" \xfe pin=\"1234\" \xff";
    let matches = pack.matches("response_body", input);

    let literal = matches
        .iter()
        .find(|finding| finding.rule_id == "literal_quotes")
        .unwrap();
    assert_eq!(literal.capture, b"secretvalue");
    assert_eq!(&input[literal.byte_start..literal.byte_end], b"secretvalue");

    let basic = matches
        .iter()
        .find(|finding| finding.rule_id == "basic_quotes")
        .unwrap();
    assert_eq!(basic.capture, b"1234");
    assert_eq!(&input[basic.byte_start..basic.byte_end], b"1234");
}

#[test]
fn toml_comments_are_accepted_outside_literals() {
    let document = r#"# pack comment
[pack]
id = "comments"
version = "1"
max_matches = 2

# rule comment
[[rules]]
id = "commented"
pattern = 'token#[0-9]+//literal'
capture_group = 0
severity = "medium"
surfaces = ["response_body"]
"#;
    let pack = RulePack::from_toml(document).unwrap();
    let matches = pack.matches("response_body", b"token#42//literal");
    assert_eq!(matches.len(), 1);
}

#[test]
fn legacy_json_and_dsl_formats_are_rejected() {
    let json = r#"{"pack":{"id":"legacy","version":"1","max_matches":1},"rules":[]}"#;
    assert!(RulePack::from_toml(json).is_err());

    let dsl = r#"pack { id = "legacy" version = "1" max_matches = 1 }"#;
    assert!(RulePack::from_toml(dsl).is_err());
}

#[test]
fn bounds_and_enums_are_enforced() {
    let bad_severity = rule_toml(
        "test",
        "1.0.0",
        10,
        r#"[[rules]]
id = "test"
pattern = 'test'
capture_group = 0
severity = "urgent"
surfaces = ["response_body"]
"#,
    );
    assert!(
        RulePack::from_toml(&bad_severity)
            .unwrap_err()
            .contains("invalid severity")
    );

    let bad_surface = rule_toml(
        "test",
        "1.0.0",
        10,
        r#"[[rules]]
id = "test"
pattern = 'test'
capture_group = 0
severity = "low"
surfaces = ["invalid_surface_name"]
"#,
    );
    assert!(
        RulePack::from_toml(&bad_surface)
            .unwrap_err()
            .contains("invalid surface")
    );

    let zero_max_matches = rule_toml(
        "test",
        "1.0.0",
        0,
        r#"[[rules]]
id = "test"
pattern = 'test'
capture_group = 0
severity = "low"
surfaces = ["response_body"]
"#,
    );
    assert!(
        RulePack::from_toml(&zero_max_matches)
            .unwrap_err()
            .contains("between 1 and 4096")
    );
}

#[test]
fn unknown_toml_fields_are_rejected() {
    let unknown_pack = r#"[pack]
id = "test"
version = "1"
max_matches = 1
unexpected = true

[[rules]]
id = "test"
pattern = 'test'
capture_group = 0
severity = "low"
surfaces = ["response_body"]
"#;
    assert!(RulePack::from_toml(unknown_pack).is_err());

    let unknown_rule = rule_toml(
        "test",
        "1",
        1,
        r#"[[rules]]
id = "test"
pattern = 'test'
capture_group = 0
severity = "low"
surfaces = ["response_body"]
when = "present"
"#,
    );
    assert!(RulePack::from_toml(&unknown_rule).is_err());
}

struct RuleFixture {
    id: &'static str,
    pattern: &'static str,
    capture_group: usize,
    severity: &'static str,
    surfaces: &'static [&'static str],
    positive: &'static str,
    negative: &'static str,
}

const NEW_RULE_FIXTURES: &[RuleFixture] = &[
    RuleFixture {
        id: "gcp_api_key",
        pattern: "\\b(AIza[0-9A-Za-z_-]{35})(?:[^0-9A-Za-z_-]|$)",
        capture_group: 1,
        severity: "high",
        surfaces: &[
            "request_message",
            "response_message",
            "response_body",
            "websocket_payload",
        ],
        positive: "AIzaAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        negative: "AIzaAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    },
    RuleFixture {
        id: "gcp_oauth_access_token",
        pattern: "\\b(ya29\\.[0-9A-Za-z_-]{20,1024})(?:[^0-9A-Za-z_-]|$)",
        capture_group: 1,
        severity: "critical",
        surfaces: &["request_message", "response_message", "response_body"],
        positive: "ya29.a0AfH6SMBx_synthetic_token_value_for_testing_oauth2_access_token_12345",
        negative: "ya29.fake_short",
    },
    RuleFixture {
        id: "gcp_service_account_key",
        pattern: r#""type"\s*:\s*"service_account"[\s\S]{1,512}?"private_key_id"\s*:\s*"[0-9a-fA-F]{40}"[\s\S]{1,512}?"private_key"\s*:\s*"-----BEGIN (?:RSA )?PRIVATE KEY-----"#,
        capture_group: 0,
        severity: "critical",
        surfaces: &["response_body"],
        positive: "{\n  \"type\": \"service_account\",\n  \"project_id\": \"my-project-123\",\n  \"private_key_id\": \"2b387b72ec1b082aa7e52189d9c43f58fb19fb48\",\n  \"private_key\": \"-----BEGIN PRIVATE KEY-----\nMIIEvQIBADANBgkqhkiG9w0BAQEFAASC\n-----END PRIVATE KEY-----\n\"\n}",
        negative: "{\n  \"type\": \"service_account\",\n  \"description\": \"regular account object without key\"\n}",
    },
    RuleFixture {
        id: "azure_storage_connection_string",
        pattern: "(?i)DefaultEndpointsProtocol=https?;\\s*AccountName=[a-z0-9]{3,24};\\s*AccountKey=([A-Za-z0-9+/]{86,88}={0,2})",
        capture_group: 1,
        severity: "critical",
        surfaces: &["response_message", "response_body"],
        positive: "DefaultEndpointsProtocol=https;AccountName=storagetest123;AccountKey=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==;EndpointSuffix=core.windows.net",
        negative: "DefaultEndpointsProtocol=https;AccountName=storagetest123;AccountKey=shortkey",
    },
    RuleFixture {
        id: "azure_sas_token",
        pattern: r#"(?:[?&]|\b)(?:sp|sp%3[Dd])=[racwdli]+[\s\S]{1,512}?(?:[?&]|\b)(?:sv|sv%3[Dd])=\d{4}-\d{2}-\d{2}[\s\S]{1,512}?(?:[?&]|\b)(?:sig|sig%3[Dd])=[A-Za-z0-9%+/=]{10,}"#,
        capture_group: 0,
        severity: "high",
        surfaces: &["request_message", "response_body", "websocket_payload"],
        positive: "?sp=r&st=2026-03-04T07:24:52Z&se=2026-04-04T15:24:52Z&spr=https&sv=2022-11-02&sr=c&sig=WSdF9YeZhvrbs%2B%2B1f8ZdDBzEe7fBJ%2BenuaXQ%2BJ9WOw0%3D",
        negative: "?action=view&sig=abcdef123456",
    },
    RuleFixture {
        id: "azure_tenant_subscription_id",
        pattern: "/(?:subscriptions|tenants)/([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})",
        capture_group: 1,
        severity: "low",
        surfaces: &["request_message", "response_body"],
        positive: "/subscriptions/91d12343-a3de-345d-b2ea-135792468abc/resourceGroups/rg1",
        negative: "/users/91d12343-a3de-345d-b2ea-135792468abc",
    },
    RuleFixture {
        id: "github_pat_classic",
        pattern: "\\b(ghp_[0-9A-Za-z]{36})\\b",
        capture_group: 1,
        severity: "critical",
        surfaces: &["request_message", "response_message", "response_body"],
        positive: "ghp_111122223333444455556666777788889999",
        negative: "ghp_11112222333344445555666677778888999",
    },
    RuleFixture {
        id: "github_pat_fine_grained",
        pattern: "\\b(github_pat_[0-9A-Za-z_]{82})\\b",
        capture_group: 1,
        severity: "critical",
        surfaces: &["request_message", "response_message", "response_body"],
        positive: "github_pat_AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        negative: "github_pat_AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    },
    RuleFixture {
        id: "github_oauth_token",
        pattern: "\\b(gho_[0-9A-Za-z]{36})\\b",
        capture_group: 1,
        severity: "high",
        surfaces: &["request_message", "response_message", "response_body"],
        positive: "gho_111122223333444455556666777788889999",
        negative: "gho_11112222333344445555666677778888999",
    },
    RuleFixture {
        id: "gitlab_pat",
        pattern: "\\b(glpat-[0-9A-Za-z_-]{20,300})\\b",
        capture_group: 1,
        severity: "critical",
        surfaces: &["request_message", "response_message", "response_body"],
        positive: "glpat-AAAAAAAAAAAAAAAAAAAA",
        negative: "glpat-short123",
    },
    RuleFixture {
        id: "slack_api_token",
        pattern: "\\b(xox[baprs]-[0-9]{10,13}-[0-9]{10,13}[a-zA-Z0-9-]*)\\b",
        capture_group: 1,
        severity: "high",
        surfaces: &[
            "request_message",
            "response_message",
            "response_body",
            "websocket_payload",
        ],
        positive: "xoxb-123456789012-123456789012-abcdef123456",
        negative: "xoxb-12345-12345-short",
    },
    RuleFixture {
        id: "slack_incoming_webhook",
        pattern: "https://hooks\\.slack\\.com/(?:services|workflows)/T[A-Z0-9]+/B[A-Z0-9]+/[A-Za-z0-9]{20,32}",
        capture_group: 0,
        severity: "high",
        surfaces: &["response_message", "response_body"],
        positive: concat!(
            "https://hooks.slack.com/services/T12345678/B12345678/",
            "abcdefghijklmnopqrstuvwx"
        ),
        negative: "https://hooks.slack.com/services/fake",
    },
    RuleFixture {
        id: "stripe_live_secret_key",
        pattern: "\\b((?:sk|rk)_live_[0-9a-zA-Z]{24,99})\\b",
        capture_group: 1,
        severity: "critical",
        surfaces: &["request_message", "response_message", "response_body"],
        positive: concat!(
            "sk_",
            "live_51AbCdeFghIjKlMnOpQrStUvWxYz012345678901234567890"
        ),
        negative: "sk_test_51AbCdeFghIjKlMnOpQrStUvWxYz012345678901234567890",
    },
    RuleFixture {
        id: "stripe_publishable_key",
        pattern: "\\b(pk_live_[0-9a-zA-Z]{24,99})\\b",
        capture_group: 1,
        severity: "low",
        surfaces: &["response_message", "response_body"],
        positive: "pk_live_51AbCdeFghIjKlMnOpQrStUvWxYz012345678901234567890",
        negative: "pk_test_51AbCdeFghIjKlMnOpQrStUvWxYz012345678901234567890",
    },
    RuleFixture {
        id: "openai_api_key",
        pattern: "\\b(sk-(?:(?:proj|svcacct|service)-[A-Za-z0-9_-]+|[A-Za-z0-9]+)T3BlbkFJ[A-Za-z0-9_-]+)\\b",
        capture_group: 1,
        severity: "critical",
        surfaces: &["request_message", "response_message", "response_body"],
        positive: concat!(
            "sk-proj-abc12345678901234567890",
            "T3BlbkFJabc12345678901234567890"
        ),
        negative: "sk-proj-abc12345678901234567890notvalidsecret",
    },
    RuleFixture {
        id: "anthropic_api_key",
        pattern: "\\b(sk-ant-(?:api03|admin01)-[A-Za-z0-9_-]{93}AA)\\b",
        capture_group: 1,
        severity: "critical",
        surfaces: &["request_message", "response_message", "response_body"],
        positive: "sk-ant-api03-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        negative: "sk-ant-api03-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    },
    RuleFixture {
        id: "huggingface_token",
        pattern: "\\b((?:hf_|api_org_)[a-zA-Z0-9]{34})\\b",
        capture_group: 1,
        severity: "high",
        surfaces: &["request_message", "response_message", "response_body"],
        positive: "hf_AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        negative: "hf_AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    },
    RuleFixture {
        id: "sendgrid_api_key",
        pattern: "\\b(SG\\.[a-zA-Z0-9_-]{22}\\.[a-zA-Z0-9_-]{43})\\b",
        capture_group: 1,
        severity: "high",
        surfaces: &["request_message", "response_message", "response_body"],
        positive: "SG.1234567890123456789012.1234567890123456789012345678901234567890123",
        negative: "SG.12345.short",
    },
    RuleFixture {
        id: "twilio_api_key",
        pattern: "\\b(SK[0-9a-fA-F]{32})\\b",
        capture_group: 1,
        severity: "high",
        surfaces: &["request_message", "response_message", "response_body"],
        positive: concat!("SK0123456789abcdef", "0123456789abcdef"),
        negative: "SK0123456789abcdef",
    },
    RuleFixture {
        id: "datadog_api_key",
        pattern: "(?i)(?:datadog|dd)[_-]?(?:api)?[_-]?key\\s*[:=]\\s*[\"']?([a-f0-9]{32})\\b",
        capture_group: 1,
        severity: "high",
        surfaces: &["request_message", "response_message", "response_body"],
        positive: "datadog_api_key = \"e4d909c290d0fb1ca068ffaddf22cbd0\"",
        negative: "md5_hash = \"e4d909c290d0fb1ca068ffaddf22cbd0\"",
    },
    RuleFixture {
        id: "aws_session_token",
        pattern: "(?i)(?:x-amz-security-token|aws_session_token)\\s*[:=]\\s*[\"']?([A-Za-z0-9+/]{80,}={0,3})",
        capture_group: 1,
        severity: "critical",
        surfaces: &["request_message", "response_message", "response_body"],
        positive: "X-Amz-Security-Token: FQoDYXdzEPf//////////wEaDEWqF2QgxkF5nqgHwiLnAQlfeWj1W6i3OGrosXS1hCM41aHMRzx15sxz9vBsHfJN2kwucHaezQK9+2n44BZrvPVXBEPgz8zJRVYZG7qQEO/xm+tfmR9I25TKhoPuowkd7aYDJSFUC0nSOoCyDam9pjunFsbD/+N9VncZ/+4UW0kzx3zOg4/kwuxz/7I5NrrrD77OZddGd4MtLaZf8wi50XcVwWX/61GSCK4qCnhRIUapQpt9WP+uOHKYfQgN74+T92Xe2boduSqLLNZuCjgzVHMSOZ9CBe5ZnQngIIAxQr4KuPEedZgIKOP7I0YRhE5auSmP+4B5eyibmp7XBQ==",
        negative: "X-Amz-Security-Token: short_token",
    },
    RuleFixture {
        id: "hashicorp_vault_token",
        pattern: "\\b(hvs\\.[A-Za-z0-9_-]{90,120}|s\\.[A-Za-z0-9_-]{24})\\b",
        capture_group: 1,
        severity: "critical",
        surfaces: &["request_message", "response_message", "response_body"],
        positive: "hvs.CAESIJ-synthetic-vault-service-token-value-with-sufficient-length-for-validation-purposes-1234567890abcdef",
        negative: "hvs.tooshort",
    },
    RuleFixture {
        id: "kubernetes_service_account_token",
        pattern: "\\b(eyJ[A-Za-z0-9_-]{10,}\\.eyJ[A-Za-z0-9_-]*(?:a3ViZXJuZXRlcy5pby9zZXJ2aWNlYWNjb3Vud|a3ViZXJuZXRlcy9zZXJ2aWNlYWNjb3Vud)[A-Za-z0-9_-]*\\.[A-Za-z0-9_-]+)\\b",
        capture_group: 1,
        severity: "critical",
        surfaces: &["request_message", "response_message", "response_body"],
        positive: "eyJhbGciOiJSUzI1NiIsImtpZCI6IiJ9.eyJpc3MiOiJrdWJlcm5ldGVzL3NlcnZpY2VhY2NvdW50Iiwia3ViZXJuZXRlcy5pby9zZXJ2aWNlYWNjb3VudC9uYW1lc3BhY2UiOiJkZWZhdWx0In0.synthetic_signature_value",
        negative: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c",
    },
    RuleFixture {
        id: "oauth2_auth_code",
        pattern: "(?:[?&]|\\b)code=([A-Za-z0-9_-]{20,128})",
        capture_group: 1,
        severity: "medium",
        surfaces: &["request_message", "response_message"],
        positive: "GET /callback?code=SplxlOBeZQQYbYS6WxSbIA9876543210abcdef HTTP/1.1\r\nHost: oauth.example.com\r\n\r\n",
        negative: "GET /status?code=404 HTTP/1.1\r\nHost: api.example.com\r\n\r\n",
    },
    RuleFixture {
        id: "oauth2_refresh_token",
        pattern: "(?i)(?:\"refresh_token\"\\s*:\\s*\"|(?:\\b|[?&])refresh_token=)([A-Za-z0-9_.~+/-]{16,512})",
        capture_group: 1,
        severity: "high",
        surfaces: &["request_message", "response_body"],
        positive: "{\"token_type\":\"Bearer\",\"expires_in\":3600,\"refresh_token\":\"r1_9876543210_abcdefghijklmnopqrstuvwxyz\"}",
        negative: "{\"token_type\":\"Bearer\",\"scope\":\"offline_access\",\"refresh_token\":\"\"}",
    },
    RuleFixture {
        id: "supabase_anon_service_key",
        pattern: "(?:sb_secret_[a-zA-Z0-9_-]{31}|eyJ[A-Za-z0-9_-]{10,}\\.eyJpc3MiOiJzdXBhYmFzZ[A-Za-z0-9_-]{10,}\\.[A-Za-z0-9_-]{16,})",
        capture_group: 0,
        severity: "high",
        surfaces: &["request_message", "response_body", "websocket_payload"],
        positive: "apikey: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImFiY2RlZiIsInJvbGUiOiJhbm9uIn0.k4PqVvR5T8W9Y1A2B3C4D5E6F7G8H9I0",
        negative: "apikey: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJhdXRoMCIsInJlZiI6ImFiY2RlZiJ9.k4PqVvR5T8W9Y1A2B3C4D5E6F7G8H9I0",
    },
    RuleFixture {
        id: "firebase_api_key",
        pattern: "\\b(AIzaSy[A-Za-z0-9_-]{33})\\b",
        capture_group: 1,
        severity: "medium",
        surfaces: &["request_message", "response_body", "websocket_payload"],
        positive: "AIzaSyAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        negative: "AIzaSyAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    },
    RuleFixture {
        id: "aws_cognito_identity",
        pattern: "\\b([a-z]{2,4}(?:-[a-z]+)+-[0-9]:[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})\\b",
        capture_group: 1,
        severity: "medium",
        surfaces: &["request_message", "response_body"],
        positive: "IdentityPoolId: \"us-east-1:12345678-abcd-1234-abcd-1234567890ab\"",
        negative: "IdentityPoolId: \"12345678-abcd-1234-abcd-1234567890ab\"",
    },
    RuleFixture {
        id: "saml_assertion_marker",
        pattern: "(?i)(?:SAMLResponse=|<(?:saml2?|samlp):(?:Assertion|Response)\\b)",
        capture_group: 0,
        severity: "medium",
        surfaces: &["request_message", "response_body"],
        positive: "<saml2:Assertion xmlns:saml2=\"urn:oasis:names:tc:SAML:2.0:assertion\" ID=\"_12345\" IssueInstant=\"2026-09-05T00:00:00Z\">",
        negative: "<simple_assertion name=\"user\" value=\"john\"></simple_assertion>",
    },
    RuleFixture {
        id: "session_cookie_spring",
        pattern: "(?i)\\bJSESSIONID=([0-9A-Fa-f]{32}(?:\\.[A-Za-z0-9_-]+)?)\\b",
        capture_group: 1,
        severity: "low",
        surfaces: &["request_message", "response_message"],
        positive: "Set-Cookie: JSESSIONID=0123456789ABCDEF0123456789ABCDEF; Path=/; HttpOnly",
        negative: "Set-Cookie: JSESSIONID=short; Path=/",
    },
    RuleFixture {
        id: "session_cookie_aspnet",
        pattern: "(?i)\\b(?:ASP\\.NET_SessionId|\\.AspNetCore\\.Session)=([A-Za-z0-9_\\-+/=%]{20,128})\\b",
        capture_group: 1,
        severity: "low",
        surfaces: &["request_message", "response_message"],
        positive: "Set-Cookie: ASP.NET_SessionId=AAAAAAAAAAAAAAAAAAAAAAAA; path=/; HttpOnly",
        negative: "Set-Cookie: ASP.NET_SessionId=123; path=/",
    },
    RuleFixture {
        id: "session_cookie_php",
        pattern: "(?i)\\bPHPSESSID=([a-zA-Z0-9]{26,40})\\b",
        capture_group: 1,
        severity: "low",
        surfaces: &["request_message", "response_message"],
        positive: "Cookie: PHPSESSID=aaaaaaaaaaaaaaaaaaaaaaaaaa; path=/",
        negative: "Cookie: PHPSESSID=short; path=/",
    },
    RuleFixture {
        id: "session_cookie_django",
        pattern: r#"(?i)\b(?:sessionid=[a-z0-9]{32}\b[\s\S]{1,512}?\bcsrftoken=[a-zA-Z0-9]{32,64}\b|csrftoken=[a-zA-Z0-9]{32,64}\b[\s\S]{1,512}?\bsessionid=[a-z0-9]{32}\b)"#,
        capture_group: 0,
        severity: "low",
        surfaces: &["request_message", "response_message"],
        positive: "Cookie: sessionid=1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d; csrftoken=uW8vO0E7qB9A1x2Y3z4W5v6U7t8S9r0Q1P2o3N4M5l6K7j8I9h0G1f2E3d4C5b6A",
        negative: "Cookie: sessionid=1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d; tracking_id=123",
    },
    RuleFixture {
        id: "session_cookie_laravel",
        pattern: "(?i)\\blaravel_session=([A-Za-z0-9%_-]{40,})\\b",
        capture_group: 1,
        severity: "low",
        surfaces: &["request_message", "response_message"],
        positive: "Set-Cookie: laravel_session=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA; Path=/; HttpOnly",
        negative: "Set-Cookie: laravel_session=abc; Path=/",
    },
    RuleFixture {
        id: "session_cookie_connect",
        pattern: "(?i)\\bconnect\\.sid=([A-Za-z0-9_\\-.%]{24,128})\\b",
        capture_group: 1,
        severity: "low",
        surfaces: &["request_message", "response_message"],
        positive: "Set-Cookie: connect.sid=AAAAAAAAAAAAAAAAAAAAAAAA.signature; Path=/; HttpOnly",
        negative: "Set-Cookie: connect.sid=short; Path=/",
    },
    RuleFixture {
        id: "server_version_banner",
        pattern: "(?im)^Server:\\s*([^\\r\\n]*\\b[A-Za-z0-9_.-]+\\/[0-9]+(?:\\.[0-9A-Za-z_.-]+)+[^\\r\\n]*)",
        capture_group: 1,
        severity: "low",
        surfaces: &["response_message"],
        positive: "HTTP/1.1 200 OK\r\nServer: Apache/2.4.50 (Unix) OpenSSL/1.1.1l\r\nContent-Type: text/html\r\n\r\nHello",
        negative: "HTTP/1.1 200 OK\r\nServer: Cloudflare\r\nContent-Type: text/html\r\n\r\nHello",
    },
    RuleFixture {
        id: "x_powered_by",
        pattern: "(?im)^X-Powered-By:\\s*([^\\r\\n]{1,256})",
        capture_group: 1,
        severity: "low",
        surfaces: &["response_message"],
        positive: "HTTP/1.1 200 OK\r\nX-Powered-By: Express\r\nContent-Type: application/json\r\n\r\n{}",
        negative: "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
    },
    RuleFixture {
        id: "spring_boot_actuator",
        pattern: "(?:[\"']_links[\"']\\s*:\\s*\\{[\\s\\S]{1,512}?[\"']href[\"']\\s*:\\s*[\"'][^\"']*/actuator|/actuator/(?:beans|env|health|heapdump|jolokia|mappings|metrics|threaddump|loggers|configprops)\\b)",
        capture_group: 0,
        severity: "high",
        surfaces: &["response_message", "response_body"],
        positive: "{\"_links\":{\"self\":{\"href\":\"http://localhost:8080/actuator\",\"templated\":false},\"health\":{\"href\":\"http://localhost:8080/actuator/health\"}}}",
        negative: "{\"message\":\"Order actuator component updated successfully\"}",
    },
    RuleFixture {
        id: "nextjs_data_route",
        pattern: "/_next/data/[A-Za-z0-9_-]+/[^\\s\"'?#]+\\.json",
        capture_group: 0,
        severity: "low",
        surfaces: &["request_message", "response_message"],
        positive: "GET /_next/data/build-id-12345/dashboard/settings.json HTTP/1.1\r\nHost: example.com\r\n\r\n",
        negative: "GET /api/v1/data/settings.json HTTP/1.1\r\nHost: example.com\r\n\r\n",
    },
    RuleFixture {
        id: "django_debug_banner",
        pattern: "(?i)(?:You(?:'|&#x27;|\\s)re seeing this error because you have <code>DEBUG = True</code>|DisallowedHost at /|Django Version:\\s*[0-9.]+|Traceback \\(most recent call last\\):[\\s\\S]{1,512}django/core/handlers)",
        capture_group: 0,
        severity: "high",
        surfaces: &["response_body"],
        positive: "<h1>DisallowedHost at /</h1>\n<p>Invalid HTTP_HOST header: &#x27;evil.com&#x27;.</p>\n<p>You&#x27;re seeing this error because you have <code>DEBUG = True</code> in your Django settings file.</p>",
        negative: "<h1>500 Internal Server Error</h1>\n<p>An unexpected error occurred. Please contact support.</p>",
    },
    RuleFixture {
        id: "flask_werkzeug_console",
        pattern: "(?i)(?:Werkzeug\\s+(?:powered\\s+)?traceback\\s+interpreter|id=[\"']interactive[\"'][^>]*console|CONSOLE_MODE\\s*=\\s*true|__debugger__\\s*=\\s*true)",
        capture_group: 0,
        severity: "critical",
        surfaces: &["response_body"],
        positive: "<!DOCTYPE HTML PUBLIC \"-//W3C//DTD HTML 4.01 Transitional//EN\">\n<h3>Werkzeug powered traceback interpreter</h3>\n<div id=\"interactive\" class=\"console\"></div>",
        negative: "<h3>Generic traceback interpreter</h3>\n<div id=\"console\"></div>",
    },
    RuleFixture {
        id: "laravel_ignition_page",
        pattern: "(?i)(?:window\\.ignition\\s*=|id=[\"']ignition-app[\"']|class=[\"'][^\"']*flare-solution|data-component=[\"']ignition[\"']|<title>[^<]*Ignition</title>)",
        capture_group: 0,
        severity: "high",
        surfaces: &["response_body"],
        positive: "<!DOCTYPE html><html><head><title>Ignition - Laravel</title></head><body><div id=\"ignition-app\"></div></body></html>",
        negative: "<!DOCTYPE html><html><head><title>Application Error</title></head><body><div id=\"error-app\"></div></body></html>",
    },
    RuleFixture {
        id: "express_body_parser_leak",
        pattern: "(?i)(?:SyntaxError:\\s*Unexpected token[^\\r\\n]*\\bnode_modules[\\\\/]body-parser\\b|at\\s+[^\\r\\n]*\\bnode_modules[\\\\/]body-parser[\\\\/]lib[\\\\/])",
        capture_group: 0,
        severity: "medium",
        surfaces: &["response_body"],
        positive: "SyntaxError: Unexpected token x in JSON at position 0\n    at parse (/app/node_modules/body-parser/lib/types/json.js:89:19)",
        negative: "SyntaxError: Unexpected token x in JSON at position 0\n    at parse (/app/custom-parser.js:12:5)",
    },
    RuleFixture {
        id: "drupal_version_marker",
        pattern: "(?im)(?:^X-Generator:\\s*Drupal\\b|<meta\\s+name=[\"']Generator[\"']\\s+content=[\"']Drupal\\s+([0-9.]+)|Drupal\\.settings\\b)",
        capture_group: 0,
        severity: "low",
        surfaces: &["response_body", "response_message"],
        positive: "<meta name=\"Generator\" content=\"Drupal 9 (https://www.drupal.org)\" />",
        negative: "<meta name=\"Generator\" content=\"CustomCMS 1.0\" />",
    },
    RuleFixture {
        id: "wordpress_version_marker",
        pattern: "(?i)(?:<meta\\s+name=[\"']generator[\"']\\s+content=[\"']WordPress\\s+([0-9.]+)|(?:/|[\"'])wp-includes/|(?:/|[\"'])wp-links-opml\\.php)",
        capture_group: 0,
        severity: "low",
        surfaces: &["response_body"],
        positive: "<meta name=\"generator\" content=\"WordPress 6.4.3\" />",
        negative: "<meta name=\"generator\" content=\"Blogger v3\" />",
    },
    RuleFixture {
        id: "sql_syntax_error_mysql",
        pattern: "(?i)(?:You have an error in your SQL syntax; check the manual that corresponds to your (?:MySQL|MariaDB) server version for the right syntax to use near |com\\.mysql\\.jdbc\\.exceptions\\.jdbc4\\.MySQLSyntaxErrorException:)",
        capture_group: 0,
        severity: "high",
        surfaces: &["response_body", "websocket_payload"],
        positive: "You have an error in your SQL syntax; check the manual that corresponds to your MySQL server version for the right syntax to use near 'WHERE id = 1' at line 1",
        negative: "Database query completed successfully without SQL syntax errors.",
    },
    RuleFixture {
        id: "sql_syntax_error_postgres",
        pattern: "(?i)(?:ERROR:\\s+syntax error at or near\\s+[\\\"']|PG::SyntaxError:|org\\.postgresql\\.util\\.PSQLException:\\s+ERROR:\\s+syntax error at)",
        capture_group: 0,
        severity: "high",
        surfaces: &["response_body", "websocket_payload"],
        positive: "ERROR: syntax error at or near \"admin\" at character 42",
        negative: "ERROR: connection refused to postgresql server on port 5432",
    },
    RuleFixture {
        id: "sql_syntax_error_mssql",
        pattern: "(?i)(?:Unclosed quotation mark (?:before|after) the character string|Line \\d+:\\s+Incorrect syntax near|com\\.microsoft\\.sqlserver\\.jdbc\\.SQLServerException:\\s+Incorrect syntax near)",
        capture_group: 0,
        severity: "high",
        surfaces: &["response_body", "websocket_payload"],
        positive: "Unclosed quotation mark after the character string 'test'.",
        negative: "The input string format is invalid for character conversion.",
    },
    RuleFixture {
        id: "sql_syntax_error_oracle",
        pattern: "\\bORA-(?:00933:\\s+SQL command not properly ended|00936:\\s+missing expression|01756:\\s+quoted string not properly terminated)\\b",
        capture_group: 0,
        severity: "high",
        surfaces: &["response_body", "websocket_payload"],
        positive: "ORA-00933: SQL command not properly ended",
        negative: "ORA-01017: invalid username/password; logon denied",
    },
    RuleFixture {
        id: "java_stack_trace",
        pattern: "(?m)^\\s+at\\s+[\\w$]+(?:\\.[\\w$]+)+\\([\\w$]+\\.java:\\d+\\)",
        capture_group: 0,
        severity: "medium",
        surfaces: &["response_body", "websocket_payload"],
        positive: "\tat org.apache.catalina.core.StandardWrapperValve.invoke(StandardWrapperValve.java:198)",
        negative: "Look at the class file org.apache.catalina.core.StandardWrapperValve",
    },
    RuleFixture {
        id: "python_stack_trace",
        pattern: "Traceback \\(most recent call last\\):[\\s\\S]{1,500}?File \\\"[^\\\"]+\\\", line \\d+, in ",
        capture_group: 0,
        severity: "medium",
        surfaces: &["response_body", "websocket_payload"],
        positive: "Traceback (most recent call last):\n  File \"/app/server/main.py\", line 42, in handle_request",
        negative: "Stack trace debugging disabled in production environment.",
    },
    RuleFixture {
        id: "php_fatal_error",
        pattern: "(?i)\\b(?:Fatal error|Parse error):\\s+(?:Uncaught\\s+[A-Za-z0-9_\\\\]+|Call to undefined function|syntax error, unexpected)[^\\r\\n]+in\\s+[^\\r\\n]+\\.php\\s+on\\s+line\\s+\\d+",
        capture_group: 0,
        severity: "medium",
        surfaces: &["response_body"],
        positive: "Fatal error: Call to undefined function get_header() in /var/www/html/index.php on line 15",
        negative: "Warning: Division by zero in /var/www/html/calc.php on line 5",
    },
    RuleFixture {
        id: "node_stack_trace",
        pattern: "(?m)^\\s+at\\s+(?:(?:async\\s+)?[\\w$]+(?:\\.[\\w$]+)*\\s+\\()?(?:/|[A-Za-z]:\\\\)[^\\r\\n]+:\\d+:\\d+\\)?",
        capture_group: 0,
        severity: "medium",
        surfaces: &["response_body", "websocket_payload"],
        positive: "    at handleRequest (/app/src/server/router.js:84:12)",
        negative: "Documentation available at /app/docs/api.md",
    },
    RuleFixture {
        id: "aspnet_custom_errors_off",
        pattern: "(?i)(?:<title>Server Error in '[^']*' Application\\.?</title>|<!--\\s*To enable the details of this specific error message to be viewable on remote machines,\\s*please create a <customErrors> tag\\s*-->)",
        capture_group: 0,
        severity: "high",
        surfaces: &["response_body"],
        positive: "<title>Server Error in '/' Application.</title>",
        negative: "<title>Application Error - 500 Internal Server Error</title>",
    },
    RuleFixture {
        id: "graphql_syntax_error",
        pattern: "(?i)\"message\"\\s*:\\s*\"Syntax Error:\\s+(?:Unexpected\\s+[\\w$]+|Cannot parse the unexpected)",
        capture_group: 0,
        severity: "low",
        surfaces: &["response_body", "websocket_payload"],
        positive: "{\"errors\":[{\"message\":\"Syntax Error: Unexpected Name \\\"foo\\\"\"}]}",
        negative: "{\"errors\":[{\"message\":\"Variable \\\"$id\\\" got invalid value\"}]}",
    },
    RuleFixture {
        id: "cloud_imds_endpoint",
        pattern: "(?:https?://)?(?:169\\.254\\.169\\.254|metadata\\.google\\.internal)(?:/|\\b)",
        capture_group: 0,
        severity: "critical",
        surfaces: &["request_message", "response_body"],
        positive: "http://169.254.169.254/latest/meta-data/",
        negative: "http://169.254.1.1/api/v1/network",
    },
    RuleFixture {
        id: "k8s_internal_dns",
        pattern: "\\b[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\\.default\\.svc\\.cluster\\.local\\b|\\b[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\\.[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\\.svc\\.cluster\\.local\\b",
        capture_group: 0,
        severity: "medium",
        surfaces: &["response_body", "websocket_payload"],
        positive: "payment-service.backend.svc.cluster.local",
        negative: "https://cluster.local.domain.com/overview",
    },
    RuleFixture {
        id: "internal_domain_leak",
        pattern: "(?i)\\b(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\\.)+(?:corp|lan|local|internal|intranet)(?::[0-9]+)?(?:[^\\w.-]|$)",
        capture_group: 0,
        severity: "medium",
        surfaces: &["response_body", "response_message"],
        positive: "dc01.ad.prod.corp",
        negative: "docs.internal-dev.example.com",
    },
    RuleFixture {
        id: "graphql_introspection_schema",
        pattern: "\"__schema\"\\s*:\\s*\\{\\s*\"types\"\\s*:\\s*\\[",
        capture_group: 0,
        severity: "medium",
        surfaces: &["response_body"],
        positive: "{\"data\":{\"__schema\":{\"types\":[{\"kind\":\"OBJECT\",\"name\":\"Query\"}]}}}",
        negative: "{\"data\":{\"schema_version\":\"1.0.0\",\"types\":[\"admin\",\"user\"]}}",
    },
    RuleFixture {
        id: "openapi_spec_body",
        pattern: "(?i)\"(?:openapi|swagger)\"\\s*:\\s*\"[23]\\.[0-9.]+\"[\\s\\S]{1,500}?\"(?:paths|components)\"\\s*:",
        capture_group: 0,
        severity: "low",
        surfaces: &["response_body"],
        positive: "{\"openapi\": \"3.0.3\", \"info\": {\"title\": \"Sample API\"}, \"paths\": {}}",
        negative: "{\"swagger_client_version\": \"2.1.0\", \"status\": \"active\"}",
    },
    RuleFixture {
        id: "postman_collection",
        pattern: "https?://schema\\.(?:get)?postman\\.com/json/collection/v2\\.[01]\\.0/collection\\.json",
        capture_group: 0,
        severity: "medium",
        surfaces: &["response_body"],
        positive: "{\"info\":{\"schema\":\"https://schema.getpostman.com/json/collection/v2.1.0/collection.json\",\"name\":\"Payment API\"}}",
        negative: "{\"name\": \"my-collection\", \"export_version\": \"2.1\"}",
    },
    RuleFixture {
        id: "env_file_marker",
        pattern: "(?m)^(?:(?:APP|DB|AWS|SECRET|API)_[A-Z0-9_]{2,32}|DATABASE_URL|SECRET_KEY|REDIS_URL)\\s*=\\s*[^\\r\\n]{1,256}",
        capture_group: 0,
        severity: "critical",
        surfaces: &["response_body"],
        positive: "DB_PASSWORD=SuperSecretPass123!\nAPP_KEY=base64:abcd1234efgh5678",
        negative: "var config = { DB_PASSWORD: 'hidden' };",
    },
    RuleFixture {
        id: "git_config_marker",
        pattern: r#"(?m)^\s*\[core\]\s*$[\s\S]{1,512}?(?:repositoryformatversion[\s\S]{1,512}?filemode|filemode[\s\S]{1,512}?repositoryformatversion)"#,
        capture_group: 0,
        severity: "critical",
        surfaces: &["response_body"],
        positive: "[core]\n\trepositoryformatversion = 0\n\tfilemode = true",
        negative: "[core]\nname = component-core",
    },
    RuleFixture {
        id: "docker_daemon_socket",
        pattern: "(?i)(?:\\\"ApiVersion\\\"\\s*:\\s*\\\"1\\.\\d+\\\"[\\s\\S]{1,500}?\\\"MinAPIVersion\\\"|Docker-Experimental:\\s+(?:true|false)|\\\"OSType\\\"\\s*:\\s*\\\"linux\\\"[\\s\\S]{1,500}?\\\"Architecture\\\")",
        capture_group: 0,
        severity: "critical",
        surfaces: &["response_body"],
        positive: "{\"ApiVersion\":\"1.43\",\"MinAPIVersion\":\"1.12\",\"GitCommit\":\"ac40704\",\"GoVersion\":\"go1.20.10\"}",
        negative: "{\"docker_installed\": true, \"version\": \"latest\"}",
    },
    RuleFixture {
        id: "param_ssrf_candidate",
        pattern: "(?im)(?:^|[?&])((?:dest|redirect_to|uri|path|feed|host|port|to|out|view))=",
        capture_group: 1,
        severity: "medium",
        surfaces: &["request_message", "websocket_payload"],
        positive: "GET /fetch?dest=https://internal.service HTTP/1.1\r\nHost: example.com\r\n\r\n",
        negative: "GET /gallery?autodestroy=true&viewmode=grid HTTP/1.1\r\nHost: example.com\r\n\r\n",
    },
    RuleFixture {
        id: "param_lfi_candidate",
        pattern: "(?im)(?:^|[?&])((?:file|document|folder|root|pg|style|doc|pdf|template|include|page))=",
        capture_group: 1,
        severity: "medium",
        surfaces: &["request_message", "websocket_payload"],
        positive: "GET /download?file=../../../../etc/passwd HTTP/1.1\r\nHost: example.com\r\n\r\n",
        negative: "GET /profile?userfile_id=12&homepage_title=hello HTTP/1.1\r\nHost: example.com\r\n\r\n",
    },
    RuleFixture {
        id: "param_rce_candidate",
        pattern: "(?im)(?:^|[?&])((?:cmd|exec|command|cli|eval|run|ping|query|code))=",
        capture_group: 1,
        severity: "high",
        surfaces: &["request_message", "websocket_payload"],
        positive: "POST /api/diagnostics HTTP/1.1\r\nHost: example.com\r\nContent-Type: application/x-www-form-urlencoded\r\n\r\ncmd=id",
        negative: "GET /status?running_state=active&reconnect=5 HTTP/1.1\r\nHost: example.com\r\n\r\n",
    },
    RuleFixture {
        id: "param_open_redirect",
        pattern: "(?im)(?:^|[?&])((?:next|url|target|rurl|destination|redir|return_to))=",
        capture_group: 1,
        severity: "low",
        surfaces: &["request_message", "websocket_payload"],
        positive: "GET /auth/login?next=https://attacker.example.com/evil HTTP/1.1\r\nHost: example.com\r\n\r\n",
        negative: "GET /products?curl_enabled=0&retargeting=false HTTP/1.1\r\nHost: example.com\r\n\r\n",
    },
    RuleFixture {
        id: "param_idor_candidate",
        pattern: "(?im)(?:^|[?&])((?:account_id|user_id|org_id|tenant_id|invoice_id|order_id))=",
        capture_group: 1,
        severity: "low",
        surfaces: &["request_message", "websocket_payload"],
        positive: "GET /api/v1/invoice?invoice_id=INV-2026-9810 HTTP/1.1\r\nHost: example.com\r\n\r\n",
        negative: "GET /api/v1/search?order_identifier=asc HTTP/1.1\r\nHost: example.com\r\n\r\n",
    },
    RuleFixture {
        id: "credit_card_number",
        pattern: "\\b(?:4[0-9]{3}(?:[- ]?[0-9]{4}){3}|5[1-5][0-9]{2}(?:[- ]?[0-9]{4}){3}|6(?:011|5[0-9]{2})(?:[- ]?[0-9]{4}){3}|3[47][0-9]{2}[- ]?[0-9]{6}[- ]?[0-9]{5})\\b",
        capture_group: 0,
        severity: "high",
        surfaces: &["response_body", "websocket_payload"],
        positive: "{\"card\": \"4000-1234-5678-9010\"}",
        negative: "{\"build\": \"9876-5432-1098-7654\", \"trace\": \"0000-0000-0000-0000\"}",
    },
    RuleFixture {
        id: "us_ssn",
        pattern: "\\b(?:00[1-9]|0[1-9][0-9]|[1-5][0-9]{2}|6[0-5][0-9]|66[0-5]|66[7-9]|6[7-9][0-9]|[78][0-9]{2})-(?:0[1-9]|[1-9][0-9])-(?:000[1-9]|00[1-9][0-9]|0[1-9][0-9]{2}|[1-9][0-9]{3})\\b",
        capture_group: 0,
        severity: "high",
        surfaces: &["response_body", "websocket_payload"],
        positive: "SSN: 123-45-6789",
        negative: "Invalid: 000-45-6789, 666-45-6789, 987-65-4321, 123-00-6789, 123-45-0000",
    },
    RuleFixture {
        id: "international_phone_number",
        pattern: "(?:^|[\\s\"'`:,])(\\+[1-9][0-9]{7,14})\\b",
        capture_group: 1,
        severity: "low",
        surfaces: &["response_body", "websocket_payload"],
        positive: "Contact: +14155552671",
        negative: "Math: x + 1234567890; Short: +12345; Invalid: +0123456789",
    },
    RuleFixture {
        id: "cors_wildcard_credentials",
        pattern: r#"(?im)(?:^access-control-allow-origin:\s*\*\s*$[\s\S]{1,512}?^access-control-allow-credentials:\s*true\b|^access-control-allow-credentials:\s*true\b[\s\S]{1,512}?^access-control-allow-origin:\s*\*\s*$)"#,
        capture_group: 0,
        severity: "high",
        surfaces: &["response_message"],
        positive: "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Credentials: true\r\n\r\n",
        negative: "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Credentials: false\r\n\r\n",
    },
    RuleFixture {
        id: "cors_null_origin",
        pattern: "(?im)^access-control-allow-origin:\\s*null\\s*$",
        capture_group: 0,
        severity: "medium",
        surfaces: &["response_message"],
        positive: "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: null\r\n\r\n",
        negative: "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: null.example.com\r\n\r\n",
    },
    RuleFixture {
        id: "directory_listing_open",
        pattern: "(?i)(?:<title>\\s*(?:Index of\\s+/|Directory listing for\\s+/)|<h1>\\s*Index of\\s+/)",
        capture_group: 0,
        severity: "medium",
        surfaces: &["response_body"],
        positive: "<html><head><title>Index of /backup</title></head></html>",
        negative: "<html><head><title>User Profiles - Index of Members</title></head></html>",
    },
    RuleFixture {
        id: "csp_unsafe_eval",
        pattern: "(?im)^content-security-policy(?:-report-only)?:\\s*[^\\r\\n]*('unsafe-eval')",
        capture_group: 1,
        severity: "low",
        surfaces: &["response_message"],
        positive: "HTTP/1.1 200 OK\r\nContent-Security-Policy: default-src 'self'; script-src 'self' 'unsafe-eval';\r\n\r\n",
        negative: "HTTP/1.1 200 OK\r\nContent-Security-Policy: default-src 'self'; script-src 'self' 'nonce-R4nd0m';\r\n\r\n",
    },
    RuleFixture {
        id: "missing_content_type_options",
        pattern: r#"(?im)^x-content-type-options:[ \t]*(?:none|0|false|off)\b"#,
        capture_group: 0,
        severity: "low",
        surfaces: &["response_message"],
        positive: "HTTP/1.1 200 OK\r\nX-Content-Type-Options: off\r\n\r\n<h1>Hello</h1>",
        negative: "HTTP/1.1 200 OK\r\nX-Content-Type-Options: nosniff\r\n\r\n<h1>Hello</h1>",
    },
];

const ALL_POSSIBLE_SURFACES: &[&str] = &[
    "request_message",
    "response_message",
    "response_body",
    "websocket_payload",
    "websocket_edited_payload",
];

#[test]
fn rule_corpus_inventory_contains_all_105_unique_rules() {
    let pack = RulePack::default_exact().expect("failed to load default exact rule pack");
    assert_eq!(pack.version(), "2026.09.05", "version must be 2026.09.05");
    let severities: std::collections::HashMap<&str, &str> = pack
        .rules()
        .iter()
        .map(|rule| (rule.id.as_str(), rule.severity.as_str()))
        .collect();

    let rule_ids: std::collections::HashSet<&str> =
        pack.rules().iter().map(|r| r.id.as_str()).collect();
    assert_eq!(
        rule_ids.len(),
        105,
        "pack must have exactly 105 unique rule IDs"
    );
    assert_eq!(pack.rules().len(), 105, "pack rules length must be 105");

    for fixture in NEW_RULE_FIXTURES {
        assert!(
            rule_ids.contains(fixture.id),
            "corpus missing newly added rule ID: {}",
            fixture.id
        );
        assert_eq!(
            severities.get(fixture.id).copied(),
            Some(fixture.severity),
            "rule '{}' severity mismatch",
            fixture.id
        );
    }
}

#[test]
fn all_77_expanded_rules_match_positive_oracle_and_exact_captures() {
    let pack = RulePack::default_exact().expect("failed to load default rule pack");

    for fixture in NEW_RULE_FIXTURES {
        for &surface in fixture.surfaces {
            let matches = pack.matches(surface, fixture.positive.as_bytes());
            let target_matches: Vec<_> = matches
                .into_iter()
                .filter(|m| m.rule_id == fixture.id)
                .collect();

            assert!(
                !target_matches.is_empty(),
                "rule '{}' failed to produce match on declared surface '{}' with positive fixture",
                fixture.id,
                surface
            );

            let primary_re = regex::bytes::Regex::new(fixture.pattern)
                .unwrap_or_else(|e| panic!("invalid regex for rule '{}': {e}", fixture.id));
            let expected_captures = primary_re
                .captures(fixture.positive.as_bytes())
                .unwrap_or_else(|| {
                    panic!(
                        "primary regex for rule '{}' did not match positive fixture",
                        fixture.id
                    )
                });
            let expected_cap = expected_captures
                .get(fixture.capture_group)
                .unwrap_or_else(|| {
                    panic!(
                        "rule '{}' missing capture group {}",
                        fixture.id, fixture.capture_group
                    )
                });

            let m = &target_matches[0];
            assert_eq!(
                m.capture,
                expected_cap.as_bytes(),
                "rule '{}' capture mismatch against primary regex",
                fixture.id
            );
            assert_eq!(
                &fixture.positive.as_bytes()[m.byte_start..m.byte_end],
                m.capture.as_slice(),
                "rule '{}' capture must match exact byte slice",
                fixture.id
            );
            assert_eq!(
                m.byte_start,
                expected_cap.start(),
                "rule '{}' start offset mismatch",
                fixture.id
            );
            assert_eq!(
                m.byte_end,
                expected_cap.end(),
                "rule '{}' end offset mismatch",
                fixture.id
            );
        }
    }
}

#[test]
fn all_77_expanded_rules_reject_negative_oracle() {
    let pack = RulePack::default_exact().expect("failed to load default rule pack");

    for fixture in NEW_RULE_FIXTURES {
        for &surface in fixture.surfaces {
            let matches = pack.matches(surface, fixture.negative.as_bytes());
            let target_matches: Vec<_> = matches
                .into_iter()
                .filter(|m| m.rule_id == fixture.id)
                .collect();

            assert!(
                target_matches.is_empty(),
                "rule '{}' unexpectedly fired on negative fixture on surface '{}'",
                fixture.id,
                surface
            );
        }
    }
}

#[test]
fn all_77_expanded_rules_do_not_fire_on_undeclared_surfaces() {
    let pack = RulePack::default_exact().expect("failed to load default rule pack");

    for fixture in NEW_RULE_FIXTURES {
        let declared: std::collections::HashSet<&str> = fixture.surfaces.iter().copied().collect();
        for &surface in ALL_POSSIBLE_SURFACES {
            if declared.contains(surface) {
                continue;
            }
            let matches = pack.matches(surface, fixture.positive.as_bytes());
            let target_matches: Vec<_> = matches
                .into_iter()
                .filter(|m| m.rule_id == fixture.id)
                .collect();

            assert!(
                target_matches.is_empty(),
                "rule '{}' fired on undeclared surface '{}'",
                fixture.id,
                surface
            );
        }
    }
}

#[test]
fn former_composite_rules_require_all_evidence_in_single_match() {
    let pack = RulePack::default_exact().expect("failed to load default rule pack");

    // 1. gcp_service_account_key: canonical JSON order (type -> private_key_id -> private_key)
    let gcp_full = "{\n  \"type\": \"service_account\",\n  \"project_id\": \"my-project-123\",\n  \"private_key_id\": \"2b387b72ec1b082aa7e52189d9c43f58fb19fb48\",\n  \"private_key\": \"-----BEGIN PRIVATE KEY-----\nMIIEvQIBADANBgkqhkiG9w0BAQEFAASC\n-----END PRIVATE KEY-----\n\"\n}";
    let gcp_m = pack.matches("response_body", gcp_full.as_bytes());
    assert!(
        gcp_m.iter().any(|m| m.rule_id == "gcp_service_account_key"),
        "gcp_service_account_key must match full canonical JSON"
    );
    let gcp_missing_pk = "{\n  \"type\": \"service_account\",\n  \"project_id\": \"my-project-123\",\n  \"private_key_id\": \"2b387b72ec1b082aa7e52189d9c43f58fb19fb48\"\n}";
    assert!(
        pack.matches("response_body", gcp_missing_pk.as_bytes())
            .iter()
            .all(|m| m.rule_id != "gcp_service_account_key"),
        "gcp_service_account_key must fail when private_key missing"
    );
    let gcp_missing_pkid = "{\n  \"type\": \"service_account\",\n  \"private_key\": \"-----BEGIN PRIVATE KEY-----\n\"\n}";
    assert!(
        pack.matches("response_body", gcp_missing_pkid.as_bytes())
            .iter()
            .all(|m| m.rule_id != "gcp_service_account_key"),
        "gcp_service_account_key must fail when private_key_id missing"
    );
    let gcp_wrong_order = "{\n  \"type\": \"service_account\",\n  \"private_key\": \"-----BEGIN PRIVATE KEY-----\n\",\n  \"private_key_id\": \"2b387b72ec1b082aa7e52189d9c43f58fb19fb48\"\n}";
    assert!(
        pack.matches("response_body", gcp_wrong_order.as_bytes())
            .iter()
            .all(|m| m.rule_id != "gcp_service_account_key"),
        "gcp_service_account_key must fail when fields are out of canonical order"
    );

    // 2. azure_sas_token: sp -> sv -> sig query order
    let sas_full = "?sp=r&st=2026-03-04T07:24:52Z&se=2026-04-04T15:24:52Z&spr=https&sv=2022-11-02&sr=c&sig=WSdF9YeZhvrbs%2B%2B1f8ZdDBzEe7fBJ%2BenuaXQ%2BJ9WOw0%3D";
    let sas_m = pack.matches("request_message", sas_full.as_bytes());
    assert!(
        sas_m.iter().any(|m| m.rule_id == "azure_sas_token"),
        "azure_sas_token must match ordered SAS token"
    );
    let sas_missing_sp = "?st=2026-03-04T07:24:52Z&se=2026-04-04T15:24:52Z&spr=https&sv=2022-11-02&sr=c&sig=WSdF9YeZhvrbs%2B%2B1f8ZdDBzEe7fBJ%2BenuaXQ%2BJ9WOw0%3D";
    assert!(
        pack.matches("request_message", sas_missing_sp.as_bytes())
            .iter()
            .all(|m| m.rule_id != "azure_sas_token"),
        "azure_sas_token must fail when sp missing"
    );
    let sas_missing_sv = "?sp=r&sig=WSdF9YeZhvrbs%2B%2B1f8ZdDBzEe7fBJ%2BenuaXQ%2BJ9WOw0%3D";
    assert!(
        pack.matches("request_message", sas_missing_sv.as_bytes())
            .iter()
            .all(|m| m.rule_id != "azure_sas_token"),
        "azure_sas_token must fail when sv missing"
    );
    let sas_wrong_order =
        "?sig=WSdF9YeZhvrbs%2B%2B1f8ZdDBzEe7fBJ%2BenuaXQ%2BJ9WOw0%3D&sp=r&sv=2022-11-02";
    assert!(
        pack.matches("request_message", sas_wrong_order.as_bytes())
            .iter()
            .all(|m| m.rule_id != "azure_sas_token"),
        "azure_sas_token must fail when parameters are out of normal order"
    );

    // 3. session_cookie_django: sessionid and csrftoken in either order
    let django_order1 = "Cookie: sessionid=1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d; csrftoken=uW8vO0E7qB9A1x2Y3z4W5v6U7t8S9r0Q1P2o3N4M5l6K7j8I9h0G1f2E3d4C5b6A";
    let django_order2 = "Cookie: csrftoken=uW8vO0E7qB9A1x2Y3z4W5v6U7t8S9r0Q1P2o3N4M5l6K7j8I9h0G1f2E3d4C5b6A; sessionid=1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d";
    assert!(
        pack.matches("request_message", django_order1.as_bytes())
            .iter()
            .any(|m| m.rule_id == "session_cookie_django"),
        "session_cookie_django must match sessionid then csrftoken"
    );
    assert!(
        pack.matches("request_message", django_order2.as_bytes())
            .iter()
            .any(|m| m.rule_id == "session_cookie_django"),
        "session_cookie_django must match csrftoken then sessionid"
    );
    let django_missing_csrf = "Cookie: sessionid=1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d; tracking_id=123";
    assert!(
        pack.matches("request_message", django_missing_csrf.as_bytes())
            .iter()
            .all(|m| m.rule_id != "session_cookie_django"),
        "session_cookie_django must fail when csrftoken missing"
    );

    // 4. git_config_marker: [core] with repositoryformatversion and filemode in either order
    let git_order1 = "[core]\n\trepositoryformatversion = 0\n\tfilemode = true";
    let git_order2 = "[core]\n\tfilemode = true\n\trepositoryformatversion = 0";
    assert!(
        pack.matches("response_body", git_order1.as_bytes())
            .iter()
            .any(|m| m.rule_id == "git_config_marker"),
        "git_config_marker must match repositoryformatversion then filemode"
    );
    assert!(
        pack.matches("response_body", git_order2.as_bytes())
            .iter()
            .any(|m| m.rule_id == "git_config_marker"),
        "git_config_marker must match filemode then repositoryformatversion"
    );
    let git_missing_filemode = "[core]\n\trepositoryformatversion = 0";
    assert!(
        pack.matches("response_body", git_missing_filemode.as_bytes())
            .iter()
            .all(|m| m.rule_id != "git_config_marker"),
        "git_config_marker must fail when filemode missing"
    );

    // 5. cors_wildcard_credentials: both headers in either order
    let cors_order1 = "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Credentials: true\r\n\r\n";
    let cors_order2 = "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Credentials: true\r\nAccess-Control-Allow-Origin: *\r\n\r\n";
    assert!(
        pack.matches("response_message", cors_order1.as_bytes())
            .iter()
            .any(|m| m.rule_id == "cors_wildcard_credentials"),
        "cors_wildcard_credentials must match origin then credentials"
    );
    assert!(
        pack.matches("response_message", cors_order2.as_bytes())
            .iter()
            .any(|m| m.rule_id == "cors_wildcard_credentials"),
        "cors_wildcard_credentials must match credentials then origin"
    );
    let cors_missing_creds = "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Credentials: false\r\n\r\n";
    assert!(
        pack.matches("response_message", cors_missing_creds.as_bytes())
            .iter()
            .all(|m| m.rule_id != "cors_wildcard_credentials"),
        "cors_wildcard_credentials must fail when credentials is not true"
    );
}

#[test]
fn missing_content_type_options_only_matches_explicit_insecure_values() {
    let pack = RulePack::default_exact().expect("failed to load default rule pack");
    for input in [
        b"".as_slice(),
        b"garbage",
        b"not an HTTP response\r\n\r\n",
        b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<h1>Hello</h1>",
        b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nX-Content-Type-Options: nosniff\r\n\r\n",
    ] {
        assert!(
            pack.matches("response_message", input)
                .iter()
                .all(|finding| finding.rule_id != "missing_content_type_options"),
            "missing_content_type_options must not match missing headers or nosniff"
        );
    }

    for insecure_val in ["none", "0", "false", "off"] {
        let msg = format!("HTTP/1.1 200 OK\r\nX-Content-Type-Options: {insecure_val}\r\n\r\n");
        let matches = pack.matches("response_message", msg.as_bytes());
        assert!(
            matches
                .iter()
                .any(|m| m.rule_id == "missing_content_type_options"),
            "missing_content_type_options must match insecure value '{insecure_val}'"
        );
    }
}

#[test]
fn arbitrary_byte_property_like_matches_do_not_panic_or_violate_bounds() {
    let pack = RulePack::default_exact().expect("failed to load default rule pack");

    // Deterministic pseudo-random bytes and pathological boundaries
    let mut inputs: Vec<Vec<u8>> = vec![
        vec![],
        vec![0; 256],
        vec![255; 256],
        (0u8..=255).collect(),
        b"\r\n\r\n".to_vec(),
        b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n".to_vec(),
    ];

    let mut seed: u64 = 0x123456789abcdef0;
    for _ in 0..20 {
        let mut chunk = Vec::with_capacity(128);
        for _ in 0..128 {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            chunk.push((seed >> 32) as u8);
        }
        inputs.push(chunk);
    }

    for &surface in ALL_POSSIBLE_SURFACES {
        for input in &inputs {
            let matches = pack.matches(surface, input);
            assert!(
                matches.len() <= pack.max_matches(),
                "matches length must respect max_matches cap"
            );
            for m in matches {
                assert!(
                    m.byte_start <= m.byte_end,
                    "start <= end for rule {}",
                    m.rule_id
                );
                assert!(
                    m.byte_end <= input.len(),
                    "byte_end must be within input bounds for rule {}",
                    m.rule_id
                );
                assert_eq!(
                    &input[m.byte_start..m.byte_end],
                    m.capture.as_slice(),
                    "capture must match exact byte slice in input for rule {}",
                    m.rule_id
                );
            }
        }
    }
}
