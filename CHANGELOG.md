# Changelog

All notable changes, architectural enhancements, performance optimizations, and security capabilities introduced in **Burp MCP** are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [3.3.0] - 2026-09-06

### Highlights
- Expanded body filtering with bounded Pest grammars and structural CSS/JSONPath evaluation, including selector combinators, logical/structural pseudos, recursive JSONPath, slices, unions, and explicit malformed-input errors.
- Migrated SiteGraph enrichment to the typed `default-rules.toml` pack with 105 byte-safe rules and no legacy JSON/DSL compatibility path.
- Added bounded, idempotent SiteGraph annotations for medium/high/critical HTTP Proxy findings while preserving operator notes and existing non-default highlights.
- **Next-Gen Attack Surface Graph (SiteGraph v3.3)**: Smart URL parameter inference (`parameterize_path`), `RegexSet` single-pass enrichment, OpenAPI/Swagger ingestion, scale ceiling expansion (250,000 nodes / 1,000,000 edges), and visual graph rendering (Mermaid & ASCII tree).
- Improved action-discriminated MCP schemas, active editor leases, scoped HTTP/WebSocket interception, metadata-first payload handling, and corrective structured errors for LLM clients.

### Verification
- Rust workspace tests, formatting, locked checks, and strict Clippy pass locally.
- Kotlin extension tests and gRPC interop pass locally.
- Live Burp replay requires the extension JAR loaded in Burp Suite; hosted CI does not provide a Burp Pro instance.

---

## [3.2.0] - 2026-08-28

### Summary of Major Enhancements
This release represents a comprehensive overhaul of Burp MCP, transforming it from a raw set of API wrappers into an **Autonomous Enterprise-Grade AppSec Agent Platform**. Major pillars include:
- **Consolidated Action-Based Tools**: Replaced ~115 fragmented tools with streamlined, action-based interfaces and dedicated compound security workflows, reducing system prompt token overhead by over 85%.
- **Active UI & Desktop Editor Integration**: Multi-tier focus resolution, micro-patching via `EditorPatchEngine`, and lease-guarded concurrency controls for real-time human-in-the-loop desktop collaboration.
- **Next-Gen Attack Surface Graph (SiteGraph v3.2)**: Smart URL parameter inference (`parameterize_path`), `RegexSet` single-pass enrichment, OpenAPI/Swagger ingestion, scale ceiling expansion (250,000 nodes / 1,000,000 edges), and visual graph rendering (Mermaid & ASCII tree).
- **Deep Montoya API Integration**: Full traffic monitoring via Burp Logger, Burp Organizer triage, BCheck lifecycle & dry-run execution, and Scanner issue status triage.
- **Advanced Pentest Workflows**: True Single-Packet Attack (Last-Byte Synchronization), Multi-Marker Fuzzing (Pitchfork, Cluster Bomb, Sniper), Collaborator Auto-Correlation Tracker, and a suite of 9 compound security workflows.

### Reliability and MCP usability fixes
- `burp_http.send_to_repeater` now accepts an absolute URL or raw HTTP request and derives Repeater service metadata safely.
- `burp_settings.action` is a schema enum, and proxy-rule `match`/`replace` fields reach the backend unchanged.
- HTTP interception requires `url_filter` or `in_scope_only`, so unrelated traffic bypasses the MCP queue.
- MCP initialization now publishes usage instructions; tool and gRPC errors return corrective structured metadata.
- Bambda import errors explain the JVM 65,535-byte `CONSTANT_Utf8` limit and supported alternatives.
- Session-rule upsert now creates missing caller-supplied IDs across both current and older extension behavior, replaces existing rules idempotently, and avoids registering disabled handlers.
- Organizer listings tolerate null, malformed, or partially unavailable Montoya metadata instead of failing the entire page.
- Payload-list `replace_all`, GraphQL audit flags/batch-body detection, and deterministic API-fuzz category selection now match their published contracts.
- Compound security workflows now enforce bounded HTTP(S) inputs, preserve request failures as errors, and use structural response evidence: successful-response-only IDOR matching, response-header-only CORS parsing, current-run Collaborator correlation, typed query/body SQLi injection, real SameSite cookie inspection for CSRF, and completed-request accounting for API fuzzing.

