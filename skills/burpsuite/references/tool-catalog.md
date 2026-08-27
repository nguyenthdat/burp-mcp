# Burp MCP Tool Catalog

Load this reference only when selecting exact tools or constructing arguments.
The live MCP schema takes precedence over this catalog. Client-visible names
may include an MCP server prefix; match by the server-local suffix shown here.

## Global constraints and semantics

- Burp-facing tools require Burp Suite with the Burp MCP extension running.
  Edition and `burp_burp_version.capabilities` determine feature availability.
- List limits are at most `500`; use `next_cursor` or `cursor` when returned.
- Proxy/scanner indexes must be at most `2147483647`.
- `burp_send_request_parallel` accepts at most `32` requests per call.
- Sitegraph limits are `1..500`; trace depth is `1..8`.
- Utility input is bounded to 16 MiB; recipes are bounded to 64 steps.
- Raw HTTP returned by `burp_proxy_detail` is text-oriented. Do not assume a
  byte-exact round trip for arbitrary binary messages.
- `burp_send_to_repeater` and `burp_send_to_intruder` open UI items; they do not
  send or attack.
- `burp_scan_start` supports passive audit or active audit (when supported by Burp edition).
- Treat `success: true` as acceptance, then verify the external effect.

Fields ending in `?` are optional. `{}` means no arguments.

---

## 1. Connection and project metadata (5 tools)

| Tool | Input | Purpose |
|---|---|---|
| `burp_burp_version` | `{}` | Extension/Burp version, edition, capabilities, and runtime limits. |
| `burp_extension_info` | `{}` | Extension filename, BApp status, and process arguments. |
| `burp_export_config` | `{}` | Export project configuration JSON. |
| `burp_import_config` | `{config}` | Import project configuration JSON; project-wide mutation. |
| `burp_inspect_config` | `{paths?}` | Export scoped project options with discovered leaf paths and UTF-8 size before import. |

---

## 2. Scope, target, and site map (5 tools)

| Tool | Input | Purpose |
|---|---|---|
| `burp_get_scope` | `{url}` | Check one URL against current Burp scope. |
| `burp_add_to_scope` | `{url}` | Add one URL to target scope. Cleanup: `burp_remove_from_scope`. |
| `burp_remove_from_scope` | `{url}` | Remove one URL from target scope. |
| `burp_target_info` | `{url?, limit?}` | Summarize hosts and technology headers from a bounded site map sample. |
| `burp_sitemap` | `{url_prefix?, limit?, cursor?}` | Page through Burp site map entries. |

---

## 3. Proxy HTTP and WebSocket evidence (6 tools)

| Tool | Input | Purpose |
|---|---|---|
| `burp_proxy_history` | `{limit?, offset?, cursor?, url_filter?, method_filter?, status_filter?, has_notes?, color?}` | Page/filter HTTP history. Prefer `cursor` after the first page. |
| `burp_proxy_detail` | `{index}` | Read the raw request/response, notes, and highlight for one index. |
| `burp_proxy_websocket_history` | `{limit?, cursor?}` | Read observed WebSocket messages; payloads are base64. |
| `burp_highlight` | `{index, color?}` | Persist a Proxy history highlight. Empty/default color clears it. |
| `burp_annotate` | `{index, note}` | Persist a Proxy history note. |
| `burp_extract_from_response` | `{index, regex, limit?}` | Extract bounded regex matches from one response. |

---

## 4. Sending and preparing HTTP requests (6 tools)

| Tool | Input | Purpose |
|---|---|---|
| `burp_send_request` | `{url, method?, body?, headers?}` | Send one structured HTTP request through Burp. |
| `burp_send_request_parallel` | `{requests}` | Send up to 32 requests concurrently. |
| `burp_send_to_repeater` | `{request, host, port?, https?, tab_name?}` | Display one raw request in Repeater without sending. |
| `burp_race_condition` | `{request, host, port?, https?, count?}` | Start a bounded concurrent comparison job. |
| `burp_convert_request` | `{request, convert_to?}` | Convert a raw HTTP request method; default target is POST. |
| `burp_export_request` | `{request, host?, format?, https?}` | Export as `curl` or Python `requests` code per the live schema. |

