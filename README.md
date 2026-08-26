# Burp MCP

[![CI](https://github.com/nguyenthdat/burp-mcp/actions/workflows/ci.yml/badge.svg)](https://github.com/nguyenthdat/burp-mcp/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Burp MCP connects MCP-compatible clients to Burp Suite through a native Rust stdio server and a Kotlin extension built on the Montoya API. The Kotlin/Rust boundary is typed protobuf over loopback gRPC; pure transformations and persistent site metadata stay in Rust.

## Capabilities

- Inspect, search, annotate, highlight, compare, and clear Proxy history.
- Send and modify HTTP requests through Burp Repeater.
- Run bounded background operations and inspect their status and paginated results.
- Start bounded crawls and audits, inspect issues, and work with Burp Collaborator.
- Query the site map, target information, scope, and cookie jar.
- Create and interact with managed WebSocket connections.
- Convert, export, and analyze text, JSON, binary data, and raw HTTP messages.
- Import Bambdas and BChecks without executing them automatically.
- Apply HTTP handlers, proxy rules, and session rules.
- Read and mutate Proxy listeners, script filters, and request/response interception rules through one operation-based configuration tool.
- Optionally persist a privacy-preserving SQLite site graph containing endpoint metadata and parameter names, never parameter values or message bodies.
- Run deterministic, binary-safe utility recipes without network, filesystem, browser, or arbitrary-code capabilities.

Some capabilities require Burp Suite Professional or a Burp feature that is available only in specific editions.

Rust serves MCP over stdio and owns the bounded reconnecting gRPC actor, typed protocol client, local utility engine, and persistent sitegraph facade. Kotlin owns Montoya state and the gRPC adapter. The default remains zero-configuration plaintext on `127.0.0.1:9877`.

The extension registers **Settings > Extensions > Burp MCP**. From that panel you can change the bind address and port, select local plaintext or remote mutual TLS, rotate certificates, and restart the gRPC server without reloading Burp.

Remote mode creates this portable bundle by default:

```text
~/.config/burp-mcp/tls/
├── ca.crt
├── server.crt
├── server.key
├── client.crt
├── client.key
└── bundle.conf
```

Keep `server.key` on the Burp machine. Copy only `ca.crt`, `client.crt`, and `client.key` to the same directory on a remote agent machine. On Unix, set `chmod 700 ~/.config/burp-mcp/tls && chmod 600 ~/.config/burp-mcp/tls/client.key`; the Rust client rejects a client key with any group/other permission bits. The Rust client discovers that directory automatically for HTTPS endpoints.
Burp/JDK 25 interactive unload/reload gate is recorded in
`docs/phase0-burp-jdk25-verification.md`.

## Architecture

```text
MCP client
    │ MCP JSON-RPC / stdio
    ▼
burp-mcp native Rust binary
    │ typed gRPC / HTTP/2 on 127.0.0.1
    ▼
Kotlin Burp extension ──► Montoya API ──► Burp Suite
```

JavaScript, Bun, npm, the legacy HTTP/JSON transport, and the external utility runtime are not part of the v3 production path.

## Requirements

- Burp Suite with support for Montoya API extensions.
- Java 25 for building the extension.
- Rust 1.88 or newer for the native MCP server.

Utility inputs are bounded to 16 MiB, recipes to 64 steps, and batches to 100 items. Binary values use tagged base64 at the MCP boundary and remain raw bytes internally.

## Install

See the complete [installation guide](docs/install.md) for macOS/Linux curl installation, checksum verification, Burp extension loading, MCP client configuration, optional skill installation, manual installation, and uninstall steps.

### Quick install on macOS or Linux

Review [`install.sh`](install.sh), then install the verified native binary with:

```sh
curl -fsSL https://raw.githubusercontent.com/nguyenthdat/burp-mcp/main/install.sh | bash
```

Install the native binary plus the repository's Burp skill:

```sh
curl -fsSL https://raw.githubusercontent.com/nguyenthdat/burp-mcp/main/install.sh \
  | bash -s -- --with-skill --agent codex
```

The installer detects Linux x86_64 and macOS arm64/x86_64, verifies the release
asset against `SHA256SUMS`, installs to `~/.local/bin`, and never uses `sudo`.
Windows users should follow the manual release-asset steps in the guide.

### Load the Burp extension

Download `burp-mcp.jar` from the [latest GitHub Release](https://github.com/nguyenthdat/burp-mcp/releases/latest), then in Burp Suite open **Extensions > Installed > Add**, select **Java**, and choose the JAR.

### Configure an MCP client

Run the installed native server:

```json
{
  "command": "/absolute/path/to/burp-mcp",
  "args": ["serve"]
}
```

The server connects to the Kotlin extension at `http://127.0.0.1:9877` by default. Verify the extension before starting the MCP client:

```sh
burp-mcp probe --endpoint http://127.0.0.1:9877
```

The optional `burp-skill` is documented in [docs/burp-skill](docs/burp-skill/SKILL.md). Sitegraph is disabled by default; use the separate [sitegraph reference](docs/sitegraph.md) before enabling it.

## Configuration

The Rust client reads TOML from `~/.config/burp-mcp/config.toml` when that file exists. Use `--config PATH` or `BURP_MCP_CONFIG` to select another file. CLI flags and environment variables override file values. A complete starting point is [`config.example.toml`](config.example.toml).

```toml
[burp]
endpoint = "http://127.0.0.1:9877"
# tls = true
# tls_dir = "/absolute/path/to/burp-mcp/tls"

[sitegraph]
enabled = false
mode = "off"
interval_seconds = 30
```

| Setting | Default | Description |
| --- | --- | --- |
| `BURP_MCP_CONFIG` / `--config` | auto-discovered | TOML configuration file. |
| `BURP_MCP_GRPC_PORT` | `9877` | Rust loopback port when an explicit endpoint is not set. |
| `BURP_MCP_GRPC_ENDPOINT` | `http://127.0.0.1:9877` | Rust endpoint. Remote endpoints must use HTTPS. |
| `BURP_MCP_TLS_DIR` | `~/.config/burp-mcp/tls` | Rust mTLS directory for HTTPS endpoints. |
| `[burp].tls` | `false` | Switches the resolved endpoint scheme to `https`; mTLS files are then loaded from `tls_dir` or the default directory. |
| `[sitegraph].enabled` | `false` | Exposes the advanced sitegraph tools and initializes the local graph. |
| `[sitegraph].project_root` | `~/.local/share/burp-mcp/sitegraph` | Root directory for project-scoped databases. Each Burp project is stored separately as `<graph_id>.sqlite`; temporary projects use `temp-<graph_id>.sqlite`. |
| `[sitegraph].mode` | `off` | Auto-index mode: `off`, `startup`, or `watch`. |
| `[sitegraph].interval_seconds` | `30` | Poll interval for sitegraph `watch` mode. |
| `[sitegraph].daemon` | auto-spawn per project database | Optional endpoint file for one already-running project daemon; normally leave unset. |
| `[sitegraph].rules_path` | `~/.config/burp-mcp/default-rules.json` | Sitegraph enrichment rules. Burp MCP initializes the embedded defaults when this file is absent and never overwrites an existing customized file. |

When TLS is enabled by `burp.tls = true` or by setting `tls_dir`, an `http://` endpoint is normalized to the equivalent `https://` endpoint before the gRPC client is created. This keeps the endpoint displayed/configured by the operator consistent with the transport security actually used.

Sitegraph remains opt-in: a project root or indexing mode alone does not expose the tools; set `[sitegraph].enabled = true` or pass `--enable-sitegraph`.

Sitegraph is project-scoped. The extension persists a stable random `graph_id` in each Burp project, and Rust stores that project under `project_root/<graph_id>.sqlite`. `project_root` selects only the parent directory; it is not a shared database path. No project identity means no graph is opened, preventing unrelated projects from falling back to one shared file.

`sitegraph_config` is read-only. It reports the active mode and interval; change them in `config.toml` (or with startup overrides) and restart `burp-mcp`.

Plaintext is accepted only on IPv4 loopback. Any non-loopback endpoint must use HTTPS with mutual TLS. Enter every DNS name or IP address clients use in the panel before generating certificates; endpoint hostname verification uses those certificate SANs. Rotating certificates invalidates previously copied client bundles.

## Security

Burp MCP is dual-use security software intended only for systems you own or are explicitly authorized to test.

- The extension listens on IPv4 loopback without TLS by default.
- Remote binding is available only with generated mutual TLS and required client certificates.
- Treat `client.key`, `server.key`, and copied TLS directories as credentials; private files are written owner-only on POSIX systems.
- High-impact tools retain the capabilities and side effects of the underlying Burp APIs.

See [CONTRIBUTING.md](CONTRIBUTING.md) for development and pull request requirements. Report suspected vulnerabilities privately using [SECURITY.md](SECURITY.md); do not disclose them in public issues.


## Build and test

Run the native and extension checks:

```sh
gradle clean test jar
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --locked
scripts/run-grpc-interop.sh
```

The extension JAR is written to `build/libs/burp-mcp.jar`. After loading it in
Burp, run the live Phase 0 probe:

```sh
cargo run -p burp-mcp --locked -- probe --endpoint http://127.0.0.1:9877
```

The probe verifies server information and byte-exact 0-byte, 1-byte, and 10 MiB
round trips. To complete the lifecycle gate, unload the extension, confirm the
probe fails, reload the same JAR, and confirm the probe passes again.

## Releases

Published GitHub Releases contain the native MCP binary, the extension JAR, checksums, and an SBOM generated from the locked Rust dependency graph.

## License

Burp MCP is available under the [MIT License](LICENSE).
