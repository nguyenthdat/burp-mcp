# Burp MCP Tool Catalog

Load this reference only when selecting exact tools or constructing arguments.
The live MCP schema takes precedence over this catalog. Client-visible names
may include an MCP server prefix; match by the server-local suffix shown here.

## Global constraints and semantics

- Burp-facing tools require Burp Suite with the Burp MCP extension running.
  Edition and `burp_burp_version.capabilities` determine feature availability.
- List limits are at most `500`; use `next_cursor` or `cursor` when returned.
- Proxy/scanner indexes must be at most `2147483647`.
- `burp_http` `send_batch` accepts at most `32` requests per call.
- Sitegraph limits are `1..500`; trace depth is `1..8`.
- Utility input is bounded to 16 MiB; recipes are bounded to 64 steps.
- Treat `success: true` as acceptance, then verify the external effect.

Fields ending in `?` are optional. `{}` means no arguments.

---

## 1. Connection & Project Metadata (2 tools)

| Tool | Input | Purpose |
|---|---|---|
| `burp_burp_version` | `{}` | Extension/Burp version, edition, capabilities, and runtime limits. |
| `burp_extension_info` | `{}` | Extension filename, BApp status, and process arguments. |

---

## 2. Core Pentesting Suite (13 tools)

| Tool | Input | Key Actions & Purpose |
|---|---|---|
| `burp_proxy` | `{action, limit?, offset?, cursor?, url_filter?, method_filter?, status_filter?, has_notes?, color?, include_bodies?, headers_only?, extract_css?, extract_json?, max_body_length?, index?, notes?, regex?}` | `history`, `detail`, `annotate`, `highlight`, `extract`, `websocket_history`. Default history uses compact metadata (`include_bodies: false`, `max_body_length: 4096`). Emits original byte length and truncation state. |
| `burp_http` | `{action, method?, url?, body?, headers?, headers_only?, extract_css?, extract_json?, max_body_length?, requests?, request?, convert_to?, host?, port?, https?, format?, tab_name?}` | `send`, `send_batch`, `convert`, `export`, `send_to_repeater`. For `send_to_repeater`, provide either absolute `url` plus optional method/body/headers or a raw `request`; raw requests derive host/port from `Host` when omitted. Default `max_body_length: 4096`. |
| `burp_target` | `{action, url?, url_prefix?, limit?, cursor?}` | `get_scope`, `add_scope`, `remove_scope`, `info`, `sitemap`. |
| `burp_scanner` | `{action, url?, audit_type?, seed_urls?, scan_configuration_id?, resource_pool_id?, timeout_seconds?, stable_seconds?, include_out_of_scope?, job_id?, limit?, offset?, cursor?, index?, status?, severity?, confidence?, notes?, format?, path?, issue_indexes?, script?, request?, response?, host?, port?, https?}` | `start_audit`, `start_crawl`, `stop`, `list_issues`, `issue_detail`, `update_issue`, `report`, `test_bcheck`, `remove`. |
| `burp_scan_config` | `{action, id?, name?, scan_type?, audit_type?, include_out_of_scope?, timeout_seconds?, stable_seconds?, resource_pool_id?, kind?, existing_pool_name?, concurrent_request_limit?, throttle_millis?, max_retries?}` | `list_configs`, `get_config`, `upsert_config`, `delete_config`, `list_pools`, `get_pool`, `upsert_pool`, `delete_pool`. |
| `burp_fuzzer` | `{action, template?, host?, port?, https?, marker?, wordlist?, payload_list_id?, payload_offset?, attack_mode?, markers?, request?, count?, single_packet_attack?, tab_name?, id?, name?, payloads?, operation?, display_name?, argument?, replacement?, generator_type?, min_value?, max_value?, step?, charset?, min_length?, max_length?}` | `fuzz` (pitchfork/cluster_bomb/sniper), `race` (with single-packet attack option), `send_to_intruder`, `list_payloads`, `get_payload_list`, `create_payload_list`, `import_payload_list`, `upsert_payloads`, `delete_payload_list`, `register_payload_processor`, `list_payload_processors`, `remove_payload_processor`, `register_payload_generator`, `list_payload_generators`, `remove_payload_generator`. |
| `burp_collaborator` | `{action, count?, target_url?, injection_point?, limit?, cursor?}` | `generate` (with target binding), `poll` (with auto-correlation), `correlate`. |
| `burp_websocket` | `{action, host?, port?, https?, path?, id?, text?, data?, limit?, cursor?, include_bodies?, max_body_length?}` | `create`, `send_text`, `send_binary`, `history`, `close`, `list`. History defaults to metadata-only; explicit frames are capped at 4096 bytes and report original length/truncation. |
| `burp_session` | `{action, id?, description?, action_type?, find?, replace?, header_name?, parameter_name?, macro_description?, url_contains?, tools?, enabled?, serial_number?, items?}` | `list_rules`, `get_rule`, `upsert_rule`, `delete_rule`, `run_macro`, `upsert_macro`, `list_macros`, `delete_macro`. |
| `burp_settings` | `{action, config?, paths?, enabled?, operation?, port?, running?, listen_mode?, listen_specific_address?, certificate_mode?, enable_http2?, support_invisible_proxying?, id?, url_contains?, phase?, rule_action?, match?, replace?, header_name?, header_value?, index?, rule?, master_enabled?, request_enabled?, response_enabled?}` | `get_proxy_settings`, `update_proxy_settings`, `export_config`, `inspect_config`, `import_config`, `intercept_state`, `set_intercept_state`, `proxy_intercept_config`, `update_proxy_intercept_config`, `register_http_handler`, `remove_http_handler`, `register_proxy_rule`, `list_proxy_rules`, `remove_proxy_rule`. `register_proxy_rule` uses `id`, `url_contains`, `phase`, `rule_action`, `match`, `replace`, `header_name`, `header_value`, `enabled`; `register_http_handler` uses `header_name`, `header_value`, `match`, `replace`. |
| `burp_logger` | `{action, limit?, offset?, cursor?, source_filter?, url_filter?, method_filter?, status_filter?, has_notes?, color?, include_bodies?, headers_only?, extract_css?, extract_json?, max_body_length?, index?}` | `query` (across proxy, repeater, scanner, intruder, extensions), `detail`, `clear`. Defaults to metadata-only (`include_bodies: false`, `max_body_length: 4096`). Emits original byte length and truncation state. |
| `burp_organizer` | `{action, request?, response?, host?, port?, https?, notes?, highlight?, limit?, cursor?, status_filter?, url_filter?}` | `add`, `list`. |
| `burp_diff` | `{action, response_a?, response_b?, index_a?, index_b?, first?, second?}` | `compare_exchanges` (send to desktop Comparer tab), `diff_responses` (similarity score, header diff, body line diff). |

