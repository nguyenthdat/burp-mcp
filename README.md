# Burp MCP

[![CI](https://github.com/nguyenthdat/burp-mcp/actions/workflows/ci.yml/badge.svg)](https://github.com/nguyenthdat/burp-mcp/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tools](https://img.shields.io/badge/Tools-111%20(96%20Core%20%2B%2015%20SiteGraph)-brightgreen.svg)](docs/features.md)

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

- **Proxy Traffic Inspection & Triage**: Search, filter (status, method, URL, regex), annotate, highlight, and view full raw HTTP/WebSocket traffic history.
- **HTTP Request Execution & Repeater**: Send structured requests, execute parallel batches (up to 32 requests), send directly to Repeater tabs, run concurrent race condition checks, convert request methods (GET ↔ POST), and export requests as `curl` commands or Python `requests` code.
- **Interception & HTTP Handlers**: Toggle master proxy interception, register custom request/response modifying handlers, and configure granular proxy rules (`forward`, `intercept`, `drop`, `edit`).
- **Proxy Settings & Listeners**: Complete programmatic management of proxy listeners, script-mode filters (Bambdas for HTTP/WebSocket history, sitemap, logger), and request/response interception rule chains.
- **Cookie Jar**: Inspect, filter by domain, and set cookies within Burp's active cookie jar.
- **Session Handling & Macros**: Create, list, execute, update, and remove scoped session handling rules and multi-request macros with parameter extraction.
- **Intruder & Declarative Fuzzing**: Open raw requests with insertion points in Intruder UI, run bounded single-marker fuzzer jobs, and register custom payload processors and generators.
- **In-Memory Payload Lists**: Create, import from file/JSON/text, update, paginate, and delete named payload lists for fuzzing and Intruder attacks.
- **Scanner & Crawl Automation**: Launch bounded passive audits, active scans, and crawls; poll background jobs; triage and inspect issues; persist custom audit findings; and generate HTML or XML Scanner reports.
- **Scan Configuration & Resource Pools**: Full CRUD for scan configurations (audit types, out-of-scope policies, timeouts) and scanner resource pools (concurrency limits, throttling, retries).
- **Burp Collaborator & Out-of-Band (OAST) Testing**: Generate unique out-of-band interaction payloads and poll for DNS, HTTP, HTTPS, and SMTP interactions to detect blind vulnerabilities (Blind SSRF, Blind SQLi, Blind XXE, Blind RCE, Log4Shell, and Deserialization).
- **Managed WebSockets**: Establish and manage outbound WebSocket connections through Burp, send text or base64 binary frames, and review message history.
- **Bambda & BCheck Script Imports**: Safely validate and import Java Bambdas and declarative BCheck scripts into Burp without unsafe automatic execution.
- **Sitegraph Engine (Advanced Opt-in)**: Project-scoped SQLite graph mapping endpoints, parameters, topology, shortest paths, clusters, downstream impact, diffs, and indexed HTTP/WebSocket evidence. Treat each graph as sensitive engagement data.
- **Offline Utility Decoder Engine**: 40+ built-in operations for encoding/decoding (Base64, Hex, URL, HTML, Unicode), cryptographic hashes (MD5, SHA-1/256/512, BLAKE3, HMAC), compression (Gzip, Zlib, Deflate, Brotli), JWT decoding/verification, and HTTP parsing.

Some capabilities require Burp Suite Professional or a Burp feature advertised
by the connected extension. The runtime tool schema and
`burp_burp_version.capabilities` are authoritative.

---

## Tools Inventory (111 Tools)

Burp MCP provides **96 tools by default**, plus **15 advanced tools** when SiteGraph is enabled.

### 1. Connection & Project Configuration (5 tools)

| Tool | Parameters | Description | Read-Only |
|---|---|---|:---:|
| `burp_burp_version` | `{}` | Return Burp Suite version, edition, extension version, capabilities, and runtime limits. | Yes |
| `burp_extension_info` | `{}` | Return extension and process metadata (JAR location, BApp status, Java arguments). | Yes |
| `burp_export_config` | `{}` | Export project configuration as a JSON string. | No |
| `burp_inspect_config` | `{paths?}` | Export scoped project options with discovered leaf paths and UTF-8 size. | Yes |
| `burp_import_config` | `{config}` | Validate and import size-bounded project configuration JSON. | No |

### 2. Scope, Target & Site Map (5 tools)

| Tool | Parameters | Description | Read-Only |
|---|---|---|:---:|
| `burp_get_scope` | `{url}` | Check whether a specific URL is in the current target scope. | Yes |
| `burp_add_to_scope` | `{url}` | Add a URL to Burp target scope. | No |
| `burp_remove_from_scope` | `{url}` | Remove a URL from Burp target scope. | No |
| `burp_target_info` | `{url?, limit?}` | Summarize hosts and technology headers from a bounded site map sample. | Yes |
| `burp_sitemap` | `{url_prefix?, limit?, cursor?}` | Page through Burp site map entries with optional URL prefix filtering. | Yes |

### 3. Proxy Evidence & WebSocket History (6 tools)

| Tool | Parameters | Description | Read-Only |
|---|---|---|:---:|
| `burp_proxy_history` | `{limit?, offset?, cursor?, url_filter?, method_filter?, status_filter?, has_notes?, color?}` | Page and filter Proxy HTTP history. | Yes |
| `burp_proxy_detail` | `{index}` | Get full raw request and response details, notes, and highlight for a history index. | Yes |
| `burp_highlight` | `{index, color?}` | Set or clear the highlight color on an item in Proxy history. | No |
| `burp_annotate` | `{index, note}` | Set or update notes on an item in Proxy history. | No |
| `burp_extract_from_response` | `{index, regex, limit?}` | Extract regex matches from a recorded response. | No |
| `burp_proxy_websocket_history` | `{limit?, cursor?}` | Page through observed WebSocket messages captured by Burp Proxy (base64 payloads). | Yes |

### 4. HTTP Sending, Preparation & Repeater (6 tools)

| Tool | Parameters | Description | Read-Only |
|---|---|---|:---:|
| `burp_send_request` | `{url, method?, body?, headers?}` | Send an HTTP request through Burp and receive the response. | No |
| `burp_send_request_parallel` | `{requests}` | Send a batch of HTTP requests concurrently (up to 32 requests). | No |
| `burp_send_to_repeater` | `{request, host, port?, https?, tab_name?}` | Display a raw HTTP request in a Burp Repeater tab without sending it. | No |
| `burp_race_condition` | `{request, host, port?, https?, count?}` | Start a bounded concurrent request comparison job. | No |
| `burp_convert_request` | `{request, convert_to?}` | Convert HTTP request method (e.g. GET ↔ POST). | No |
| `burp_export_request` | `{request, host?, format?, https?}` | Export a request as raw text, `curl` command, or Python `requests` code. | No |

### 5. Interception, Handlers & Proxy Settings (17 tools)

| Tool | Parameters | Description | Read-Only |
|---|---|---|:---:|
| `burp_intercept_state` | `{}` | Read current master Proxy interception state (enabled/disabled). | Yes |
| `burp_set_intercept_state` | `{enabled}` | Toggle master Proxy interception state. | No |
| `burp_intercept_controller` | `{enabled?, timeout_seconds?}` | Read or configure the MCP-owned HTTP interception queue. Pending messages auto-forward on timeout. | No |
| `burp_intercepted_messages` | `{limit?, cursor?}` | Page pending HTTP requests and responses, including lossless base64 messages. | Yes |
| `burp_control_intercepted_message` | `{id, action, message_base64?}` | Forward, drop, or send one paused HTTP message to Burp's manual Intercept tab; optionally replace the full message. | No |
| `burp_websocket_intercept_controller` | `{enabled?, timeout_seconds?}` | Read or configure MCP-owned WebSocket interception. | No |
| `burp_intercepted_websocket_messages` | `{limit?, cursor?}` | Page pending intercepted WebSocket messages. | Yes |
| `burp_control_intercepted_websocket_message` | `{id, action, payload_base64?}` | Forward, drop, or send one paused WebSocket message to Burp's manual Intercept tab; optionally replace its payload. | No |
| `burp_register_http_handler` | `{header_name?, header_value?, match_text?, replace?}` | Register a bounded HTTP request handler rule for header injection or string replacement. | No |
| `burp_remove_http_handler` | `{}` | Remove all registered HTTP handler rules. | No |
| `burp_register_proxy_rule` | `{url_contains, id?, phase?, action?, match_text?, replace?, header_name?, header_value?, enabled?}` | Register a Proxy request/response rule (`forward`, `intercept`, `drop`, `edit`). | No |
| `burp_list_proxy_rules` | `{}` | List registered Proxy request and response rules. | Yes |
| `burp_remove_proxy_rule` | `{id?}` | Remove a Proxy rule by ID, or clear all rules. | No |
| `burp_proxy_intercept_config` | `{}` | Read Proxy request, response, WebSocket interception filters, and response modification settings. | Yes |
| `burp_update_proxy_intercept_config` | `{master_intercept_enabled?, request_do_intercept?, response_do_intercept?, ...}` | Patch Proxy interception filters and response modification options. | No |
| `burp_proxy_settings` | `{}` | Read Proxy listeners, script filters, and interception settings together. | Yes |
| `burp_update_proxy_settings` | `{operation, port?, running?, listen_mode?, ...}` | Granular mutation of Proxy listeners, script filters, or interception rules. | No |

### 6. Cookies (2 tools)

| Tool | Parameters | Description | Read-Only |
|---|---|---|:---:|
| `burp_cookie_jar` | `{limit?, domain?}` | List cookies in Burp's cookie jar with domain, path, value, and expiration. | Yes |
| `burp_cookie_jar_set` | `{name, value, domain, path?, expiration?}` | Set or update a cookie in Burp's cookie jar. | No |

### 7. Sessions & Macros (9 tools)

| Tool | Parameters | Description | Read-Only |
|---|---|---|:---:|
| `burp_session_create_rule` | `{id?, description?, action_type?, find?, replace?, header_name?, parameter_name?, macro_description?, url_contains?, tools?, enabled?}` | Create a scoped session handling rule. | No |
| `burp_session_get_rule` | `{id}` | Get a session handling rule by ID. | Yes |
| `burp_session_update_rule` | `{id, description?, action_type?, find?, replace?, ...}` | Update an existing session handling rule by ID. | No |
| `burp_session_list_rules` | `{}` | List all registered session handling rules. | Yes |
| `burp_session_delete_rule` | `{id}` | Delete a session handling rule by ID. | No |
| `burp_macro_create` | `{description, serial_number?, items}` | Create or replace a Burp session macro definition. | No |
| `burp_macro_list` | `{}` | List all defined Burp session macros. | Yes |
| `burp_macro_run` | `{description}` | Execute requests in a named session macro and inspect results. | No |
| `burp_macro_remove` | `{description}` | Remove a session macro by description. | No |

### 8. Intruder & Declarative Fuzzing (8 tools)

| Tool | Parameters | Description | Read-Only |
|---|---|---|:---:|
| `burp_inline_fuzzer` | `{template, host, port?, https?, marker?, wordlist, payload_list_id?, payload_offset?}` | Start a bounded input matrix fuzzing job against a raw request template. | No |
| `burp_send_to_intruder` | `{request, host, port?, https?, tab_name?}` | Open a raw request in Burp Intruder UI without starting an attack. | No |
| `burp_intruder_payload_processor_register` | `{id, display_name, operation, argument?, replacement?}` | Register a declarative Intruder payload processor. | No |
| `burp_intruder_payload_processor_list` | `{}` | List registered Intruder payload processors. | Yes |
| `burp_intruder_payload_processor_remove` | `{id}` | Deregister an Intruder payload processor by ID. | No |
| `burp_intruder_payload_generator_register` | `{id, display_name, payloads, max_output_count?, payload_list_id?, payload_offset?}` | Register a declarative Intruder payload generator. | No |
| `burp_intruder_payload_generator_list` | `{}` | List registered Intruder payload generators. | Yes |
| `burp_intruder_payload_generator_remove` | `{id}` | Deregister an Intruder payload generator by ID. | No |

### 9. Payload Lists (6 tools)

| Tool | Parameters | Description | Read-Only |
|---|---|---|:---:|
| `burp_payload_list_create` | `{id, display_name, payloads}` | Create a named in-memory payload list. | No |
| `burp_payload_list_import` | `{id, display_name, content, format?, keep_empty?}` | Import a payload list from newline text or JSON array. | No |
| `burp_payload_list_list` | `{}` | List metadata for all in-memory payload lists. | Yes |
| `burp_payload_list_get` | `{id, offset?, limit?}` | Read a paginated slice of payloads from a list. | Yes |
| `burp_payload_list_update` | `{id, operation, payloads?, index?, indexes?, display_name?}` | Append, prepend, insert, replace, remove, or clear entries in a payload list. | No |
| `burp_payload_list_delete` | `{id}` | Delete a payload list by ID. | No |

### 10. Scanner Execution, Crawl & Findings (7 tools)

| Tool | Parameters | Description | Read-Only |
|---|---|---|:---:|
| `burp_scan_start` | `{url, audit_type?, scan_configuration_id?, resource_pool_id?, timeout_seconds?, stable_seconds?, include_out_of_scope?}` | Start a passive stateless audit or active scan with bounded options. | No |
| `burp_scan_stop` | `{job_id}` | Stop an active Burp audit job by job ID. | No |
| `burp_scan_remove` | `{job_id}` | Remove a terminal scan/crawl job from the registry. | No |
| `burp_crawl` | `{seed_urls, scan_configuration_id?, resource_pool_id?, timeout_seconds?, stable_seconds?, include_out_of_scope?}` | Start a bounded Burp crawl from seed URLs. | No |
| `burp_scan_issues` | `{limit?, cursor?, severity_filter?, confidence_filter?, url_filter?, index?}` | Page through Scanner issues with severity/confidence filters. | Yes |
| `burp_scan_issue_detail` | `{index}` | Get complete details and HTTP evidence for a Scanner issue index. | Yes |
| `burp_scanner_generate_report` | `{format, path, issue_indexes?}` | Generate an HTML or XML Burp Scanner report for selected issues. | No |

### 11. Scanner Configuration & Resource Pools (10 tools)

| Tool | Parameters | Description | Read-Only |
|---|---|---|:---:|
| `burp_scan_config_list` | `{}` | List built-in and project-persisted scan configurations. | Yes |
| `burp_scan_config_get` | `{id}` | Get a scan configuration by ID. | Yes |
| `burp_scan_config_create` | `{id?, name, scan_type, audit_type?, include_out_of_scope?, timeout_seconds?, stable_seconds?, resource_pool_id?}` | Create a persisted scan configuration. | No |
| `burp_scan_config_update` | `{id?, name, scan_type, audit_type?, include_out_of_scope?, timeout_seconds?, stable_seconds?, resource_pool_id?}` | Update a persisted scan configuration by ID. | No |
| `burp_scan_config_delete` | `{id}` | Delete a persisted scan configuration by ID. | No |
| `burp_scan_pool_list` | `{}` | List scanner resource pool definitions. | Yes |
| `burp_scan_pool_get` | `{id}` | Get a scanner resource pool definition by ID. | Yes |
| `burp_scan_pool_create` | `{id?, name, kind, existing_pool_name?, concurrent_request_limit?, throttle_millis?, max_retries?}` | Create a scanner resource pool definition. | No |
| `burp_scan_pool_update` | `{id?, name, kind, existing_pool_name?, concurrent_request_limit?, throttle_millis?, max_retries?}` | Update a scanner resource pool definition by ID. | No |
| `burp_scan_pool_delete` | `{id}` | Delete a scanner resource pool definition by ID. | No |

### 12. Background Jobs (3 tools)

| Tool | Parameters | Description | Read-Only |
|---|---|---|:---:|
| `burp_job_status` | `{job_id}` | Check the progress, status, and summary of a background job. | Yes |
| `burp_job_result` | `{job_id, limit?, cursor?}` | Read paginated result items from a completed background job. | Yes |
| `burp_job_cancel` | `{job_id}` | Cancel an in-progress background job. | No |

### 13. Collaborator & Custom Findings (3 tools)

| Tool | Parameters | Description | Read-Only |
|---|---|---|:---:|
| `burp_collaborator_generate` | `{count?}` | Generate bounded Collaborator payloads for out-of-band testing. | No |
| `burp_collaborator_poll` | `{limit?, cursor?}` | Page DNS, HTTP, HTTPS, or SMTP interactions observed by the extension's active Collaborator context. | Yes |
| `burp_add_issue` | `{name, url, detail?, remediation?, severity?, confidence?}` | Add a custom typed security issue to the Burp site map. | No |

### 14. Managed WebSockets (6 tools)

| Tool | Parameters | Description | Read-Only |
|---|---|---|:---:|
| `burp_websocket_create` | `{host, port?, https?, path?}` | Open a managed WebSocket connection through Burp. | No |
| `burp_websocket_send_text` | `{id, text}` | Send a text message over a managed WebSocket connection. | No |
| `burp_websocket_send_binary` | `{id, data}` | Send base64-encoded binary data over a managed WebSocket connection. | No |
| `burp_websocket_history` | `{id?, limit?, cursor?}` | Read message history (sent and received) for managed WebSocket connections. | Yes |
| `burp_websocket_close` | `{id}` | Close a managed WebSocket connection. | No |
| `burp_websocket_list` | `{}` | List all active managed WebSocket connection IDs. | Yes |

### 15. Bambda & BCheck Script Imports (2 tools)

| Tool | Parameters | Description | Read-Only |
|---|---|---|:---:|
| `burp_bambda_import` | `{script}` | Validate and import a complete Bambda YAML script without executing it. | No |
| `burp_bcheck_import` | `{script, enabled?}` | Validate and import a complete BCheck script definition into Burp. | No |

### 16. Persistent Sitegraph (15 tools, Opt-in)

*Requires starting the server with `--enable-sitegraph` or `BURP_MCP_ENABLE_SITEGRAPH=true`.*

| Tool | Parameters | Description | Read-Only |
|---|---|---|:---:|
| `sitegraph_sync` | `{url_prefix?}` | Synchronize bounded Burp observations into the active project's local SQLite graph. | No |
| `sitegraph_status` | `{}` | Get local sitegraph synchronization and schema status. | Yes |
| `sitegraph_stats` | `{}` | Return graph node and edge counts and last sync time. | Yes |
| `sitegraph_config` | `{}` | Read active auto-index settings; change them in configuration and restart. | Yes |
| `sitegraph_projects` | `{}` | List the active project-scoped graph identity. | Yes |
| `sitegraph_search` | `{query, limit?, cursor?}` | Search normalized endpoints with metadata filters. | Yes |
| `sitegraph_history_search` | `{query, source?, limit?, cursor?}` | Search indexed raw HTTP/WebSocket evidence with bounded pagination. | Yes |
| `sitegraph_endpoint_detail` | `{id}` | Get full normalized endpoint metadata and adjacency counts. | Yes |
| `sitegraph_neighbors` | `{id, limit?, cursor?}` | Page adjacent inbound and outbound graph nodes. | Yes |
| `sitegraph_trace` | `{id, max_depth?, limit?}` | Trace graph relationships to a depth of 1..8 hops. | Yes |
| `sitegraph_shortest_path` | `{from_id, to_id, max_depth?}` | Find the shortest directed path between two graph nodes. | Yes |
| `sitegraph_clusters` | `{limit?}` | Cluster project endpoints by origin and path segments. | Yes |
| `sitegraph_impact` | `{id, max_depth?, limit?}` | Perform downstream impact analysis from a seed node. | Yes |
| `sitegraph_diff` | `{since, limit?, cursor?}` | Query nodes changed since a specific Unix timestamp. | Yes |
| `sitegraph_export` | `{profile?, format?, snapshot_id?, cursor?, limit?}` | Export bounded metadata or exact-evidence pages; exact evidence is sensitive. | Yes |

### 17. Offline Decoder Engine (1 tool)

| Tool | Parameters | Description | Read-Only |
|---|---|---|:---:|
| `decoder` | `{input, operation?, args?, steps?, query?, describe?, magic?}` | Execute deterministic transformations, multi-step recipes, search catalog, or get magic decode suggestions. | Yes |

Supported operation categories:
- **Encoding/Decoding**: `base64.encode`, `base64.decode`, `base64url.encode`, `base64url.decode`, `hex.encode`, `hex.decode`, `url.encode`, `url.decode`, `html.encode`, `html.decode`, `unicode.escape`, `unicode.unescape`.
- **Hashes & Checksums**: `md5`, `sha1`, `sha256`, `sha512`, `blake3`, `hmac.sha256`, `hmac.sha512`, `entropy`, `length`, `strings.extract`.
- **Compression**: `gzip.compress`, `gzip.decompress`, `zlib.compress`, `zlib.decompress`, `deflate.compress`, `deflate.decompress`, `brotli.compress`, `brotli.decompress`.
- **Web & Security**: `jwt.decode`, `jwt.verify_hs256`, `cookie.parse`, `query.parse`, `query.build`, `http.parse`, `http.set_body`, `http.update_content_length`.
- **JSON & Text**: `json.pretty`, `json.minify`, `json.query`, `text.uppercase`, `text.lowercase`, `text.reverse`, `text.split`, `text.join`, `regex.extract`, `regex.replace`.

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

The extension JAR is written to `build/libs/burp-mcp.jar`. The live Burp/Pro
interop scenario is local-only; see [the installation guide](docs/install.md)
for the release verification steps.

## Releases

Published GitHub Releases contain the native MCP binary, extension JAR,
checksums, and an SBOM generated from the locked Rust dependency graph.

---

## License

Burp MCP is released under the [MIT License](LICENSE).
