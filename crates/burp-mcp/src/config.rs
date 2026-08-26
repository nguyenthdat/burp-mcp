use anyhow::{Context, Result, bail};
use serde::Deserialize;
use sitegraph::enrichment::{DEFAULT_RULE_PACK, RulePack};
use std::path::{Path, PathBuf};

pub const DEFAULT_CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub burp: BurpConfig,
    pub sitegraph: SitegraphConfig,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BurpConfig {
    pub endpoint: Option<String>,
    pub port: Option<u16>,
    pub tls: bool,
    pub tls_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SitegraphConfig {
    pub enabled: bool,
    pub project_root: Option<PathBuf>,
    pub daemon: Option<PathBuf>,
    pub rules_path: Option<PathBuf>,
    pub mode: String,
    pub interval_seconds: u64,
}

impl Default for SitegraphConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            project_root: None,
            daemon: None,
            rules_path: None,
            mode: "off".to_owned(),
            interval_seconds: 30,
        }
    }
}

pub fn load(path: &Path) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read configuration {}", path.display()))?;
    let config = toml::from_str(&content)
        .with_context(|| format!("failed to parse TOML configuration {}", path.display()))?;
    validate(&config, path)?;
    Ok(config)
}

fn validate(config: &Config, path: &Path) -> Result<()> {
    if let Some(port) = config.burp.port
        && port == 0
    {
        bail!("configuration {} sets burp.port to zero", path.display());
    }
    if !matches!(config.sitegraph.mode.as_str(), "off" | "startup" | "watch") {
        bail!(
            "configuration {} sets sitegraph.mode to an invalid value",
            path.display()
        );
    }
    if config.sitegraph.interval_seconds == 0 {
        bail!(
            "configuration {} sets sitegraph.interval_seconds to zero",
            path.display()
        );
    }
    Ok(())
}

pub fn default_path() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("burp-mcp")
        .join(DEFAULT_CONFIG_FILE)
}

pub fn default_rules_path() -> PathBuf {
    default_path().with_file_name("default-rules.json")
}

pub fn ensure_rules_file(path: &Path) -> Result<()> {
    if path.is_file() {
        RulePack::from_path(path).map_err(|error| {
            anyhow::anyhow!("invalid sitegraph rules file {}: {error}", path.display())
        })?;
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create rules directory {}", parent.display()))?;
    }
    let temporary = path.with_extension("json.new");
    std::fs::write(&temporary, DEFAULT_RULE_PACK)
        .with_context(|| format!("failed to initialize rules file {}", path.display()))?;
    std::fs::rename(&temporary, path)
        .with_context(|| format!("failed to install rules file {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::load;

    #[test]
    fn loads_burp_and_sitegraph_settings() {
        let directory = tempfile::tempdir().expect("temporary directory must exist");
        let path = directory.path().join("config.toml");
        std::fs::write(
            &path,
            r#"
[burp]
endpoint = "http://127.0.0.1:10077"
tls_dir = "/tmp/burp-mcp-tls"

[sitegraph]
enabled = true
project_root = "/tmp/burp-mcp-sitegraph"
mode = "watch"
interval_seconds = 45
"#,
        )
        .expect("configuration fixture must be written");

        let config = load(&path).expect("configuration must parse");
        assert_eq!(
            Some("http://127.0.0.1:10077"),
            config.burp.endpoint.as_deref()
        );
        assert_eq!(
            Some(std::path::Path::new("/tmp/burp-mcp-tls")),
            config.burp.tls_dir.as_deref()
        );
        assert!(config.sitegraph.enabled);
        assert_eq!(
            Some(std::path::Path::new("/tmp/burp-mcp-sitegraph")),
            config.sitegraph.project_root.as_deref()
        );
        assert_eq!("watch", config.sitegraph.mode);
        assert_eq!(45, config.sitegraph.interval_seconds);
    }

    #[test]
    fn rejects_unknown_settings() {
        let directory = tempfile::tempdir().expect("temporary directory must exist");
        let path = directory.path().join("config.toml");
        std::fs::write(&path, "[sitegraph]\nenabled = true\nunknown = 1\n")
            .expect("configuration fixture must be written");

        let error = load(&path).expect_err("unknown settings must fail");
        assert!(
            error
                .to_string()
                .contains("failed to parse TOML configuration")
        );
    }

    #[test]
    fn initializes_default_rules_once_without_overwriting_custom_rules() {
        let directory = tempfile::tempdir().expect("temporary directory must exist");
        let path = directory.path().join("default-rules.json");

        super::ensure_rules_file(&path).expect("default rules must initialize");
        assert_eq!(
            std::fs::read(&path).unwrap(),
            sitegraph::enrichment::DEFAULT_RULE_PACK
        );

        let custom = br#"{
          "id":"custom","version":"1","max_matches":1,
          "rules":[{"id":"custom","pattern":"custom","capture_group":0,"severity":"low","surfaces":["response_body"]}]
        }"#;
        std::fs::write(&path, custom).unwrap();
        super::ensure_rules_file(&path).expect("custom rules must be retained");
        assert_eq!(std::fs::read(&path).unwrap(), custom);
    }
}
