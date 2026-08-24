# Burp MCP Live Feature Test Guide

This document is used to test `burp-mcp` against a running Burp Suite instance. Each section is an independent test case; the tester can mark it as `PASS`, `FAIL`, or `BLOCKED` and record the minimum required output.

## 0. Safety rules and scope

Burp MCP is a dual-use tool. Test only on applications and data that the tester owns or is explicitly authorized to access.

- Use a local target fixture or approved staging environment. Do not use production.
- Record the target boundary before testing: scheme, host, port, path.
- Enable Burp Proxy/HTTP history; disable interception before sending automated traffic.
- Do not include tokens, cookies, passwords, Collaborator payloads, or sensitive response bodies in the report.
- Every temporary scope entry, handler, proxy rule, session rule, macro, cookie, scan job, WebSocket, and payload list created during testing must be removed after the test.
- Keep each request bounded. Use the maximum page size allowed by the runtime schema; use `next_cursor` when a response has `truncated: true`.
- Test active scanning, crawling, fuzzing, race conditions, and Collaborator only when permitted by the scope/authorization.

## 1. Test environment and data preparation

### 1.1 Components

- Burp Suite Professional when testing Scanner, Collaborator, or Pro features.
- Java 25 extension `burp-mcp.jar` loaded under **Extensions → Installed**.
- Native MCP server:

```json
{
  "command": "/absolute/path/to/burp-mcp",
  "args": ["serve"]
}
```

- Default endpoint: `http://127.0.0.1:9877`.
- If the port is changed, the same value must be used for the extension (`BURP_MCP_GRPC_PORT` or `-Dburp.mcp.grpc.port`) and Rust (`BURP_MCP_GRPC_ENDPOINT`).

### 1.2 Recommended target fixture

Use a test application with the following endpoints:

- `GET /health` returns `200`.
- `GET /echo?value=one` echoes the query or body.
- `POST /echo` accepts JSON.
- `GET /redirect` returns a redirect.
- `GET /error` returns `500`.
- A WebSocket endpoint.
- An endpoint with an intentional issue for a Scanner issue or `burp_add_issue`.
- An endpoint with a sufficiently small request/response for fuzzing with no more than 3–5 payloads.

Set the placeholders before starting:

```text
TARGET_ORIGIN=https://target.test
TARGET_URL=https://target.test/health
TARGET_PREFIX=https://target.test/
TEST_COOKIE=burp-mcp-live-test
TEMP_HANDLER_HEADER=X-Burp-MCP-Live-Test
TEMP_RULE_ID=<record-after-create>
TEMP_JOB_ID=<record-after-start>
TEMP_WS_ID=<record-after-create>
TEMP_PAYLOAD_LIST_ID=<record-after-create>
```

## 2. Connection, metadata, and capability discovery

### LIVE-001 — `burp_burp_version`

**Objective:** confirm the MCP ↔ extension ↔ Burp connection.

```json
{}
```

**Expected:** successful response; includes the extension/Burp version, edition/capabilities, page limit, response limit, and RPC timeout.

**Evidence:** save the version; do not save sensitive information.

### LIVE-002 — `burp_extension_info`

```json
{}
```

**Expected:** returns extension/process metadata when supported.

### LIVE-003 — `burp_intercept_state`

```json
{}
```

**Expected:** the interception state can be read. Before any automated request, the value must be disabled; if enabled, stop and handle it manually.

### LIVE-004 — `burp_get_scope`

```json
{"url":"${TARGET_URL}"}
```

**Expected:** returns the correct `in_scope` value for the current Burp Target scope.

### LIVE-005 — `burp_target_info`

```json
{"url_prefix":"${TARGET_PREFIX}","limit":20}
```

**Expected:** host/technology summary from the Site map; bounded results.

## 3. Proxy history and Site map — read-only

### LIVE-010 — `burp_proxy_history`

```json
{"limit":20}
```

Method/status filters can be added according to the runtime schema.

**Expected:** history page; verify `total`, `truncated`, `next_cursor`, URL/method/status.

### LIVE-011 — `burp_proxy_history_filtered`

```json
{"url_filter":"${TARGET_PREFIX}","limit":20}
```

**Expected:** returns only entries matching the URL filter. Repeat using `cursor` if truncated.

### LIVE-012 — `burp_proxy_detail`

```json
{"index":<index_from_LIVE-010>}
```

**Expected:** request/response details match the selected entry. Do not include the full body in the report unless necessary.

