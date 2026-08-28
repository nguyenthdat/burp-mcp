# Burp MCP Feature Reference

This preliminary catalog is a reference for live testing. It explains what each tool does and the minimum observable behavior to verify. The live MCP schema remains authoritative for exact argument names, types, and currently advertised tools.

## Live-test conventions

- Test only authorized targets. Record the Burp version, edition, extension version, and target boundary first.
- Call `burp_burp_version` before edition-dependent cases and record advertised capabilities.
- For every mutation: capture the baseline, make the smallest isolated change, verify the external effect, then restore the baseline.
- `success: true` means the operation was accepted. It does not prove the effect; use a corresponding read tool, Burp UI state, target behavior, or graph state.
- Follow `next_cursor` for paginated results. Most list limits are at most `500`; `burp_send_request_parallel` accepts at most `32` requests.
- `burp_send_to_repeater` and `burp_send_to_intruder` open UI items; they do not send a request or start an attack.
- `burp_proxy_detail` returns text-oriented raw HTTP. Do not use it to prove a byte-exact binary round trip.
- Redact cookie values, configuration secrets, Collaborator secrets, and sensitive request/response data from reports.

## Inventory

The default v3 runtime registers **100 tools**. Enabling the advanced sitegraph adds **15 tools**.

| Feature group | Tools |
|---|---:|
| Connection and project configuration | 5 |
| Scope, target, and site map | 5 |
| Proxy evidence & Token optimization | 6 |
| HTTP sending and preparation | 6 |
| Handlers and interception | 17 |
| Active editor UI | 4 |
| Cookies | 2 |
| Sessions and macros | 9 |
| Intruder, Race conditions & Multi-marker fuzzing | 8 |
| Payload lists | 6 |
| Scanner execution, findings & BCheck dry-run | 9 |
| Scanner configuration | 10 |
| Background jobs | 3 |
| Collaborator & Auto-correlation | 3 |
| Managed WebSockets | 6 |
| Logger API (Traffic across all tools) | 3 |
| Organizer API | 2 |
| Response Comparer & Diff engine | 2 |
| Compound Security Workflows (CORS, IDOR, Auth Matrix) | 3 |
| Consolidated Action-Based Tools | 7 |
| Sitegraph (advanced opt-in) | 14 |
| Offline decoder | 1 |
## Connection and project configuration

| Tool | Purpose | Preliminary live check |
|---|---|---|
| `burp_burp_version` | Return the Burp version, edition, extension version, capabilities, and runtime limits. | Compare version and edition with the Burp UI; use capabilities as prerequisites for later cases. |
| `burp_extension_info` | Return extension and process metadata, including filename, BApp status, and process arguments. | Compare the result with the loaded extension; expect no mutation. |
| `burp_export_config` | Export project configuration as JSON. | Confirm the output parses as JSON; do not copy secrets into the report. |
| `burp_inspect_config` | Export selected project-option paths and report discovered leaf paths and UTF-8 size. | Inspect one narrow path and confirm paths/size before any import. |
| `burp_import_config` | Validate and import size-bounded project configuration JSON. | Change one harmless option, read/export it back, then restore the baseline. |

## Scope, target, and site map

| Tool | Purpose | Preliminary live check |
|---|---|---|
| `burp_get_scope` | Check whether one URL is in the current Burp target scope. | Query a URL with a known scope state and compare it with Target scope. |
| `burp_add_to_scope` | Add one URL to target scope. | Add an isolated fixture URL, verify with `burp_get_scope`, then remove it if it was initially out of scope. |
| `burp_remove_from_scope` | Remove one URL from target scope. | Remove only a fixture URL, verify the state, then restore it if initially in scope. |
| `burp_target_info` | Summarize hosts and technology headers from a bounded site-map sample. | Use a fixture URL prefix with recorded traffic and inspect the host/technology summary. |
| `burp_sitemap` | Page through site-map entries, optionally filtered by URL prefix. | Generate fixture traffic, find its entry, and follow `next_cursor` when present. |

## Proxy evidence

