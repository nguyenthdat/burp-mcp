# Burp MCP v3 — Migration Plan

> **Status:** Draft  
> **Created:** 2026-08-21  
> **Target:** Replace the TypeScript/Bun bridge and CyberChef runtime with a native Rust MCP server; connect Rust to the Kotlin Burp extension through gRPC; add a persistent sitemap graph.

## 1. Goals

1. Use **gRPC over HTTP/2 with Protocol Buffers** for the internal connection between the Kotlin Burp extension and the Rust process.
2. Remove the complete TypeScript/JavaScript client and Bun/npm runtime dependency.
3. Replace CyberChef with a Rust-native, binary-safe utility engine.
4. Keep Kotlin focused on the Montoya API and Burp-owned lifecycle/state.
5. Use Rust for:
   - MCP JSON-RPC over stdio;
   - public MCP tool definitions and JSON Schema;
   - gRPC client and reconnect logic;
   - local utilities;
   - sitemap indexing, persistence, graph queries and future analysis features.
6. Preserve existing `burp_*` tool names where practical to minimize client breakage.
7. Release the new architecture as `v3.0.0`.

## 2. Non-goals for the first release

- Full compatibility with every CyberChef operation.
- Arbitrary SQL or Cypher execution through MCP.
- Vector or semantic search in the initial sitemap graph MVP.
- Distributed or cross-project graph federation.
- Running Rust inside the Burp JVM through JNI.
- Exposing gRPC directly to MCP clients. MCP remains JSON-RPC over stdio.

## 3. Current architecture

```text
MCP client
    │ MCP JSON-RPC / stdio
    ▼
TypeScript/Bun bridge
    ├── HTTP + JSON + Bearer token ──► Kotlin Burp extension ──► Montoya API
    └── Bun worker ──► CyberChef
```

Current responsibilities are concentrated in several places:

- `src/main/kotlin/.../McpHttpServer.kt` combines transport, authentication, Gson DTOs, tool registration, Burp operations and long-lived state.
- `bridge/src/main.ts` starts the MCP bridge.
- `bridge/src/rpc.ts` implements MCP JSON-RPC dispatch.
- `bridge/src/burp-provider.ts` calls the Kotlin extension over HTTP/JSON.
- `bridge/src/cyberchef-*` implements CyberChef discovery and execution.
- `package.json`, `bun.lock`, `tsconfig.json` and `biome.json` own the current bridge build and release workflow.
- The current sitemap tool returns a flat list and does not provide persistent indexing or graph traversal.

## 4. Target architecture

```text
MCP client
    │ MCP JSON-RPC / stdio
    ▼
burp-mcp native Rust binary
    ├── rmcp server
    ├── MCP tool catalog and JSON Schema
    ├── Rust utility engine
    ├── sitemap graph/index and SQLite storage
    │
    │ gRPC over HTTP/2 on loopback
    ▼
Kotlin Burp extension
    ├── gRPC service server
    ├── typed Burp service adapters
    └── Montoya API
```

### 4.1 Kotlin responsibilities

Kotlin will only own functionality that must run in the Burp process:

- Montoya API integration.
- Proxy history and annotations.
- HTTP requests through Burp.
- Repeater and Intruder integration.
- Scanner, crawl and audit state.
- Collaborator state.
- WebSocket state.
- HTTP/proxy/session handler registration.
- Burp configuration and extension lifecycle.
- Loopback-only gRPC server.

Kotlin should not own:

- MCP protocol handling.
- Public MCP JSON Schema.
- Base64, hex, hash, JWT decode, entropy or other pure utility operations.
- Sitemap graph persistence and graph query logic.

### 4.2 Rust responsibilities

Rust will own:

- MCP server over stdio using `rmcp`.
- Public tool descriptions and input/output schemas.
- gRPC client.
- Connection lifecycle, timeout, retry and reconnect.
- Mapping gRPC status and protobuf error details to MCP errors.
- Rust-native utility operation registry and recipe engine.
- Sitemap ingestion, normalization, persistence and graph queries.
- Output truncation, pagination and resource limits.
- Native CLI and release binaries.

## 5. gRPC transport decision

The previously considered RPC option is removed from the plan because its Java/Kotlin runtime is not a sufficiently reliable foundation for this project. The internal boundary will use the mature gRPC ecosystem instead:

- Kotlin/JVM: `grpc-java` with Protocol Buffers code generation.
- Rust: `tonic` with `prost` code generation.
- Transport: HTTP/2 over a loopback-only TCP listener.
- Public protocol: MCP JSON-RPC over stdio remains in Rust.

### 5.1 Required gRPC spike

Before the full migration, implement a minimal Rust-to-Kotlin interoperability prototype:

```proto
service BurpService {
  rpc Ping(PingRequest) returns (PingResponse);
  rpc EchoBytes(EchoBytesRequest) returns (EchoBytesResponse);
  rpc ProxyHistory(ProxyHistoryRequest) returns (ProxyHistoryResponse);
}
```

