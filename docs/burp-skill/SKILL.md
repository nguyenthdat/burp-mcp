---
name: burp-skill
description: >-
  Operate an already configured burp-mcp server for authorized web application
  security work: inspect Burp Proxy HTTP/WebSocket history, scope, and site map;
  replay requests; use Repeater/Intruder, passive audits, bounded fuzz/race
  jobs, Collaborator, managed WebSockets, macros/session rules, write and
  import Bambda or BCheck scripts, run offline decoding, and query the
  persistent site graph. Use when asked to assess, investigate, reproduce,
  annotate, automate, or document a target through Burp Suite.
  Do not use to implement or debug burp-mcp itself, or to test a target without
  explicit authorization.
---

# Burp Skill

Use the existing `burp-mcp` server as a controlled interface to Burp Suite. The
goal is reproducible security evidence with bounded side effects, not maximum
tool activity.

Tool names below are server-local names. A client may expose them with an MCP
server prefix. The runtime tool schema is authoritative; never invent a field
that is absent from the exposed schema.

## Non-negotiable boundaries

1. **Require authorization.** Identify the exact target and allowed activity
   before sending traffic, fuzzing, crawling, polling Collaborator, or changing
   Burp state. Existing Burp scope is a routing control, not proof of
   authorization. If authorization or the target boundary is missing, ask one
   direct question before any active action.
2. **Start read-only.** Prefer Proxy history, site map, scanner issues, target
   info, sitegraph, and offline decoder operations. Add traffic or mutate Burp
   state only when the task requires it.
3. **Preserve operator state.** Record every temporary scope entry, intercept
   change, handler, proxy rule, session rule, macro, cookie, job, and managed
   WebSocket. Restore or remove it before finishing unless the user explicitly
   asks to keep it.
4. **Do not enable interception in unattended flows.** Intercepted requests can
   remain blocked in the Burp UI, and this MCP surface has no forward/drop
   operation. Read `burp_intercept_state` before automated traffic; if it is
   enabled, ask before temporarily disabling it and restore the original state.
5. **Do not request active scanning.** `burp_scan` supports `mode: "passive"`;
   `mode: "active"` returns an unsupported error. Burp edition and advertised
   capabilities may further limit crawl, audit, Collaborator, or Scanner tools.
6. **Keep every operation bounded.** Page through results, use narrow URL
   filters/prefixes, keep parallel batches small, and use the minimum useful
   fuzz wordlist or race count. Never create an unbounded polling loop.
7. **Verify effects, not acknowledgements.** A successful handler, session,
   cookie, scope, macro, request, or job call proves only that the call was
   accepted. Verify the observable result through Proxy detail, job results,
   the cookie/rule list, or the relevant read API.
8. **Minimize sensitive output.** Do not echo authentication tokens, session
   cookies, full bodies, or Collaborator payloads unless they are necessary
   evidence. The local sitegraph is intended for endpoint metadata and
   parameter names, not parameter values or message bodies.

## Workflow

### 1. Establish connection and boundary

1. Inspect the available Burp MCP tools and their schemas.
2. Call `burp_burp_version`. Record the extension version, Burp edition,
   advertised capabilities, page limit, response limit, and RPC timeout.
3. Call `burp_extension_info` only when extension/process metadata matters.
4. State the authorized target as scheme, host, port, and path boundary. Use
   `burp_get_scope` to inspect scope. Add a URL with `burp_add_to_scope` only
   when the user authorized that state change; track temporary additions.
5. Read `burp_intercept_state` before generating automated traffic.
6. Choose one workflow below. Do not call unrelated tools merely because they
   are available.

If the connection call fails, report that Burp Suite plus the Burp MCP
extension must be running and connected. Do not simulate results or retry in a
busy loop.

### 2. Observe existing evidence

Use the smallest useful path:

1. Inventory: `burp_target_info` and `burp_sitemap` with a target URL prefix.
2. Traffic: `burp_proxy_history` with URL, method, status, notes, or highlight filters.
3. Detail: call `burp_proxy_detail` only for selected history indexes.
4. WebSockets: use `burp_proxy_websocket_history`; decode a payload with
   `decoder` only when needed.
5. Findings: use `burp_scan_issues`, then `burp_scan_issue_detail` for selected
   indexes.
6. Triage: use `burp_highlight` or `burp_annotate` only when persistent Burp
   annotations are part of the requested result.

Honor `truncated` and follow `next_cursor`. Stop when enough evidence answers
the question; do not dump an entire project by default.

### 3. Reproduce or vary one request

1. Obtain the source request from `burp_proxy_detail`, or construct a structured
   request from the user-provided URL, method, headers, and body.
2. Preserve all irrelevant bytes and change only the field under test. Use
   `decoder` operations such as `http.parse`, `http.set_body`, or
   `http.update_content_length` when they make the change safer.
3. Choose the execution surface deliberately:
   - `burp_send_to_repeater`: open a raw request for human review; it does not
     send the request.
   - `burp_send_request`: send one structured request and return the response.
   - `burp_send_request_parallel`: compare a bounded set of independent
     requests; at most 32 per call.
   - `burp_send_to_intruder`: open one raw request in Intruder; it does not run
     an attack.
4. Re-read the resulting Proxy entry and compare status, headers, length, and
   the minimum relevant body fragment. Do not treat status code alone as proof.
5. Use `burp_convert_request`, `burp_export_request`, or
   `burp_extract_from_response` only for a concrete conversion, handoff, or
   extraction need.

### 4. Run a bounded asynchronous check

