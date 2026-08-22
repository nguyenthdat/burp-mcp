# MCP Server Best Practices

This reference contains protocol-wide design rules. Apply it together with the
pinned MCP specification, the selected SDK's release documentation, and the
target-client guide such as
[`goose_coding_mcp.md`](./goose_coding_mcp.md).

## 1. Start from tasks, not endpoint count

Build the smallest coherent surface that completes representative user tasks.
Use composable primitive tools for operations that combine well; add workflow
tools only when they materially reduce ambiguity, round trips, cost, or error
rate. Expand from observed task failures rather than wrapping an entire API by
default.

Before implementation, record:

- target clients and transports;
- required MCP capabilities and protocol revision;
- SDK/runtime versions and documentation source/commit/date;
- realistic read and mutation tasks;
- authorization, data sensitivity, and audit requirements;
- expected scale, latency, pagination, timeout, cancellation, and consistency.

## 2. Identity and naming

Distinguish four identities:

1. **MCP server implementation name** advertised during initialization.
2. **Package/artifact name** in the language ecosystem.
3. **Executable/launcher command** used by a client.
4. **Client extension ID/display name** used for namespacing and permissions.

They may follow different ecosystem conventions. Keep each stable and document
the mapping.

### Tool names

- Use `snake_case` unless the target SDK imposes another convention.
- Prefix tools with a stable service/domain identifier when the client may load
  multiple servers: `github_create_issue`, `repo_trace_callers`.
- Start with a precise action: `get`, `list`, `search`, `trace`, `preview`,
  `apply`, `create`, `update`, or `delete`.
- Avoid generic names such as `run`, `query`, `manage`, or `execute` unless the
  scope is unambiguous from the complete name and schema.
- Keep read, preview, mutation, and deletion operations separate so clients and
  policy systems can reason about side effects.

## 3. Tool contracts

### Descriptions

A tool description should state:

- the exact operation and important non-goals;
- side effects and external systems contacted;
- required permissions and scope;
- selection/filter semantics;
- pagination, maximum depth, and truncation behavior;
- consistency/freshness guarantees;
- likely errors and recovery steps.

Descriptions must match implementation behavior. Never promise idempotency,
atomicity, strong consistency, or complete coverage unless the server enforces
and tests it.

### Input schemas

- Use the SDK's typed schema mechanism and generate JSON Schema where possible.
- Mark required versus optional fields accurately.
- Add type constraints, enums, ranges, length limits, and defaults.
- Use mutually exclusive parameters instead of ambiguous precedence.
- Reject unknown or conflicting inputs when silently ignoring them is unsafe.
- Accept stable IDs rather than display names for mutations; provide a discovery
  tool to resolve IDs.
- Bound every user-controlled collection, string, path, query, recursion depth,
  and time range.

### Output schemas

Define `outputSchema` where supported and return protocol
`structuredContent` for machine-readable results. Also return concise text when
human clients or compatibility modes require it. `structuredContent` is an MCP
protocol concept, not a feature unique to one SDK.

Do not require every tool to add a `response_format` parameter. Add distinct
JSON/Markdown representations only when callers genuinely need both. Structured
and text forms must be semantically equivalent.

Useful metadata includes:

- stable IDs and human-readable labels;
- canonical path/URI and line/range information;
- result count for the current page;
- `has_more`, an opaque continuation cursor, and truncation reason;
- source revision, snapshot, index generation, or freshness;
- partial/skipped/coverage state;
- warning and correlation/request IDs.

Keep defaults compact. Offer an explicit detail mode or resource URI for large
payloads instead of returning unbounded source or documents.

## 4. Pagination and bounded context

Prefer the upstream system's native cursor. Offset pagination is acceptable for
stable collections that support it efficiently, but it can duplicate or skip
items in mutable datasets.

A cursor response commonly includes:

```json
{
  "items": [],
  "count": 20,
  "has_more": true,
  "next_cursor": "opaque-token"
}
```

Rules:

- honor `limit` and enforce a server-side maximum;
- make cursors opaque to clients;
- document cursor lifetime and stale-cursor errors;
- do not load all records merely to return one page;
- make `total_count` optional—it may require an expensive full scan;
- expose truncation explicitly rather than silently dropping results;
- paginate protocol list operations such as tools/resources when the SDK and
  client support it;
- test empty, first, middle, final, stale, malformed, and repeated cursors.

## 5. Transports and deployment

### stdio

Use for local, client-owned processes and single-session integrations.

- Keep stdout exclusively for MCP protocol traffic.
- Send logs and diagnostics to stderr.
- Handle EOF, cancellation, and process termination.
- Do not assume the process working directory is an authorization boundary.
- Avoid global state that leaks between sessions when a launcher reuses a
  process unexpectedly.

### Streamable HTTP

Use for remote services, independently managed deployments, and multi-client
access.

- Require TLS outside trusted local development.
- Define stateful versus stateless behavior from required capabilities.
- Validate authentication and authorization on every request.
- Enforce request/body limits, timeouts, cancellation, rate limits, and
  concurrency limits.
- Validate `Origin` where relevant and protect local deployments from DNS
  rebinding.
- Define reconnect, resumability, session expiry, load balancing, and sticky
  session behavior when sessions are used.
- Protect upstream requests against SSRF and use outbound allowlists where
  appropriate.

### Legacy SSE

SSE transport may remain for compatibility with older clients/servers. Do not
select it for new deployments when Streamable HTTP is available. Keep migration
and compatibility tests separate from the primary path.

Transport does not determine whether communication can be bidirectional; stdio
can carry protocol requests, responses, and notifications. Choose based on
process ownership, network boundary, scaling, auth, and state requirements.

## 6. Capabilities and graceful degradation

Negotiate capabilities during initialization. Do not call or expose behavior
that the peer did not advertise.

