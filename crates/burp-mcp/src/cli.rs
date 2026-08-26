use crate::config::Config;
use clap::{Args, Parser, Subcommand};
use std::path::{Path, PathBuf};

pub const DEFAULT_ENDPOINT: &str = "http://127.0.0.1:9877";

#[derive(Debug, Parser)]
#[command(name = "burp-mcp", version, about = "Native MCP server for Burp Suite")]
pub struct Cli {
    /// TOML configuration file. Defaults to ~/.config/burp-mcp/config.toml when present.
    #[arg(long, global = true, env = "BURP_MCP_CONFIG")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run the MCP server over standard input and output.
    Serve(ServeArgs),
    /// Verify the Kotlin RPC adapter and transport limits.
    Probe(ProbeArgs),
    #[command(name = "__sitegraph-daemon", hide = true)]
    SitegraphDaemon(SitegraphDaemonArgs),
}

#[derive(Debug, Clone, Args)]
pub struct SitegraphDaemonArgs {
    #[arg(long)]
    pub graph_path: PathBuf,
    #[arg(long)]
    pub graph_id: String,
    #[arg(long)]
    pub endpoint_file: PathBuf,
    #[arg(long)]
    pub rules_path: PathBuf,
}

#[derive(Debug, Clone, Args)]
pub struct ServeArgs {
    /// Optional endpoint file for an already-running shared sitegraph daemon.
    #[arg(long, env = "BURP_MCP_SITEGRAPH_DAEMON")]
    pub sitegraph_daemon: Option<PathBuf>,

    /// Burp RPC endpoint. Plaintext is limited to IPv4 loopback; remote endpoints require HTTPS and mTLS.
    #[arg(long, env = "BURP_MCP_GRPC_ENDPOINT")]
    pub endpoint: Option<String>,

    /// Burp RPC port, used when --endpoint is not set.
    #[arg(long, env = "BURP_MCP_GRPC_PORT", value_parser = parse_port)]
    pub port: Option<u16>,

    /// Directory containing ca.crt, client.crt, and client.key for mTLS.
    #[arg(long, env = "BURP_MCP_TLS_DIR")]
    pub tls_dir: Option<PathBuf>,

    /// SQLite sitegraph path. Defaults to the platform data directory when sitegraph is enabled.
    #[arg(long, env = "BURP_MCP_GRAPH_PATH")]
    pub graph_path: Option<PathBuf>,

    /// Sitegraph enrichment rules JSON. Initialized from embedded defaults when absent.
    #[arg(long, env = "BURP_MCP_SITEGRAPH_RULES")]
    pub sitegraph_rules_path: Option<PathBuf>,

    /// Enable the advanced sitegraph tools and local SQLite graph.
    #[arg(long, env = "BURP_MCP_ENABLE_SITEGRAPH", num_args = 0..=1, default_missing_value = "true")]
    pub enable_sitegraph: Option<bool>,

    /// Sitegraph indexing mode. Auto-index is opt-in.
    #[arg(long, env = "BURP_MCP_SITEGRAPH_MODE", value_parser = parse_sitegraph_mode)]
    pub sitegraph_mode: Option<String>,

    /// Poll interval for watch mode.
    #[arg(long, env = "BURP_MCP_SITEGRAPH_INTERVAL_SECONDS")]
    pub sitegraph_interval_seconds: Option<u64>,

    /// Serve MCP over standard input and output.
    #[arg(long, default_value_t = true)]
    pub stdio: bool,
}

