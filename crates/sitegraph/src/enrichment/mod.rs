use regex::bytes::{Regex, RegexSet};
use std::collections::HashSet;
use std::path::Path;

pub mod parser;

pub const DEFAULT_RULE_PACK: &str = include_str!("rules/default-rules.rules");

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
    rules: Vec<CompiledRule>,
    set: RegexSet,
    max_matches: usize,
}

#[derive(Debug, Clone)]
struct CompiledRule {
    id: String,
    regex: Regex,
    capture_group: usize,
    severity: String,
    surfaces: HashSet<String>,
}

impl RulePack {
    pub fn default_exact() -> Result<Self, String> {
        Self::from_dsl(DEFAULT_RULE_PACK)
    }

    pub fn from_path(path: &Path) -> Result<Self, String> {
        let document = std::fs::read_to_string(path)
            .map_err(|error| format!("failed to read rule pack {}: {error}", path.display()))?;
        Self::from_dsl(&document)
    }

    pub fn from_dsl(document: &str) -> Result<Self, String> {
        let parsed = parser::parse_rule_pack(document)?;
        let mut rules = Vec::with_capacity(parsed.rules.len());
        for raw_rule in parsed.rules {
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
            id: parsed.id,
            version: parsed.version,
            rules,
            set,
            max_matches: parsed.max_matches,
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

#[cfg(test)]
mod tests;
