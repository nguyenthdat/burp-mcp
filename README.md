# Burp MCP

[![CI](https://github.com/nguyenthdat/burp-mcp/actions/workflows/ci.yml/badge.svg)](https://github.com/nguyenthdat/burp-mcp/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tools](https://img.shields.io/badge/Tools-43%20Default%20%2B%201%20SiteGraph-brightgreen.svg)](docs/features.md)

Burp MCP connects MCP-compatible clients to Burp Suite through a native Rust
stdio server and a Kotlin extension built on the Montoya API. The Kotlin/Rust
boundary is typed protobuf over loopback gRPC; pure transformations and
persistent site data stay in Rust. Features that depend on Burp Scanner,
Collaborator, or other edition-specific APIs remain capability-gated.

---

## Architecture

Burp MCP has two runtime tiers:

1. **Native Rust MCP server (`burp-mcp`)**: serves MCP over stdio and owns the
   bounded reconnecting gRPC client, offline utility engine, and optional
   project-scoped SQLite sitegraph.
2. **Kotlin Burp extension (`burp-mcp.jar`)**: runs inside Burp Suite, uses the
   Montoya API, and exposes typed gRPC on loopback by default or remote mTLS.

```text
┌─────────────────────────────────────────────────────────────┐
│             MCP Client (Claude, Cursor, Zed, Codex, etc.)   │
└──────────────────────────────┬──────────────────────────────┘
                               │ MCP JSON-RPC (stdio)
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                 burp-mcp (Native Rust Server)               │
│  ├─ Offline Utility Decoder Engine (40+ transforms)        │
│  ├─ Optional SQLite Sitegraph Engine                        │
│  └─ Reconnecting Typed Protocol Client                      │
└──────────────────────────────┬──────────────────────────────┘
                               │ Typed Protobuf over gRPC
                               │ Loopback (127.0.0.1:9877) or Remote mTLS
                               ▼
┌─────────────────────────────────────────────────────────────┐
│            Burp MCP Extension (Kotlin / Montoya API)        │
│  ├─ Settings UI Panel (Port, Security, Certificate Rotation)│
│  ├─ Proxy, Repeater, Intruder, Scanner, WebSocket Facades   │
│  └─ Session Rules, Macros, Cookie Jar, Collaborator Engine  │
└──────────────────────────────┬──────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                    Burp Suite Desktop                       │
└─────────────────────────────────────────────────────────────┘
```

---

## Features & Capabilities

- **Proxy Traffic Inspection & Triage**: Search, filter (status, method, URL, regex), annotate, highlight, and view full raw HTTP/WebSocket traffic history with compact metadata by default (`include_bodies: false`) and smart field projection (`extract_css`, `extract_json`, `headers_only`, truncation).
- **Logger API Integration**: Complete traffic visibility across all Burp tools (`Proxy`, `Repeater`, `Scanner`, `Intruder`, `Extensions`) via `burp_logger_history`, `burp_logger_detail`, and `burp_clear_logger`.
- **Organizer Integration**: Send important request/response pairs directly into Burp Organizer and query/filter saved entries via `burp_organizer_send` and `burp_organizer_list`.
- **Active editor UI integration**: Capture and guardedly replace focused editable HTTP text editors with short-lived token/hash leases; edit WebSocket payloads through an MCP-provided extension tab with lossless Base64.
- **Interception & HTTP Handlers**: Toggle master proxy interception, register custom request/response modifying handlers, and configure granular proxy rules (`forward`, `intercept`, `drop`, `edit`).
- **True Single-Packet Attack (Last-Byte Sync)**: Synchronized race condition testing via `burp_race_condition` (`single_packet_attack: true`).
- **Multi-Marker Fuzzing**: Advanced matrix fuzzing supporting `pitchfork`, `cluster_bomb`, and `sniper` attack modes via `burp_inline_fuzzer`.
- **Collaborator Auto-Correlation Tracker**: Automatic mapping between injected parameter/URL origins and out-of-band DNS/HTTP interaction callbacks.
- **Response Comparer & Diffing**: Compute similarity scores, header diffs, and unified line diffs between HTTP responses with `burp_diff_responses` and `burp_send_to_comparer`.
- **Compound Security Workflows**: High-level automated workflows for IDOR verification (`burp_verify_idor`), CORS auditing (`burp_check_cors`), and Access Control Matrix testing (`burp_auth_matrix`).
- **Action-Based Pentesting Suite**: Streamlined ~15 action-based tools for modern AI agents to dramatically reduce context-window overhead and tool hallucinations.
- **Cookie Jar**: Inspect, filter by domain, and set cookies within Burp's active cookie jar.
- **Session Handling & Macros**: Create, list, execute, update, and remove scoped session handling rules and multi-request macros with parameter extraction.
- **In-Memory Payload Lists**: Create, import from file/JSON/text, update, paginate, and delete named payload lists for fuzzing and Intruder attacks.
- **Scanner & Crawl Automation**: Launch bounded passive audits, active scans, and crawls; poll background jobs; triage and inspect issues; update issue statuses (False Positive/Ignored); and test/dry-run BCheck scripts via `burp_test_bcheck`.
- **Sitegraph Engine (Advanced Opt-in)**: Project-scoped SQLite graph mapping endpoints, parameters, topology, shortest paths, clusters, downstream impact, diffs, and indexed HTTP/WebSocket evidence. Treat each graph as sensitive engagement data.
- **Offline Utility Decoder Engine**: 40+ built-in operations for encoding/decoding (Base64, Hex, URL, HTML, Unicode), cryptographic hashes (MD5, SHA-1/256/512, BLAKE3, HMAC), compression (Gzip, Zlib, Deflate, Brotli), JWT decoding/verification, and HTTP parsing.
Some capabilities require Burp Suite Professional or a Burp feature advertised
by the connected extension. The runtime tool schema and
`burp_burp_version.capabilities` are authoritative.

## Tools Inventory (43 Default + 1 SiteGraph)

Burp MCP registers **43 tools by default** (42 Burp tools + 1 offline Decoder tool), plus **1 SiteGraph tool** when SiteGraph is enabled with `--enable-sitegraph`.
### 1. Connection & Project Configuration (2 tools)

| Tool | Parameters | Description | Read-Only |
|---|---|---|:---:|
| `burp_burp_version` | `{}` | Return Burp Suite version, edition, extension version, capabilities, and runtime limits. | Yes |
| `burp_extension_info` | `{}` | Return extension and process metadata (JAR location, BApp status, Java arguments). | Yes |

### 2. Core Pentesting Suite (13 tools)

| Tool | Key Actions | Description | Read-Only |
|---|---|---|:---:|
| `burp_proxy` | `history`, `detail`, `annotate`, `highlight`, `extract`, `websocket_history` | Proxy HTTP/WebSocket history inspection & annotation. | No |
| `burp_http` | `send`, `send_batch`, `convert`, `export`, `send_to_repeater` | Send requests, batch testing, format export, and Repeater UI bridge. | No |
| `burp_target` | `get_scope`, `add_scope`, `remove_scope`, `info`, `sitemap` | Scope checking/mutation and Site Map exploration. | No |
| `burp_scanner` | `start_audit`, `start_crawl`, `stop`, `list_issues`, `issue_detail`, `update_issue`, `report`, `test_bcheck`, `remove` | Automated scanning, issue triage, and BCheck test runner. | No |
| `burp_scan_config` | `list_configs`, `get_config`, `upsert_config`, `delete_config`, `list_pools`, `get_pool`, `upsert_pool`, `delete_pool` | Full CRUD for scan configurations and resource pools. | No |
| `burp_fuzzer` | `fuzz`, `race`, `send_to_intruder`, `list_payloads`, `upsert_payloads`, `register_payload_processor`, `register_payload_generator` | Multi-marker fuzzing (`pitchfork`/`cluster_bomb`/`sniper`), single-packet race attack, and payload management. | No |
| `burp_collaborator` | `generate`, `poll`, `correlate` | Out-of-band OAST testing with origin correlation tracking. | No |
| `burp_websocket` | `create`, `send_text`, `send_binary`, `history`, `close`, `list` | Outbound managed WebSocket connections. | No |
| `burp_session` | `list_rules`, `get_rule`, `upsert_rule`, `delete_rule`, `run_macro`, `upsert_macro`, `list_macros`, `delete_macro` | Session handling rules and multi-request macros. | No |
| `burp_settings` | `get_proxy_settings`, `update_proxy_settings`, `export_config`, `inspect_config`, `import_config`, `intercept_state`, `set_intercept_state`, `proxy_intercept_config`, `update_proxy_intercept_config`, `register_http_handler`, `remove_http_handler`, `register_proxy_rule`, `list_proxy_rules`, `remove_proxy_rule` | Proxy listeners, intercept settings, handlers, and configuration. | No |
| `burp_logger` | `query`, `detail`, `clear` | Comprehensive traffic logger across all Burp tools. | No |
| `burp_organizer` | `add`, `list` | Burp Organizer item storage and triage. | No |
| `burp_diff` | `diff_responses`, `compare_exchanges` | HTTP response diffing, similarity scoring, and Comparer UI bridge. | Yes |

### 3. Compound Security Workflows (9 tools)

| Tool | Description | Read-Only |
|---|---|:---:|
| `burp_verify_idor` | Automated IDOR verification across two user authorization contexts (User A vs User B). | No |
| `burp_check_cors` | Automated CORS vulnerability auditing with origin reflection analysis. | No |
| `burp_auth_matrix` | Automated role-based access control matrix across multiple endpoints. | No |
| `burp_audit_jwt` | Automated JWT vulnerability audit (None algorithm, RS256 -> HS256 key confusion, and claim tampering). | No |
| `burp_verify_ssrf` | Automated SSRF verification with Collaborator interaction polling and payload correlation. | No |
| `burp_verify_sqli_blind` | Differential boolean-based and timing statistical blind SQL injection verification. | No |
| `burp_audit_graphql` | Automated GraphQL security audit (Introspection, Field Suggestions, and Query Batching). | No |
| `burp_verify_csrf_samesite` | Automated CSRF risk audit, SameSite cookie evaluation, and auto-generated HTML PoC form. | No |
| `burp_api_fuzz_orchestrator` | Automated specification-driven API fuzzing from OpenAPI 2.0 / 3.0 or Swagger documents. | No |
### 4. Active UI & Desktop Editor Integration (3 tools)

| Tool | Parameters | Description | Read-Only |
|---|---|---|:---:|
| `burp_editor_get` | `{target_hint?, ttl_seconds?}` | Capture active or last-focused editor tab with rich metadata, selection offsets, and UTF-8 decoded text. | Yes |
| `burp_editor_patch` | `{token, expected_sha256, mode?, text?, ...}` | Surgically modify active Burp editor contents (`replace_selection`, `set_header`, `json_patch`, `set_param`, `regex`, `replace_all`) with automatic Content-Length and CRLF calculation. | No |
| `burp_editor_renew_lease` | `{token, extend_seconds?}` | Extend the lifetime of an active Burp editor lease token. | No |
### 5. Cookies & Findings (3 tools)

| Tool | Parameters | Description | Read-Only |
|---|---|---|:---:|
| `burp_cookie_jar` | `{limit?, domain?}` | List cookies in Burp's cookie jar with domain, path, value, and expiration. | Yes |
| `burp_cookie_jar_set` | `{name, value, domain, path?, expiration?}` | Set or update a cookie in Burp's cookie jar. | No |
| `burp_add_issue` | `{name, url, detail?, remediation?, severity?, confidence?}` | Add a custom typed security issue to the Burp site map. | No |

### 6. Background Jobs (3 tools)

| Tool | Parameters | Description | Read-Only |
|---|---|---|:---:|
| `burp_job_status` | `{job_id}` | Check the progress, status, and summary of a background job. | Yes |
| `burp_job_result` | `{job_id, limit?, cursor?}` | Read paginated result items from a completed background job. | Yes |
| `burp_job_cancel` | `{job_id}` | Cancel an in-progress background job. | No |

### 7. Custom Script Imports (2 tools)

| Tool | Parameters | Description | Read-Only |
|---|---|---|:---:|
| `burp_bambda_import` | `{script}` | Validate and import a complete Bambda YAML script without executing it. | No |
| `burp_bcheck_import` | `{script, enabled?}` | Validate and import a complete BCheck script definition into Burp. | No |

### 8. MCP Interception Queues (6 tools)

| Tool | Parameters | Description | Read-Only |
|---|---|---|:---:|
| `burp_intercept_controller` | `{enabled?, timeout_seconds?}` | Read or configure the MCP-owned HTTP interception queue. Pending messages auto-forward on timeout. | No |
| `burp_intercepted_messages` | `{limit?, cursor?}` | Page pending HTTP requests and responses, including lossless base64 messages. | Yes |
| `burp_control_intercepted_message` | `{id, action, message_base64?}` | Forward, drop, or send one paused HTTP message to Burp's manual Intercept tab; optionally replace the full message. | No |
| `burp_websocket_intercept_controller` | `{enabled?, timeout_seconds?}` | Read or configure MCP-owned WebSocket interception. | No |
| `burp_intercepted_websocket_messages` | `{limit?, cursor?}` | Page pending intercepted WebSocket messages. | Yes |
| `burp_control_intercepted_websocket_message` | `{id, action, payload_base64?}` | Forward, drop, or send one paused WebSocket message to Burp's manual Intercept tab; optionally replace its payload. | No |

### 9. Offline Decoder Engine (1 tool)

| Tool | Parameters | Description | Read-Only |
|---|---|---|:---:|
| `decoder` | `{input, operation?, args?, steps?, query?, describe?, magic?}` | Execute deterministic transformations, multi-step recipes, search catalog, or get magic decode suggestions. | Yes |

### 10. Persistent Sitegraph (1 tool, Opt-in)

*Requires starting the server with `--enable-sitegraph` or `BURP_MCP_ENABLE_SITEGRAPH=true`.*

| Tool | Parameters | Description | Read-Only |
|---|---|---|:---:|
| `sitegraph` | `{action, url_prefix?, query?, id?, from_id?, to_id?, limit?, cursor?, max_depth?, since?, profile?, format?, snapshot_id?}` | SiteGraph attack surface graph analyzer (`status`, `sync`, `search`, `neighbors`, `trace`, `shortest_path`, `clusters`, `impact`, `diff`, `export`, `history_search`, `endpoint_detail`, `projects`, `config`). | No |

---

## Installation

See the complete [installation guide](docs/install.md) for supported platforms,
checksum verification, Burp extension loading, MCP client configuration,
optional skill installation, manual installation, and uninstall steps.

### 1. One-Line Installer (macOS & Linux)

Review [`install.sh`](install.sh), then install the verified native binary:

```sh
curl -fsSL https://raw.githubusercontent.com/nguyenthdat/burp-mcp/main/install.sh | bash
```

Install the binary plus the optional `burpsuite` agent skill:

```sh
curl -fsSL https://raw.githubusercontent.com/nguyenthdat/burp-mcp/main/install.sh \
  | bash -s -- --with-skill
```

The script detects Linux x86_64 and macOS arm64/x86_64, verifies the selected
asset against `SHA256SUMS`, installs to `~/.local/bin/burp-mcp`, and never uses
`sudo`. Windows users should follow the manual release-asset steps in the
installation guide.

### 2. Load the Burp Extension

1. Download `burp-mcp.jar` from the [Latest GitHub Release](https://github.com/nguyenthdat/burp-mcp/releases/latest).
2. In Burp Suite, navigate to **Extensions > Installed**.
3. Click **Add**, choose extension type **Java**, and select `burp-mcp.jar`.
4. The extension starts its loopback gRPC listener on `127.0.0.1:9877`.

### 3. Verify Connection

Test that the native server connects to the loaded Burp extension:

```sh
burp-mcp probe --endpoint http://127.0.0.1:9877
```

---

## MCP Client Configuration

### Claude Desktop

Edit `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) or `%APPDATA%\Claude\claude_desktop_config.json` (Windows):

```json
{
  "mcpServers": {
    "burp": {
      "command": "/Users/you/.local/bin/burp-mcp",
      "args": ["serve"]
    }
  }
}
```

### Cursor

In `.cursor/mcp.json` or Global Cursor Settings:

```json
{
  "mcpServers": {
    "burp": {
      "command": "/Users/you/.local/bin/burp-mcp",
      "args": ["serve"]
    }
  }
}
```

### Claude Code

```sh
claude mcp add burp /Users/you/.local/bin/burp-mcp -- serve
```

### Zed

Add to your Zed `settings.json`:

```json
{
  "context_servers": {
    "burp": {
      "command": {
        "path": "/Users/you/.local/bin/burp-mcp",
        "args": ["serve"]
      }
    }
  }
}
```

---

## Configuration

The Rust client reads TOML from `~/.config/burp-mcp/config.toml` when that file
exists. Use `--config PATH` or `BURP_MCP_CONFIG` to select another file. CLI
flags and environment variables override file values. A complete starting point
is [`config.example.toml`](config.example.toml).

| Option / setting | Environment variable | Default | Description |
|---|---|---|---|
| `--endpoint <URL>` / `[burp].endpoint` | `BURP_MCP_GRPC_ENDPOINT` | `http://127.0.0.1:9877` | Target gRPC endpoint for the Burp extension. |
| `--port <PORT>` / `[burp].port` | `BURP_MCP_GRPC_PORT` | `9877` | Loopback port when no endpoint is set. |
| `--tls-dir <PATH>` / `[burp].tls_dir` | `BURP_MCP_TLS_DIR` | `~/.config/burp-mcp/tls` | mTLS directory for HTTPS endpoints. |
| `--enable-sitegraph` / `[sitegraph].enabled` | `BURP_MCP_ENABLE_SITEGRAPH` | `false` | Enable the 15 `sitegraph_*` tools. |
| `--sitegraph-project-root <PATH>` / `[sitegraph].project_root` | `BURP_MCP_SITEGRAPH_PROJECT_ROOT` | `~/.local/share/burp-mcp/sitegraph` | Parent directory for project-scoped SQLite databases. |
| `--sitegraph-rules-path <PATH>` / `[sitegraph].rules_path` | `BURP_MCP_SITEGRAPH_RULES` | `~/.config/burp-mcp/default-rules.json` | Sitegraph enrichment rules. |
| `--sitegraph-mode <MODE>` / `[sitegraph].mode` | `BURP_MCP_SITEGRAPH_MODE` | `off` | Auto-index mode: `off`, `startup`, or `watch`. |
| `--sitegraph-interval-seconds <SECS>` / `[sitegraph].interval_seconds` | `BURP_MCP_SITEGRAPH_INTERVAL_SECONDS` | `30` | Poll interval for `watch` mode. |

Sitegraph remains opt-in: a project root or indexing mode alone does not expose
its tools. Each Burp project receives an independent database. `sitegraph_config`
is read-only; change configuration and restart `burp-mcp`.

## Security & Remote mTLS Setup

- **Local Plaintext**: Plaintext gRPC is accepted only on IPv4 loopback (`127.0.0.1`).
- **Remote mTLS**: Any non-loopback endpoint requires mutual TLS (mTLS).
- The Burp extension registers **Settings > Extensions > Burp MCP** to generate certificates, configure SANs, and manage the TLS bundle.

```text
~/.config/burp-mcp/tls/
├── ca.crt
├── server.crt
├── server.key       <-- Keep on Burp machine only
├── client.crt
├── client.key       <-- Copy to remote client machine (chmod 600)
└── bundle.conf
```

On Unix, ensure strict permissions:
```sh
chmod 700 ~/.config/burp-mcp/tls
chmod 600 ~/.config/burp-mcp/tls/client.key
```

For the sitegraph privacy boundary, project partitioning, daemon behavior, and
retention guidance, read [the sitegraph reference](docs/sitegraph.md) before
enabling it.

---

Burp MCP is dual-use security software intended only for systems you own or are
explicitly authorized to test. Treat TLS private keys, Burp traffic, sitegraph
databases, Collaborator secrets, and session material as credentials or
sensitive engagement data.

See [CONTRIBUTING.md](CONTRIBUTING.md) for development requirements. Report
suspected vulnerabilities privately using [SECURITY.md](SECURITY.md).

---

## Development & Build

### Requirements
- **Java 25** (for extension build)
- **Rust 1.88+** (for MCP binary)
- **Gradle 9.7+**

### Building and Testing

```sh
# Build Kotlin extension JAR
gradle clean test jar

# Run Rust formatting and linter
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run all workspace unit and contract tests
cargo test --workspace --locked

# Run gRPC interop suite
scripts/run-grpc-interop.sh
```

The extension JAR is written to `build/libs/burp-mcp.jar`. The live Burp/Pro
interop scenario is local-only; see [the installation guide](docs/install.md)
for the release verification steps.

## Releases

Published GitHub Releases contain the native MCP binary, extension JAR,
checksums, and an SBOM generated from the locked Rust dependency graph.

---

## License

Burp MCP is released under the [MIT License](LICENSE).
