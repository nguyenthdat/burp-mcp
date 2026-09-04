---
name: burpsuite
description: >-
  Operate an already configured burp-mcp server for authorized web application
  security testing, penetration testing, vulnerability assessment, and Burp Suite
  automation: inspect Proxy HTTP/WebSocket history, scope, and site map; craft
  and replay requests in Repeater; run bounded parallel fuzzing, race condition
  jobs, and Collaborator out-of-band checks; manage sessions, cookies, macros,
  and WebSockets; orchestrate passive/active scans, custom configurations, and
  resource pools; author and import Bambda or BCheck scripts; perform offline
  payload transformations; and query the SQLite site graph. Use when asked to
  assess, investigate, reproduce, verify, fuzz, annotate, or document web security
  vulnerabilities through Burp Suite. Do not use to implement or debug the burp-mcp
  codebase itself, or to test targets without explicit authorization.
---

# Burp Suite Security Testing Skill

Use the native `burp-mcp` server as a controlled, deterministic interface to Burp Suite. Focus on reproducible security evidence, defensive test isolation, and bounded side effects.

Tool names in this skill are server-local names. Use the exact names returned by the connected server's `tools/list`; MCP hosts may expose qualified bindings such as `mcp__burp_mcp_burp_http` or `tool.mcp__burp_mcp_burp_http`, so do not assume `tool.burp_http` exists in an eval kernel. The runtime MCP schema is authoritative.

---

## Decision Index: When to Load Specific References

Load supporting reference files only when the task enters a specific domain:

| Domain / Task | Reference to Load | Purpose |
|---|---|---|
| **AppSec Testing Methodologies & OAST** | [`references/appsec-testing-guide.md`](references/appsec-testing-guide.md) | OWASP WSTG testing techniques for Auth, IDOR, SQLi, XSS, OAST (Collaborator blind SSRF/RCE/XXE/SQLi), Race conditions, and WebSockets. |
| **Complete Tool Schema & Inputs** | [`references/tool-catalog.md`](references/tool-catalog.md) | Parameter lists, constraints, and schemas for the current tool runtime. |
| **PortSwigger Desktop Workflows** | [`references/burp-workflows.md`](references/burp-workflows.md) | Standard desktop workflows for Proxy, Scope, Repeater, Intruder, and Scanner. |
| **Graph-Based Attack Surface** | [`references/sitegraph.md`](references/sitegraph.md) | SQLite sitegraph partitioning, modes, clustering, shortest path, and impact analysis. |
| **Scanner Custom Checks** | [`references/bcheck-authoring.md`](references/bcheck-authoring.md) | Writing, reviewing, validating, and importing declarative BCheck scripts. |
| **Table Filters & Java Scripts** | [`references/bambda-authoring.md`](references/bambda-authoring.md) | Writing, reviewing, and importing Java Bambda scripts for Proxy, Logger, and Repeater. |

---

## Safety Contract & Testing Discipline

1. **Verify Authorization**: Identify the exact target scheme, host, port, and allowed path boundaries before sending active traffic, fuzzing, crawling, polling Collaborator, or modifying Burp state. Scope is a routing boundary, not legal authorization.
2. **Token Efficiency First**: Prefer compact metadata history (`burp_proxy` / `burp_logger` default `include_bodies: false`, `max_body_length: 4096` default). Use server-side projection (`headers_only`, `extract_json: "$.data..."`, `extract_css: "form#login"`) or fetch single entries (`burp_proxy` action `detail`, `burp_logger` action `detail`) to preserve client context window. Original byte lengths and truncation state are emitted.
3. **Start Read-Only**: Prefer Proxy history (`burp_proxy` action `history`), Logger traffic (`burp_logger` action `query`), Target site map (`burp_target` action `sitemap`), target info (`burp_target` action `info`), Scanner issues (`burp_scanner` action `list_issues`), sitegraph queries (`sitegraph` action `search`), and offline decoder operations (`decoder`) before generating active traffic.
4. **Preserve Operator State**: Record every temporary scope addition, intercept state change, HTTP handler, proxy rule, session rule, macro, cookie, background job, and managed WebSocket connection. Restore or remove it during cleanup.
5. **Interception Discipline**: Do not enable proxy interception in unattended flows. The MCP HTTP controller requires a narrow `url_filter` or `in_scope_only: true`; set a bounded timeout, resolve pending messages, disable it, and restore original state upon completion.
6. **Rate & Concurrency Bounds**: Limit parallel requests (at most 32 via `burp_http` action `send_batch`), cap scan concurrency via custom resource pools (`burp_scan_config` action `upsert_pool`), and pace background job polling.
7. **Protect Secrets & Redact Evidence**: Never log or disclose private keys, session cookies, auth tokens, or Collaborator secrets in reports.
## Procedural Security Assessment Workflow