For a raw request, keep `Host`, path, body framing, target host, port, and `https` consistent. When using structured sends, pass a full URL.

---

## 5. Handlers, interception, and proxy settings (11 tools)

| Tool | Input | Purpose and cleanup |
|---|---|---|
| `burp_intercept_state` | `{}` | Read the current Proxy interception state. |
| `burp_set_intercept_state` | `{enabled}` | Set Proxy interception state. Read and restore the original state around temporary changes. |
| `burp_register_http_handler` | `{header_name?, header_value?, match_text?, replace?}` | Add-header or replace-text rule. Cleanup: `burp_remove_http_handler`. |
| `burp_remove_http_handler` | `{}` | Clear HTTP handler rules. |
| `burp_register_proxy_rule` | `{url_contains, id?, phase?, action?, match_text?, replace?, header_name?, header_value?, enabled?}` | Create/replace a request or response rule. Actions: `forward`, `intercept`, `drop`, `edit`. Cleanup: `burp_remove_proxy_rule`. |
| `burp_list_proxy_rules` | `{}` | List configured Proxy rules and their enabled state. |
| `burp_remove_proxy_rule` | `{id?}` | Remove one rule by ID, or clear all rules when `id` is omitted. |
| `burp_proxy_intercept_config` | `{}` | Legacy focused read of Proxy request, response, WebSocket interception filters and response modification settings. Prefer `burp_proxy_settings` for new workflows. |
| `burp_update_proxy_intercept_config` | `{master_intercept_enabled?, request_do_intercept?, request_auto_content_length?, request_fix_missing_new_lines?, response_do_intercept?, response_auto_content_length?, websocket_client_to_server?, websocket_server_to_client?, websocket_in_scope_only?, request_rules?, response_rules?, replace_request_rules?, replace_response_rules?, ...}` | Legacy bulk patch; replacing rule arrays requires the matching `replace_*_rules` flag. Prefer granular `burp_update_proxy_settings` operations. |
| `burp_proxy_settings` | `{}` | Read listeners, script filters, and request/response interception settings together. |
| `burp_update_proxy_settings` | `{operation, port?, running?, listen_mode?, listen_specific_address?, certificate_mode?, enable_http2?, support_invisible_proxying?, target?, mode?, script?, script_id?, script_name?, kind?, index?, rule?, master_enabled?, request_enabled?, response_enabled?}` | Unified mutation tool. Operations: `listener_upsert`, `listener_delete`, `script_filter_upsert`, `script_filter_delete`, `intercept_rule_upsert`, `intercept_rule_delete`, `intercept_toggle`. |

## MCP-owned interception queues (6 tools)

| Tool | Input | Purpose and cleanup |
|---|---|---|
| `burp_intercept_controller` | `{enabled?, timeout_seconds?}` | Read/configure the MCP-owned HTTP queue. Pending messages auto-forward on timeout; disable and drain it during cleanup. |
| `burp_intercepted_messages` | `{limit?, cursor?}` | Page pending HTTP requests/responses, including lossless base64 messages. |
| `burp_control_intercepted_message` | `{id, action, message_base64?}` | Forward, drop, or send one paused HTTP message to Burp's manual Intercept tab; optionally replace the complete message. |
| `burp_websocket_intercept_controller` | `{enabled?, timeout_seconds?}` | Read/configure the MCP-owned WebSocket queue. |
| `burp_intercepted_websocket_messages` | `{limit?, cursor?}` | Page pending WebSocket text/binary messages. |
| `burp_control_intercepted_websocket_message` | `{id, action, payload_base64?}` | Forward, drop, or send one paused WebSocket message to Burp's manual Intercept tab; optionally replace its payload. |

---

## 6. Cookies (2 tools)

| Tool | Input | Purpose |
|---|---|---|
| `burp_cookie_jar` | `{limit?, domain?}` | List cookies in cookie jar; values are sensitive. |
| `burp_cookie_jar_set` | `{name, value, domain, path?, expiration?}` | Set a cookie. Verify by listing it; expire temporary cookies during cleanup. |

---

## 7. Sessions and macros (9 tools)

