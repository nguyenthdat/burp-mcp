# Burp MCP Feature Reference

This catalog is the definitive reference for Burp MCP tools, actions, and verification contracts.

## Live-test conventions

- Test only authorized targets. Record the Burp version, edition, extension version, and target boundary first.
- Call `burp_burp_version` before edition-dependent cases and record advertised capabilities.
- For every mutation: capture the baseline, make the smallest isolated change, verify the external effect, then restore the baseline.
- `success: true` means the operation was accepted. It does not prove the effect; use a corresponding read tool, Burp UI state, target behavior, or graph state.
- Redact cookie values, configuration secrets, Collaborator secrets, and sensitive request/response data from reports.

## Inventory

Burp MCP registers **43 tools by default** (42 Burp tools + 1 offline Decoder tool), plus **1 SiteGraph tool** when SiteGraph is enabled with `--enable-sitegraph`.

| Feature group | Tools |
|---|---:|
| Connection and project configuration | 2 |
| Core Pentesting Suite | 13 |
| Compound Security Workflows (IDOR, CORS, Auth Matrix, JWT, SSRF, SQLi, GraphQL, CSRF, API Fuzz) | 9 |
| Active UI & Desktop Editor Integration | 3 |
| Cookies & Findings | 3 |
| Background Job Control | 3 |
| Custom Script Imports | 2 |
| MCP Interception Queues | 6 |
| Offline Utility Decoder | 1 |
| SiteGraph (Advanced Opt-in) | 1 |
| **Total Default Tools** | **43** |
| **Total with SiteGraph** | **44** |

## 1. Connection & Project Configuration (2 tools)

| Tool | Purpose | Preliminary live check |
|---|---|---|
| `burp_burp_version` | Return the Burp version, edition, extension version, capabilities, and runtime limits. | Compare version and edition with the Burp UI; use capabilities as prerequisites for later cases. |
| `burp_extension_info` | Return extension and process metadata, including filename, BApp status, and process arguments. | Compare the result with the loaded extension; expect no mutation. |

---

## 2. Core Pentesting Suite (13 tools)

### `burp_proxy`
Burp Proxy tool.
- **Actions**:
  - `history`: Page and filter Proxy HTTP history with compact metadata (`include_bodies: false` by default), `headers_only`, `extract_css`, `extract_json`, `max_body_length`.
  - `detail`: Get full request/response details for a specific Proxy history index with optional projection.
  - `annotate`: Add/edit notes on a Proxy history entry.
  - `highlight`: Set/clear color highlight on a Proxy history entry.
  - `extract`: Extract regex matches from a Proxy history response.
  - `websocket_history`: Page observed WebSocket message history with base64 payloads.

### `burp_http`
Burp HTTP client & Repeater bridge.
- **Actions**:
  - `send`: Send one HTTP request through Burp with optional headers, projection, and extraction.
  - `send_batch`: Send parallel HTTP requests (up to 32).
  - `convert`: Convert raw HTTP request method (e.g. GET ↔ POST).
  - `export`: Export request as raw text, `curl`, or Python `requests` code.
  - `send_to_repeater`: Open a raw request in Burp Repeater UI under a named tab.

### `burp_target`
Burp Target & Scope manager.
- **Actions**:
  - `get_scope`: Check if a URL is in the current target scope.
  - `add_scope`: Add a URL to Burp target scope.
  - `remove_scope`: Remove a URL from Burp target scope.
  - `info`: Summarize target hosts and technology headers from site map.
  - `sitemap`: Page through Burp site map entries with optional URL prefix.

### `burp_scanner`
Burp Scanner automation.
- **Actions**:
  - `start_audit`: Start passive stateless audit or active scan with configuration options.
  - `start_crawl`: Start crawler from seed URLs.
  - `stop`: Stop an active audit job by job ID.
  - `list_issues`: Page through discovered Scanner issues.
  - `issue_detail`: Get full details and evidence for one Scanner issue index.
  - `update_issue`: Update an issue status (`false_positive`, `ignored`, `confirmed`) and notes.
  - `report`: Generate HTML or XML Scanner reports.
  - `test_bcheck` / `dry_run`: Dry-run a BCheck script against sample HTTP exchange.
  - `remove`: Remove a completed/stopped scan job from the registry.

### `burp_scan_config`
Scanner configurations and resource pools manager.
- **Actions**:
  - `list_configs`: List built-in and project-persisted scan configurations.
  - `get_config`: Get a scan configuration by ID.
  - `upsert_config`: Create or update a scan configuration.
  - `delete_config`: Delete a scan configuration by ID.
  - `list_pools`: List scanner resource pool definitions.
  - `get_pool`: Get a resource pool definition by ID.
  - `upsert_pool`: Create or update a resource pool (concurrency limit, throttle, retries).
  - `delete_pool`: Delete a resource pool by ID.