The spike must test:

- Kotlin server binds only to loopback;
- Rust client connects using `tonic`;
- zero-byte, one-byte and 10 MiB payloads;
- binary byte-for-byte round trips;
- multiple concurrent unary calls;
- deadlines and cancellation;
- Kotlin server restart and Rust reconnect;
- graceful shutdown and Burp extension unload;
- macOS, Linux and Windows where possible.

### 5.2 Pass criteria

The migration continues with gRPC when:

- Rust and Kotlin interoperate reliably;
- concurrent calls do not deadlock or return corrupted responses;
- raw binary messages remain byte-exact;
- deadlines cancel pending calls cleanly;
- reconnect creates a fresh channel and succeeds after server restart;
- unloading the extension closes the listener and worker pool;
- the runtime works inside Burp on JDK 25.

### 5.3 gRPC implementation rules

- Define `.proto` files as the only cross-language contract.
- Pin `protoc`, Kotlin/JVM gRPC plugins, `grpc-java`, `prost` and `tonic` versions.
- Set explicit maximum inbound and outbound message sizes on both sides.
- Set deadlines on every call; never allow an unbounded RPC.
- Use server/client streaming for large lists and event feeds instead of oversized unary responses.
- Use `Status` codes and structured protobuf error details rather than parsing exception strings.
- Use a bounded RPC actor in Rust so generated tonic clients do not leak into MCP handlers.
- Keep the service API typed; do not make `invoke(toolName, jsonParams)` the final contract.

## 6. Repository structure

Proposed structure:

```text
burp-mcp/
├── Cargo.toml
├── Cargo.lock
├── proto/
│   ├── common.proto
│   ├── burp.proto
│   ├── proxy.proto
│   ├── http.proto
│   ├── scanner.proto
│   ├── sitemap.proto
│   └── websocket.proto
├── crates/
│   ├── burp-mcp/
│   │   └── CLI, MCP stdio server and composition root
│   ├── burp-grpc/
│   │   └── generated tonic/prost client and gRPC actor
│   ├── burp-tools/
│   │   └── MCP wrappers, descriptions and JSON Schema
│   ├── utility-core/
│   │   └── data model, operation registry and recipe engine
│   ├── utility-tools/
│   │   └── codecs, hashes, compression, JWT and HTTP utilities
│   └── sitegraph/
│       └── ingestion, graph storage and query engine
├── src/main/kotlin/
│   └── Kotlin Burp extension and typed service adapters
├── src/test/kotlin/
├── test-fixtures/
│   ├── rpc/
│   ├── tools/
│   ├── utility/
│   └── sitegraph/
└── .github/workflows/
```

Generated gRPC code must come from the same `.proto` files:

- Rust generation through `tonic-build`/`prost-build`.
- Kotlin/JVM generation through the Gradle protobuf plugin and gRPC Java plugin.
- `protoc` and all runtime/plugin versions must be pinned.
- CI must fail when generated code is stale or non-reproducible.

## 7. gRPC API design

### 7.1 Do not make JSON the final contract

A final method such as this is not acceptable:

```text
invoke(toolName, jsonParams)
```

It may exist temporarily as `invokeLegacy` during migration, but the final boundary must use typed services and messages.

### 7.2 Proposed services

Use explicit protobuf services instead of capability objects:

```proto
service BurpService {
  rpc ServerInfo(ServerInfoRequest) returns (ServerInfoResponse);
}

service ProxyService {
  rpc History(ProxyHistoryRequest) returns (ProxyHistoryResponse);
  rpc Detail(ProxyDetailRequest) returns (ProxyDetailResponse);
  rpc Search(ProxySearchRequest) returns (ProxySearchResponse);
}

service HttpService {
  rpc Send(HttpRequest) returns (HttpResponse);
  rpc SendParallel(SendParallelRequest) returns (SendParallelResponse);
}

service ScannerService {
  rpc StartScan(StartScanRequest) returns (JobResponse);
  rpc GetResults(ScanResultsRequest) returns (ScanResultsResponse);
}

service SitemapService {
  rpc Snapshot(SitemapSnapshotRequest) returns (SitemapSnapshotResponse);
}
```

Keep service boundaries aligned with the Kotlin facades. Rust should expose a single typed client layer that hides generated gRPC stubs from MCP handlers.

### 7.3 Long-running operations

Scan, crawl, Intruder and bulk fuzzing operations must not block one RPC call until completion.

Use a job ID model for the first version:

```proto
service JobService {
  rpc GetStatus(JobStatusRequest) returns (JobStatusResponse);
  rpc Cancel(JobCancelRequest) returns (JobCancelResponse);
  rpc GetResult(JobResultRequest) returns (JobResultResponse);
}
```