### LIVE-013 — `burp_sitemap`

```json
{"url_prefix":"${TARGET_PREFIX}","limit":20}
```

**Expected:** Site map entries match the prefix; bounded page.

### LIVE-014 — `burp_scan_issues`

```json
{"url_prefix":"${TARGET_PREFIX}","limit":20}
```

**Expected:** list of existing issues and pagination.

### LIVE-015 — `burp_scan_issue_detail`

```json
{"index":<issue_index_from_LIVE-014>}
```

**Expected:** issue details match the index.

### LIVE-016 — `burp_proxy_intercept_config`

First call with `{}` to read the current configuration and save the baseline. Test mutations only while an operator is monitoring the Burp UI.

Safe change example:

```json
{"master_intercept_enabled":false}
```

**Expected:** response reflects the master state; automated traffic is not held. Restore the entire baseline after the test. Do not enable interception in an unattended test because MCP has no forward/drop operation.

## 4. HTTP requests, Repeater, annotations, and scope

### LIVE-020 — `burp_send_request`

```json
{
  "method":"GET",
  "url":"${TARGET_URL}",
  "headers":{"X-Burp-MCP-Live-Test":"read-only"}
}
```

**Expected:** status/response matches the fixture; the request appears in Proxy history if Burp is configured to record it.

### LIVE-021 — `burp_send_request_parallel`

Use no more than 3 requests identical to LIVE-020; do not use a production target.

**Expected:** response count matches the input count; each response preserves the correct order or has an identifying key according to the schema.

### LIVE-022 — `burp_send_to_repeater`

Send a selected fixture request.

**Expected:** action succeeds; verify in Burp Repeater or through an appropriate read API.

### LIVE-022B — `burp_send_to_intruder`

Send the raw fixture request, host, port, scheme, and test tab name.

```json
{
  "request":"GET /health HTTP/1.1\r\nHost: target.test\r\n\r\n",
  "host":"target.test",
  "port":443,
  "https":true,
  "tab_name":"burp-mcp-live-test"
}
```

**Expected:** action succeeds; the request appears in Intruder with the correct service and tab name. Do not start an attack outside an authorized bounded test case.

### LIVE-023 — `burp_highlight`

```json
{"index":<history_index>,"color":"red"}
```

**Expected:** action succeeds; call `burp_proxy_detail` or use the Burp UI to confirm the color annotation.

### LIVE-024 — `burp_annotate`

```json
{"index":<history_index>,"note":"burp-mcp live test"}
```

**Expected:** the note is saved; read the history/details again to verify.

### LIVE-025 — `burp_add_to_scope` / `burp_remove_from_scope`

```json
{"url":"${TARGET_URL}"}
```

**Expected:** the scope check transitions to the correct state and then returns to its initial state. Use only an authorized URL; record whether it was already in scope.

## 5. Cookie jar

### LIVE-030 — `burp_cookie_jar`

```json
{"url":"${TARGET_URL}"}
```

**Expected:** reads the cookie jar applicable to the URL. Redact cookie values in the report.

### LIVE-031 — `burp_cookie_jar_set`

```json
{
  "url":"${TARGET_URL}",
  "name":"burp_mcp_live_test",
  "value":"temporary",
  "domain":"target.test",
  "path":"/"
}
```

**Expected:** set succeeds; read it back using `burp_cookie_jar`; then delete the cookie through the Burp UI/API if supported by the schema. Do not use a real cookie.

## 6. Configuration, handlers, proxy rules, and session rules

### LIVE-040 — `burp_export_config` / `burp_inspect_config`

```json
{"paths":[]}
```

**Expected:** export returns valid JSON; inspect returns leaf paths and `size_bytes`. Do not commit or send a configuration containing secrets.

### LIVE-041 — `burp_import_config`

Use a minimal project-options JSON file that was backed up beforehand.

**Expected:** import succeeds; verify one harmless setting. Restore the backup after the test.

### LIVE-042 — `burp_register_http_handler` / `burp_remove_http_handler`

Register a handler that only adds the `X-Burp-MCP-Live-Test: true` header on the target fixture.

**Expected:** registration succeeds; send a request and verify the header/behavior; remove the handler; send another request and verify that the handler no longer runs.

### LIVE-043 — `burp_register_proxy_rule` / `burp_list_proxy_rules` / `burp_remove_proxy_rule`

Create a harmless rule that matches only `${TARGET_PREFIX}` and modifies a test note/header.

