# Burp MCP Tool Catalog

Load this reference only when selecting exact tools or constructing arguments.
The live MCP schema takes precedence over this catalog. Client-visible names
may include an MCP server prefix; match by the server-local suffix shown here.

## Global constraints and semantics

- Burp-facing tools require Burp Suite with the Burp MCP extension running.
  Edition and `burp_burp_version.capabilities` determine feature availability.
- List limits are at most `500`; use `next_cursor` when returned.
- Proxy/scanner indexes must be at most `2147483647`.
- `burp_send_request_parallel` accepts at most `32` requests per call.
- Sitegraph limits are `1..500`; trace depth is `1..8`.
- Utility input is bounded to 16 MiB; recipes are bounded to 64 steps.
- Raw HTTP returned by `burp_proxy_detail` is text-oriented. Do not assume a
  byte-exact round trip for arbitrary binary messages.
- `burp_send_to_repeater` and `burp_send_to_intruder` open UI items; they do not
  send or attack.
- `burp_scan` supports passive mode only. Active mode is explicitly
  unsupported.
- Treat `success: true` as acceptance, then verify the external effect.

Fields ending in `?` are optional. `{}` means no arguments.

## Connection and project metadata

| Tool | Input | Purpose |
|---|---|---|
| `burp_burp_version` | `{}` | Extension/Burp version, edition, capabilities, and limits. |
| `burp_extension_info` | `{}` | Extension filename, BApp status, and process arguments. |
| `burp_export_config` | `{}` | Export project configuration JSON. |
| `burp_import_config` | `{config}` | Import project configuration JSON; project-wide mutation. |
| `burp_inspect_config` | `{paths?}` | Export scoped project options with discovered leaf paths and UTF-8 size before import. |

## Scope, target, and site map

| Tool | Input | Purpose |
|---|---|---|
| `burp_get_scope` | `{url}` | Check one URL against current Burp scope. |
| `burp_add_to_scope` | `{url}` | Add one URL to target scope. Cleanup: `burp_remove_from_scope`. |
| `burp_remove_from_scope` | `{url}` | Remove one URL from target scope. |
| `burp_target_info` | `{url?, limit?}` | Summarize hosts and technology headers from a bounded sample. |
| `burp_sitemap` | `{url_prefix?, limit?, cursor?}` | Page through Burp site-map entries. |

## Proxy HTTP and WebSocket evidence

| Tool | Input | Purpose |
|---|---|---|
| `burp_proxy_history` | `{limit?, offset?, cursor?, url_filter?, method_filter?, status_filter?, has_notes?, color?}` | Page/filter HTTP history. Prefer `cursor` after the first page. |
| `burp_proxy_detail` | `{index}` | Read the raw request/response, notes, and highlight for one index. |
| `burp_proxy_websocket_history` | `{limit?, cursor?}` | Read observed WebSocket messages; payloads are base64. |
| `burp_highlight` | `{index, color?}` | Persist a Proxy history highlight. Empty/default color clears it. |
| `burp_annotate` | `{index, note}` | Persist a Proxy history note. |
| `burp_extract_from_response` | `{index, regex, limit?}` | Extract bounded regex matches from one response. |

## Sending and preparing HTTP requests

| Tool | Input | Purpose |
|---|---|---|
| `burp_send_request` | `{url, method?, body?, headers?}` | Send one structured HTTP request through Burp. |
| `burp_send_request_parallel` | `{requests: [{url, method?, body?, headers?}, ...]}` | Send up to 32 requests concurrently. |
| `burp_send_to_repeater` | `{request, host, port?, https?, tab_name?}` | Display one raw request in Repeater without sending. |
| `burp_send_to_intruder` | `{request, host, port?, https?, tab_name?}` | Display one raw request in Intruder without starting an attack. |
| `burp_convert_request` | `{request, convert_to?}` | Convert a raw HTTP request method; default target is POST. |
| `burp_export_request` | `{request, host?, format?, https?}` | Export as `curl` or Python `requests` code per the live schema. |

For a raw request, keep `Host`, path, body framing, target host, port, and
`https` consistent. When using structured sends, pass a full URL.

## Background jobs and Scanner

### Starters

| Tool | Input | Purpose |
|---|---|---|
| `burp_race_condition` | `{request, host, port?, https?, count?}` | Start a bounded concurrent comparison job. |
| `burp_inline_fuzzer` | `{template, host, port?, https?, marker?, wordlist}` | Start a bounded single-marker input matrix. |
| `burp_scan` | `{url, mode?}` | Start passive audit with `mode: "passive"`; active is unsupported. |
| `burp_crawl` | `{url}` | Start a bounded crawl when the runtime supports it. |

### Lifecycle

| Tool | Input | Purpose |
|---|---|---|
| `burp_job_status` | `{job_id}` | Read current job state and error. |
| `burp_job_result` | `{job_id, limit?, cursor?}` | Page results and summary counters. |
| `burp_job_cancel` | `{job_id}` | Cancel a queued/running job. |

### Findings

