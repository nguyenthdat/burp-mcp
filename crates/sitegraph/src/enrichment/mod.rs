use regex::bytes::{Regex, RegexSet};
use serde::Deserialize;
use std::collections::HashSet;

pub const DEFAULT_RULE_PACK: &[u8] = include_bytes!("rules/default-rules.json");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleMatch {
    pub rule_id: String,
    pub byte_start: usize,
    pub byte_end: usize,
    pub capture: Vec<u8>,
    pub severity: String,
}

#[derive(Debug)]
pub struct RulePack {
    id: String,
    version: String,
    rules: Vec<CompiledRule>,
    set: RegexSet,
    max_matches: usize,
}
#[derive(Debug)]
struct CompiledRule {
    id: String,
    regex: Regex,
    capture_group: usize,
    severity: String,
    surfaces: HashSet<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRulePack {
    id: String,
    version: String,
    max_matches: usize,
    rules: Vec<RawRule>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRule {
    id: String,
    pattern: String,
    capture_group: usize,
    severity: String,
    surfaces: Vec<String>,
}

impl RulePack {
    pub fn default_exact() -> Result<Self, String> {
        Self::from_json(DEFAULT_RULE_PACK)
    }

    pub fn from_path(path: &std::path::Path) -> Result<Self, String> {
        let document = std::fs::read(path)
            .map_err(|error| format!("failed to read rule pack {}: {error}", path.display()))?;
        Self::from_json(&document)
    }

    pub fn from_json(document: &[u8]) -> Result<Self, String> {
        let raw: RawRulePack = serde_json::from_slice(document)
            .map_err(|error| format!("invalid rule pack JSON: {error}"))?;
        validate_name("rule pack id", &raw.id)?;
        validate_name("rule pack version", &raw.version)?;
        if raw.max_matches == 0 || raw.max_matches > 4_096 {
            return Err("rule pack max_matches must be between 1 and 4096".to_owned());
        }
        if raw.rules.is_empty() || raw.rules.len() > 512 {
            return Err("rule pack must contain between 1 and 512 rules".to_owned());
        }
        let mut rule_ids = HashSet::with_capacity(raw.rules.len());
        let mut rules = Vec::with_capacity(raw.rules.len());
        for raw_rule in raw.rules {
            validate_name("rule id", &raw_rule.id)?;
            if !rule_ids.insert(raw_rule.id.clone()) {
                return Err(format!("duplicate rule id: {}", raw_rule.id));
            }
            if raw_rule.pattern.is_empty() || raw_rule.pattern.len() > 16 * 1_024 {
                return Err(format!(
                    "rule {} pattern is empty or too large",
                    raw_rule.id
                ));
            }
            if raw_rule.surfaces.is_empty() {
                return Err(format!(
                    "rule {} must declare at least one surface",
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
        let patterns = rules.iter().map(|r| r.regex.as_str());
        let set = RegexSet::new(patterns)
            .map_err(|e| format!("failed to build compiled rule set: {e}"))?;
        Ok(Self {
            id: raw.id,
            version: raw.version,
            rules,
            set,
            max_matches: raw.max_matches,
        })
    }
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn matches(&self, surface: &str, input: &[u8]) -> Vec<RuleMatch> {
        let mut matches = Vec::new();
        let set_matches = self.set.matches(input);
        for idx in set_matches.into_iter() {
            let rule = &self.rules[idx];
            if !rule.surfaces.contains(surface) {
                continue;
            }
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

fn validate_name(label: &str, value: &str) -> Result<(), String> {
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

#[cfg(test)]
mod tests;