**Expected:** the rule appears in the list; fixture traffic demonstrates the effect; after removal, the rule no longer appears in the list.

### LIVE-044 — Session rule CRUD

Use `burp_session_create_rule` to create a harmless rule scoped to `${TARGET_PREFIX}`. Record the returned stable `id`, then exercise `burp_session_get_rule`, `burp_session_update_rule`, `burp_session_list_rules`, and `burp_session_delete_rule`.

**Expected:** create/get/list return the same ID and fields; update replaces the selected rule without affecting other rules; delete removes only the selected ID; get after delete returns not found.

## 7. Macro

### LIVE-050 — `burp_macro_create`

Create a macro containing 1–2 fixture requests with no secrets.

**Expected:** creation succeeds and returns a serial/identifier.

### LIVE-051 — `burp_macro_list`

**Expected:** the newly created macro appears.

### LIVE-052 — `burp_macro_run`

Run the macro against the fixture.

**Expected:** response/result is correct; verify the request count or history.

### LIVE-053 — `burp_macro_remove`

Delete the newly created macro.

**Expected:** removal succeeds; the macro no longer appears in the list.

## 8. Intruder, bounded fuzzing, and race jobs

### LIVE-060 — Payload processor CRUD

Use:

- `burp_intruder_payload_processor_register`
- `burp_intruder_payload_processor_list`
- `burp_intruder_payload_processor_remove`

**Expected:** the processor appears with the correct configuration and disappears after removal. Do not register a processor with side effects outside the fixture.

### LIVE-061 — Payload generator CRUD

Use:

- `burp_intruder_payload_generator_register`
- `burp_intruder_payload_generator_list`
- `burp_intruder_payload_generator_remove`

**Expected:** generator list/remove follows the correct lifecycle.

### LIVE-062 — `burp_payload_list_create`

Create a list with no more than 3 payloads: `one`, `two`, `three`.

**Expected:** returns the list ID, count, and fingerprint.

### LIVE-063 — `burp_payload_list_get`

Retrieve the list by ID, first page.

**Expected:** payload count/content is correct; pagination is correct.

### LIVE-064 — `burp_payload_list_update`

Add/remove one test payload.

**Expected:** count/fingerprint changes correctly; retrieve the list again to verify.

### LIVE-065 — `burp_payload_list_import`

Import a small text/JSON payload list.

**Expected:** the list is created successfully with the selected format; verify through get.

### LIVE-066 — `burp_payload_list_list` / `burp_payload_list_delete`

**Expected:** the ID appears in the list; after deletion, the ID disappears.

### LIVE-067 — `burp_inline_fuzzer`

Use a fixture request containing the `FUZZ` marker, a wordlist with no more than 3 entries, and the correct host/port/scheme.

**Expected:** returns a job ID; use `burp_job_status` and `burp_job_result` until a terminal state; request count and substitution count are correct.

### LIVE-068 — `burp_race_condition`

Use a harmless fixture request and the smallest count allowed by the schema.

**Expected:** returns a job ID; status/result is bounded; do not use an endpoint that creates real data.

## 9. Scanner, crawl, and scan catalog

### LIVE-070 — `burp_scan_config_list`

```json
{}
```

**Expected:** includes built-in configurations: lightweight, fast, balanced, deep, and passive snapshot; each entry has scan type, audit type, timing, pool ID, and source.

### LIVE-071 — `burp_scan_config_get`

```json
{"id":"built-in-passive"}
```

**Expected:** returns the exact configuration; the built-in entry is immutable.

### LIVE-072 — `burp_scan_config_create`

```json
{
  "name":"Live test passive",
  "scan_type":"audit",
  "audit_type":"passive",
  "timeout_seconds":30,
  "stable_seconds":0,
  "resource_pool_id":"built-in-default"
}
```

**Expected:** entry with `source: extension` and a new ID. Record the ID for update/delete.

### LIVE-073 — `burp_scan_config_update` / `burp_scan_config_delete`

Change the name/timing to valid values, then delete the test entry.

**Expected:** update is reflected by get; delete succeeds; get after deletion returns not found.

### LIVE-074 — `burp_scan_pool_list`

**Expected:** includes `built-in-default`; clearly returns `scanner_supported` and `support_message`. With the current Montoya API, Scanner resource-pool binding may be reported as unsupported.

### LIVE-075 — `burp_scan_pool_create` / `burp_scan_pool_get` / `burp_scan_pool_update` / `burp_scan_pool_delete`