Flow:

```text
start operation -> jobId
job status(jobId)
job cancel(jobId)
job result(jobId)
```

Server-streaming job events may be added after the basic job lifecycle is stable.

### 7.4 Pagination and streaming

Proxy history, sitemap entries, issues and graph synchronization must use bounded pages:

```text
offset/cursor
limit
items
total
truncated
nextCursor
```

Default and maximum limits must be enforced at the Kotlin boundary and again at the MCP boundary. Use server-streaming only where the stream can be cancelled and bounded by a deadline.

### 7.5 Protocol Buffer rules

- Use `proto3` and keep `.proto` files as the source of truth.
- Never reuse field numbers.
- Reserve removed field numbers and names.
- Add new fields with higher numbers.
- Use `bytes` for raw HTTP request and response data.
- Use `string` only for validated UTF-8 data.
- Preserve duplicate headers and original header order with repeated header entries.
- Do not normalize ambiguous `Content-Length` or `Transfer-Encoding` framing.
- Use explicit enum values and reserve removed enum values.
- Define structured errors through gRPC status codes and protobuf error details.
- Avoid putting arbitrary JSON in `google.protobuf.Struct` except for a temporary migration method.

Example:

```proto
message RpcError {
  ErrorCode code = 1;
  string message = 2;
  bool retryable = 3;
  string details = 4;
}
```

## 8. Loopback transport and local trust model

The gRPC endpoint is local-only and does not use application-level authentication. The first version uses HTTP/2 over TCP on `127.0.0.1`; therefore the local machine is the trust boundary.

Connection flow:

```text
connect to 127.0.0.1:<port>
  -> create a gRPC channel
  -> call typed service methods
  -> reconnect the channel after server restart
```

Configuration:

- `BURP_MCP_PORT`
- `-Dburp.mcp.port`

`BURP_MCP_TOKEN`, `~/.burp-mcp-token` and `-Dburp.mcp.token` are removed with the old HTTP transport.

Requirements:

- Bind exclusively to `127.0.0.1`, never `0.0.0.0` or an externally reachable interface.
- Rust connects only to the loopback endpoint; the host is not user-configurable in the initial release.
- Fail startup if the requested bind address is not loopback.
- Apply gRPC max inbound/outbound message limits, deadlines, concurrency limits and stream limits.
- Document that any local process, including a process owned by another local user, may be able to connect to the TCP endpoint.
- Unix domain sockets may be added later on macOS/Linux for stronger local endpoint ownership, while loopback TCP remains the cross-platform baseline.

## 9. Rust MCP server design

Use the official Rust MCP SDK, `rmcp`, for stdio JSON-RPC.

Public MCP types should use:

- `serde` for serialization;
- `schemars` for JSON Schema;
- typed input and output structs;
- structured tool errors.

Generated tonic/prost types must remain internal to `burp-grpc` and must not leak into MCP tool APIs.

### 9.1 gRPC actor

Run the tonic gRPC client behind a dedicated actor:

```text
rmcp tool handlers
    │ bounded mpsc commands
    ▼
GrpcActor on a dedicated Tokio task
    │ oneshot response per command
    ▼
tonic gRPC client
```

The actor owns:

- the connection;
- the gRPC channel and generated service clients;
- reconnect and backoff;
- per-call timeout;
- bounded in-flight calls;
- conversion from protobuf responses to Rust domain DTOs.

This keeps generated gRPC clients and retry behavior out of MCP handlers and provides centralized backpressure.

### 9.2 Offline behavior

The Rust MCP server should start even when Burp is not running.

When Burp is offline:

- `burp_*` calls return an actionable connection error;
- local `utility_*` tools continue to work;
- graph query tools continue to work against the last synchronized database;
- `sitegraph_sync` reports that the source is unavailable.

## 10. Public MCP tool surface

Recommended namespaces:

```text
burp_*        Operations requiring the Burp/Montoya API
utility_*     Pure Rust utility operations
sitegraph_*   Sitemap indexing and graph queries
```

Examples:

```text
burp_proxy_history
burp_proxy_detail
burp_send_request
burp_scan_start

utility_run
utility_batch
utility_search_operations
utility_describe_operation
utility_magic

sitegraph_sync
sitegraph_status
sitegraph_search
sitegraph_neighbors
sitegraph_trace
sitegraph_diff
sitegraph_export
```

### 10.1 Tool compatibility

- Preserve existing advertised `burp_*` names where possible.
- Capture current TypeScript descriptions and schemas as fixtures before deleting the bridge.
- Use snapshot tests to prevent accidental public schema changes.
- Return a clear migration note for tools that intentionally change behavior.

### 10.2 Capability-based visibility

During the Rust/Kotlin handshake, Kotlin should report capabilities such as:

```text
proxy.read
proxy.write
scanner.read
scanner.active
crawler
collaborator
websocket
bambda
bcheck
```