| Tool | Purpose | Preliminary live check |
|---|---|---|
| `burp_proxy_history` | Page and filter Proxy HTTP history with compact metadata by default (`include_bodies: false`). Supports `headers_only`, `extract_css`, `extract_json`, `max_body_length`. | Generate fixture traffic, test metadata listing, and verify projection/extraction. |
| `burp_proxy_detail` | Return request, response, notes, and highlight for one Proxy history index with optional `headers_only`, `extract_css`, `extract_json`, and truncation. | Compare method, path, status, and text body with the fixture. |
| `burp_highlight` | Set or clear the highlight color of one Proxy history item. | Save the original color, set a test color, read it back, then restore it. |
| `burp_annotate` | Set notes on one Proxy history item. | Save the original note, write a unique marker, read it back, then restore it. |
| `burp_extract_from_response` | Extract bounded regular-expression matches from one recorded response. | Use a response with a deterministic marker and verify matches and limit behavior. |
| `burp_proxy_websocket_history` | Page through WebSocket messages observed by Burp Proxy; payloads are base64. | Generate fixture WebSocket traffic, decode the payload, and verify direction/listener metadata. |

## Logger, Organizer, Diffing & Compound Security Workflows

| Tool | Purpose | Preliminary live check |
|---|---|---|
| `burp_logger_history` | Page full HTTP traffic from Burp Logger across all tools (Proxy, Repeater, Scanner, Intruder, Extensions). | Query traffic across tools, filter by source or URL, verify compact metadata and extraction. |
| `burp_logger_detail` | Retrieve full request/response for a specific Logger entry index. | Inspect details of scanner or repeater request. |
| `burp_clear_logger` | Clear in-memory Logger traffic buffer. | Clear and verify history returns empty. |
| `burp_organizer_send` | Send request/response exchange into Burp Organizer with notes and highlight color. | Send a test item, verify acceptance. |
| `burp_organizer_list` | List and filter items stored in Burp Organizer. | Query items by status/URL filter. |
| `burp_diff_responses` | Compare two HTTP response texts or history entries, computing similarity ratio, header diffs, and body diff. | Compare two different responses, verify similarity score and header/body diff. |
| `burp_send_to_comparer` | Send two raw HTTP messages directly to Burp Comparer UI tab. | Send two test payloads, verify in Burp Comparer. |
| `burp_test_bcheck` | Dry-run and syntax check a BCheck script against sample request/response. | Pass a valid BCheck script and sample HTTP exchange, verify rule matching. |
| `burp_update_scan_issue_status` | Update a scanner issue status (False Positive, Ignored, Confirmed) and notes. | Update an issue index, verify updated status. |
| `burp_verify_idor` | Compound workflow to verify IDOR between two authorization tokens/headers. | Test two roles against an endpoint, inspect similarity and differential verdict. |
| `burp_check_cors` | Compound workflow to audit CORS configuration with origin reflections and credentials. | Test target URL with test origins, review CORS findings. |
| `burp_auth_matrix` | Compound workflow to evaluate role-based access control matrix across multiple endpoints. | Pass endpoint list and role headers, review access matrix. |
| `burp_proxy` | Consolidated action-based proxy tool (`history`, `detail`, `annotate`, `highlight`, `extract`). | Call actions via unified schema. |
| `burp_http` | Consolidated action-based HTTP tool (`send`, `send_batch`, `convert`, `export`, `send_to_repeater`). | Call actions via unified schema. |
| `burp_target` | Consolidated action-based target tool (`get_scope`, `add_scope`, `remove_scope`, `info`, `sitemap`). | Call actions via unified schema. |
| `burp_scanner` | Consolidated action-based scanner tool (`start_audit`, `start_crawl`, `stop`, `list_issues`, `issue_detail`, `update_issue`, `report`). | Call actions via unified schema. |
| `burp_fuzzer` | Consolidated action-based fuzzer tool (`fuzz`, `race`, `send_to_intruder`, `list_payloads`, `upsert_payloads`). | Call actions via unified schema. |
| `burp_collaborator` | Consolidated action-based collaborator tool (`generate`, `poll`, `correlate`). | Call actions via unified schema. |
| `burp_diff` | Consolidated action-based diffing tool (`diff_responses`, `compare_exchanges`). | Call actions via unified schema. |
## HTTP sending and preparation