---

## 3. Compound Security Workflows (9 tools)

| Tool | Input | Purpose |
|---|---|---|
| `burp_verify_idor` | `{url, method?, body?, headers?, original_auth_header, victim_auth_header, auth_header_name?, match_pattern?}` | Automated IDOR verification across two user authorization contexts (User A vs User B). |
| `burp_check_cors` | `{url, method?, test_origins?, headers?}` | Automated CORS vulnerability auditing with origin reflections, wildcard checks, and credentials evaluation. |
| `burp_auth_matrix` | `{endpoints, method?, body?, roles}` | Automated role-based access control matrix across multiple endpoints. |
| `burp_audit_jwt` | `{url, method?, headers?, jwt_token, auth_header_name?, public_key_pem?, tamper_claims?}` | Automated JWT vulnerability audit (None algorithm, RS256 -> HS256 key confusion, and claim tampering). |
| `burp_verify_ssrf` | `{target_url, method?, headers?, body?, injection_points, wait_seconds?}` | Automated SSRF verification with Collaborator interaction polling and payload correlation. |
| `burp_verify_sqli_blind` | `{url, method?, param_name, param_type?, sleep_seconds?}` | Differential boolean-based and timing statistical blind SQL injection verification. |
| `burp_audit_graphql` | `{endpoint, headers?, test_batching?, test_introspection?, test_field_suggestions?}` | Automated GraphQL security audit (Introspection, Field Suggestions, and Query Batching). |
| `burp_verify_csrf_samesite` | `{url, method?, body?, session_cookie_name}` | Automated CSRF risk audit, SameSite cookie evaluation, and auto-generated HTML PoC form. |
| `burp_api_fuzz_orchestrator` | `{spec_content, target_base_url, auth_headers?, fuzz_categories?}` | Automated specification-driven API fuzzing from OpenAPI 2.0 / 3.0 or Swagger documents. |
---

## 4. Active UI & Desktop Editor Integration (3 tools)

