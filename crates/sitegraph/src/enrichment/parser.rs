use pest::Parser;
use pest_derive::Parser;
use std::collections::HashSet;

#[derive(Parser)]
#[grammar = "enrichment/grammar.pest"]
pub struct RuleDslParser;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedRulePack {
    pub id: String,
    pub version: String,
    pub max_matches: usize,
    pub rules: Vec<ParsedRule>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedRule {
    pub id: String,
    pub pattern: String,
    pub capture_group: usize,
    pub severity: String,
    pub surfaces: Vec<String>,
}

const VALID_SEVERITIES: &[&str] = &["critical", "high", "medium", "low"];
const VALID_SURFACES: &[&str] = &[
    "request_message",
    "response_message",
    "response_body",
    "websocket_payload",
    "websocket_edited_payload",
];

pub fn parse_rule_pack(input: &str) -> Result<ParsedRulePack, String> {
    let pairs = RuleDslParser::parse(Rule::file, input)
        .map_err(|error| format!("failed to parse rule pack DSL: {error}"))?;

    let mut pack_opt: Option<(Option<String>, Option<String>, Option<usize>)> = None;
    let mut parsed_rules = Vec::new();

    for file_pair in pairs {
        for inner in file_pair.into_inner() {
            match inner.as_rule() {
                Rule::pack_decl => {
                    if pack_opt.is_some() {
                        return Err("multiple pack declarations in rule pack DSL".to_owned());
                    }
                    let mut id = None;
                    let mut version = None;
                    let mut max_matches = None;

                    for field in inner.into_inner() {
                        let field_inner = field.into_inner().next().ok_or("empty pack field")?;
                        match field_inner.as_rule() {
                            Rule::id_field => {
                                if id.is_some() {
                                    return Err("duplicate pack field: id".to_owned());
                                }
                                let str_pair = field_inner
                                    .into_inner()
                                    .next()
                                    .ok_or("missing pack id value")?;
                                id = Some(parse_string_val(str_pair)?);
                            }
                            Rule::version_field => {
                                if version.is_some() {
                                    return Err("duplicate pack field: version".to_owned());
                                }
                                let str_pair = field_inner
                                    .into_inner()
                                    .next()
                                    .ok_or("missing pack version value")?;
                                version = Some(parse_string_val(str_pair)?);
                            }
                            Rule::max_matches_field => {
                                if max_matches.is_some() {
                                    return Err("duplicate pack field: max_matches".to_owned());
                                }
                                let int_pair = field_inner
                                    .into_inner()
                                    .next()
                                    .ok_or("missing pack max_matches value")?;
                                let val: usize = int_pair
                                    .as_str()
                                    .parse()
                                    .map_err(|e| format!("invalid max_matches integer: {e}"))?;
                                max_matches = Some(val);
                            }
                            _ => {}
                        }
                    }

                    pack_opt = Some((id, version, max_matches));
                }
                Rule::rule_decl => {
                    let mut rule_parts = inner.into_inner();
                    let id_pair = rule_parts.next().ok_or("missing rule id")?;
                    let rule_id = parse_string_val(id_pair)?;

                    let mut pattern = None;
                    let mut capture_group = None;
                    let mut severity = None;
                    let mut surfaces = None;

                    for field in rule_parts {
                        let field_inner = field.into_inner().next().ok_or("empty rule field")?;
                        match field_inner.as_rule() {
                            Rule::pattern_field => {
                                if pattern.is_some() {
                                    return Err(format!(
                                        "duplicate rule field 'pattern' in rule '{rule_id}'"
                                    ));
                                }
                                let str_pair = field_inner
                                    .into_inner()
                                    .next()
                                    .ok_or("missing pattern value")?;
                                pattern = Some(parse_string_val(str_pair)?);
                            }
                            Rule::capture_group_field => {
                                if capture_group.is_some() {
                                    return Err(format!(
                                        "duplicate rule field 'capture_group' in rule '{rule_id}'"
                                    ));
                                }
                                let int_pair = field_inner
                                    .into_inner()
                                    .next()
                                    .ok_or("missing capture_group value")?;
                                let val: usize = int_pair.as_str().parse().map_err(|e| {
                                    format!(
                                        "invalid capture_group integer in rule '{rule_id}': {e}"
                                    )
                                })?;
                                capture_group = Some(val);
                            }
                            Rule::severity_field => {
                                if severity.is_some() {
                                    return Err(format!(
                                        "duplicate rule field 'severity' in rule '{rule_id}'"
                                    ));
                                }
                                let sev_pair = field_inner
                                    .into_inner()
                                    .next()
                                    .ok_or("missing severity value")?;
                                let sev_val = match sev_pair.as_rule() {
                                    Rule::severity_val => sev_pair.as_str().to_owned(),
                                    Rule::string_val => parse_string_val(sev_pair)?,
                                    _ => sev_pair.as_str().trim().trim_matches('"').to_owned(),
                                };
                                severity = Some(sev_val);
                            }
                            Rule::surfaces_field => {
                                if surfaces.is_some() {
                                    return Err(format!(
                                        "duplicate rule field 'surfaces' in rule '{rule_id}'"
                                    ));
                                }
                                let list_pair = field_inner
                                    .into_inner()
                                    .next()
                                    .ok_or("missing surfaces list")?;
                                let mut surfs = Vec::new();
                                for item in list_pair.into_inner() {
                                    let item_inner =
                                        item.into_inner().next().ok_or("empty surface item")?;
                                    let s = match item_inner.as_rule() {
                                        Rule::surface_name => item_inner.as_str().to_owned(),
                                        Rule::string_val => parse_string_val(item_inner)?,
                                        _ => {
                                            item_inner.as_str().trim().trim_matches('"').to_owned()
                                        }
                                    };
                                    surfs.push(s);
                                }
                                surfaces = Some(surfs);
                            }
                            _ => {}
                        }
                    }

                    let pattern = pattern.ok_or_else(|| {
                        format!("rule '{rule_id}' is missing required field 'pattern'")
                    })?;
                    let capture_group = capture_group.ok_or_else(|| {
                        format!("rule '{rule_id}' is missing required field 'capture_group'")
                    })?;
                    let severity = severity.ok_or_else(|| {
                        format!("rule '{rule_id}' is missing required field 'severity'")
                    })?;
                    let surfaces = surfaces.ok_or_else(|| {
                        format!("rule '{rule_id}' is missing required field 'surfaces'")
                    })?;

                    parsed_rules.push(ParsedRule {
                        id: rule_id,
                        pattern,
                        capture_group,
                        severity,
                        surfaces,
                    });
                }
                Rule::EOI => {}
                _ => {}
            }
        }
    }

    let (id_opt, ver_opt, max_m_opt) =
        pack_opt.ok_or("missing pack declaration in rule pack DSL")?;
    let pack_id = id_opt.ok_or("pack missing required field 'id'")?;
    let pack_version = ver_opt.ok_or("pack missing required field 'version'")?;
    let max_matches = max_m_opt.ok_or("pack missing required field 'max_matches'")?;

    validate_name("rule pack id", &pack_id)?;
    validate_name("rule pack version", &pack_version)?;

    if max_matches == 0 || max_matches > 4_096 {
        return Err("rule pack max_matches must be between 1 and 4096".to_owned());
    }

    if parsed_rules.is_empty() || parsed_rules.len() > 512 {
        return Err("rule pack must contain between 1 and 512 rules".to_owned());
    }

    let mut rule_ids = HashSet::with_capacity(parsed_rules.len());
    for rule in &parsed_rules {
        validate_name("rule id", &rule.id)?;
        if !rule_ids.insert(rule.id.clone()) {
            return Err(format!("duplicate rule id: {}", rule.id));
        }

        if !VALID_SEVERITIES.contains(&rule.severity.as_str()) {
            return Err(format!(
                "rule '{}' has invalid severity '{}' (expected one of: {})",
                rule.id,
                rule.severity,
                VALID_SEVERITIES.join(", ")
            ));
        }

        if rule.surfaces.is_empty() {
            return Err(format!(
                "rule {} must declare at least one surface",
                rule.id
            ));
        }

        for surface in &rule.surfaces {
            if !VALID_SURFACES.contains(&surface.as_str()) {
                return Err(format!(
                    "rule '{}' has invalid surface '{}' (expected one of: {})",
                    rule.id,
                    surface,
                    VALID_SURFACES.join(", ")
                ));
            }
        }

        if rule.pattern.is_empty() || rule.pattern.len() > 16 * 1_024 {
            return Err(format!("rule {} pattern is empty or too large", rule.id));
        }
    }

    Ok(ParsedRulePack {
        id: pack_id,
        version: pack_version,
        max_matches,
        rules: parsed_rules,
    })
}

fn parse_string_val(pair: pest::iterators::Pair<'_, Rule>) -> Result<String, String> {
    let inner = pair
        .into_inner()
        .next()
        .ok_or("expected inner string representation")?;
    match inner.as_rule() {
        Rule::raw_string => parse_raw_string(inner.as_str()),
        Rule::quoted_string => parse_quoted_string(inner.as_str()),
        _ => Err(format!("unexpected string token: {:?}", inner.as_rule())),
    }
}

fn parse_raw_string(raw: &str) -> Result<String, String> {
    let after_r = raw.strip_prefix('r').ok_or("invalid raw string prefix")?;
    let hash_count = after_r.chars().take_while(|&c| c == '#').count();
    let prefix_len = 1 + hash_count + 1; // 'r' + '#' * count + '"'
    let suffix_len = 1 + hash_count; // '"' + '#' * count
    if raw.len() < prefix_len + suffix_len {
        return Err("raw string literal too short".to_owned());
    }
    let body = &raw[prefix_len..raw.len() - suffix_len];
    Ok(body.to_owned())
}

fn parse_quoted_string(quoted: &str) -> Result<String, String> {
    if !quoted.starts_with('"') || !quoted.ends_with('"') || quoted.len() < 2 {
        return Err("quoted string missing bounding quotes".to_owned());
    }
    let inner = &quoted[1..quoted.len() - 1];
    let mut result = String::with_capacity(inner.len());
    let mut chars = inner.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next().ok_or("incomplete escape in string literal")? {
                '"' => result.push('"'),
                '\\' => result.push('\\'),
                '/' => result.push('/'),
                'b' => result.push('\x08'),
                'f' => result.push('\x0c'),
                'n' => result.push('\n'),
                'r' => result.push('\r'),
                't' => result.push('\t'),
                'u' => {
                    let mut hex = String::with_capacity(4);
                    for _ in 0..4 {
                        hex.push(chars.next().ok_or("incomplete unicode escape")?);
                    }
                    let code = u32::from_str_radix(&hex, 16)
                        .map_err(|e| format!("invalid hex in unicode escape \\u{hex}: {e}"))?;
                    let unicode_char = char::from_u32(code)
                        .ok_or_else(|| format!("invalid unicode code point: {code}"))?;
                    result.push(unicode_char);
                }
                other => {
                    result.push('\\');
                    result.push(other);
                }
            }
        } else {
            result.push(ch);
        }
    }
    Ok(result)
}

pub fn validate_name(label: &str, value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(format!("{label} contains unsupported characters"));
    }
    Ok(())
}