| Tool | Purpose | Preliminary live check |
|---|---|---|
| `burp_send_request` | Send one HTTP request through Burp and return the response. | Send a harmless fixture request and verify status/body plus its Burp history entry. |
| `burp_send_request_parallel` | Send a bounded batch of HTTP requests concurrently. | Send a small batch with distinguishable responses and verify every per-request result/error. |
| `burp_send_to_repeater` | Open a raw HTTP request in Repeater without sending it. | Use a unique `tab_name`; verify the tab and target, not a response. |
| `burp_race_condition` | Start a bounded concurrent-request comparison job. | Use an idempotent fixture endpoint, poll the job, and verify request/result counts. |
| `burp_convert_request` | Convert a raw request between methods such as GET and POST. | Convert a request with known parameters; verify method, parameters, body, and headers. |
| `burp_export_request` | Export a request as raw text, `curl`, or Python `requests` code. | Exercise all three formats and verify method, URL, headers, and body semantics. |

### Active editor UI

| Tool | Purpose | Preliminary live check |
|---|---|---|
| `burp_active_editor_get` | Capture the focused text editor and return a short-lived token plus content hash. | Focus an editable HTTP text editor in Burp, call the tool, and compare returned text. |
| `burp_active_editor_set` | Replace captured HTTP editor text after token/hash validation. | Change the editor locally between get/set to confirm stale writes are rejected; otherwise verify the visible editor text. |
| `burp_websocket_editor_get` | Capture the focused MCP `ExtensionProvidedWebSocketMessageEditor` tab with lossless Base64 payload. | Open and focus the **MCP** WebSocket tab, select a message, and compare payload/direction metadata. |
| `burp_websocket_editor_set` | Replace the captured extension-tab payload after token/hash validation and mark it modified for Burp. | Apply a harmless binary fixture, verify the tab payload, then use a normal explicit Burp action to send it. |

## Handlers and interception

There are two distinct rule systems:

- `burp_register_proxy_rule` manages runtime request/response handlers owned by the extension.
- Interception rules in `burp_proxy_settings` are Burp Proxy Intercept project settings.

| Tool | Purpose | Preliminary live check |
|---|---|---|
| `burp_register_http_handler` | Register a bounded request handler that adds a header or replaces text. | Apply a unique marker to a narrow fixture, send a request, verify the effect, then clear handlers. |
| `burp_remove_http_handler` | Remove registered HTTP handler rules. | Clear the fixture handler and confirm a later request is no longer modified. |
| `burp_register_proxy_rule` | Create or replace a request/response rule with `forward`, `intercept`, `drop`, or `edit`. | Scope the rule to a unique URL marker, verify it via list/traffic, then remove it. |
| `burp_list_proxy_rules` | List configured runtime Proxy rules and enabled state. | Confirm the fixture ID, phase, action, and enabled state. |
| `burp_remove_proxy_rule` | Remove one Proxy rule by ID, or clear all rules when ID is omitted. | Remove only the fixture ID; use clear-all only in an isolated test project. |
| `burp_intercept_state` | Read the master Proxy interception state. | Capture this baseline before any Intercept mutation. |
| `burp_set_intercept_state` | Enable or disable master Proxy interception. | Toggle while monitoring the UI, read it back, and restore immediately to avoid blocked traffic. |
| `burp_intercept_controller` | Read or configure the MCP-owned HTTP interception queue and timeout; intercepted messages auto-forward on timeout. | Enable only for a narrow fixture, confirm the timeout, then disable it and verify `pending` returns to zero. |
| `burp_intercepted_messages` | Page HTTP requests/responses paused by the MCP intercept controller, including lossless base64 messages. | Generate one scoped fixture message and verify direction, phase, URL, and pagination without logging sensitive payloads. |
| `burp_control_intercepted_message` | Forward, drop, or send one MCP-paused HTTP message to Burp's manual Intercept tab; optionally replace the complete message from base64. | Use a harmless fixture, preserve the original bytes unless replacement is required, and ensure no message remains pending. |
| `burp_websocket_intercept_controller` | Read or configure MCP-owned interception for Proxy WebSocket text and binary messages. | Enable only around fixture traffic, verify timeout/pending state, then disable it. |
| `burp_intercepted_websocket_messages` | Page text/binary WebSocket messages paused by the MCP controller. | Verify message type, direction, phase, base64 payload, and cursor behavior on a fixture connection. |
| `burp_control_intercepted_websocket_message` | Forward, drop, or send one paused WebSocket message to Burp's manual Intercept tab; optionally replace payload bytes from base64. | Act on one fixture ID, verify the returned action/state, then drain or disable the controller. |
| `burp_proxy_intercept_config` | Legacy focused read of request, response, WebSocket interception filters, and response modification. | Compare the result with the Proxy settings UI; prefer `burp_proxy_settings` for new flows. |
| `burp_update_proxy_intercept_config` | Legacy bulk patch of interception settings; replacing rule arrays requires matching replace flags. | Use a complete baseline and restore it exactly; prefer granular operations below. |
| `burp_proxy_settings` | Read listeners, script filters, and request/response interception settings and rules together. | Capture a baseline and compare listener ports and Intercept settings with the Burp UI. |
| `burp_update_proxy_settings` | Perform one listener, script-filter, interception-rule, or interception-toggle mutation selected by `operation`. | Exercise one operation at a time, read back with `burp_proxy_settings`, then restore/delete the fixture. |