Create a bounded private test pool, for example with concurrency 1, throttle 100 ms, and retries 0.

**Expected:** CRUD persistence is correct; do not delete a pool referenced by a configuration; delete the configuration first, then delete the pool.

### LIVE-076 — `burp_scan_start` passive

```json
{
  "url":"${TARGET_URL}",
  "audit_type":"passive",
  "include_out_of_scope":false
}
```

**Expected:** completed stateless snapshot, operation `scanner_passive_snapshot`, `scan_type: passive`, `stateless: true`; `burp_job_result` includes the issue count.

### LIVE-077 — `burp_scan_start` active

Run only against an authorized target; set `audit_type: active`, configure the smallest timeout sufficient for the test, and set `include_out_of_scope: true` only when the target is out of scope.

**Expected:** job ID with operation `scanner_audit`; poll `burp_job_status`; retrieve `burp_job_result`; then call `burp_scan_stop` if it is still running and `burp_scan_remove` when terminal.

### LIVE-078 — `burp_crawl`

```json
{
  "seed_urls":["${TARGET_URL}"],
  "timeout_seconds":60,
  "stable_seconds":2,
  "include_out_of_scope":false
}
```

**Expected:** job ID; bounded crawl; request/error counts; cleanup when terminal. An out-of-scope seed must fail without explicit opt-in.

### LIVE-079 — Generic job tools

Use a job ID from LIVE-067, LIVE-068, LIVE-076, LIVE-077, or LIVE-078:

- `burp_job_status`
- `burp_job_result`
- `burp_job_cancel`

**Expected:** state transitions are correct; result is not returned before the job finishes if the contract requires a terminal state; cursor/page works; cancel does not turn into a late completion.

## 10. Collaborator

Run only with an authorized Collaborator server/project.

### LIVE-080 — `burp_collaborator_generate`

```json
{"count":1}
```

**Expected:** returns the correct bounded number of identifiers. Redact the payload from the report.

### LIVE-081 — `burp_collaborator_poll`

```json
{"limit":20}
```

**Expected:** bounded response; page metadata is correct. Do not run an infinite polling loop.

## 11. WebSocket

### LIVE-090 — `burp_websocket_create`

Create a connection to the WebSocket fixture.

**Expected:** returns a WebSocket ID.

### LIVE-091 — `burp_websocket_send_text`

Send the text `burp-mcp-live-test`.

**Expected:** send succeeds; the peer/server receives the correct message.

### LIVE-092 — `burp_websocket_send_binary`

Send small test bytes, for example `01 02 03`.

**Expected:** the binary message round-trip preserves the exact bytes.

### LIVE-093 — `burp_websocket_history`

**Expected:** history includes text/binary direction and a bounded page.

### LIVE-094 — `burp_websocket_list` / `burp_websocket_close`

**Expected:** the ID appears before close and disappears/is closed after close.

### LIVE-095 — `burp_proxy_websocket_history`

**Expected:** WebSocket history can be read from Proxy/Burp and is distinguishable from managed WebSocket history.

## 12. Bambda, BCheck, and scanner findings

### LIVE-100 — `burp_bambda_import`

Import a harmless script but do not execute it.

**Expected:** import/validation status is clear; no traffic or automatic side effects occur.

### LIVE-101 — `burp_bcheck_import`

Import a harmless BCheck in a disabled state if supported by the schema.

**Expected:** status/errors are clear; it does not run automatically.

### LIVE-102 — `burp_add_issue`

Add a synthetic issue to the fixture URL with no secrets.

**Expected:** action succeeds; the issue appears in `burp_scan_issues`; details can be read using `burp_scan_issue_detail`.

### LIVE-103 — `burp_scanner_generate_report`

Generate a report from the fixture issue using a small format.

**Expected:** report succeeds, with correct file/output metadata. Use a temporary directory and delete it after the test.

## 13. Offline HTTP/data utilities

These tests do not require target traffic and should run before active tests.

### LIVE-110 — `burp_convert_request`

Convert the fixture `GET` request to `POST`.

**Expected:** method/body/header transformation is correct and does not corrupt the HTTP syntax.

### LIVE-111 — `burp_export_request`

Export the fixture request in raw/text form.

**Expected:** valid raw request; binary-safe if the input contains bytes.

### LIVE-112 — `burp_extract_from_response`

Extract the status/header/body fragment from the fixture response.

