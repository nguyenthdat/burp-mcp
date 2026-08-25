use clap::{Args, Parser, Subcommand};

pub const DEFAULT_ENDPOINT: &str = "http://127.0.0.1:9877";

#[derive(Debug, Parser)]
#[command(name = "burp-mcp", version, about = "Native MCP server for Burp Suite")]
pub struct Cli {
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
    pub graph_path: std::path::PathBuf,
    #[arg(long)]
    pub graph_id: String,
    #[arg(long)]
    pub endpoint_file: std::path::PathBuf,
}

#[derive(Debug, Clone, Args)]
pub struct ServeArgs {
    /// Burp RPC endpoint. Plaintext is limited to IPv4 loopback; remote endpoints require HTTPS and mTLS.
    /// Optional endpoint file for an already-running shared sitegraph daemon.
    #[arg(long, env = "BURP_MCP_SITEGRAPH_DAEMON")]
    pub sitegraph_daemon: Option<std::path::PathBuf>,
    #[arg(long, env = "BURP_MCP_GRPC_ENDPOINT", value_parser = parse_endpoint)]
    pub endpoint: Option<String>,

    /// Burp RPC port, used when --endpoint is not set.
    #[arg(long, env = "BURP_MCP_GRPC_PORT", value_parser = parse_port)]
    pub port: Option<u16>,

    /// Directory containing ca.crt, client.crt, and client.key for remote mTLS.
    #[arg(long, env = "BURP_MCP_TLS_DIR")]
    pub tls_dir: Option<String>,
    /// SQLite sitegraph path. Defaults to the platform data directory when sitegraph is enabled.
    #[arg(long, env = "BURP_MCP_GRAPH_PATH")]
    pub graph_path: Option<String>,

    /// Enable the advanced sitegraph tools and local SQLite graph.
    ///
    /// Disabled by default; set this explicitly for a manual opt-in.
    #[arg(long, env = "BURP_MCP_ENABLE_SITEGRAPH", default_value_t = false)]
    pub enable_sitegraph: bool,

    /// Sitegraph indexing mode. Auto-index is opt-in.
    #[arg(long, env = "BURP_MCP_SITEGRAPH_MODE", default_value = "off", value_parser = parse_sitegraph_mode)]
    pub sitegraph_mode: String,

    /// Poll interval for watch mode.
    #[arg(
        long,
        env = "BURP_MCP_SITEGRAPH_INTERVAL_SECONDS",
        default_value_t = 30
    )]
    pub sitegraph_interval_seconds: u64,

    /// Serve MCP over standard input and output.
    #[arg(long, default_value_t = true)]
    pub stdio: bool,
}

impl Default for ServeArgs {
    fn default() -> Self {
        Self {
            endpoint: None,
            port: None,
            tls_dir: None,
            sitegraph_daemon: None,
            graph_path: None,
            enable_sitegraph: false,
            sitegraph_mode: "off".to_owned(),
            sitegraph_interval_seconds: 30,
            stdio: true,
        }
    }
}

impl ServeArgs {
    pub fn resolved_endpoint(&self) -> Result<String, String> {
        resolve_endpoint(self.endpoint.as_deref(), self.port)
    }

    pub fn resolved_tls_dir(&self) -> Option<std::path::PathBuf> {
        resolve_tls_dir(&self.resolved_endpoint().ok()?, self.tls_dir.as_deref())
    }

    pub fn resolved_graph_path(&self) -> std::path::PathBuf {
        self.graph_path
            .as_deref()
            .map(std::path::PathBuf::from)
            .unwrap_or_else(default_graph_path)
    }
}

#[derive(Debug, Clone, Args)]
pub struct ProbeArgs {
    /// Burp RPC endpoint. Plaintext is limited to IPv4 loopback; remote endpoints require HTTPS and mTLS.
    #[arg(long, env = "BURP_MCP_GRPC_ENDPOINT", value_parser = parse_endpoint)]
    pub endpoint: Option<String>,

    /// Burp RPC port, used when --endpoint is not set.
    #[arg(long, env = "BURP_MCP_GRPC_PORT", value_parser = parse_port)]
    pub port: Option<u16>,

    /// Directory containing ca.crt, client.crt, and client.key for remote mTLS.
    #[arg(long, env = "BURP_MCP_TLS_DIR")]
    pub tls_dir: Option<String>,
}

impl ProbeArgs {
    pub fn resolved_endpoint(&self) -> Result<String, String> {
        resolve_endpoint(self.endpoint.as_deref(), self.port)
    }

    pub fn resolved_tls_dir(&self) -> Option<std::path::PathBuf> {
        resolve_tls_dir(&self.resolved_endpoint().ok()?, self.tls_dir.as_deref())
    }
}