### `burp_update_proxy_settings` operations

| Operation | Purpose | Main fields |
|---|---|---|
| `listener_upsert` | Create or replace a listener by port. | `port`; optional `running`, `listen_mode`, `listen_specific_address`, `certificate_mode`, `enable_http2`, `support_invisible_proxying`. |
| `listener_delete` | Delete a listener by port. | `port`. Never remove the only test listener without a recovery path. |
| `script_filter_upsert` | Set a Settings/Script filter for Proxy history, WebSocket history, site map, or Logger. | `target`; optional `mode`, `script`, `script_id`, `script_name`. |
| `script_filter_delete` | Reset a target to Settings mode/default script metadata. | `target`. |
| `intercept_rule_upsert` | Append a rule when `index` is omitted, or replace one rule at a zero-based index. | `kind` (`request` or `response`), `rule`, optional `index`. |
| `intercept_rule_delete` | Delete one interception rule. | `kind`, zero-based `index`. |
| `intercept_toggle` | Change master, request, and/or response interception flags. | At least one of `master_enabled`, `request_enabled`, `response_enabled`. |

Script-filter targets: `proxy_http_history`, `proxy_websocket_history`, `sitemap`, `logger_capture`, and `logger_display`.

## Cookies

| Tool | Purpose | Preliminary live check |
|---|---|---|
| `burp_cookie_jar` | List cookies with optional domain filtering and pagination. Cookie values are sensitive. | Query only the fixture domain and redact values from the report. |
| `burp_cookie_jar_set` | Create or update a cookie. | Set a fixture cookie and read it back. Montoya has no direct delete API; expire it for cleanup when possible. |

## Sessions and macros

Session tools manage rules owned by this extension; they are not unrestricted CRUD over every Burp Session Handling Rule in the UI.

| Tool | Purpose | Preliminary live check |
|---|---|---|
| `burp_session_create_rule` | Create a scoped MCP session rule and return a stable ID. | Create a narrowly scoped fixture rule, verify request behavior, then delete it. |
| `burp_session_get_rule` | Get one session rule by ID. | Compare every returned field with the created fixture. |
| `burp_session_update_rule` | Replace one session rule by ID. | Change one field, get it again, and verify the new behavior. |
| `burp_session_list_rules` | List registered session rules and their scope/action. | Confirm the fixture ID and current fields. |
| `burp_session_delete_rule` | Delete one session rule by ID. | Delete the fixture and confirm it is absent and no longer affects traffic. |
| `burp_macro_create` | Create a macro definition from request items and parameter mappings. | Create a harmless fixture macro, list/run it, then remove it. |
| `burp_macro_list` | List managed macro definitions. | Confirm fixture description, serial number, and items. |
| `burp_macro_run` | Run a macro by description and return each item result. | Verify item count, statuses, response markers, and parameter extraction if configured. |
| `burp_macro_remove` | Remove a macro by description. | Remove the fixture and confirm it no longer appears in the list. |