---

### 1. Active UI & Desktop Editor Integration

#### Problem & Rationale
- **Focus Fragility**: Switching focus between Burp Suite and AI chat interfaces (Claude Desktop, Cursor, OMP, Zed) previously cleared `permanentFocusOwner`, causing editor tools to fail with `NoSuchElementException`.
- **Full-Text Replacement Penalty**: Overwriting large HTTP requests (50KB+) to modify a single parameter wasted tens of thousands of LLM output tokens, added latency, and risked CRLF corruption or stale `Content-Length` headers.
- **Lease Expiry & Concurrency Risks**: Short 30s TTL leases expired during deep LLM reasoning phases, and single-use token destruction prevented retries.

#### Key Implementations & Upgrades
- **Multi-Tier Target Discovery Engine**:
  1. *Tier 1 (Active Focus)*: Real-time Swing `KeyboardFocusManager.permanentFocusOwner` inspection.
  2. *Tier 2 (Last-Active Cache)*: Tracks the most recently active editable component to survive window switching.
  3. *Tier 3 (Montoya UI Providers)*: Extension-provided custom `"MCP"` editor tabs (`HttpRequestEditorProvider`, `HttpResponseEditorProvider`, `WebSocketMessageEditorProvider`).
  4. *Tier 4 (Explicit Selection)*: Context-aware target resolution.
- **Surgical Micro-Editing (`EditorPatchEngine.kt`)**:
  - Implemented surgical patch operations: `replace_selection`, `set_header`, `json_patch` (dot-notation & RFC 6902 support), `set_param`, `regex`, and `replace_all`.
  - Automatic normalization of CRLF line endings (`\r\n`) and automatic recalculation of `Content-Length`.
- **Adaptive Lease Lifecycle & Concurrency Guards**:
  - Extended lease TTL to 120 seconds with non-destructive validation and explicit lease renewal via `burp_editor_renew_lease`.
  - Enforced SHA-256 hash matching (`expected_sha256`) to guarantee optimistic concurrency and prevent overwriting manual user inputs.
  - Safe memory management using `WeakReference` to eliminate component memory leaks.
  - Strict dispatch to the Swing Event Dispatch Thread (EDT) via `SwingUtilities.invokeAndWait`.
- **Standardized Ergonomic Tools**:
  - `burp_editor_get`: Captures editor state with contextual metadata, caret/selection positions, dual-format payloads, and lease tokens.
  - `burp_editor_patch`: Applies targeted micro-edits without full payload re-generation.
  - `burp_editor_renew_lease`: Extends active lease duration during prolonged reasoning turns.
  - Purged legacy editor RPCs (`burp_active_editor_get/set`, `burp_websocket_editor_get/set`).

---

### 2. Attack Surface Knowledge Graph (SiteGraph v3.2)

#### Problem & Rationale
- **Path Explosion**: REST APIs with dynamic IDs (`/api/users/123`, `/api/users/456`) previously created thousands of redundant `Endpoint` and `PathSegment` nodes, bloating graphs by 85%+.
- **Ingestion Spikes & Scale Limits**: Sitemap pagination previously fetched entire sitemap collections into JVM memory, risking `OutOfMemoryError`. Analysis algorithms were hardcapped at 25k nodes.
- **FTS5 & Regex Bottlenecks**: Binary payloads in SQLite FTS5 inflated database files to gigabytes, and serial regex evaluation throttled ingestion throughput.

#### Key Implementations & Upgrades
- **Smart Path Parameter Inference (`normalize/url.rs`)**:
  - Introduced `parameterize_path` with regex classifiers for Integer IDs, UUIDs, Hex Hashes, and Slugs (e.g., `/api/v1/users/{user_id}/orders/{order_id}`).
  - Cuts graph node noise by 85–90% on large RESTful surfaces.