impl Default for ServeArgs {
    fn default() -> Self {
        Self {
            sitegraph_daemon: None,
            endpoint: None,
            port: None,
            tls_dir: None,
            graph_path: None,
            sitegraph_rules_path: None,
            enable_sitegraph: None,
            sitegraph_mode: None,
            sitegraph_interval_seconds: None,
            stdio: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ServeConfig {
    pub sitegraph_daemon: Option<PathBuf>,
    pub endpoint: String,
    pub tls_dir: Option<PathBuf>,
    pub graph_path: PathBuf,
    pub enable_sitegraph: bool,
    pub rules_path: PathBuf,
    pub sitegraph_mode: String,
    pub sitegraph_interval_seconds: u64,
    pub stdio: bool,
}

impl ServeArgs {
    pub fn resolve(self, file: &Config) -> Result<ServeConfig, String> {
        let tls_dir = self.tls_dir.or_else(|| file.burp.tls_dir.clone());
        let endpoint = resolve_endpoint(
            self.endpoint.as_deref().or(file.burp.endpoint.as_deref()),
            self.port.or(file.burp.port),
            file.burp.tls || tls_dir.is_some(),
        )?;
        let tls_dir = resolve_tls_dir(&endpoint, tls_dir.as_deref());
        let sitegraph_mode = self
            .sitegraph_mode
            .unwrap_or_else(|| file.sitegraph.mode.clone());
        parse_sitegraph_mode(&sitegraph_mode)?;
        let sitegraph_interval_seconds = self
            .sitegraph_interval_seconds
            .unwrap_or(file.sitegraph.interval_seconds);
        if sitegraph_interval_seconds == 0 {
            return Err("sitegraph interval must be positive".to_owned());
        }
        Ok(ServeConfig {
            sitegraph_daemon: self
                .sitegraph_daemon
                .or_else(|| file.sitegraph.daemon.clone()),
            endpoint,
            tls_dir,
            graph_path: self
                .graph_path
                .or_else(|| file.sitegraph.graph_path.clone())
                .unwrap_or_else(default_graph_path),
            enable_sitegraph: self.enable_sitegraph.unwrap_or(file.sitegraph.enabled),
            sitegraph_mode,
            rules_path: self
                .sitegraph_rules_path
                .or_else(|| file.sitegraph.rules_path.clone())
                .unwrap_or_else(crate::config::default_rules_path),
            sitegraph_interval_seconds,
            stdio: self.stdio,
        })
    }
}

#[derive(Debug, Clone, Args, Default)]
pub struct ProbeArgs {
    /// Burp RPC endpoint. Plaintext is limited to IPv4 loopback; remote endpoints require HTTPS and mTLS.
    #[arg(long, env = "BURP_MCP_GRPC_ENDPOINT")]
    pub endpoint: Option<String>,

    /// Burp RPC port, used when --endpoint is not set.
    #[arg(long, env = "BURP_MCP_GRPC_PORT", value_parser = parse_port)]
    pub port: Option<u16>,

    /// Directory containing ca.crt, client.crt, and client.key for mTLS.
    #[arg(long, env = "BURP_MCP_TLS_DIR")]
    pub tls_dir: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ProbeConfig {
    pub endpoint: String,
    pub tls_dir: Option<PathBuf>,
}

impl ProbeArgs {
    pub fn resolve(self, file: &Config) -> Result<ProbeConfig, String> {
        let tls_dir = self.tls_dir.or_else(|| file.burp.tls_dir.clone());
        let endpoint = resolve_endpoint(
            self.endpoint.as_deref().or(file.burp.endpoint.as_deref()),
            self.port.or(file.burp.port),
            file.burp.tls || tls_dir.is_some(),
        )?;
        Ok(ProbeConfig {
            tls_dir: resolve_tls_dir(&endpoint, tls_dir.as_deref()),
            endpoint,
        })
    }
}

pub fn resolve_config_path(explicit: Option<PathBuf>) -> Option<PathBuf> {
    explicit.or_else(|| {
        let path = crate::config::default_path();
        path.is_file().then_some(path)
    })
}

fn resolve_endpoint(
    endpoint: Option<&str>,
    port: Option<u16>,
    tls: bool,
) -> Result<String, String> {
    let endpoint = if let Some(endpoint) = endpoint {
        if tls && endpoint.starts_with("http://") {
            format!("https://{}", &endpoint["http://".len()..])
        } else {
            endpoint.to_owned()
        }
    } else {
        match (tls, port) {
            (true, Some(port)) => format!("https://127.0.0.1:{port}"),
            (true, None) => DEFAULT_ENDPOINT.replacen("http://", "https://", 1),
            (false, Some(port)) => format!("http://127.0.0.1:{port}"),
            (false, None) => DEFAULT_ENDPOINT.to_owned(),
        }
    };
    parse_endpoint(&endpoint)
}

fn default_graph_path() -> PathBuf {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("burp-mcp/graphs/default.sqlite")
}

fn resolve_tls_dir(endpoint: &str, explicit: Option<&Path>) -> Option<PathBuf> {
    if !endpoint.starts_with("https://") {
        return None;
    }
    Some(
        explicit
            .map(Path::to_path_buf)
            .unwrap_or_else(default_tls_dir),
    )
}

fn default_tls_dir() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("burp-mcp/tls")
}

fn parse_endpoint(endpoint: &str) -> Result<String, String> {
    let uri: http::Uri = endpoint
        .parse()
        .map_err(|_| "Burp RPC endpoint must be a valid URI".to_owned())?;
    let scheme = uri
        .scheme_str()
        .ok_or_else(|| "Burp RPC endpoint requires http or https".to_owned())?;
    let host = uri
        .host()
        .ok_or_else(|| "Burp RPC endpoint requires a host".to_owned())?;
    let port = uri
        .port_u16()
        .ok_or_else(|| "Burp RPC endpoint requires an explicit port".to_owned())?;
    if scheme == "http" && host == "127.0.0.1" && port > 0 {
        return Ok(endpoint.to_owned());
    }
    if scheme == "https" && port > 0 {
        return Ok(endpoint.to_owned());
    }
    Err("Burp RPC endpoint must be http://127.0.0.1:<port> or https://<host>:<port>".to_owned())
}

fn parse_port(port: &str) -> Result<u16, String> {
    let port = port
        .parse::<u16>()
        .map_err(|_| "Burp RPC endpoint port must be an integer from 1 to 65535".to_owned())?;
    if port == 0 {
        return Err("Burp RPC endpoint port must be positive".to_owned());
    }
    Ok(port)
}

fn parse_sitegraph_mode(value: &str) -> Result<String, String> {
    match value {
        "off" | "startup" | "watch" => Ok(value.to_owned()),
        _ => Err("sitegraph mode must be off, startup, or watch".to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::{Cli, Command, DEFAULT_ENDPOINT};
    use crate::config::Config;
    use clap::Parser;
    use std::path::{Path, PathBuf};

    #[test]
    fn defaults_to_serve_configuration() {
        let cli = Cli::try_parse_from(["burp-mcp"]).expect("default CLI must parse");
        assert!(cli.command.is_none());
    }

    #[test]
    fn parses_probe_endpoint() {
        let cli =
            Cli::try_parse_from(["burp-mcp", "probe", "--endpoint", "http://127.0.0.1:10077"])
                .expect("probe CLI must parse");
        let Some(Command::Probe(args)) = cli.command else {
            panic!("expected probe command")
        };
        assert_eq!(
            "http://127.0.0.1:10077",
            args.resolve(&Config::default()).unwrap().endpoint
        );
    }

    #[test]
    fn rejects_remote_plaintext_endpoint_during_resolution() {
        let cli = Cli::try_parse_from(["burp-mcp", "serve", "--endpoint", "http://10.10.0.8:9877"])
            .expect("raw endpoint must parse before TLS settings are merged");
        let Some(Command::Serve(args)) = cli.command else {
            panic!("expected serve command")
        };
        let error = args.resolve(&Config::default()).unwrap_err();
        assert!(error.contains("https"));
    }

    #[test]
    fn tls_directory_upgrades_explicit_http_endpoint_to_https() {
        let cli = Cli::try_parse_from([
            "burp-mcp",
            "probe",
            "--endpoint",
            "http://127.0.0.1:9877",
            "--tls-dir",
            "/tmp/burp-mcp-tls",
        ])
        .expect("TLS probe must parse");
        let Some(Command::Probe(args)) = cli.command else {
            panic!("expected probe command")
        };
        let resolved = args.resolve(&Config::default()).unwrap();
        assert_eq!("https://127.0.0.1:9877", resolved.endpoint);
        assert_eq!(Some(PathBuf::from("/tmp/burp-mcp-tls")), resolved.tls_dir);
    }

    #[test]
    fn tls_directory_upgrades_port_endpoint_to_https() {
        let cli = Cli::try_parse_from([
            "burp-mcp",
            "probe",
            "--port",
            "10077",
            "--tls-dir",
            "/tmp/burp-mcp-tls",
        ])
        .expect("TLS port must parse");
        let Some(Command::Probe(args)) = cli.command else {
            panic!("expected probe command")
        };
        assert_eq!(
            "https://127.0.0.1:10077",
            args.resolve(&Config::default()).unwrap().endpoint
        );
    }

    #[test]
    fn file_tls_setting_upgrades_endpoint_and_uses_default_bundle() {
        let mut file = Config::default();
        file.burp.endpoint = Some("http://burp-vm.test:9877".to_owned());
        file.burp.tls = true;

        let cli =
            Cli::try_parse_from(["burp-mcp", "probe"]).expect("probe configuration must parse");
        let Some(Command::Probe(args)) = cli.command else {
            panic!("expected probe command")
        };
        let resolved = args.resolve(&file).unwrap();
        assert_eq!("https://burp-vm.test:9877", resolved.endpoint);
        assert!(resolved.tls_dir.is_some());
    }

    #[test]
    fn file_config_enables_sitegraph_and_cli_overrides_mode() {
        let mut file = Config::default();
        file.burp.port = Some(10077);
        file.sitegraph.enabled = true;
        file.sitegraph.graph_path = Some(PathBuf::from("/tmp/burp-mcp-graph.sqlite"));
        file.sitegraph.mode = "watch".to_owned();
        file.sitegraph.interval_seconds = 45;
        let cli = Cli::try_parse_from(["burp-mcp", "serve", "--sitegraph-mode", "startup"])
            .expect("serve configuration must parse");
        let Some(Command::Serve(args)) = cli.command else {
            panic!("expected serve command")
        };
        let resolved = args.resolve(&file).unwrap();
        assert_eq!("http://127.0.0.1:10077", resolved.endpoint);
        assert!(resolved.enable_sitegraph);
        assert_eq!("startup", resolved.sitegraph_mode);
        assert_eq!(45, resolved.sitegraph_interval_seconds);
        assert_eq!(Path::new("/tmp/burp-mcp-graph.sqlite"), resolved.graph_path);
    }

    #[test]
    fn sitegraph_mode_does_not_implicitly_enable_sitegraph() {
        let cli = Cli::try_parse_from(["burp-mcp", "serve", "--sitegraph-mode", "watch"])
            .expect("sitegraph mode must parse");
        let Some(Command::Serve(args)) = cli.command else {
            panic!("expected serve command")
        };
        let resolved = args.resolve(&Config::default()).unwrap();
        assert!(!resolved.enable_sitegraph);
        assert_eq!("watch", resolved.sitegraph_mode);
    }

    #[test]
    fn default_endpoint_is_stable() {
        assert_eq!("http://127.0.0.1:9877", DEFAULT_ENDPOINT);
        assert_eq!(
            DEFAULT_ENDPOINT,
            super::resolve_endpoint(None, None, false).unwrap()
        );
    }
}