## Intruder and bounded payloads

`burp_send_to_intruder` only opens the Intruder UI. The bounded fuzzer returns a background job instead of starting an unrestricted Intruder attack.

| Tool | Purpose | Preliminary live check |
|---|---|---|
| `burp_inline_fuzzer` | Start a bounded input-matrix job by substituting a marker in a raw request/template. | Use a tiny wordlist, poll the job, and verify each input/status/length result. |
| `burp_send_to_intruder` | Open a request in Intruder with optional insertion-point ranges. | Verify the Intruder tab and positions; do not expect an attack to start. |
| `burp_intruder_payload_processor_register` | Register one bounded declarative Intruder payload processor. | Register a simple fixture operation, verify it in list/UI, then remove it. |
| `burp_intruder_payload_processor_list` | List processors registered by the extension. | Confirm fixture ID, display name, and operation. |
| `burp_intruder_payload_processor_remove` | Remove a registered processor by ID. | Remove the fixture and confirm it disappears. |
| `burp_intruder_payload_generator_register` | Register a finite payload generator backed by inline payloads or a payload list. | Register two or three values, verify metadata/count, then remove it. |
| `burp_intruder_payload_generator_list` | List generators registered by the extension. | Confirm fixture source, ID, display name, and count. |
| `burp_intruder_payload_generator_remove` | Remove a registered generator by ID. | Remove the fixture and confirm it disappears. |

## Payload lists

Payload lists are managed by the extension and can feed the bounded fuzzer or an Intruder payload generator.

| Tool | Purpose | Preliminary live check |
|---|---|---|
| `burp_payload_list_create` | Create a payload list from inline values. | Create a small fixture list and verify count/content with get/list. |
| `burp_payload_list_import` | Import a payload list from a file visible to the extension process. | Use a known fixture file and verify encoding/line format/count. |
| `burp_payload_list_list` | List payload-list metadata. | Confirm fixture ID, name, source, and count. |
| `burp_payload_list_get` | Return metadata and a page of payloads for one list. | Verify payload representation and pagination. |
| `burp_payload_list_update` | Rename a list or replace its payloads. | Update the fixture and read it back. |
| `burp_payload_list_delete` | Delete one payload list by ID. | Delete the fixture and confirm get/list no longer returns it. |

## Scanner execution and findings

Availability depends on Burp edition. Long-running operations return background jobs and must be polled to a terminal state.

| Tool | Purpose | Preliminary live check |
|---|---|---|
| `burp_scan_start` | Start a passive stateless audit or an active audit with explicit bounded scan options. | Use only an authorized fixture and the edition-supported mode; poll the returned job and inspect status/result. |
| `burp_scan_stop` | Stop an unfinished crawl/audit and record its terminal state. | Start a sufficiently long fixture job, stop it, and confirm it is no longer running. |
| `burp_scan_remove` | Remove a terminal scan job from the registry. | Remove a completed/stopped fixture and confirm status reports it absent. |
| `burp_crawl` | Start a bounded crawl from explicit seed URLs, configuration, scope, and timing. | Verify scope validation, terminal state, and expected site-map growth. |
| `burp_scan_issues` | Page/filter Scanner issues by index, severity, confidence, or URL. | Use a known fixture issue and verify filtering and pagination. |
| `burp_scan_issue_detail` | Return full detail for one Scanner issue index. | Compare name, severity, confidence, URL, and evidence with the fixture. |
| `burp_scanner_generate_report` | Generate an HTML or XML Scanner report for selected issue indexes or all issues. | Generate a small fixture report, verify format/path/issue count, then remove the temporary report. |
## Scanner configuration