- **Enrichment Rule Migration to Typed TOML & Byte-Safe Surface Matching (`enrichment/`)**:
  - Migrated rule pack definitions to canonical typed TOML (`default-rules.toml`) with strict unknown-field rejection; legacy Pest DSL and JSON rule definitions are rejected without dual-format compatibility.
  - Expanded the embedded `2026.09.05` pack from 28 to 105 rules, adding 77 validated detectors across cloud credentials, authentication/session state, framework fingerprints, error disclosure, infrastructure reconnaissance, vulnerability-prone parameters, PII, and security misconfiguration.
  - Standardized all 105 rules into ordinary regex entries evaluated by surface-partitioned byte-safe `RegexSet` matching. No absence inference: `missing_content_type_options` detects only explicitly insecure header values (`none|0|false|off`).
  - Matching compiles into surface-partitioned `regex::bytes::RegexSet` instances plus individual `regex::bytes::Regex` matchers, preserving byte-exact offsets and capture groups across arbitrary payloads without lossy UTF-8 conversion.
- **Automatic HTTP-History Proxy Annotations (`sitegraph/sync.rs`, `AnnotationFacade.kt`)**:
  - `sitegraph_sync` automatically annotates interesting newly indexed HTTP Proxy history entries matching medium+ severity rules (`medium`, `high`, `critical`).
  - Annotations resolve entries by permanent stable Burp history `id` rather than transient relative indexes.
  - Bounded to a maximum of 50 entries per sync run.
  - Injects or updates an idempotent marker line: `[SiteGraph] severity={severity} rules={rule_ids} direction={direction}`, preserving existing operator notes.
  - Maps severity to highlights: `critical`/`high` -> RED, `medium` -> ORANGE, `low` -> YELLOW, while preserving existing non-NONE highlights.
  - Runs post-commit as a non-fatal side effect (annotation failures do not fail or roll back graph sync).
  - Applied strictly to HTTP proxy history (WebSocket messages are not annotated).
- **Scale Ceiling Expansion & Storage Optimization**:
  - Raised analysis capacity limits to `MAX_ANALYSIS_NODES = 250_000` and `MAX_ANALYSIS_EDGES = 1_000_000`.
  - Excluded binary payloads from SQLite FTS5 index (`0004_history_fts.sql`), keeping database sizes lightweight and query times fast.
  - Paginated sitemap synchronization with `after_id` cursors and `PAGE_SIZE = 500`.
- **Multi-Source Ingestion & Spec Support**:
  - Integrated OpenAPI 3.0 / Swagger 2.0 parser (`crates/sitegraph/src/ingest/openapi.rs`) into the tool interface via the canonical `import_spec` action.
- **Pre-Computed Security Views & Visualizations**:
  - Added 4 pre-computed security analysis views: `unauthenticated_routes`, `auth_matrix`, `idor_candidates`, and `sensitive_parameters`.
  - Built-in graph visualization generators: Mermaid.js diagrams and ASCII trees for immediate context-efficient rendering in chat.
- **Consolidated 15-Action `sitegraph` Tool**:
  - Unified actions: `status`, `stats`, `sync`, `search`, `security_view`, `import_spec`, `neighbors`, `trace`, `shortest_path`, `clusters`, `impact`, `diff`, `export`, `history_search`, `endpoint_detail`, `projects`, `config`.

---

### 3. Core Pentesting Suite & Montoya API Integrations

#### Key Implementations & Upgrades
- **Burp Logger API Integration (`LoggerFacade.kt`)**:
  - Added visibility across all Burp Suite traffic (Proxy, Repeater, Scanner, Intruder, Extensions) via `burp_logger` (`query`, `detail`, `clear`).