fn resolve_endpoint(endpoint: Option<&str>, port: Option<u16>) -> Result<String, String> {
    if let Some(endpoint) = endpoint {
        return Ok(endpoint.to_owned());
    }
    if let Some(port) = port {
        return Ok(format!("http://127.0.0.1:{port}"));
    }
    Ok(DEFAULT_ENDPOINT.to_owned())
}

fn default_graph_path() -> std::path::PathBuf {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| std::path::PathBuf::from(home).join(".local/share"))
        })
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    base.join("burp-mcp/graphs/default.sqlite")
}

fn resolve_tls_dir(endpoint: &str, explicit: Option<&str>) -> Option<std::path::PathBuf> {
    if !endpoint.starts_with("https://") {
        return None;
    }
    Some(
        explicit
            .map(std::path::PathBuf::from)
            .unwrap_or_else(default_tls_dir),
    )
}

fn default_tls_dir() -> std::path::PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| std::path::PathBuf::from(home).join(".config"))
        })
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("burp-mcp")
        .join("tls")
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
    use clap::Parser;

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
            panic!("expected probe command");
        };
        assert_eq!("http://127.0.0.1:10077", args.resolved_endpoint().unwrap());
    }

    #[test]
    fn parses_probe_port() {
        let cli = Cli::try_parse_from(["burp-mcp", "probe", "--port", "10077"])
            .expect("probe port must parse");
        let Some(Command::Probe(args)) = cli.command else {
            panic!("expected probe command");
        };
        assert_eq!("http://127.0.0.1:10077", args.resolved_endpoint().unwrap());
    }

    #[test]
    fn rejects_remote_plaintext_endpoint() {
        let error =
            Cli::try_parse_from(["burp-mcp", "serve", "--endpoint", "http://10.10.0.8:9877"])
                .expect_err("remote plaintext endpoint must fail");
        assert!(error.to_string().contains("https"));
    }

    #[test]
    fn accepts_remote_https_and_explicit_tls_directory() {
        let cli = Cli::try_parse_from([
            "burp-mcp",
            "probe",
            "--endpoint",
            "https://burp-vm.test:9877",
            "--tls-dir",
            "/tmp/burp-mcp-tls",
        ])
        .expect("remote mTLS CLI must parse");
        let Some(Command::Probe(args)) = cli.command else {
            panic!("expected probe command")
        };
        assert_eq!(
            "https://burp-vm.test:9877",
            args.resolved_endpoint().unwrap()
        );
        assert_eq!(
            std::path::PathBuf::from("/tmp/burp-mcp-tls"),
            args.resolved_tls_dir().unwrap()
        );
    }

    #[test]
    fn default_endpoint_is_stable() {
        assert_eq!("http://127.0.0.1:9877", DEFAULT_ENDPOINT);
    }

    #[test]
    fn sitegraph_is_disabled_by_default_and_requires_explicit_opt_in() {
        let cli = Cli::try_parse_from(["burp-mcp", "serve"]).expect("serve CLI must parse");
        let Some(Command::Serve(args)) = cli.command else {
            panic!("expected serve command");
        };
        assert!(!args.enable_sitegraph);

        let cli = Cli::try_parse_from(["burp-mcp", "serve", "--enable-sitegraph"])
            .expect("sitegraph opt-in must parse");
        let Some(Command::Serve(args)) = cli.command else {
            panic!("expected serve command");
        };
        assert!(args.enable_sitegraph);
    }

    #[test]
    fn sitegraph_mode_without_enable_flag_does_not_enable_sitegraph() {
        let cli = Cli::try_parse_from(["burp-mcp", "serve", "--sitegraph-mode", "watch"])
            .expect("sitegraph mode must parse");
        let Some(Command::Serve(args)) = cli.command else {
            panic!("expected serve command");
        };
        assert!(!args.enable_sitegraph);
        assert_eq!("watch", args.sitegraph_mode);
    }

    #[test]
    fn sitegraph_opt_in_accepts_manual_path_and_mode() {
        let cli = Cli::try_parse_from([
            "burp-mcp",
            "serve",
            "--enable-sitegraph",
            "--graph-path",
            "/tmp/burp-mcp-graph.sqlite",
            "--sitegraph-mode",
            "startup",
        ])
        .expect("manual sitegraph configuration must parse");
        let Some(Command::Serve(args)) = cli.command else {
            panic!("expected serve command");
        };
        assert!(args.enable_sitegraph);
        assert_eq!("startup", args.sitegraph_mode);
        assert_eq!(
            std::path::Path::new("/tmp/burp-mcp-graph.sqlite"),
            args.resolved_graph_path()
        );
    }
}