| Tool | Purpose | Preliminary live check |
|---|---|---|
| `burp_scan_config_list` | List available scan configurations. | Confirm built-in/project configurations and any fixture config. |
| `burp_scan_config_get` | Get one scan configuration by ID. | Compare name, kind, source, and settings. |
| `burp_scan_config_create` | Create a scan configuration. | Create an isolated fixture, verify it with get/list, then delete it. |
| `burp_scan_config_update` | Replace a scan configuration by ID. | Change one fixture field and read it back. |
| `burp_scan_config_delete` | Delete a scan configuration by ID. | Delete the fixture and confirm it is absent. |
| `burp_scan_pool_list` | List scan resource pools. | Confirm available pools and any fixture pool. |
| `burp_scan_pool_get` | Get one resource pool by ID. | Compare concurrency, throttle, retry, kind, and source fields. |
| `burp_scan_pool_create` | Create a scan resource pool. | Create a low-concurrency fixture, verify it, then delete it. |
| `burp_scan_pool_update` | Replace resource-pool settings by ID. | Change one fixture limit and read it back. |
| `burp_scan_pool_delete` | Delete a resource pool by ID. | Delete only an unused fixture pool and confirm it is absent. |

## Background jobs

| Tool | Purpose | Preliminary live check |
|---|---|---|
| `burp_job_status` | Return state, progress, and summary for one background job. | Poll a valid job to a terminal state and test the unknown-ID error contract. |
| `burp_job_result` | Return a paginated background-job result. | Call after terminal state and verify totals, cursor, errors, and operation-specific fields. |
| `burp_job_cancel` | Cancel an unfinished job and record its final state. | Cancel an isolated fixture and confirm it becomes terminal and stops producing work. |

## Collaborator and custom findings

Collaborator normally requires Burp Suite Professional and a working Collaborator configuration.

| Tool | Purpose | Preliminary live check |
|---|---|---|
| `burp_collaborator_generate` | Generate one or more Collaborator payloads. | Generate a small number and keep secrets within the authorized test workflow. |
| `burp_collaborator_poll` | Poll interactions for a payload secret with a bounded timeout. | Trigger a fixture DNS/HTTP interaction and verify type/time/properties; mark BLOCKED if the environment cannot deliver it. |
| `burp_add_issue` | Add a custom audit issue to the Burp site map. | Use a uniquely named fixture issue and verify it in the UI/API without creating duplicates. |

## Managed WebSockets

These are connections created and owned by the extension. They are distinct from WebSocket traffic observed in Proxy history.

| Tool | Purpose | Preliminary live check |
|---|---|---|
| `burp_websocket_create` | Open a managed WebSocket connection through Burp. | Connect to a fixture echo server and retain the returned ID. |
| `burp_websocket_send_text` | Send a text message on a managed connection. | Send a marker and verify echo/history/direction. |
| `burp_websocket_send_binary` | Send binary data on a managed connection. | Send a small payload and verify byte-equivalent echo. |
| `burp_websocket_close` | Close a managed connection. | Close the fixture ID and verify closed state/error behavior on later sends. |
| `burp_websocket_list` | List managed connections and their state. | Confirm fixture ID, URL, and state before and after close. |
| `burp_websocket_history` | Page through messages for one managed connection. | Verify text/binary entries, direction, sequence, and pagination. |

## Bambda and BCheck import

Import does not mean execution. Import only reviewed source with a unique fixture name.

| Tool | Purpose | Preliminary live check |
|---|---|---|
| `burp_bambda_import` | Validate and import Bambda source into Burp. | Import a harmless fixture and verify it appears in the UI; cleanup may be manual. |
| `burp_bcheck_import` | Validate and import BCheck source into Burp. | Import a harmless fixture and verify parser/UI acceptance; do not expect a scan to start. |

## Sitegraph (advanced opt-in)

Sitegraph is a Rust-owned, project-scoped SQLite graph. It stores normalized structure plus project-local exact HTTP/WebSocket evidence used by `sitegraph_history_search` and exact export. Treat the database as sensitive engagement data. It is disabled and omitted from the MCP tool inventory by default in v3. Restart `burp-mcp` with `--enable-sitegraph` (or `BURP_MCP_ENABLE_SITEGRAPH=true`) to expose these tools; setting only `--sitegraph-project-root` or `--sitegraph-mode` does not enable them.