| Tool | Input | Purpose |
|---|---|---|
| `burp_session_create_rule` | `{id?, description?, action_type?, find?, replace?, header_name?, parameter_name?, macro_description?, url_contains?, tools?, enabled?}` | Create a session rule and return its stable ID. |
| `burp_session_get_rule` | `{id}` | Get one session rule by stable ID. |
| `burp_session_update_rule` | `{id?, description?, action_type?, find?, replace?, header_name?, parameter_name?, macro_description?, url_contains?, tools?, enabled?}` | Replace one session rule by ID. |
| `burp_session_list_rules` | `{}` | List registered session rules and scope. |
| `burp_session_delete_rule` | `{id}` | Delete one session rule by ID. |
| `burp_macro_create` | `{description, serial_number?, items}` | Create or replace a Burp Settings > Sessions > Macros definition. |
| `burp_macro_list` | `{}` | List Burp session macros. |
| `burp_macro_run` | `{description}` | Execute requests from one named macro. |
| `burp_macro_remove` | `{description}` | Remove one named macro. |

---

## 8. Intruder and bounded payloads (8 tools)

| Tool | Input | Purpose |
|---|---|---|
| `burp_inline_fuzzer` | `{template, host, port?, https?, marker?, wordlist, payload_list_id?, payload_offset?}` | Start a bounded single-marker request matrix job. |
| `burp_send_to_intruder` | `{request, host, port?, https?, tab_name?}` | Open one request in Burp Intruder without starting an attack. |
| `burp_intruder_payload_processor_register` | `{id, display_name, operation, argument?, replacement?}` | Register one bounded declarative Intruder payload processor. |
| `burp_intruder_payload_processor_list` | `{}` | List registered declarative Intruder payload processors. |
| `burp_intruder_payload_processor_remove` | `{id}` | Deregister one declarative Intruder payload processor. |
| `burp_intruder_payload_generator_register` | `{id, display_name, payloads, max_output_count?, payload_list_id?, payload_offset?}` | Register one bounded declarative Intruder payload generator. |
| `burp_intruder_payload_generator_list` | `{}` | List registered declarative Intruder payload generators. |
| `burp_intruder_payload_generator_remove` | `{id}` | Deregister one declarative Intruder payload generator. |

---

## 9. Payload lists (6 tools)

| Tool | Input | Purpose |
|---|---|---|
| `burp_payload_list_create` | `{id, display_name, payloads}` | Create one bounded in-memory payload list. |
| `burp_payload_list_import` | `{id, display_name, content, format?, keep_empty?}` | Import a bounded payload list from newline text or JSON string array. |
| `burp_payload_list_list` | `{}` | List bounded in-memory payload lists. |
| `burp_payload_list_get` | `{id, offset?, limit?}` | Read one bounded page from a payload list. |
| `burp_payload_list_update` | `{id, operation, payloads?, index?, indexes?, display_name?}` | Append, prepend, insert, replace, remove, clear, or rename a payload list. |
| `burp_payload_list_delete` | `{id}` | Delete one payload list. |

---

## 10. Scanner execution and findings (7 tools)

| Tool | Input | Purpose |
|---|---|---|
| `burp_scan_start` | `{url, audit_type?, scan_configuration_id?, resource_pool_id?, timeout_seconds?, stable_seconds?, include_out_of_scope?}` | Start passive stateless audit or active audit with bounded scan options. |
| `burp_scan_stop` | `{job_id}` | Stop a running active Burp audit by job ID. |
| `burp_scan_remove` | `{job_id}` | Remove a terminal Burp audit, passive snapshot, or crawl by job ID. |
| `burp_crawl` | `{seed_urls, scan_configuration_id?, resource_pool_id?, timeout_seconds?, stable_seconds?, include_out_of_scope?}` | Start a bounded Burp crawl with explicit seeds, configuration, and scope. |
| `burp_scan_issues` | `{limit?, cursor?, severity_filter?, confidence_filter?, url_filter?, index?}` | Page through Burp Scanner issues. |
| `burp_scan_issue_detail` | `{index}` | Read complete details for one Scanner issue index. |
| `burp_scanner_generate_report` | `{format, path, issue_indexes?}` | Generate an HTML or XML Burp Scanner report for selected issue indexes (or all issues when omitted). |