### `burp_fuzzer`
Fuzzer & Intruder suite.
- **Actions**:
  - `fuzz`: Run bounded request matrix fuzzing supporting `pitchfork`, `cluster_bomb`, and `sniper` modes with multi-marker substitution.
  - `race`: Run concurrent race condition check with optional Last-Byte Sync Single-Packet Attack (`single_packet_attack: true`).
  - `send_to_intruder`: Open request in Burp Intruder UI.
  - `list_payloads`: List managed in-memory payload lists.
  - `get_payload_list`: Page items from a named payload list.
  - `create_payload_list`: Create a named in-memory payload list.
  - `import_payload_list`: Import a payload list from raw text/JSON.
  - `upsert_payloads`: Update or replace payload list items.
  - `delete_payload_list`: Delete a payload list.
  - `register_payload_processor`: Register custom declarative Intruder payload processor.
  - `list_payload_processors`: List registered payload processors.
  - `remove_payload_processor`: Deregister payload processor.
  - `register_payload_generator`: Register finite Intruder payload generator.
  - `list_payload_generators`: List registered payload generators.
  - `remove_payload_generator`: Deregister payload generator.

### `burp_collaborator`
Out-of-Band (OAST) testing engine.
- **Actions**:
  - `generate`: Generate bounded Collaborator payloads with optional `target_url` and `injection_point` binding.
  - `poll`: Poll for DNS/HTTP interactions (automatically enriched with origin correlation metadata).
  - `correlate`: Retrieve correlation mapping for active payloads.

### `burp_websocket`
Managed WebSocket connection manager.
- **Actions**:
  - `create`: Establish outbound managed WebSocket connection.
  - `send_text`: Send text frame on managed WebSocket.
  - `send_binary`: Send base64 binary frame on managed WebSocket.
  - `history`: Page message history for a connection.
  - `close`: Close a managed WebSocket connection.
  - `list`: List active managed WebSocket IDs.

### `burp_session`
Session Handling Rules & Macros manager.
- **Actions**:
  - `list_rules`: List registered MCP session rules.
  - `get_rule`: Get session rule details by ID.
  - `upsert_rule`: Create or update a session rule.
  - `delete_rule`: Delete a session rule by ID.
  - `run_macro`: Execute requests from a session macro definition.
  - `upsert_macro`: Create or replace a macro definition.
  - `list_macros`: List managed macro definitions.
  - `delete_macro`: Remove a macro definition.

### `burp_settings`
Proxy Settings & Configuration manager.
- **Actions**:
  - `get_proxy_settings`: Read listeners, script filters, and intercept settings.
  - `update_proxy_settings`: Mutate listeners, filters, or rules (`operation`: `listener_upsert`, `listener_delete`, `script_filter_upsert`, `script_filter_delete`, `intercept_rule_upsert`, `intercept_rule_delete`, `intercept_toggle`).
  - `export_config`: Export project configuration as JSON.
  - `inspect_config`: Inspect selected project options before import.
  - `import_config`: Import project configuration JSON.
  - `intercept_state`: Read master Proxy intercept state.
  - `set_intercept_state`: Enable/disable master Proxy intercept.
  - `proxy_intercept_config`: Read legacy intercept filters and response modification.
  - `update_proxy_intercept_config`: Patch intercept filters and response modification.
  - `register_http_handler`: Register bounded HTTP request handler rule.
  - `remove_http_handler`: Remove/clear HTTP handler rules.
  - `register_proxy_rule`: Register request/response Proxy rule (`forward`, `intercept`, `drop`, `edit`).
  - `list_proxy_rules`: List runtime Proxy rules.
  - `remove_proxy_rule`: Remove one or clear all Proxy rules.

### `burp_logger`
Burp Logger traffic inspector.
- **Actions**:
  - `query`: Page HTTP traffic across all tools (`proxy`, `repeater`, `scanner`, `intruder`, `extension`) with compact metadata and extraction.
  - `detail`: Read full request/response for one Logger index.
  - `clear`: Clear in-memory Logger traffic buffer.

### `burp_organizer`
Burp Organizer interface.
- **Actions**:
  - `add`: Send request/response exchange into Burp Organizer with notes and highlight.
  - `list`: List and filter entries in Burp Organizer.

### `burp_diff`
Response Comparer & Diff engine.
- **Actions**:
  - `diff_responses`: Compare two responses (strings or indexes), computing similarity score ($0.0 \dots 1.0$), header diffs, and unified line diffs.
  - `compare_exchanges`: Send two raw payloads to Burp Comparer UI tab.

---

## 3. Compound Security Workflows (9 tools)