| Tool | Input | Purpose |
|---|---|---|
| `burp_editor_get` | `{target_hint?, ttl_seconds?}` | Capture active or last-focused editor tab with rich metadata, selection offsets, and UTF-8 decoded text. |
| `burp_editor_patch` | `{token, expected_sha256, mode?, text?, payload_base64?, selection_replacement?, header_name?, header_value?, header_remove?, regex_pattern?, regex_replacement?, regex_replace_all?, regex_case_insensitive?, json_path?, json_value?, param_name?, param_value?, param_remove?, param_type?}` | Surgically modify active Burp editor contents (`replace_selection`, `set_header`, `json_patch`, `set_param`, `regex`, `replace_all`) with automatic Content-Length and CRLF calculation. |
| `burp_editor_renew_lease` | `{token, extend_seconds?}` | Extend the lifetime of an active Burp editor lease token. |

---

## 5. Cookies & Findings (3 tools)

| Tool | Input | Purpose |
|---|---|---|
| `burp_cookie_jar` | `{domain?, limit?}` | List cookies with optional domain filtering and pagination. |
| `burp_cookie_jar_set` | `{name, value, domain?, path?, expiration?}` | Create or update a cookie in Burp's active cookie jar. |
| `burp_add_issue` | `{name, url, detail?, remediation?, severity?, confidence?}` | Add a typed custom audit issue to the Burp site map. |

---

## 6. Background Jobs (3 tools)

| Tool | Input | Purpose |
|---|---|---|
| `burp_job_status` | `{job_id}` | Return state, progress, and error details for a background job. |
| `burp_job_result` | `{job_id, limit?, cursor?}` | Return paginated background job results. |
| `burp_job_cancel` | `{job_id}` | Cancel an unfinished background job. |

---

## 7. Custom Script Imports (2 tools)

| Tool | Input | Purpose |
|---|---|---|
| `burp_bambda_import` | `{script}` | Validate and import a Bambda definition. JVM `CONSTANT_Utf8` entries are limited to 65,535 bytes; do not embed large bundles. |
| `burp_bcheck_import` | `{script, enabled}` | Validate and import declarative BCheck definition into Burp Scanner. |

---

## 8. MCP Interception Queues (6 tools)

| Tool | Input | Purpose |
|---|---|---|
| `burp_intercept_controller` | `{enabled?, timeout_seconds?, url_filter?, in_scope_only?}` | Configure scoped HTTP interception. Enabling requires `url_filter` or `in_scope_only: true`; non-matching traffic bypasses the queue. |
| `burp_intercepted_messages` | `{limit?, cursor?, include_bodies?, max_body_length?}` | List HTTP messages currently paused by MCP intercept controller. Defaults to metadata-only; explicit bodies are capped at 4096 bytes and report original length/truncation. |
| `burp_control_intercepted_message` | `{id, action, message_base64?, max_body_length?}` | Forward, drop, or send an MCP-paused HTTP message to manual Intercept; optionally replace the complete raw message. The returned message is capped at 4096 bytes by default. |
| `burp_websocket_intercept_controller` | `{enabled?, timeout_seconds?}` | Read or configure MCP-controlled WebSocket interception queue. |
| `burp_intercepted_websocket_messages` | `{limit?, cursor?, include_bodies?, max_body_length?}` | List WebSocket messages currently paused by MCP controller. Defaults to metadata-only; explicit payloads are capped at 4096 bytes and report original length/truncation. |
| `burp_control_intercepted_websocket_message` | `{id, action, payload_base64?, max_body_length?}` | Forward, drop, or send a paused WebSocket frame to manual Intercept; optionally replace its payload. The returned payload is capped at 4096 bytes by default. |

---

## 9. Offline Utility Decoder (1 tool)

| Tool | Input | Purpose |
|---|---|---|
| `decoder` | `{input, operation?, args?, query?, describe?, magic?, steps?}` | 40+ built-in operations for encoding, decoding, hashing, compression, JWT, and parsing. |

---

## 10. SiteGraph (Advanced Opt-in) (1 tool)

| Tool | Input | Purpose |
| `sitegraph` | `{action, url_prefix?, query?, id?, from_id?, to_id?, limit?, cursor?, max_depth?, since?, profile?, format?, snapshot_id?, view_name?, spec_content?}` | SiteGraph attack surface graph analyzer (`status`, `stats`, `sync`, `search`, `security_view`, `import_spec`, `neighbors`, `trace`, `shortest_path`, `clusters`, `impact`, `diff`, `export`, `history_search`, `endpoint_detail`, `projects`, `config`). |