Rust should expose or enable only tools supported by the current Burp edition and extension capabilities. Utility and stored graph query tools remain independent of Burp.

## 11. Rust utility engine

The replacement is a Rust-native utility engine, not a full port of CyberChef.

### 11.1 Data model

```rust
enum DataValue {
    Text(String),
    Bytes(Vec<u8>),
    Json(serde_json::Value),
}
```

Each operation defines:

- stable operation ID;
- name and description;
- accepted input kinds;
- typed argument schema;
- output kind;
- deterministic/pure flags;
- size and time limits.

### 11.2 Tool design

Do not create one MCP tool for every utility operation. A large tool list wastes client context.

Expose a small generic API:

- `utility_run`
- `utility_batch`
- `utility_search_operations`
- `utility_describe_operation`
- `utility_magic`

Example recipe:

```json
{
  "input": {
    "kind": "text",
    "value": "SGVsbG8="
  },
  "steps": [
    {
      "op": "base64.decode",
      "args": {}
    },
    {
      "op": "sha256",
      "args": {}
    }
  ]
}
```

### 11.3 MVP operations

#### Encoding

- Base64 and Base64URL encode/decode.
- Hex encode/decode.
- URL percent encode/decode.
- HTML entity encode/decode.
- Unicode escape/unescape.

#### Hash and MAC

- MD5.
- SHA-1.
- SHA-256.
- SHA-512.
- BLAKE3.
- HMAC SHA-256 and SHA-512.

MD5 and SHA-1 remain available for security analysis but must be marked cryptographically weak.

#### Compression

- gzip.
- zlib/deflate.
- Brotli.

#### JSON and text

- JSON pretty/minify/query.
- Case conversion.
- Regex extract/replace with limits.
- Entropy calculation.
- Printable-string extraction.
- Length, reverse, split and join.

#### Web and security

- JWT decode and inspection.
- JWT verification when the caller supplies a key.
- Cookie parsing.
- Query-string parse/build.
- HTTP request/response parsing.
- Header and body transforms.
- Safe `Content-Length` updating.

### 11.4 Magic analysis

`utility_magic` should be deterministic and bounded:

- maximum recursion depth;
- maximum candidate count;
- confidence score;
- no network access;
- no arbitrary code execution;
- no uncontrolled recursive decompression.

### 11.5 Utility security

- No network-capable operations.
- No JavaScript execution.
- No implicit filesystem access.
- Limit input and output sizes.
- Limit execution time.
- Limit decompression ratio and total decompressed bytes.
- Do not log request bodies, cookies, tokens or keys by default.

### 11.6 CyberChef migration

Before deleting CyberChef:

1. Build an input/output fixture corpus from the operations currently used by tests and documented workflows.
2. Implement the supported Rust subset.
3. Run differential tests against saved expected outputs.
4. Document unsupported operations.
5. Remove the CyberChef dependency, worker, catalog and Bun compatibility layer.

The project should advertise a supported Rust utility catalog, not claim full CyberChef compatibility.

## 12. Sitemap graph

### 12.1 Graph model

Initial node types:

- `Origin`: scheme, host and port.
- `Endpoint`: HTTP method and normalized path.
- `PathSegment`.
- `Parameter`: query, header, cookie, path or body parameter.
- `ResponseShape`: status, content type, length and fingerprint.
- `Technology`.
- `Issue`.
- `Artifact`: HTML form, script reference, OpenAPI path or GraphQL operation.

Initial edge types:

```text
ORIGIN_HAS_ENDPOINT
ENDPOINT_HAS_PARAMETER
ENDPOINT_OBSERVED_RESPONSE
ENDPOINT_REDIRECTS_TO
ENDPOINT_LINKS_TO
ARTIFACT_DISCOVERS_ENDPOINT
ISSUE_AFFECTS_ENDPOINT
ENDPOINT_USES_TECHNOLOGY
```

Every node and edge should include evidence metadata:

- source event ID or sitemap reference;
- observation timestamp;
- extraction method;
- confidence;
- parse error or limitation when relevant.

### 12.2 Initial ingestion flow

```text
Rust calls SitemapService.snapshot(cursor, limit)
    -> Kotlin reads Montoya site map in bounded pages
    -> Rust normalizes and parses entries
    -> Rust performs idempotent graph upserts
    -> SQLite transaction commits the new graph state
```

Stable fingerprints must prevent duplicates during repeated synchronization.

### 12.3 Incremental synchronization

After snapshot sync is stable, add one of:

- `eventsSince(sequence, limit)` using a monotonic event log; or
- a callback capability through which Kotlin publishes metadata events.

Event publication must never block a Burp proxy or HTTP callback thread. Events should enter a bounded queue and degrade safely when overloaded.

### 12.4 Storage

Use SQLite for the MVP:

```text
~/.local/share/burp-mcp/graphs/<graph-name>.sqlite
```

Reasons:

- no external database service;
- single-binary distribution;
- transactions and crash recovery;
- FTS support;
- simple backup and migration;
- enough capacity for the initial graph size.

### 12.5 Privacy defaults

Persist only normalized metadata by default:

- normalized URL/path;
- HTTP method;
- parameter names, not values;
- status and content type;
- links and relationships;
- issue metadata;
- hashes and fingerprints.

Do not persist these without explicit opt-in:

- Authorization headers;
- Cookie values;
- query parameter values;
- request bodies;
- response bodies;
- JWTs, API keys or other detected secrets.

### 12.6 Graph tools

MVP tools:

```text
sitegraph_sync
sitegraph_status
sitegraph_stats
sitegraph_search
sitegraph_neighbors
sitegraph_trace
sitegraph_endpoint_detail
sitegraph_diff
sitegraph_export
```

Every query must be bounded and return:

- `total`;
- `truncated`;
- `nextCursor` when applicable;
- `lastSyncedAt`;
- evidence summary.

Do not expose arbitrary SQL or Cypher in the MVP.

### 12.7 Rust sitegraph implementation stack

The sitemap graph MVP will use a SQLite-backed relational graph model implemented in Rust.

The selected libraries:

| Concern | Library | Reason |
| --- | --- | --- |
| Async runtime | `tokio` | Native async runtime for Rust services and gRPC integration |
| SQLite persistence | `sqlx` | Async SQLite access, migrations, compile-time checked queries |
| Embedded SQLite runtime | `rusqlite` with `bundled` | Consistent SQLite version across macOS/Linux/Windows binaries |
| Serialization | `serde` | Typed domain model serialization |
| JSON handling | `serde_json` | MCP responses and metadata storage |
| URL normalization | `url` | Deterministic origin, path and endpoint normalization |
| HTTP model | `http` | Standard request/response metadata representation |
| Stable identifiers | `blake3` | Fast deterministic fingerprints for nodes and edges |
| Graph algorithms | `petgraph` | Optional in-memory graph analysis and traversal algorithms |
| Error handling | `thiserror` + `anyhow` | Typed library errors and binary-level error propagation |
| Logging | `tracing` | Structured diagnostics |
| Time handling | `time` | Lightweight timestamp handling with serde support |
| Property testing | `proptest` | Validate normalization and idempotent graph behavior |

### 12.7.1 Storage architecture

SQLite is the source of truth for the persistent graph.

The graph uses a relational adjacency model:

nodes
-----
id
kind
stable_hash
created_at
updated_at
metadata JSON
edges
-----
id
from_id
to_id
kind
evidence_id
created_at
metadata JSON

The database stores:

- Origin nodes.
- Endpoint nodes.
- Path segments.
- Parameters.
- Response fingerprints.
- Technologies.
- Issues.
- Artifacts.
- Evidence metadata.

Every node and edge must have deterministic identifiers to guarantee idempotent synchronization.

### 12.7.2 Repository architecture

The `sitegraph` crate should be structured as:

sitegraph/
src/
├── model/
│   ├── node.rs
│   ├── edge.rs
│   ├── endpoint.rs
│   └── evidence.rs
│
├── normalize/
│   ├── url.rs
│   ├── headers.rs
│   └── fingerprint.rs
│
├── storage/
│   ├── sqlite.rs
│   ├── migrations.rs
│   ├── nodes.rs
│   └── edges.rs
│
├── graph/
│   ├── traversal.rs
│   ├── neighbors.rs
│   └── diff.rs
│
├── ingest/
│   ├── sitemap.rs
│   ├── html.rs
│   └── openapi.rs
│
└── export/
    ├── json.rs
    └── csv.rs

### 12.7.3 Graph traversal

The MVP does not require a dedicated graph database.

Traversal is implemented with SQLite recursive CTE queries:

- `sitegraph_neighbors`
- `sitegraph_trace`
- bounded relationship traversal
- deterministic pagination

`petgraph` is reserved for optional in-memory analysis such as:

- shortest path analysis;
- dependency impact;
- endpoint clustering.

### 12.7.4 Search

Use SQLite FTS5 for endpoint and artifact search.

Requirements:

- no Elasticsearch dependency;
- local-only indexing;
- deterministic search results;
- migration-managed FTS schema.

### 12.7.5 Privacy and security defaults

The sitegraph database stores metadata only:

Persist:

- normalized URLs;
- HTTP methods;
- parameter names;
- status codes;
- content types;
- response fingerprints;
- relationships;
- evidence timestamps.

Do not persist by default:

- Authorization headers;
- Cookie values;
- query values;
- request bodies;
- response bodies;
- JWTs;
- API keys or secrets.

### 12.7.6 Testing requirements

The sitegraph implementation must verify:

- repeated synchronization creates no duplicate nodes or edges;
- stable fingerprints remain unchanged;
- URL normalization is deterministic;
- migrations work on clean and existing databases;
- graph traversal respects depth and result limits;
- pagination does not skip or duplicate results;
- sensitive data redaction works correctly.

## 13. Migration phases

## Phase 0 — ADR and Rust/Kotlin gRPC spike

**Estimate:** 2–4 days.

Tasks:

- Write an ADR for the Rust/Kotlin boundary.
- Implement the minimal Rust `tonic` client and Kotlin/JVM gRPC server prototype.
- Test loopback-only binding, binary payloads, concurrency, deadlines, reconnect and unload.
- Confirm the pinned gRPC/protobuf versions and code-generation workflow.
- Record benchmark and interoperability results.

Exit criteria:

- The gRPC stack and pinned versions are documented.
- The spike passes all mandatory interoperability criteria.
- No full migration work starts before gRPC works inside Burp on JDK 25.

## Phase 1 — Refactor Kotlin without changing behavior

**Estimate:** 1–2 weeks.

Split `McpHttpServer.kt` into typed domain services:

```text
burp/
├── ProxyFacade.kt
├── HttpFacade.kt
├── RepeaterFacade.kt
├── IntruderFacade.kt
├── ScannerFacade.kt
├── SitemapFacade.kt
├── ScopeFacade.kt
├── CollaboratorFacade.kt
├── WebSocketFacade.kt
└── ConfigurationFacade.kt
```

Tasks:

- Isolate the legacy HTTP authentication/token management so it can be deleted during cutover.
- Separate transport from business logic.
- Replace `JsonObject` in business services with Kotlin DTOs.
- Keep the HTTP adapter temporarily for compatibility.
- Extract lifecycle and resource cleanup into explicit components.
- Capture current tool names, descriptions and JSON schemas as fixtures.
- Classify every current tool as Kotlin/Burp-bound or Rust/pure.

Exit criteria:

- Existing HTTP and behavior tests still pass.
- Domain services no longer depend on NanoHTTPD.
- Pure utilities are identified for removal from Kotlin.

## Phase 2 — Protobuf contracts and Kotlin gRPC server

**Estimate:** 1–2 weeks.

Tasks:

- Add `proto/*.proto`.
- Add Gradle protobuf and gRPC Java code generation.
- Implement typed gRPC services without application-level authentication.
- Implement gRPC status and structured protobuf error mapping.
- Implement pagination, deadlines and message limits.
- Implement graceful gRPC server shutdown.
- Add temporary `InvokeLegacy` only if needed to unblock migration.
- Allow temporary transport modes:

```text
-D burp.mcp.transport=http
-D burp.mcp.transport=grpc
-D burp.mcp.transport=dual
```

Exit criteria:

- Rust tonic integration tests call a read-only Kotlin gRPC service successfully.
- Loopback binding, deadlines, reconnect and shutdown tests pass.
- Inbound/outbound message and concurrency limits are enforced.

## Phase 3 — Rust MCP core

**Estimate:** 1–2 weeks.

Tasks:

- Create the Cargo workspace.
- Implement the `rmcp` stdio server.
- Implement port configuration and local connection discovery.
- Implement `GrpcActor`, deadlines and reconnect behavior.
- Port public tool descriptions and schemas from TypeScript.
- Implement offline behavior.
- Port the first read-only tools:
  - health/version;
  - proxy history/detail;
  - sitemap;
  - target information;
  - scope read;
  - scan issues.

Exit criteria:

- MCP initialize, tools/list and tools/call run through Rust.
- Public tool schema snapshots pass.
- The first tool group no longer requires TypeScript.

## Phase 4 — Port all Burp tools

**Estimate:** 3–5 weeks.

Port tools in risk order.

### Batch 1 — Read-only

- Proxy history/detail/search.
- Sitemap and target information.
- Cookie jar read.
- Scanner results.
- Extension and Burp version.

### Batch 2 — HTTP request operations

- Send request.
- Send parallel requests.
- Repeater operations.
- Request conversion/export.
- Response extraction.

### Batch 3 — Stateful mutation

- Notes and highlights.
- Scope mutation.
- Configuration import/export.
- HTTP handlers.
- Proxy rules.
- Session rules.

### Batch 4 — Long-running and high-impact

- Intruder, fuzzer and race-condition operations.
- Scanner and crawl.
- Collaborator.
- WebSocket.
- Bambda and BCheck.

Exit criteria:

- Every advertised `burp_*` tool has a parity or migration test.
- `invokeLegacy` is removed.
- Kotlin business code no longer receives or returns Gson `JsonObject`.
- Long-running operations are job-based.
- Large result sets are paginated.

## Phase 5 — Rust utility MVP

**Estimate:** 2–3 weeks; may run in parallel with Phase 4.

Tasks:

- Implement `utility-core` and the operation registry.
- Implement the MVP operations.
- Implement recipes, batch execution and bounded Magic analysis.
- Build differential fixtures from current CyberChef behavior.
- Move pure Kotlin utilities to Rust.
- Remove CyberChef runtime dependencies after parity is accepted.

Exit criteria:

- No runtime dependency on CyberChef.
- The supported operation catalog is documented.
- All operations are binary-safe and resource-bounded.
- No utility operation can initiate network access.

## Phase 6 — Sitemap graph MVP

**Estimate:** 2–4 weeks; may run in parallel after Phase 3.

Tasks:

- Implement paginated sitemap snapshots.
- Create SQLite schema and migrations.
- Define stable IDs and idempotent upserts.
- Normalize origins, endpoints and parameters.
- Extract HTML links, forms and redirects.
- Link scanner issues to endpoints.
- Implement search, neighbors, trace, detail, diff and export.
- Implement redaction and privacy tests.

Exit criteria:

- Repeated sync does not produce duplicate nodes or edges.
- Query results are deterministic and paginated.
- The graph survives Rust process restarts.
- Depth and result limits are enforced.
- Secrets and raw bodies are not persisted by default.

## Phase 7 — Cutover and cleanup

**Estimate:** 1–2 weeks.

Remove:

```text
bridge/
package.json
bun.lock
tsconfig.json
biome.json
CyberChef workers/catalog/runtime
TypeScript and JavaScript bridge tests
npm publish workflow
```

Kotlin cleanup:

- Remove NanoHTTPD.
- Remove Gson when no longer used.
- Stop reading the project version from `package.json`.
- Remove the HTTP/JSON server.
- Retain only the gRPC server and typed Montoya adapters.

Release cleanup:

- Replace Bun/npm CI with Cargo and Gradle jobs.
- Add cross-language gRPC contract tests.
- Add native binary build matrix.
- Generate checksums and SBOM.
- Publish `v3.0.0`.

Exit criteria:

- No TypeScript, JavaScript, Bun, Node or npm runtime requirement.
- No CyberChef dependency.
- Rust MCP and Kotlin gRPC are the only production path.

## 14. Compatibility and deprecation

### 14.1 Burp tools

Keep `burp_*` public names where possible. Any intentional semantic change must include:

- a migration note;
- a schema diff;
- a parity or replacement test;
- a changelog entry.

### 14.2 CyberChef tools

Preferred clean break for `v3`:

- remove `cyberchef_*` names;
- introduce `utility_*` names;
- document the supported Rust operation set.

If a grace period is necessary, a late `v2.x` Rust preview may provide temporary aliases such as:

```text
cyberchef_bake -> utility_run
```

Aliases must emit a deprecation warning and should be removed in `v3`.

## 15. Testing strategy

### 15.1 Schema and contract tests

- Detect protobuf field-number reuse and incompatible contract changes.
- Compile the same `.proto` files for Rust and Java/Kotlin.
- Verify generated-code reproducibility.
- Snapshot MCP tool names, descriptions and JSON Schema.
- Test backward-compatible protobuf additions.

### 15.2 Cross-language integration tests

- Start a Kotlin gRPC test server.
- Connect with the Rust client.
- Test that the server binds only to `127.0.0.1` and rejects non-loopback bind configuration.
- Test fragmented writes and abrupt disconnects.
- Test timeouts and reconnect.
- Test byte-exact raw HTTP payloads.
- Test clean server shutdown.

### 15.3 Utility tests

- Differential fixtures based on current behavior.
- Property tests for encode/decode round trips.
- Fuzz HTTP and structured-input parsers.
- Decompression bomb tests.
- Input/output/time limit tests.

### 15.4 Graph tests

- Idempotent synchronization.
- Stable node and edge IDs.
- Deterministic endpoint normalization.
- Pagination without duplicate or skipped results.
- Bounded BFS/traversal.
- Database migration and crash recovery.
- Redaction tests for Authorization, Cookie, JWT, query values and API keys.

### 15.5 Burp lifecycle tests

- No gRPC work executes on a Burp proxy callback thread.
- Long operations run in bounded executors.
- Extension unload deregisters all handlers.
- Extension unload closes the gRPC server, jobs and worker pools.
- Reconnect does not retain stale session/job/WebSocket capabilities.

## 16. CI and release

CI jobs:

1. `rust` — format, clippy, unit tests and build.
2. `kotlin` — Gradle test and JAR build.
3. `proto` — generate and verify protobuf/gRPC outputs.
4. `interop` — Rust/Kotlin gRPC integration tests.
5. `utility` — fixtures, properties and fuzz smoke tests.
6. `sitegraph` — migrations, sync and query tests.
7. `release-smoke` — run the packaged Rust MCP binary over stdio.

Target release artifacts:

```text
burp-mcp.jar
burp-mcp-darwin-aarch64.tar.gz
burp-mcp-darwin-x86_64.tar.gz
burp-mcp-linux-x86_64.tar.gz
burp-mcp-linux-aarch64.tar.gz
burp-mcp-windows-x86_64.zip
SHA256SUMS
SBOM
```

Example MCP client configuration:

```json
{
  "command": "/path/to/burp-mcp",
  "args": ["serve", "--stdio"]
}
```

## 17. Estimates

Estimate for one full-time developer:

| Work item | Estimate |
| --- | ---: |
| Rust/Kotlin gRPC spike | 2–4 days |
| Kotlin decomposition | 1–2 weeks |
| Kotlin gRPC server and Rust tonic client | 1–2 weeks |
| Rust MCP core | 1–2 weeks |
| Migrate 80+ Burp tools | 3–5 weeks |
| Rust utility MVP | 2–3 weeks |
| Sitemap graph MVP | 2–4 weeks |
| Release and hardening | 1–2 weeks |

Expected critical path for one developer: **12–16 weeks**.

With two developers:

- Developer A: Kotlin, gRPC/protobuf and Burp tools.
- Developer B: Rust MCP, utility engine and sitegraph.

Expected MVP: approximately **8–10 weeks**, depending on the gRPC spike and the Kotlin decomposition work.

## 18. Risks and mitigations

| Risk | Impact | Mitigation |
| --- | --- | --- |
| gRPC/protobuf contract or plugin versions drift | Rust/Kotlin interoperability breaks | Pin `protoc`, plugins and runtimes; verify generated outputs and interop in CI |
| `McpHttpServer.kt` is highly coupled | Regression during migration | Refactor behind existing HTTP tests before changing transport |
| Tool schema drift | MCP clients break | Snapshot current TypeScript tool schemas and compare continuously |
| Raw HTTP data is normalized | Security workflows become incorrect | Use protobuf `bytes`, preserve duplicate headers and add byte-exact fixtures |
| Long calls block gRPC or Burp threads | Burp freezes or calls time out | Job model, deadlines, bounded executors and cancellation |
| Utility expansion creates unsafe behavior | Network/file abuse or resource exhaustion | Pure registry, no network, strict size/time/decompression limits |
| Sitemap graph persists secrets | Sensitive local data exposure | Metadata-only defaults, redaction tests and explicit opt-in for raw data |
| Too many utility MCP tools consume context | Poor agent behavior | Generic operation registry with search/describe/run tools |
| Cross-platform native releases become complex | Release delay | Start with CI-supported targets and automate a release matrix early |

## 19. Definition of Done for v3

- [ ] No TypeScript or JavaScript production bridge.
- [ ] No Bun, Node or npm runtime requirement.
- [ ] No CyberChef dependency or worker.
- [ ] MCP server is a native Rust binary using `rmcp`.
- [ ] Rust and Kotlin communicate through loopback-only gRPC without application-level authentication.
- [ ] Existing advertised Burp tools are ported or have documented replacements.
- [ ] Kotlin no longer exposes the HTTP/JSON transport.
- [ ] Kotlin domain services no longer depend on Gson `JsonObject`.
- [ ] Raw HTTP messages remain byte-exact and preserve duplicate headers.
- [ ] Long-running operations use jobs and support status/cancellation.
- [ ] Utility execution has strict resource limits and no network access.
- [ ] Sitemap graph supports sync, status, search, neighbors, trace, detail, diff and export.
- [ ] Graph persistence does not store secrets or raw bodies by default.
- [ ] CI tests Gradle, Cargo, protobuf generation and cross-language gRPC.
- [ ] Releases contain the extension JAR, native binaries, checksums and SBOM.

## 20. Recommended pull request sequence

1. **PR 1 — Baseline contracts:** ADR, current tool/schema fixtures and migration inventory; no behavior changes.
2. **PR 2 — gRPC spike:** Rust tonic/Kotlin gRPC interoperability and decision report.
3. **PR 3 — Kotlin decomposition:** split `McpHttpServer.kt` into typed domain services while retaining the legacy HTTP compatibility adapter.
4. **PR 4 — Rust foundation:** Cargo workspace, `rmcp` stdio server, tonic client and `GrpcActor`.
5. **PR 5 — Read-only Burp tools:** proxy history, sitemap, scope and issue queries.
6. **PR 6 — Remaining Burp tools:** stateful, long-running and high-impact operations.
7. **PR 7 — Rust utilities:** operation registry, recipe engine and removal of CyberChef.
8. **PR 8 — Sitegraph MVP:** snapshot sync, SQLite graph and bounded query tools.
9. **PR 9 — v3 cutover:** remove HTTP, TypeScript, Bun/npm and publish native release artifacts.

The first implementation task is **Phase 0**, because the Rust/Kotlin gRPC interoperability spike determines the service contract and critical path for every later phase.