| Tool | Purpose | Preliminary live check |
|---|---|---|
| `sitegraph_config` | Read the effective auto-index mode and sync interval. Changes must be made in Burp MCP `config.toml` and applied by restart. | Compare the response with the selected configuration file. |
| `sitegraph_sync` | Synchronize the current Burp site-map snapshot into the graph. | Generate known entries, sync, and inspect inserted/updated counts. |
| `sitegraph_status` | Return local sitegraph synchronization and schema status. | Compare index freshness/schema status before and after a fixture sync. |
| `sitegraph_stats` | Return graph ID, mode, project counts, and node/edge totals. | Compare totals before and after fixture sync. |
| `sitegraph_projects` | List known graph project partitions. | Confirm the active graph/project metadata. |
| `sitegraph_search` | Search graph nodes by metadata query. | Find a known fixture host, path, or parameter; expect no raw body/value. |
| `sitegraph_history_search` | Full-text search indexed HTTP request/response and WebSocket payload evidence, optionally filtered by `http`, `websocket`, or `all`. | Search a unique fixture marker, verify source filtering and pagination, and avoid copying sensitive snippets into reports. |
| `sitegraph_neighbors` | Return inbound/outbound neighboring nodes filtered by edge type. | Use a node with a known edge and verify direction/type. |
| `sitegraph_endpoint_detail` | Return endpoint metadata and adjacency counts. | Compare method/path/status/parameters with the fixture. |
| `sitegraph_shortest_path` | Find a shortest path between two nodes. | Use connected fixture nodes and verify endpoints and edge sequence. |
| `sitegraph_impact` | Run a bounded impact traversal from one or more seeds. | Verify depth, visited nodes, and truncation on a small fixture graph. |
| `sitegraph_clusters` | List bounded connected components/clusters. | Confirm known fixture nodes belong to the expected component. |
| `sitegraph_diff` | Report node/edge changes between Unix timestamps. | Sync a baseline, add a fixture, sync again, then diff the two times. |
| `sitegraph_trace` | Trace paths from a source node with direction/depth/edge filters. | Use a small known graph and verify paths never exceed requested depth. |
| `sitegraph_export` | Export a bounded metadata or exact-evidence page as JSON, or metadata as CSV. | Verify metadata export omits raw bodies; request `profile=exact` only when authorized and treat its base64 evidence as sensitive. |

## v3 release checklist

- Confirm the release branch is merged into `main`; the release workflow rejects commits outside `main`.
- Keep the stable version consistent in `Cargo.toml`, `Cargo.lock`, `build.gradle.kts`, the MCP server handler, and `JarPackagingTest.kt`.
- Create an annotated Git tag matching the workspace version (`vX.Y.Z`) and publish a non-prerelease GitHub Release from that exact tag commit.
- Run `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo test --workspace --locked`, and `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`.
- Run `gradle --no-daemon clean test jar -Pversion="X.Y.Z"`; verify the JAR manifest, packaged sitegraph rule-pack checksum, and absence of test classes.
- Run `scripts/run-grpc-interop.sh` and `cargo run -p burp-mcp --locked -- probe --endpoint http://127.0.0.1:9877` against the release extension.
- Build the release bundle, verify `SHA256SUMS`, and inspect the SBOM before publishing assets.
- Smoke-test default MCP startup: `sitegraph_*` tools are absent. Then restart with `--enable-sitegraph` and verify the 15 advanced tools are present.
- Record that sitegraph is intentionally manual opt-in for v3; sitemap graph expansion remains follow-up release scope.

## Offline decoder

| Tool | Purpose | Preliminary live check |
|---|---|---|
| `decoder` | Perform one offline transform, execute a multi-step recipe, search the operation catalog, describe an operation, or produce deterministic magic suggestions. | Test query/describe, one round trip, a multi-step recipe, invalid-operation errors, the 16 MiB input bound, and the 64-step recipe bound. |

`decoder` has no network, filesystem, browser, or arbitrary-code capability. Discover exact operations through its runtime `query` and `describe` modes instead of hard-coding an operation list here.

## Live-test report checklist

Record for every case:

1. Tool name and redacted input.
2. Edition/capability prerequisite.
3. Observable result: response fields, Burp UI state, target effect, or graph state.
4. Cleanup action and read-back proving the baseline was restored.
5. `PASS`, `FAIL`, or `BLOCKED`. Never report PASS from `success: true` alone.