Use only within the explicit authorized boundary:

- `burp_race_condition` for a bounded concurrent comparison of one raw request.
- `burp_inline_fuzzer` for one request template and one bounded wordlist; use a
  unique marker such as `FUZZ`.
- `burp_scan` with `mode: "passive"` for passive audit processing.
- `burp_crawl` when the runtime capability and Burp edition support it.

For every returned job ID:

1. Store the job ID and operation.
2. Poll `burp_job_status` at a paced interval until the returned state is
   terminal. Do not tight-loop.
3. On success, paginate `burp_job_result` with `next_cursor`.
4. On failure, report the job error as evidence; do not relabel it as a clean
   result.
5. On abort, timeout, scope change, or user cancellation, call
   `burp_job_cancel` and report the final observed state.

Use `burp_collaborator_generate` only for a specific authorized out-of-band
check. Poll with `burp_collaborator_poll` for a bounded period, correlate the
interaction ID and time, then stop.

### 5. Work with WebSockets

For live protocol testing, use `burp_websocket_create`, then text or base64
binary sends, and always `burp_websocket_close` in cleanup. Use
`burp_websocket_list` to detect leaked managed connections. Keep Proxy
WebSocket history separate from managed outbound connections: history observes
Burp traffic; managed tools create new traffic.

### 6. Change Burp automation state only when required

Persistent/high-impact tools include configuration import, HTTP handlers,
proxy rules, session rules, macros, cookies, scope, annotations, Bambda/BCheck
imports, and intercept state.

Before a change:

1. Read the current state when a read API exists.
2. Define the expected observable effect and cleanup operation.
3. Apply one change.
4. Generate one controlled verification request.
5. Verify the effect in `burp_proxy_detail` or the corresponding list/read tool.
6. Remove the temporary change immediately.

Use `burp_export_config` before `burp_import_config`. Imports can affect the
whole Burp project; never import configuration as a speculative fix. Create or
run macros only from reviewed raw requests, and remove temporary macros
afterward.

### 7. Write or import a Bambda or BCheck

Treat both formats as persistent executable Scanner/extension content. Import
only when the user requests a Burp library change; otherwise return reviewed
source without importing it.

1. Choose the format by contract:
   - Bambda: Java-based filters, custom columns, Repeater actions,
     match-and-replace, or Java/Montoya scan checks.
   - BCheck: declarative Scanner checks using BCheck control flow and request
     actions.
2. Load the relevant authoring reference before drafting or reviewing source:
   - [`references/bambda-authoring.md`](./references/bambda-authoring.md)
   - [`references/bcheck-authoring.md`](./references/bcheck-authoring.md)
3. Review the complete source, not a fragment. Reject embedded credentials,
   arbitrary process execution, unbounded loops/requests, unrelated I/O,
   destructive behavior, and passive checks that generate traffic.
4. For active BChecks, calculate the maximum request count and import with
   `enabled: false`. Do not enable merely because the script compiles.
5. For Bambdas, preserve a stable UUID when updating and take `function`,
   `location`, variables, and return type from a current Burp template.
6. Accept an import only when the tool returns `success: true`,
   `LOADED_WITHOUT_ERRORS`, and no import errors.
7. Import success is compilation/load evidence, not behavioral proof. Verify on
   controlled positive and negative fixtures in Burp. The current MCP surface
   cannot list, execute, enable/disable, or delete imported scripts; report that
   limitation instead of claiming cleanup.

### 8. Use offline transforms and the sitegraph

For deterministic local transforms, use `decoder` without generating network
traffic:

1. Search the catalog with `query` or inspect one operation with `describe`.
2. Use exactly one `operation`, a bounded `steps` recipe, or `magic: true`.
3. Preserve tagged input types: `text`, base64-backed `bytes`, or `json`.
4. Treat MD5 and SHA-1 output as compatibility data, not secure cryptography.

For persistent target structure:

1. `sitegraph_sync` with the narrowest useful URL prefix.
2. Check `sitegraph_status` or `sitegraph_stats`.
3. Use `sitegraph_search`, then `sitegraph_endpoint_detail`.
4. Use `sitegraph_neighbors` or `sitegraph_trace` only from a known endpoint ID.
5. Use `sitegraph_diff` for changes since a known Unix timestamp.
6. Use `sitegraph_export` only when the user needs a bounded JSON/CSV handoff.

Read [`references/tool-catalog.md`](./references/tool-catalog.md) when choosing
exact tools, fields, limits, decoder operation IDs, or cleanup pairs. Load the
Bambda/BCheck authoring references only for script creation, review, or import.

## Evidence standard

A finding needs:

- authorized target and relevant scope boundary;
- exact request variant or history index;
- response status plus the decisive header/body/length observation;
- repeatability or a clearly stated one-shot condition;
- confidence and plausible alternative explanations;
- cleanup state.

Use `burp_add_issue` only after validating the finding and only when the user
wants it persisted in Burp. Do not turn a payload reflection, timeout, length
change, or tool error into a vulnerability claim without demonstrating the
security impact.

## Output

Return a compact assessment record:

```text
Target: <authorized boundary>
Burp: <edition/version and relevant capability>
Actions: <tools and bounded parameters, excluding secrets>
Evidence: <history indexes, request variants, decisive observations>
Findings: <validated result, confidence, impact; or no demonstrated issue>
State changes: <created, restored, removed, or intentionally retained>
Limitations: <unsupported feature, edition restriction, truncation, or error>
```

Never claim full coverage from a bounded page, passive audit, sitegraph sync,
or a single request path.