For tools, resources, prompts, roots, sampling, elicitation, logging,
notifications, and other optional features:

- implement a documented fallback;
- handle capability absence and version mismatch;
- bound server-initiated requests;
- support cancellation and timeout;
- avoid assuming one client UI or content type;
- test with every claimed target client.

Draft protocol features require an explicit compatibility record and revalidation
on upgrades.

## 7. Errors and partial failure

Distinguish protocol errors from tool execution errors according to the MCP
specification and SDK. A tool failure should communicate:

- what operation failed;
- whether any state changed;
- whether the failure is validation, auth, authorization, not-found, conflict,
  rate limit, timeout, cancellation, dependency, or internal;
- whether retry is safe and, if known, when;
- a request/correlation ID for server logs;
- an actionable next step without exposing internals or secrets.

Do not send stack traces, credentials, raw authorization headers, SQL, internal
filesystem paths, or upstream response bodies to the model by default. Log
safely server-side and redact sensitive fields.

For batch or multi-step operations, return per-item outcomes and an explicit
partial-success state. Never claim atomicity if the upstream API cannot provide
it.

## 8. Tool annotations

Set MCP tool annotations accurately:

| Annotation | Meaning |
|---|---|
| `readOnlyHint` | The tool does not modify its environment |
| `destructiveHint` | The tool may perform destructive updates |
| `idempotentHint` | Repeating the same request adds no further effect |
| `openWorldHint` | The tool interacts with external entities |

Defaults and exact semantics depend on the protocol revision; verify them in the
pinned specification. Annotations are hints for clients and models, not a
security boundary. Enforce authorization and policy server-side.

## 9. Security baseline

### Authentication and authorization

- Use the authentication model required by the current MCP authorization
  specification and deployment environment.
- Validate issuer, audience/resource, expiry, signature, and scopes/claims.
- Authorize each tool and object, not just the connection.
- Apply least privilege and tenant/user isolation.
- Keep credentials in a secret store or injected environment, never source or
  shared configuration.
- Rotate credentials and redact them from errors, telemetry, and tool results.

### Input and execution safety

- Canonicalize paths and reject traversal and symlink escape.
- Validate URLs, schemes, ports, hostnames, redirects, and outbound destinations.
- Use parameterized API/database clients; never interpolate untrusted input into
  shell, SQL, templates, or code.
- Prefer allowlisted bounded commands over arbitrary shell tools.
- Enforce file, byte, token, result, recursion, CPU, memory, and wall-clock
  limits.
- Sanitize untrusted content before rendering and label it as data, not
  instructions.
- Protect against confused-deputy behavior and indirect prompt injection.

### Mutations

- Separate inspect/preview/apply/delete.
- Offer dry-run or patch preview.
- Require expected revision/hash for stale-write protection.
- Use idempotency keys when the upstream supports them.
- Record auditable actor, scope, request, outcome, and correlation ID.
- Make cleanup/rollback behavior explicit.

## 10. Observability without leakage

Emit structured logs to the transport-appropriate diagnostic stream. Include:

- server and protocol version;
- request/correlation ID;
- tool name, duration, outcome, and retry category;
- upstream status and rate-limit metadata;
- root/repository identity and index generation where relevant;
- cancellation, timeout, truncation, and partial-result signals.

Do not log complete prompts, source, tokens, headers, environment variables, or
personal data by default. Make verbose/debug logging opt-in, time-bounded, and
safe to share after redaction.

## 11. Testing strategy

### Deterministic protocol/contract tests

- initialization and capability negotiation;
- paginated tools/resources/prompts listing;
- schema validation and unknown/conflicting inputs;
- valid tool calls and typed structured output;
- tool errors versus protocol errors;
- cancellation, timeout, retries, rate limits, and partial success;
- malformed frames/requests and oversized input/output;
- transport shutdown, reconnect, and session expiry;
- optional capability absence and graceful fallback.

### Security tests

- missing, invalid, expired, wrong-audience, and under-scoped credentials;
- object/tenant authorization;
- path traversal, symlink escape, command/template/SQL injection;
- SSRF, redirect, origin, and DNS rebinding behavior;
- secret redaction in errors/logs/results;
- mutation preview, stale revision, idempotency, and rollback/partial failure.

### Target-client acceptance

Test actual representative tasks through every claimed client and mode. For
Goose coding integrations, follow
[`goose_coding_mcp.md`](./goose_coding_mcp.md). Direct SDK or Inspector success
alone does not prove client tool discovery, namespacing, permission behavior,
context usage, or result compatibility.

### Reproducibility

Record:

```yaml
validated_on: YYYY-MM-DD
protocol_revision: YYYY-MM-DD
sdk_package: package-name
sdk_version: x.y.z
runtime_version: x.y.z
target_clients:
  - client and version
```

Pin dependencies and commit the authoritative lockfile. Re-run extracted
examples and acceptance suites when the SDK, protocol, runtime, or target client
changes.

## 12. Definition of done

- [ ] Representative tasks and client capabilities shaped the surface.
- [ ] Package, runtime, SDK, and protocol versions are pinned and recorded.
- [ ] Tool names, descriptions, schemas, and annotations are accurate.
- [ ] Outputs are typed, compact, paginated, and explicit about truncation and
      freshness.
- [ ] Errors distinguish validation, auth, conflict, rate limit, timeout,
      cancellation, dependency, and internal failure.
- [ ] Authentication, authorization, path/URL controls, mutation safeguards,
      and redaction are enforced server-side.
- [ ] Supported transports pass lifecycle and failure tests.
- [ ] Optional capabilities degrade gracefully.
- [ ] Logs are useful without leaking secrets or unnecessary data.
- [ ] Deterministic protocol tests and target-client task tests pass.
