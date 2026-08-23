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
}

#[derive(Debug, Clone, Args)]
pub struct ServeArgs {
    /// Burp RPC endpoint. Only IPv4 loopback endpoints are accepted.
    #[arg(long, env = "BURP_MCP_GRPC_ENDPOINT", value_parser = parse_endpoint)]
    pub endpoint: Option<String>,

    /// Burp RPC port, used when --endpoint is not set.
    #[arg(long, env = "BURP_MCP_GRPC_PORT", value_parser = parse_port)]
    pub port: Option<u16>,
    /// SQLite sitegraph path. Defaults to the platform data directory.
    #[arg(long, env = "BURP_MCP_GRAPH_PATH")]
    pub graph_path: Option<String>,

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
            stdio: true,
            graph_path: None,
            sitegraph_mode: "off".to_owned(),
            sitegraph_interval_seconds: 30,
        }
    }
}

impl ServeArgs {
    pub fn resolved_endpoint(&self) -> Result<String, String> {
        resolve_endpoint(self.endpoint.as_deref(), self.port)
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
    /// Burp RPC endpoint. Only IPv4 loopback endpoints are accepted.
    #[arg(long, env = "BURP_MCP_GRPC_ENDPOINT", value_parser = parse_endpoint)]
    pub endpoint: Option<String>,

    /// Burp RPC port, used when --endpoint is not set.
    #[arg(long, env = "BURP_MCP_GRPC_PORT", value_parser = parse_port)]
    pub port: Option<u16>,
}

impl ProbeArgs {
    pub fn resolved_endpoint(&self) -> Result<String, String> {
        resolve_endpoint(self.endpoint.as_deref(), self.port)
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

fn parse_endpoint(endpoint: &str) -> Result<String, String> {
    let Some(port) = endpoint.strip_prefix("http://127.0.0.1:") else {
        return Err("Burp RPC endpoint must be http://127.0.0.1:<port>".to_owned());
    };
    parse_port(port)?;
    Ok(endpoint.to_owned())
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
    fn rejects_non_loopback_endpoint() {
        let error = Cli::try_parse_from(["burp-mcp", "serve", "--endpoint", "http://0.0.0.0:9877"])
            .expect_err("non-loopback endpoint must fail");
        assert!(error.to_string().contains("127.0.0.1"));
    }

    #[test]
    fn default_endpoint_is_stable() {
        assert_eq!("http://127.0.0.1:9877", DEFAULT_ENDPOINT);
    }
}