---

## 11. Scanner configuration and resource pools (10 tools)

| Tool | Input | Purpose |
|---|---|---|
| `burp_scan_config_list` | `{}` | List built-in and project-persisted scan configurations. |
| `burp_scan_config_get` | `{id}` | Get one scan configuration by ID. |
| `burp_scan_config_create` | `{id?, name, scan_type, audit_type?, include_out_of_scope?, timeout_seconds?, stable_seconds?, resource_pool_id?}` | Create a bounded persisted scan configuration. |
| `burp_scan_config_update` | `{id?, name, scan_type, audit_type?, include_out_of_scope?, timeout_seconds?, stable_seconds?, resource_pool_id?}` | Update a persisted scan configuration by ID. |
| `burp_scan_config_delete` | `{id}` | Delete a persisted scan configuration by ID. |
| `burp_scan_pool_list` | `{}` | List scanner resource pool definitions and runtime support. |
| `burp_scan_pool_get` | `{id}` | Get one scanner resource pool definition by ID. |
| `burp_scan_pool_create` | `{id?, name, kind, existing_pool_name?, concurrent_request_limit?, throttle_millis?, max_retries?}` | Create a persisted scanner resource pool definition. |
| `burp_scan_pool_update` | `{id?, name, kind, existing_pool_name?, concurrent_request_limit?, throttle_millis?, max_retries?}` | Update a persisted scanner resource pool definition. |
| `burp_scan_pool_delete` | `{id}` | Delete a persisted scanner resource pool definition. |

---

## 12. Background jobs (3 tools)

| Tool | Input | Purpose |
|---|---|---|
| `burp_job_status` | `{job_id}` | Read current job state and error. |
| `burp_job_result` | `{job_id, limit?, cursor?}` | Page results and summary counters. |
| `burp_job_cancel` | `{job_id}` | Cancel a queued/running job. |

---

## 13. Collaborator and custom findings (3 tools)

| Tool | Input | Purpose |
|---|---|---|
| `burp_collaborator_generate` | `{count?}` | Generate a bounded set of Collaborator payloads. |
| `burp_collaborator_poll` | `{limit?, cursor?}` | Page interactions from the extension's active Collaborator context. |
| `burp_add_issue` | `{name, url, detail?, remediation?, severity?, confidence?}` | Persist one validated typed issue in Burp. |

---

## 14. Managed WebSockets (6 tools)

| Tool | Input | Purpose |
|---|---|---|
| `burp_websocket_create` | `{host, port?, https?, path?}` | Create a managed WebSocket through Burp. |
| `burp_websocket_send_text` | `{id, text}` | Send one text message. |
| `burp_websocket_send_binary` | `{id, data}` | Send base64-encoded binary data. |
| `burp_websocket_history` | `{id?, limit?, cursor?}` | Read messages sent to or received from managed WebSocket connections. |
| `burp_websocket_close` | `{id}` | Close one managed connection. |
| `burp_websocket_list` | `{}` | List active managed connection IDs. |

Cleanup pair: every successful create must end with close unless the user asks to leave the connection open.

---

## 15. Script imports (2 tools)

| Tool | Input | Purpose |
|---|---|---|
| `burp_bambda_import` | `{script}` | Import a complete Bambda YAML document without executing it. |
| `burp_bcheck_import` | `{script, enabled?}` | Import a complete BCheck definition without running a scan. Default authoring workflow: `enabled: false`. |

Imports persist in Burp. Treat source as executable code: review it and require explicit user intent. Import success requires `success: true`, status `LOADED_WITHOUT_ERRORS`, and an empty error list. The current MCP surface has no list, test, run, enable/disable-after-import, or delete operation for imported scripts. For authoring and review rules, load [`bambda-authoring.md`](./bambda-authoring.md) or [`bcheck-authoring.md`](./bcheck-authoring.md).

---

## 16. Persistent sitegraph (15 tools, Advanced Opt-in)

Sitegraph is disabled by default in v3 (`--enable-sitegraph` to enable).