### 1. Connection & Scope Setup
1. Call `burp_burp_version` to verify Burp edition, extension version, capabilities (e.g. Scanner, Collaborator support), and message limits.
2. Call `burp_extension_info` if process arguments or JAR metadata are required.
3. Check target scope using `burp_target` action `get_scope`. Add authorized URLs via `burp_target` action `add_scope` only when explicitly permitted.
4. Check proxy interception state with `burp_settings` action `intercept_state`.

### 2. Attack Surface Reconnaissance & Inventory
1. Sample site map hosts and server technologies with `burp_target` action `info`.
2. Page through historical target entries with `burp_target` action `sitemap` using `url_prefix` filtering.
3. When SiteGraph is active (`--enable-sitegraph`), run `sitegraph` action `sync` with `url_prefix`, inspect `sitegraph` action `stats`, cluster endpoints with `sitegraph` action `clusters`, and search parameter names with `sitegraph` action `search`.

### 3. Traffic Analysis & Evidence Extraction
1. Search Proxy HTTP history with `burp_proxy` action `history` (compact metadata by default, `max_body_length: 4096`). Inspect full request/response or extract patterns via `burp_proxy` action `detail`.
2. Monitor overall Burp traffic across all tools (Repeater, Scanner, Intruder, Extensions) with `burp_logger` action `query` and `burp_logger` action `detail`.
3. Store critical pentest findings in Burp Organizer via `burp_organizer` action `add`, and list saved items with `burp_organizer` action `list`.
4. Compare HTTP response variations or verify authorization discrepancies with `burp_diff` action `diff_responses` (similarity score, header diffs, line diffs) or send to UI Comparer via `burp_diff` action `compare_exchanges`.
5. Review WebSocket frame history with `burp_proxy` action `websocket_history`.
6. Annotate or highlight critical requests with `burp_proxy` action `annotate` or `burp_proxy` action `highlight`.

### 4. Vulnerability Testing & Compound Workflows
Follow specific vulnerability methodologies in [`references/appsec-testing-guide.md`](references/appsec-testing-guide.md):

- **Compound Automated Workflows**:
  - **IDOR Verification**: Use `burp_verify_idor` to test authorization bypass between two user tokens/headers with automated response diffing.
  - **CORS Audit**: Use `burp_check_cors` to audit origin reflection, wildcard allowances, and credentials trust.
  - **Access Control Matrix**: Use `burp_auth_matrix` to evaluate role-based access control across multiple endpoints.
- **Manual Request Crafting**:
  - Replay modified requests via `burp_http` action `send`.
  - Send requests to Repeater UI with `burp_http` action `send_to_repeater`, using either an absolute `url` plus optional method/body/headers or a raw `request` (authority derives from `Host` when omitted).
  - Export requests to `curl` or Python `requests` using `burp_http` action `export`.
  - Convert request methods (GET ↔ POST) using `burp_http` action `convert`.
- **Differential & Parallel Testing**:
  - Execute up to 32 concurrent requests with `burp_http` action `send_batch` to test access controls, IDOR, or rate limits.
- **Race Conditions & Single-Packet Attack**:
  - Test limit overruns and concurrency bugs using `burp_fuzzer` action `race` with Last-Byte Synchronization (`single_packet_attack: true`).
- **Out-of-Band (OAST) & Auto-Correlation**:
  - Generate identifiers with `burp_collaborator` action `generate` (binding `target_url` and `injection_point`).
  - Poll for DNS/HTTP interactions with `burp_collaborator` action `poll` (returns auto-correlated origin metadata).
- **Live WebSockets**:
  - Open managed connections with `burp_websocket` action `create`, transmit text (`burp_websocket` action `send_text`) or base64 binary frames (`burp_websocket` action `send_binary`), inspect history (`burp_websocket` action `history`), and close with `burp_websocket` action `close`.