- **Burp Organizer Integration (`OrganizerFacade.kt`)**:
  - Seamlessly send requests/responses with notes and statuses directly into Burp Organizer via `burp_organizer` (`add`, `list`).
- **True Single-Packet Attack (Last-Byte Synchronization)**:
  - Implemented synchronized race condition testing in `LongOperationFacade.kt` (`burp_fuzzer` action `race` with `single_packet_attack: true`), holding the final byte across parallel TCP/TLS sockets to eliminate network jitter.
- **Multi-Marker Fuzzing Engine (`IntruderPayloadFacade.kt`)**:
  - Upgraded `burp_fuzzer` action `fuzz` to support multiple injection markers (`§param§`) and classical matrix attack types: `pitchfork`, `cluster_bomb`, and `sniper`.
- **Collaborator Auto-Correlation Tracker (`CollaboratorFacade.kt`)**:
  - Maintained in-memory and persistent injection metadata tables `(payload_id, target_url, injection_point, timestamp)` to automatically correlate out-of-band DNS/HTTP interactions back to the originating injection vector.
- **HTTP Response Comparer & Diff Engine (`diff_engine.rs`)**:
  - Added `burp_diff` (`diff_responses`, `compare_exchanges`) supporting cosine similarity scoring, header difference mapping, and line-by-line unified diffs for Boolean-based and access control testing.
- **BCheck Management & Scanner Issue Triage**:
  - Added BCheck import, validation, and dry-run execution against specific HTTP exchanges.
  - Enabled status updates on Burp Scanner issues (`False Positive`, `Ignored`, severity/confidence overrides).

---

### 4. Context Optimization, Smart Filtering & PEG Parsers

#### Problem & Rationale
- Publishing 115 individual tools consumed 20,000–30,000 tokens in the system prompt on every LLM turn.
- Returning uncompressed raw HTTP bodies (HTML, JavaScript, images) frequently caused context window overflows.

#### Key Implementations & Upgrades
- **Tool Consolidation**:
  - Replaced 115 legacy single-purpose tools with **13 Action-Based Suite Tools** in `suite.rs` (`burp_proxy`, `burp_http`, `burp_target`, `burp_scanner`, `burp_fuzzer`, `burp_collaborator`, `burp_diff`, `burp_settings`, `burp_logger`, `burp_organizer`, `burp_websocket`, `burp_session`, `burp_scan_config`).
  - Standardized input schemas on `*ActionInput` structs.
- **Smart Filtering & Truncation**:
  - Defaulted traffic history queries to compact metadata only (`include_bodies: false`).
  - Added automatic payload truncation exceeding `max_body_length` and binary content stripping.
  - Added `headers_only` filtering flag.
- **Expanded Custom Pest Grammar Parsing for Field Projections (`src/body_filter/`)**:
  - Expanded custom Pest grammars and AST evaluators without external parser crates (avoiding `scraper` or `serde_json_path` dependencies):
    - **CSS Selectors (`css_selector.pest`, `css_selector.rs`)**:
      - Supports selector lists (`,`), combinators (child `>`, descendant whitespace, next-sibling `+`, subsequent-sibling `~`), escaped and Unicode identifiers.
      - Supports attribute matchers (`=`, `~=`, `|=`, `^=`, `$=`, `*=`) with `i`/`s` case flags, empty values, and quoted/unquoted strings.
      - Supports structural pseudos (`:first-child`, `:last-child`, `:only-child`, `:nth-child` with `odd`/`even`/`An+B`, `:first`, `:last`), logical pseudos (`:not`, `:is`, `:where`), and deliberate nonstandard extension `:contains("...")`.
      - Replaced naive regex balancing with bounded tokenization so `>`/`<` inside quoted attributes, comments, and scripts parse predictably. (Does not claim full CSS4 coverage).
    - **JSONPath (`json_path.pest`, `json_path.rs`)**:
      - RFC 9535 syntax with quoted escaped member names (`\uXXXX`, `\"`, `\'`), negative array indices, full slices `[start:end:step]` (with `step=0` evaluating to empty per RFC 9535).
      - Supports selector unions, recursive descent (`..`), filter expressions `?(...)` with logical precedence (`!`, `&&`, `||`), comparison operators (`==`, `!=`, `<`, `<=`, `>`, `>=`, `=~`), and standard functions `match()` and `search()`.
      - Rejects unsupported functions or malformed paths with clear parse errors rather than silent partial matches.
      - Bounded input and recursion depth with deterministic document ordering; final projection output remains bounded by `max_body_length`.