| Tool | Input | Purpose |
|---|---|---|
| `burp_scan_issues` | `{limit?, cursor?}` | Page through Scanner issues. |
| `burp_scan_issue_detail` | `{index}` | Read complete details for one issue index. |
| `burp_add_issue` | `{name, url, detail?, remediation?, severity?, confidence?}` | Persist one validated typed issue in Burp. |
| `burp_scanner_generate_report` | `{format, path, issue_indexes?}` | Generate an HTML or XML Scanner report. Omit indexes for all issues; destination must not exist. |

Scanner reports, scoped configuration inspection, typed Proxy listener configuration, script filters, and interception rules are backed by Burp project-option export/import through the extension. Native Dashboard task enumeration/resource pools, Sequencer, Repeater/Intruder execution results, Session UI CRUD, and pending Intercept-editor message control remain outside the stable Montoya surface; do not represent lower-level HTTP wrappers as equivalents.

## Collaborator

| Tool | Input | Purpose |
|---|---|---|
| `burp_collaborator_generate` | `{count?}` | Generate a bounded set of Collaborator payloads. |
| `burp_collaborator_poll` | `{limit?, cursor?}` | Page observed interactions. |

Associate each payload with one request variant before sending it. Poll for a
bounded interval; absence of an interaction is not proof of safety.

## Managed WebSockets

| Tool | Input | Purpose |
|---|---|---|
| `burp_websocket_create` | `{host, port?, https?, path?}` | Create a managed WebSocket through Burp. |
| `burp_websocket_send_text` | `{id, text}` | Send one text message. |
| `burp_websocket_send_binary` | `{id, data}` | Send base64-encoded binary data. |
| `burp_websocket_list` | `{}` | List active managed connection IDs. |
| `burp_websocket_close` | `{id}` | Close one managed connection. |

Cleanup pair: every successful create must end with close unless the user asks
to leave the connection open.

## Handlers, interception, cookies, sessions, and macros

| Tool | Input | Purpose and cleanup |
|---|---|---|
| `burp_intercept_state` | `{}` | Read the current Proxy interception state. |
| `burp_set_intercept_state` | `{enabled}` | Set Proxy interception state. Read and restore the original state around temporary changes. |
| `burp_register_http_handler` | `{header_name?, header_value?, match?, replace?}` | Add-header or replace-text rule. Cleanup: `burp_remove_http_handler`. |
| `burp_remove_http_handler` | `{}` | Clear HTTP handler rules. |
| `burp_register_proxy_rule` | `{url_contains, id?, phase?, action?, intercept?, match?, replace?, header_name?, header_value?, enabled?}` | Create/replace a request or response rule. Actions: `forward`, `intercept`, `drop`, `edit`. Cleanup: `burp_remove_proxy_rule`. |
| `burp_list_proxy_rules` | `{}` | List configured Proxy rules and their enabled state. |
| `burp_remove_proxy_rule` | `{id?}` | Remove one rule by ID, or clear all rules when `id` is omitted. |
| `burp_proxy_intercept_config` | `{}` | Legacy focused read of Proxy request, response, WebSocket interception filters and response modification settings. Prefer `burp_proxy_settings` for new workflows. |
| `burp_update_proxy_intercept_config` | `{master_intercept_enabled?, request_do_intercept?, response_do_intercept?, request_rules?, response_rules?, websocket_client_to_server?, websocket_server_to_client?, websocket_in_scope_only?, ...}` | Legacy bulk patch; replacing rule arrays requires the matching `replace_*_rules` flag. Prefer granular `burp_update_proxy_settings` operations. |
| `burp_proxy_settings` | `{}` | Read listeners, script filters, and request/response interception settings together. |
| `burp_update_proxy_settings` | `{operation, ...}` | Unified mutation tool. Operations: `listener_upsert`, `listener_delete`, `script_filter_upsert`, `script_filter_delete`, `intercept_rule_upsert`, `intercept_rule_delete`, `intercept_toggle`. |

| `burp_cookie_jar` | `{limit?, domain?}` | List cookies; values are sensitive. |
| `burp_cookie_jar_set` | `{name, value, domain, path?, expiration?}` | Set a cookie. Verify by listing it; expire temporary cookies during cleanup. |
| `burp_session_create_rule` | `{id?, find?, replace?, description?, action_type?, header_name?, parameter_name?, macro_description?, url_contains?, tools?, enabled?}` | Create a session rule and return its stable ID. |
| `burp_session_get_rule` | `{id}` | Get one session rule by stable ID. |
| `burp_session_update_rule` | `{id, find?, replace?, description?, action_type?, header_name?, parameter_name?, macro_description?, url_contains?, tools?, enabled?}` | Replace one session rule by ID. |
| `burp_session_list_rules` | `{}` | List current session rules. |
| `burp_session_delete_rule` | `{id}` | Delete one session rule by ID. |
| `burp_macro_list` | `{}` | List Burp session macros. |
| `burp_macro_create` | `{description, serial_number?, items}` | Create/replace a macro; see item shape below. |
| `burp_macro_run` | `{description}` | Execute requests from one named macro. |
| `burp_macro_remove` | `{description}` | Remove one named macro. |

