use regex::bytes::{Regex, RegexSet, RegexSetBuilder};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub const DEFAULT_RULE_PACK: &str = include_str!("rules/default-rules.toml");

const VALID_SEVERITIES: &[&str] = &["critical", "high", "medium", "low"];
const VALID_SURFACES: &[&str] = &[
    "request_message",
    "response_message",
    "response_body",
    "websocket_payload",
    "websocket_edited_payload",
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRulePackDoc {
    pack: RawPackHeader,
    rules: Vec<RawRule>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPackHeader {
    id: String,
    version: String,
    max_matches: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRule {
    id: String,
    pattern: String,
    capture_group: usize,
    severity: String,
    surfaces: Vec<String>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleMatch {
    pub rule_id: String,
    pub byte_start: usize,
    pub byte_end: usize,
    pub capture: Vec<u8>,
    pub severity: String,
}

#[derive(Debug, Clone)]
pub struct RulePack {
    id: String,
    version: String,
    pub(crate) rules: Vec<CompiledRule>,
    partitions: HashMap<String, SurfacePartition>,
    max_matches: usize,
}

#[derive(Debug, Clone)]
struct SurfacePartition {
    set: RegexSet,
    rule_indices: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct CompiledRule {
    pub id: String,
    pub regex: Regex,
    pub capture_group: usize,
    pub severity: String,
    pub surfaces: HashSet<String>,
}

impl RulePack {
    pub fn default_exact() -> Result<Self, String> {
        Self::from_toml(DEFAULT_RULE_PACK)
    }

    pub fn from_path(path: &Path) -> Result<Self, String> {
        let document = std::fs::read_to_string(path)
            .map_err(|error| format!("failed to read rule pack {}: {error}", path.display()))?;
        Self::from_toml(&document)
            .map_err(|error| format!("failed to parse TOML rule pack {}: {error}", path.display()))
    }

    pub fn from_toml(document: &str) -> Result<Self, String> {
        let parsed: RawRulePackDoc = toml::from_str(document)
            .map_err(|error| format!("failed to parse TOML rule pack: {error}"))?;

        validate_name("rule pack id", &parsed.pack.id)?;
        validate_name("rule pack version", &parsed.pack.version)?;

        if parsed.pack.max_matches == 0 || parsed.pack.max_matches > 4_096 {
            return Err("rule pack max_matches must be between 1 and 4096".to_owned());
        }

        if parsed.rules.is_empty() || parsed.rules.len() > 512 {
            return Err("rule pack must contain between 1 and 512 rules".to_owned());
        }

        let mut rule_ids = HashSet::with_capacity(parsed.rules.len());
        let mut rules = Vec::with_capacity(parsed.rules.len());

        for raw_rule in parsed.rules {
            validate_name("rule id", &raw_rule.id)?;
            if !rule_ids.insert(raw_rule.id.clone()) {
                return Err(format!("duplicate rule id: {}", raw_rule.id));
            }

            if !VALID_SEVERITIES.contains(&raw_rule.severity.as_str()) {
                return Err(format!(
                    "rule '{}' has invalid severity '{}' (expected one of: {})",
                    raw_rule.id,
                    raw_rule.severity,
                    VALID_SEVERITIES.join(", ")
                ));
            }

            if raw_rule.surfaces.is_empty() {
                return Err(format!(
                    "rule {} must declare at least one surface",
                    raw_rule.id
                ));
            }

            for surface in &raw_rule.surfaces {
                if !VALID_SURFACES.contains(&surface.as_str()) {
                    return Err(format!(
                        "rule '{}' has invalid surface '{}' (expected one of: {})",
                        raw_rule.id,
                        surface,
                        VALID_SURFACES.join(", ")
                    ));
                }
            }

            if raw_rule.pattern.is_empty() || raw_rule.pattern.len() > 16 * 1_024 {
                return Err(format!(
                    "rule {} pattern is empty or too large",
                    raw_rule.id
                ));
            }

            let regex = Regex::new(&raw_rule.pattern)
                .map_err(|error| format!("invalid regex for rule {}: {error}", raw_rule.id))?;
            if raw_rule.capture_group >= regex.captures_len() {
                return Err(format!(
                    "rule {} capture_group {} does not exist",
                    raw_rule.id, raw_rule.capture_group
                ));
            }

            rules.push(CompiledRule {
                id: raw_rule.id,
                regex,
                capture_group: raw_rule.capture_group,
                severity: raw_rule.severity,
                surfaces: raw_rule.surfaces.into_iter().collect(),
            });
        }

        let mut surface_to_rule_indices: HashMap<String, Vec<usize>> = HashMap::new();
        for (idx, rule) in rules.iter().enumerate() {
            for surface in &rule.surfaces {
                surface_to_rule_indices
                    .entry(surface.clone())
                    .or_default()
                    .push(idx);
            }
        }

        let mut partitions = HashMap::with_capacity(surface_to_rule_indices.len());
        for (surface, indices) in surface_to_rule_indices {
            let patterns: Vec<&str> = indices
                .iter()
                .map(|&idx| rules[idx].regex.as_str())
                .collect();
            let set = RegexSetBuilder::new(&patterns)
                .size_limit(64 * 1024 * 1024)
                .build()
                .map_err(|e| {
                    format!("failed to build compiled rule set for surface {surface}: {e}")
                })?;
            partitions.insert(
                surface,
                SurfacePartition {
                    set,
                    rule_indices: indices,
                },
            );
        }

        Ok(Self {
            id: parsed.pack.id,
            version: parsed.pack.version,
            rules,
            partitions,
            max_matches: parsed.pack.max_matches,
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn max_matches(&self) -> usize {
        self.max_matches
    }

    pub fn rules(&self) -> &[CompiledRule] {
        &self.rules
    }

    pub fn matches(&self, surface: &str, input: &[u8]) -> Vec<RuleMatch> {
        let mut matches = Vec::new();
        let Some(partition) = self.partitions.get(surface) else {
            return matches;
        };
        let set_matches = partition.set.matches(input);
        for idx in set_matches.into_iter() {
            let rule = &self.rules[partition.rule_indices[idx]];
            for captures in rule.regex.captures_iter(input) {
                let Some(found) = captures.get(rule.capture_group) else {
                    continue;
                };
                matches.push(RuleMatch {
                    rule_id: rule.id.clone(),
                    byte_start: found.start(),
                    byte_end: found.end(),
                    capture: found.as_bytes().to_vec(),
                    severity: rule.severity.clone(),
                });
                if matches.len() == self.max_matches {
                    return matches;
                }
            }
        }
        matches
    }
}

#[cfg(test)]
mod tests;