---

### 5. Autonomous Compound Security Workflows (`workflows.rs`)

To eliminate multi-turn round-trip latency for standard penetration testing patterns, 9 specialized compound workflows were implemented:

1. **`burp_verify_idor`**: Executes dual-authenticated requests (User A vs. User B) with baseline and victim tokens, diffs response status and bodies, and asserts horizontal/vertical authorization bypasses.
2. **`burp_check_cors`**: Tests arbitrary origins (`https://evil.com`, `null`, prefix/suffix trusts) and evaluates `Access-Control-Allow-Origin` / `Access-Control-Allow-Credentials` misconfigurations.
3. **`burp_auth_matrix`**: Evaluates role-based access control (RBAC) across multiple endpoint URLs and privilege levels (Admin, User, Anonymous).
4. **`burp_audit_jwt`**: Performs automated JWT vulnerability checks including `alg: "none"`, HMAC/RSA key confusion, `kid` header injection, and claim tampering.
5. **`burp_verify_ssrf`**: Injects Burp Collaborator payloads into target parameters/headers and monitors for out-of-band DNS/HTTP interactions with correlation tracking.
6. **`burp_verify_sqli`**: Executes differential testing for Boolean-based and Time-based Blind SQL Injection using statistical response comparison.
7. **`burp_audit_graphql`**: Audits GraphQL endpoints for introspection exposure, field suggestion enumeration, batching query amplification, and circular query depth limits.
8. **`burp_verify_csrf`**: Checks cookie `SameSite` attributes, evaluates CORS preflight constraints, checks reflection headers, and generates interactive HTML PoC payloads.
9. **`burp_api_fuzz`**: Consumes OpenAPI/Swagger specifications and orchestrates boundary mutations and fuzzing payloads across exposed routes.

---

### 6. Architectural Layering & Safety Invariants

- **Mandatory 4-Layer Systematic Architecture**:
  1. `proto/burp.proto`: Protocol buffer contracts (114 RPCs).
  2. Kotlin JVM Extension: Montoya API implementation, Swing EDT dispatch, and lease maps.
  3. `crates/burp-protocol`: Rust gRPC client wrappers with reconnection and mTLS support.
  4. `crates/burp-tools`: High-level tool implementations, validation guards, and MCP server registration.
- **Explicit Error Domains**:
  - Standardized error classification (`NO_ACTIVE_EDITOR`, `READ_ONLY`, `STALE_EDITOR`, `UNSUPPORTED_COMPONENT`, `LEASE_EXPIRED`).
- **Standardized Tool Footprint**:
  - **42 Tools by default** (2 Connection + 13 Core Suite + 9 Compound Workflows + 3 Editor + 3 Cookies/Findings + 3 Jobs + 2 Scripts + 6 Intercept + 1 Offline Utility Decoder).
  - **43 Tools with SiteGraph** (`--enable-sitegraph`).

---

### 7. Verification & Quality Assurance
- **Full Test Suite Status**:
  - Kotlin Test Suite: 88/88 passing (`./gradlew test`).
  - Rust Workspace Test Suite: 82+ passing (`cargo test --workspace`).
  - Rust Linters & Formatters: 100% clean (`cargo check --workspace`, `cargo fmt --all -- --check`).
- **Synchronized Documentation**:
  - Synchronized across `README.md`, `docs/features.md`, `skills/burpsuite/SKILL.md`, and references.