`burp_update_proxy_settings` arguments by operation:

- Listener upsert: `port`, plus optional `running`, `listen_mode`, `listen_specific_address`, `certificate_mode`, `enable_http2`, and `support_invisible_proxying`. Listener delete: `port`.
- Script-filter upsert: `target`, optional `mode`, `script`, `script_id`, and `script_name`. Targets: `proxy_http_history`, `proxy_websocket_history`, `sitemap`, `logger_capture`, `logger_display`. Delete: `target`.
- Interception-rule upsert: `kind` (`request` or `response`) and `rule`; omit `index` to append or provide it to replace one rule. Delete: `kind` and `index`.
- Interception toggle: one or more of `master_enabled`, `request_enabled`, and `response_enabled`.

Read the baseline before every temporary Proxy mutation and restore it afterward. Rule indexes are zero-based and refer to the list returned by `burp_proxy_settings`.

Macro item shape:

```json
{
  "request": "GET / HTTP/1.1\r\nHost: example.test\r\n\r\n",
  "url": "https://example.test/",
  "method": "GET",
  "response": "",
  "status_code": 0,
  "cookies_received": "",
  "request_parameters": [
    {
      "name": "csrf",
      "original_value": "",
      "parameter_handling": "preset_value",
      "preset_value": "",
      "type": ""
    }
  ],
  "custom_parameters": []
}
```

Only `request` and `url` are required per item. Review raw requests before
creating or running a macro.

## Script imports

| Tool | Input | Purpose |
|---|---|---|
| `burp_bambda_import` | `{script}` | Import a complete Bambda YAML document without executing it. |
| `burp_bcheck_import` | `{script, enabled}` | Import a complete BCheck definition without running a scan. Default authoring workflow: `enabled: false`. |

Imports persist in Burp. Treat source as executable code: review it and require
explicit user intent. Import success requires `success: true`, status
`LOADED_WITHOUT_ERRORS`, and an empty error list. The current MCP surface has no
list, test, run, enable/disable-after-import, or delete operation for imported
scripts. For authoring and review rules, load
[`bambda-authoring.md`](./bambda-authoring.md) or
[`bcheck-authoring.md`](./bcheck-authoring.md).

## Offline `decoder`

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

- Encoding: `base64.encode`, `base64.decode`, `base64url.encode`,
  `base64url.decode`, `hex.encode`, `hex.decode`, `url.encode`, `url.decode`,
  `html.encode`, `html.decode`, `unicode.escape`, `unicode.unescape`.
- JSON/text: `json.pretty`, `json.minify`, `json.query`, `text.uppercase`,
  `text.lowercase`, `text.reverse`, `text.split`, `text.join`,
  `regex.extract`, `regex.replace`.
- Inspection/hashes: `entropy`, `strings.extract`, `length`, `md5`, `sha1`,
  `sha256`, `sha512`, `blake3`, `hmac.sha256`, `hmac.sha512`.
- Compression: `gzip.compress`, `gzip.decompress`, `zlib.compress`,
  `zlib.decompress`, `deflate.compress`, `deflate.decompress`,
  `brotli.compress`, `brotli.decompress`.
- Web/security: `jwt.decode`, `jwt.verify_hs256`, `cookie.parse`,
  `query.parse`, `query.build`, `http.parse`, `http.set_body`,
  `http.update_content_length`.

Use `describe` before supplying operation-specific `args` unless the live
schema or a previous result already established them. JWT decode does not
verify a signature. MD5 and SHA-1 are marked cryptographically weak.

## Persistent sitegraph

| Tool | Input | Purpose |
|---|---|---|
| `sitegraph_sync` | `{url_prefix?}` | Synchronize bounded sitemap and issue metadata into local SQLite. |
| `sitegraph_status` | `{}` | Read synchronization/schema status. |
| `sitegraph_stats` | `{}` | Read node/edge counts and last sync time. |
| `sitegraph_search` | `{query, limit?, cursor?}` | Search normalized endpoints. |
| `sitegraph_endpoint_detail` | `{id}` | Read one endpoint by stable ID. |
| `sitegraph_neighbors` | `{id, limit?, cursor?}` | Page adjacent nodes. |
| `sitegraph_trace` | `{id, max_depth?, limit?}` | Traverse relationships to depth 1..8. |
| `sitegraph_diff` | `{since, limit?, cursor?}` | Read nodes changed since a Unix timestamp. |
| `sitegraph_export` | `{format?, limit?, cursor?}` | Export a bounded metadata page as JSON or CSV. |

Sync before querying when freshness matters. Sitegraph persistence is
privacy-preserving metadata storage, not an archive of HTTP bodies or parameter
values.

## Cleanup checklist

- Restore original intercept state.
- Remove temporary scope entries, HTTP/proxy/session rules, and macros.
- Expire temporary cookies and verify the cookie jar.
- Close every managed WebSocket.
- Cancel unfinished jobs and record the final state.
- Leave annotations, imported scripts/checks, configuration, or created issues
  only when the user explicitly requested persistence.