**Expected:** the value matches the fixture; output limits are bounded.

### LIVE-113 — Decoder/utility

Use the corresponding decoder/runtime-schema tool for a JSON, base64, URL, or raw HTTP sample.

**Expected:** deterministic output; no network, filesystem, browser, or arbitrary-code side effects.

## 14. Sitegraph

Sitegraph stores endpoint metadata and parameter names; it does not store parameter values or message bodies.

### LIVE-120 — `sitegraph_status`

**Expected:** graph path/project/index status, node/edge counts.

### LIVE-121 — `sitegraph_sync`

Sync from the Burp Site map using `TARGET_PREFIX` or bounded input.

**Expected:** endpoint count increases/remains correct; parameter values/bodies are not written.

### LIVE-122 — `sitegraph_search`

Search endpoint/method/parameter metadata.

**Expected:** finds the fixture endpoint; secret/body values are not visible.

### LIVE-123 — `sitegraph_endpoint_detail`

Retrieve details for an endpoint from the search result.

**Expected:** method/path/host/parameters/metadata; no body/token.

### LIVE-124 — `sitegraph_neighbors`

**Expected:** returns bounded relationships between endpoint nodes.

### LIVE-125 — `sitegraph_trace`

Trace a call/data path using a valid node.

**Expected:** correct path/edges, or a clearly empty result if no path exists.

### LIVE-126 — `sitegraph_shortest_path`

**Expected:** bounded shortest path with no loop.

### LIVE-127 — `sitegraph_clusters`

**Expected:** stable and bounded cluster summary.

### LIVE-128 — `sitegraph_impact`

**Expected:** impact set by node/endpoint; no body/value leakage.

### LIVE-129 — `sitegraph_diff`

Compare two snapshot/project graphs.

**Expected:** added/removed/changed metadata is correct.

### LIVE-130 — `sitegraph_export`

Export graph metadata to a temporary directory.

**Expected:** output is readable and contains only published graph data; delete the file after the test.

### LIVE-131 — `sitegraph_config` / `sitegraph_projects` / `sitegraph_stats`

**Expected:** config/project/statistics read/write according to the schema; no unintended project changes.

## 15. Cleanup checklist

After each test group:

- [ ] Delete temporary scope entries.
- [ ] Disable/remove HTTP handlers.
- [ ] Delete proxy rules and session rules.
- [ ] Delete macros.
- [ ] Delete test cookies.
- [ ] Delete payload lists.
- [ ] Stop/remove background jobs.
- [ ] Close managed WebSockets.
- [ ] Restore the imported Burp config.
- [ ] Delete synthetic issues if the Burp API/UI allows it.
- [ ] Delete report/export files from the temporary directory.
- [ ] Do not leave interception enabled if the tester changed the state.

## 16. Report template

```markdown
# Burp MCP Live Test Report

- Date:
- Tester:
- Burp version/edition:
- burp-mcp version:
- OS/arch:
- MCP client:
- gRPC endpoint:
- Authorized target boundary:
- Burp scope before test:

## Summary

- PASS:
- FAIL:
- BLOCKED:
- Not run:

## Cases

| ID | Feature/tool | Status | Input summary | Expected | Actual | Evidence | Cleanup |
| --- | --- | --- | --- | --- | --- | --- | --- |
| LIVE-001 | `burp_burp_version` | PASS/FAIL/BLOCKED | metadata | connection info | ... | screenshot/log | N/A |

## Failures

For every failure include:

- exact tool name and sanitized input;
- timestamp;
- sanitized response/error;
- Burp edition/version;
- whether traffic or Burp state changed;
- cleanup result;
- reproducibility count.

## Security notes

- No secrets included:
- Target authorization reference:
- Temporary state restored:
```

## 17. Known limitations

- The gRPC service listens on `127.0.0.1` without application-level authentication. Any local process can connect while Burp is running.
- Scanner active/crawl behavior depends on Burp edition, project state, scope and Montoya API support.
- Montoya API `2026.7` does not expose resource-pool binding for `Scanner.startCrawl`/`Scanner.startAudit`; resource-pool CRUD is available for cataloging and explicit capability reporting, but unsupported bindings must not be reported as applied.
- Full Rust workspace tests require repository contract fixtures that may be supplied separately in the test environment.
- Runtime MCP schemas are authoritative. If a deployed binary exposes a different field name or omits a tool, mark the case `BLOCKED` and record the advertised schema instead of inventing input.