### 5. Multi-Marker Fuzzing & Payload Lists
1. **Multi-Marker Fuzzing**: Use `burp_fuzzer` action `fuzz` supporting `attack_mode` (`"pitchfork"`, `"cluster_bomb"`, `"sniper"`) and `markers` mapping.
2. **Payload Lists**: Create in-memory lists with `burp_fuzzer` action `create_payload_list` or import from text/JSON with `burp_fuzzer` action `import_payload_list`. Manage items with `burp_fuzzer` action `upsert_payloads`.
3. **Declarative Intruder Integration**: Register processors (`burp_fuzzer` action `register_payload_processor`) and generators (`burp_fuzzer` action `register_payload_generator`) or open raw requests in Intruder UI (`burp_fuzzer` action `send_to_intruder`).

### 6. Scanner Automation & Custom Checks
1. Start passive audits or bounded active scans using `burp_scanner` action `start_audit` or `start_crawl`.
2. Configure custom scan configurations (`burp_scan_config` action `upsert_config`) and rate-limiting resource pools (`burp_scan_config` action `upsert_pool`).
3. For long-running scans, crawls, or fuzzing jobs:
   - Poll progress with `burp_job_status`.
   - Retrieve paginated results upon completion with `burp_job_result`.
   - Cancel unfinished jobs with `burp_job_cancel` or `burp_scanner` action `stop`.
4. Page Scanner findings with `burp_scanner` action `list_issues`, detail issue evidence with `burp_scanner` action `issue_detail`, update issue status (False Positive / Ignored / Confirmed) with `burp_scanner` action `update_issue`, and generate HTML/XML reports with `burp_scanner` action `report`.
5. Dry-run and test BCheck rules against sample request/response exchanges using `burp_scanner` action `test_bcheck`.
### 7. Custom Extensions (Bambda & BCheck)
- **BChecks**: When creating custom declarative Scanner checks, load [`references/bcheck-authoring.md`](references/bcheck-authoring.md). Validate syntax and import with `burp_bcheck_import` (default `enabled: false`).
- **Bambdas**: Load [`references/bambda-authoring.md`](references/bambda-authoring.md) and import complete YAML with `burp_bambda_import`. Never embed large bundles: JVM `CONSTANT_Utf8` entries are limited to 65,535 bytes; use a bounded proxy rule or external streaming proxy.

### 8. Offline Transformations (Decoder)
For deterministic local encoding, decoding, hashing, compression, or parsing without network traffic, use `decoder`:
- Search operations with `query` or inspect arguments with `describe`.
- Execute single operations, multi-step recipes (`steps`), or deterministic `magic: true` decode suggestions.
- Tagged input: `text`, `bytes` (base64), or `json`.

---

## Evidence & Reporting Standard

Every validated finding must include:
1. **Vulnerability Classification**: Title, CWE identifier, and CVSS v3.1 score / severity.
2. **Target & Context**: Method, URL, parameter name, and authentication state.
3. **Reproducible Proof of Concept (PoC)**:
   - Complete HTTP request.
   - Decisive response headers, status code, and highlighted response body extract.
   - Out-of-band evidence (Collaborator interaction ID, client IP, interaction type, timestamp) when applicable.
4. **Impact Analysis**: Demonstrable security risk (not speculative).
5. **Remediation**: Specific, actionable fix instructions.

Optionally persist verified custom issues into Burp's site map using `burp_add_issue`.

---

## Mandatory Cleanup Protocol

Before completing any task, execute cleanup and report restored state:
- [ ] Reset proxy interception state (`burp_settings` action `set_intercept_state`).
- [ ] Remove temporary HTTP handlers (`burp_settings` action `remove_http_handler`).
- [ ] Remove temporary proxy rules (`burp_settings` action `remove_proxy_rule`).
- [ ] Remove temporary session rules (`burp_session` action `delete_rule`).
- [ ] Remove temporary macros (`burp_session` action `delete_macro`).
- [ ] Close all open managed WebSockets (`burp_websocket` action `close`).
- [ ] Remove temporary scope additions (`burp_target` action `remove_scope`).
- [ ] Cancel/remove any running background jobs (`burp_job_cancel`, `burp_scanner` action `remove`).
