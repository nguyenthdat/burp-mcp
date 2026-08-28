# Burp Suite workflow reference

Use this reference for common Burp workflows. The links are first-party PortSwigger documentation. The runtime `burp-mcp` tool schema remains authoritative for MCP fields and capabilities.

## Safety baseline

- Work only on systems the operator owns or is explicitly authorized to test. Prefer [Web Security Academy](https://portswigger.net/web-security) labs, staging, or other non-production targets while learning.
- Define a narrow [Target scope](https://portswigger.net/burp/documentation/desktop/tools/target/scope) before active work. Scope filters and redirect rules reduce mistakes; they do not prove authorization.
- Start with passive observation. PortSwigger distinguishes passive auditing, which observes normal traffic, from active auditing, which sends modified requests. Medium active checks can trigger alerts or state changes; intrusive checks can damage data or cause outages. See [Scanner auditing](https://portswigger.net/burp/documentation/scanner/auditing) and [audit settings](https://portswigger.net/burp/documentation/scanner/scan-configurations/audit-settings).
- Do not use Intruder's denial-of-service mode as a default or unattended workflow. State-changing endpoints, brute force, active audit, fuzzing, race tests, and Collaborator traffic require explicit authorization and bounds.

## Proxy and browser setup

Burp's embedded browser is preconfigured and is the safest default. For an external browser:

1. Verify a running Proxy listener under **Tools > Proxy**. The conventional listener is `127.0.0.1:8080`; choose another local free port if occupied. See the [listener checklist](https://portswigger.net/burp/documentation/desktop/external-browser-config/check-listener).
2. Configure the browser's HTTP proxy to the listener. The [external browser guide](https://portswigger.net/burp/documentation/desktop/external-browser-config) and [Firefox instructions](https://portswigger.net/burp/documentation/desktop/external-browser-config/browser-config-firefox) use `127.0.0.1:8080` for all protocols.
3. Install Burp's CA certificate only in the intended testing browser profile. Follow the [CA certificate guide](https://portswigger.net/burp/documentation/desktop/external-browser-config/certificate). Never disclose the CA private key; see [certificate management](https://portswigger.net/burp/documentation/desktop/tools/proxy/manage-certificates).
4. Use **Proxy > Intercept** for deliberate inspection/editing, then turn interception off for normal browsing so requests do not remain blocked. HTTP history continues recording proxied traffic when interception is off. See [intercepting HTTP traffic](https://portswigger.net/burp/documentation/desktop/getting-started/intercepting-http-traffic) and [intercept controls](https://portswigger.net/burp/documentation/desktop/tools/proxy/intercept-messages).

`burp-mcp` captures and surgically modifies focused or last-active text editors and
extension-provided **MCP** tabs across HTTP requests/responses and WebSocket messages
through `burp_editor_get` and `burp_editor_patch`.

The `burp_editor_get` tool leverages multi-tier target discovery:
1. Direct Swing focus on editable editors / extension-provided tabs.
2. Explicit `target_hint` targeting (`"repeater"`, `"request"`, `"websocket"`).
3. Last-Active Editor cache (retained across window switching).
4. Staged buffer from the desktop context menu (**"Send to MCP Active Buffer"**).

The `burp_editor_patch` tool performs surgical mutations without transmitting full payloads:
- `replace_selection`: Replaces only the currently highlighted text slice.
- `set_header`: Adds, updates, or deletes headers with automatic `Content-Length` recomputation.
- `json_patch`: Modifies nested JSON fields via dot-notation.
- `set_param`: Updates query or body parameters.
- `regex`: Regular expression search and replacement.
- Automatic CRLF (`\r\n`) header normalization.
- Adaptive leases with configurable TTL (default 120s) and renewal via `burp_editor_renew_lease`.
## MCP-owned interception queues

The MCP queues are separate from master Proxy Intercept state and Proxy history.
Use them only for a narrow authorized fixture:

1. Read/configure `burp_intercept_controller` or
   `burp_websocket_intercept_controller` with a bounded timeout.
2. Generate one scoped message.
3. Page the matching pending queue; retain one stable ID.
4. Forward, drop, or send that ID to manual Intercept. Replace complete HTTP
   messages or WebSocket payload bytes only from reviewed base64.
5. Confirm `pending` is zero and disable the controller. Messages auto-forward
   on timeout, but timeout is a failsafe rather than cleanup.

## Scope and logging

Use [Target scope](https://portswigger.net/burp/documentation/desktop/tools/target/scope) to include exact authorized protocols, hosts, ports, and paths and exclude unsafe or irrelevant areas. PortSwigger's [test-scope workflow](https://portswigger.net/burp/documentation/desktop/testing-workflow/test-scope) recommends excluding unauthorized, unsafe, and irrelevant URLs before testing.

- Scope can filter site map and Proxy history, constrain logging/interception, constrain Repeater/Intruder redirects, and control Professional live tasks.
- Normal scope uses URL prefixes; advanced scope adds protocol, host/IP, port, and path rules.
- Scope changes do not retroactively delete history. Treat existing history as retained engagement data.

[Proxy HTTP history](https://portswigger.net/burp/documentation/desktop/tools/proxy/http-history) records browser-proxied HTTP traffic even when interception is off. Its [display filters](https://portswigger.net/burp/documentation/desktop/tools/proxy/http-history/filter-settings) hide entries but do not delete them. Use narrow filters and pagination before clearing anything.

[Logger](https://portswigger.net/burp/documentation/desktop/tools/logger) records HTTP generated by Burp tools, Scanner, session handling, and extensions. Use `burp_logger_history` to inspect traffic across all sources (Proxy, Repeater, Scanner, Intruder, Extensions) and `burp_logger_detail` for full payloads.

[Organizer](https://portswigger.net/burp/documentation/desktop/tools/organizer) allows triage and tracking of interesting HTTP requests/responses. Use `burp_organizer_send` to save annotated requests/responses to Organizer and `burp_organizer_list` to review saved items.

## Response Diffing & Comparer

Use `burp_diff_responses` to compare two HTTP responses (from strings or proxy/logger indexes) and calculate:
- Similarity ratio ($0.0 \dots 1.0$)
- Status code delta and response length differences
- Header differences (added, removed, modified)
- Unified line-by-line body diff

Use `burp_send_to_comparer` to send two raw HTTP payloads directly to Burp's desktop Comparer tab for visual inspection.

## Race Conditions & Single-Packet Attack

Standard multi-threading is often insufficient for tight race conditions due to TCP network jitter. Use `burp_race_condition` with `single_packet_attack: true`:
- Establishes $N$ parallel connection streams.
- Pre-sends request prefix to warm up connections.
- Uses Last-Byte Synchronization to simultaneously release the final request byte across all connections in a single microsecond-synchronized burst.

## Multi-Marker Fuzzing

Use `burp_inline_fuzzer` with `attack_mode`:
- `pitchfork`: Lockstep parallel iteration across multiple payload sets (`(u1, p1), (u2, p2)...`).
- `cluster_bomb`: Cartesian product across all payload sets (`(u1, p1), (u1, p2)...`).
- `sniper`: Tests each marker position sequentially with individual payload lists.

## Compound Security Workflows

High-level automated workflows reduce round-trips:
- `burp_verify_idor`: Automated IDOR verification sending requests with original and victim auth headers and analyzing similarity / pattern leaks.
- `burp_check_cors`: Comprehensive CORS audit against test origins (`evil.com`, `null`, subdomain trusts) checking `Access-Control-Allow-Origin` and `Access-Control-Allow-Credentials`.
- `burp_auth_matrix`: Matrix testing of endpoints against multiple roles (Admin, User, Unauthenticated/Guest) with automated access control violation detection.
## Repeater

[Repeater](https://portswigger.net/burp/documentation/desktop/tools/repeater) modifies and resends HTTP or WebSocket messages. Common uses:

- vary one parameter and compare response status, length, headers, body, and timing;
- reproduce a sequence of dependent requests in a stable order;
- manually verify a Scanner finding before reporting it;
- retain request history in a named tab for evidence.

Typical flow from PortSwigger's [reissuing requests guide](https://portswigger.net/burp/documentation/desktop/getting-started/reissuing-http-requests): select an HTTP history entry, send it to Repeater, send a baseline unchanged request, modify one variable, resend, and use history navigation to compare results.

Keep redirect handling in-scope unless the operator asks otherwise. Redirects and cookie processing can cross boundaries or alter session state.

## Intruder

[Intruder](https://portswigger.net/burp/documentation/desktop/tools/intruder) repeats a request with payloads at defined positions. PortSwigger lists identifier enumeration, controlled fuzzing, subdomain discovery, data harvesting, and login testing among its [uses](https://portswigger.net/burp/documentation/desktop/tools/intruder/uses).

Safe operating pattern:

1. Capture a stable baseline request in HTTP history.
2. Highlight only the intended value and send it to Intruder.
3. Verify the `§` payload positions and choose the least complex attack type, commonly Sniper for one payload set tested one position at a time.
4. Use a small explicit payload list and low request/concurrency bounds.
5. Sort and compare response status/length and inspect anomalous responses.
6. Keep redirects in-scope. Redirect following can log out a session or trigger additional state changes.

See [getting started with Intruder](https://portswigger.net/burp/documentation/desktop/tools/intruder/getting-started) and [attack settings](https://portswigger.net/burp/documentation/desktop/tools/intruder/configure-attack/settings).

## Scanner

[Burp Scanner](https://portswigger.net/burp/documentation/scanner) is available in Professional/DAST editions. Crawling discovers content and builds a map; auditing sends requests and analyzes behavior. The desktop [scan workflow](https://portswigger.net/burp/documentation/desktop/running-scans) supports crawl, full crawl-and-audit, and auditing selected items.

Burp MCP's `burp_scan_start` interface supports passive and active audits. Other bounded crawl/audit tools depend on the connected edition and advertised capabilities. Never assume an active scan is safe because it is automated.

For permitted scans:

- begin with a precise start URL and scope;
- prefer a low-impact preset such as Lightweight or Fast before broader modes; see [preset scan modes](https://portswigger.net/burp/documentation/scanner/scan-configurations/preset-scan-modes);
- disable checks that are outside authorization or may modify state;
- monitor task status and Logger, and stop on scope drift or unexpected effects;
- manually verify findings in Repeater before reporting them.

PortSwigger's [first scan guide](https://portswigger.net/burp/documentation/desktop/getting-started/running-your-first-scan) recommends learning on non-production targets and never scanning third-party systems without owner authorization.

## Burp Collaborator (OAST)

[Burp Collaborator](https://portswigger.net/burp/documentation/desktop/tools/collaborator) is Burp's out-of-band application security testing (OAST) infrastructure. It detects blind vulnerabilities where a target system interacts with an external server over DNS, HTTP/HTTPS, or SMTP.

Safe OAST operating pattern:

1. Generate distinct payloads with `burp_collaborator_generate` and assign one
   unique payload to each injection parameter or header.
2. Inject payloads into suspected parameters (`url`, `redirect`, `dest`,
   `webhook`, `callback`), headers (`Host`, `X-Forwarded-For`, `Referer`), or
   XML/template/database contexts.
3. Poll at a paced interval with `burp_collaborator_poll({limit, cursor?})`.
4. Distinguish `DNS` interactions from `HTTP`/`HTTPS` interactions.
5. Detailed OAST vulnerability patterns (Blind SSRF, Blind SQLi, Blind XXE, Blind RCE, Log4Shell, and Deserialization) are documented in [`appsec-testing-guide.md`](./appsec-testing-guide.md).

## Extensions

[Burp extensions](https://portswigger.net/burp/documentation/desktop/extend-burp/extensions) can modify traffic, send requests, add Scanner checks, and access Burp state. Treat every extension as executable code.

- For BApp Store installs, review source and resource/traffic impact; PortSwigger reviews submissions but does not guarantee quality or security. See [BApp installation](https://portswigger.net/burp/documentation/desktop/extend-burp/extensions/installing/bapp-store).
- Review manually loaded JAR/Python/Ruby/BApp code and provenance before using [manual installation](https://portswigger.net/burp/documentation/desktop/extend-burp/extensions/installing/manual-install).
- Check Logger after enabling an extension to understand generated traffic.

## Site map terminology

Do not conflate these three concepts:

1. Burp **Target > Site map** is Burp's internal hierarchical view, populated by proxy browsing, Scanner, content discovery, and inferred content. Gray items may be inferred and never requested. See [Target site map](https://portswigger.net/burp/documentation/desktop/tools/target/site-map).
2. A target's `/sitemap.xml` is a server resource. Scanner crawl settings can optionally request it and extract links. See [crawl settings](https://portswigger.net/burp/documentation/scanner/scan-configurations/crawl-settings).
3. Burp MCP **sitegraph** is a separate optional Rust/SQLite metadata graph. It neither means nor automatically requests `/sitemap.xml`. Read [the dedicated sitegraph reference](./sitegraph.md) before enabling it.