| Tool | Purpose | Preliminary live check |
|---|---|---|
| `burp_verify_idor` | Automated IDOR verification across two user authorization contexts (User A vs User B). | Send requests with original and victim auth headers, verify similarity and differential verdict. |
| `burp_check_cors` | Automated CORS vulnerability auditing with origin reflections, wildcard checks, and credentials evaluation. | Pass target URL, review generated findings across test origins. |
| `burp_auth_matrix` | Automated role-based access control matrix across multiple endpoints and user roles. | Submit matrix of endpoints and role headers, evaluate access violations. |
| `burp_audit_jwt` | Automated JWT vulnerability audit (None algorithm, RS256 -> HS256 key confusion, and claim tampering). | Provide target JWT, verify rejection of forged tokens. |
| `burp_verify_ssrf` | Automated SSRF verification with Collaborator interaction polling and payload correlation. | Provide target URL and injection points, verify callback detection. |
| `burp_verify_sqli_blind` | Differential boolean-based and timing statistical blind SQL injection verification. | Provide target parameter, verify cosine diff score and timing delays. |
| `burp_audit_graphql` | Automated GraphQL security audit (Introspection, Field Suggestions, and Query Batching). | Provide GraphQL endpoint, review enabled introspection/batching. |
| `burp_verify_csrf_samesite` | Automated CSRF risk audit, SameSite cookie evaluation, and auto-generated HTML PoC form. | Provide target endpoint, inspect cookie flags and generated HTML PoC. |
| `burp_api_fuzz_orchestrator` | Automated specification-driven API fuzzing from OpenAPI 2.0 / 3.0 or Swagger documents. | Provide OpenAPI spec string, verify bounded batch mutations and anomaly detection. |

---

## 4. Active UI & Desktop Editor Integration (3 tools)
| Tool | Purpose | Preliminary live check |
|---|---|---|
| `burp_editor_get` | Capture the active or last-focused Burp editor tab (HTTP Request/Response or WebSocket) with rich metadata, selection offsets, and UTF-8 decoded text. | Focus a tab, test fallback to Last-Active or Staged Buffer, verify rich metadata. |
| `burp_editor_patch` | Surgically modify the active Burp editor contents (`replace_selection`, `set_header`, `json_patch`, `set_param`, `regex`, `replace_all`) with automatic Content-Length calculation and CRLF normalization. | Apply a surgical patch, verify zero formatting corruption and 90% token reduction. |
| `burp_editor_renew_lease` | Extend the lifetime of an active Burp editor lease token. | Renew active token, verify extended expiry timestamp. |
## 5. Cookies & Findings (3 tools)

| Tool | Purpose | Preliminary live check |
|---|---|---|
| `burp_cookie_jar` | List cookies with optional domain filtering and pagination. | Inspect active cookie jar. |
| `burp_cookie_jar_set` | Create or update a cookie in Burp's active cookie jar. | Set a test cookie and verify in cookie jar. |
| `burp_add_issue` | Add a typed custom audit issue to the Burp site map. | Add custom finding into site map. |

---

## 6. Background Jobs (3 tools)

| Tool | Purpose | Preliminary live check |
|---|---|---|
| `burp_job_status` | Return state, progress, and error details for a background job. | Poll job to terminal state. |
| `burp_job_result` | Return paginated background job results. | Retrieve results upon job completion. |
| `burp_job_cancel` | Cancel an unfinished background job. | Stop in-flight job. |

---

## 7. Custom Script Imports (2 tools)

| Tool | Purpose | Preliminary live check |
|---|---|---|
| `burp_bambda_import` | Validate and import Bambda script definition into Burp. | Import YAML Bambda definition. |
| `burp_bcheck_import` | Validate and import declarative BCheck definition into Burp Scanner. | Import BCheck script. |

---

## 8. MCP Interception Queues (6 tools)

| Tool | Purpose | Preliminary live check |
|---|---|---|
| `burp_intercept_controller` | Read or configure MCP-controlled HTTP request/response interception queue. | Configure bounded queue timeout. |
| `burp_intercepted_messages` | List HTTP messages currently paused by MCP intercept controller. | Inspect paused requests/responses. |
| `burp_control_intercepted_message` | Forward, drop, or edit MCP-paused HTTP message. | Resolve paused message. |
| `burp_websocket_intercept_controller` | Read or configure MCP-controlled WebSocket interception queue. | Configure WebSocket intercept queue. |
| `burp_intercepted_websocket_messages` | List WebSocket messages currently paused by MCP controller. | Inspect paused text/binary frames. |
| `burp_control_intercepted_websocket_message` | Forward, drop, or edit paused WebSocket frame. | Resolve paused WebSocket frame. |

---

## 9. Offline Utility Decoder (1 tool)

| Tool | Purpose | Preliminary live check |
|---|---|---|
| `decoder` | 40+ built-in operations for encoding, decoding, hashing, compression, JWT, and parsing. | Execute single transforms or multi-step recipes. |

---

## 10. SiteGraph (Advanced Opt-in) (1 tool)

| Tool | Purpose | Preliminary live check |
|---|---|---|
| `sitegraph` | SiteGraph attack surface graph analyzer (`status`, `sync`, `search`, `neighbors`, `trace`, `shortest_path`, `clusters`, `impact`, `diff`, `export`, `history_search`, `endpoint_detail`, `projects`, `config`). | Enable with `--enable-sitegraph`, sync site map, and run graph traversals. |