| Tool | Input | Purpose |
|---|---|---|
| `sitegraph_sync` | `{url_prefix?}` | Synchronize bounded sitemap, history, issue, technology, and WebSocket observations into local SQLite. |
| `sitegraph_status` | `{}` | Read synchronization/schema status. |
| `sitegraph_stats` | `{}` | Read node/edge counts and last sync time. |
| `sitegraph_config` | `{}` | Read active auto-index configuration; edit configuration and restart to change it. |
| `sitegraph_projects` | `{}` | List the active project-scoped graph identity. |
| `sitegraph_search` | `{query, limit?, cursor?}` | Search normalized endpoints. |
| `sitegraph_history_search` | `{query, source?, limit?, cursor?}` | Search indexed raw evidence; `source` is `all`, `http`, or `websocket`. |
| `sitegraph_endpoint_detail` | `{id}` | Read one endpoint by stable ID. |
| `sitegraph_neighbors` | `{id, limit?, cursor?}` | Page adjacent nodes. |
| `sitegraph_trace` | `{id, max_depth?, limit?}` | Traverse relationships to depth 1..8. |
| `sitegraph_shortest_path` | `{from_id, to_id, max_depth?}` | Find shortest directed path between two nodes. |
| `sitegraph_clusters` | `{limit?}` | Cluster active project endpoints by origin and path segments. |
| `sitegraph_impact` | `{id, max_depth?, limit?}` | List bounded downstream impact from one active project graph node. |
| `sitegraph_diff` | `{since, limit?, cursor?}` | Read nodes changed since a Unix timestamp. |
| `sitegraph_export` | `{profile?, format?, snapshot_id?, cursor?, limit?}` | Export bounded metadata or exact-evidence pages. Exact evidence contains sensitive base64 traffic. |

Sync before querying when freshness matters. Normalized metadata excludes parameter values, but indexed HTTP/WebSocket evidence and `profile=exact` exports can contain sensitive raw traffic.

---

## 17. Offline `decoder` (1 tool)

Input is one tagged value:

```json
{"kind":"text","value":"..."}
{"kind":"bytes","base64":"..."}
{"kind":"json","value":{"key":"value"}}
```

Select exactly one mode per call:

```json
{"input": {"kind":"text","value":"SGVsbG8="}, "operation":"base64.decode", "args":{}}
{"input": {"kind":"text","value":"jwt"}, "describe":"jwt.decode"}
{"input": {"kind":"text","value":"..."}, "query":"gzip"}
{"input": {"kind":"text","value":"..."}, "magic":true}
{"input": {"kind":"text","value":"..."}, "steps":[{"op":"url.decode","args":{}},{"op":"base64.decode","args":{}}]}
```

Available operation IDs:

- Encoding: `base64.encode`, `base64.decode`, `base64url.encode`, `base64url.decode`, `hex.encode`, `hex.decode`, `url.encode`, `url.decode`, `html.encode`, `html.decode`, `unicode.escape`, `unicode.unescape`.
- JSON/text: `json.pretty`, `json.minify`, `json.query`, `text.uppercase`, `text.lowercase`, `text.reverse`, `text.split`, `text.join`, `regex.extract`, `regex.replace`.
- Inspection/hashes: `entropy`, `strings.extract`, `length`, `md5`, `sha1`, `sha256`, `sha512`, `blake3`, `hmac.sha256`, `hmac.sha512`.
- Compression: `gzip.compress`, `gzip.decompress`, `zlib.compress`, `zlib.decompress`, `deflate.compress`, `deflate.decompress`, `brotli.compress`, `brotli.decompress`.
- Web/security: `jwt.decode`, `jwt.verify_hs256`, `cookie.parse`, `query.parse`, `query.build`, `http.parse`, `http.set_body`, `http.update_content_length`.

Use `describe` before supplying operation-specific `args` unless the live schema or a previous result already established them. JWT decode does not verify a signature. MD5 and SHA-1 are marked cryptographically weak.

---

## Cleanup checklist

- Restore original intercept state.
- Remove temporary scope entries, HTTP/proxy/session rules, and macros.
- Expire temporary cookies and verify the cookie jar.
- Close every managed WebSocket.
- Cancel unfinished jobs and record the final state.
- Leave annotations, imported scripts/checks, configuration, or created issues only when the user explicitly requested persistence.
