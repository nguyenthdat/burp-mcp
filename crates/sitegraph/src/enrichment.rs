use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleMatch {
    pub kind: &'static str,
    pub byte_start: usize,
    pub byte_end: usize,
    pub capture: Vec<u8>,
}

pub struct RulePack {
    pub id: &'static str,
    pub version: &'static str,
    rules: Vec<(&'static str, Regex)>,
    max_matches: usize,
}

impl RulePack {
    pub fn default_exact() -> Result<Self, regex::Error> {
        Ok(Self {
            id: "default",
            version: "1",
            rules: vec![
                (
                    "secret_like_value",
                    Regex::new(r#"(?i)(?:token|secret|api[_-]?key|authorization)\s*[:=]\s*([^\s&\"']+)"#)?,
                ),
                (
                    "jwt",
                    Regex::new(r"eyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}")?,
                ),
            ],
            max_matches: 128,
        })
    }

    pub fn matches(&self, input: &[u8]) -> Vec<RuleMatch> {
        let text = String::from_utf8_lossy(input);
        let mut matches = Vec::new();
        for (kind, rule) in &self.rules {
            for captures in rule.captures_iter(&text) {
                let found = captures.get(1).or_else(|| captures.get(0));
                let Some(found) = found else { continue };
                matches.push(RuleMatch {
                    kind,
                    byte_start: found.start(),
                    byte_end: found.end(),
                    capture: found.as_str().as_bytes().to_vec(),
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
mod tests {
    use super::*;

    #[test]
    fn default_rules_keep_exact_secret_and_jwt_captures() {
        let pack = RulePack::default_exact().unwrap();
        let findings = pack.matches(b"token=exact-value eyJ12345678.abcdefgh.ijklmnop");
        assert!(findings.iter().any(|finding| finding.capture == b"exact-value"));
        assert!(findings.iter().any(|finding| finding.kind == "jwt"));
        assert!(findings.len() <= 128);
        assert_eq!(pack.id, "default");
        assert_eq!(pack.version, "1");
    }
}
