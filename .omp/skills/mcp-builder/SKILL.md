---
name: mcp-builder
description: >-
  Build, review, modernize, and test production-quality MCP servers for external
  services, with first-class integration guidance for Goose coding workflows.
  Use for MCP tools, resources, prompts, transports, security, evaluations, or
  Goose extension integration in Python, TypeScript/Node, Go, Rust, Java,
  Kotlin, C#/.NET, PHP, Ruby, or Swift.
metadata:
  domain: mcp
  audience: software-engineer
  primary_reference: reference/goose_coding_mcp.md
---

# MCP Builder

Build an MCP server that helps an agent complete real tasks safely—not a thin
wrapper around every endpoint in an API. This skill covers protocol design,
implementation, client integration, verification, and evaluation. When the
server will be used by Goose for software development, load
[`reference/goose_coding_mcp.md`](./reference/goose_coding_mcp.md) as a required
companion.

## Non-negotiable principles

1. **Preserve the repository runtime.** Inspect the existing lockfile,
   package manager, build system, and test commands before choosing an SDK.
   For a new Python project prefer `uv`; for a new JavaScript/TypeScript
   project prefer Bun unless the project already standardizes on another tool.
2. **Pin what you build against.** Record the SDK version, protocol revision,
   runtime version, and documentation retrieval date. Do not implement against
   an unbounded `latest` dependency and assume the examples will remain valid.
3. **Design from tasks and client capabilities.** Start with representative
   user workflows, not endpoint count. Identify whether the target client is
   Goose traditional tool calling, Goose Code Mode, another MCP client, or all
   three.
4. **Keep the tool surface focused.** Use clear, action-oriented names, narrow
   schemas, bounded output, native cursor pagination, and actionable errors.
   Filter large extensions to the tools needed for the current workflow.
5. **Treat metadata as metadata.** Tool annotations, descriptions, and Goose
   approval modes improve safety and UX but are not authorization. Enforce
   authentication, authorization, path boundaries, and destructive-operation
   policy in the server.
6. **Test the protocol and the target client.** A successful unit test is not
   proof that initialization, capability negotiation, cancellation, transport,
   output schemas, or Goose tool selection work in practice.

## Workflow

### 1. Establish the contract

Write a short implementation brief containing:

- target service, API version, auth mechanism, rate limits, and data sensitivity;
- 5–10 realistic tasks and the minimum tool calls each task needs;
- target clients and modes (stdio, Streamable HTTP, Goose traditional mode,
  Goose Code Mode, CI, or other clients);
- read-only versus mutating operations and their approval/audit requirements;
- expected scale, latency, pagination, retry, cancellation, and consistency
  behavior;
- required MCP capabilities: tools, resources, prompts, roots, sampling,
  elicitation, notifications, or MCP Apps.

If a task can be completed by Goose's built-in Developer or Analyze extensions,
avoid adding a duplicate generic shell, file editor, or call-graph surface.
Add differentiated domain or repository intelligence instead.

### 2. Research and pin versions

Read, in this order:

1. [`reference/mcp_best_practices.md`](./reference/mcp_best_practices.md) for
   protocol-level invariants.
2. The official MCP specification and the SDK documentation for the chosen
   language and pinned release.
3. [`reference/goose_coding_mcp.md`](./reference/goose_coding_mcp.md) when Goose
   is a target client.
4. The relevant language guide below.
5. The upstream service API documentation and security guidance.

Use the available official-documentation or repository reader; do not assume a
client-specific tool such as `WebFetch`. Prefer the repository's existing
package manager and lockfile. If documentation is from a mutable branch, note
the commit or retrieval date and verify all copied APIs against the pinned SDK.

### 3. Design the MCP surface

Create a capability matrix before coding:

| Capability | Tool/resource/prompt | Read-only? | Input limits | Output contract | Failure/retry policy |
|---|---|---:|---|---|---|
| Search/discovery |  |  |  |  |  |
| Read/detail |  |  |  |  |  |
| Mutation |  |  |  |  |  |
| Workspace/context |  |  |  |  |  |

For every tool:

- use a stable service prefix and action/resource name, such as
  `repo_search_symbols` or `repo_trace_callers`;
- give each parameter a type, constraint, default, and useful description;
- separate preview/dry-run from commit/apply operations;
- declare a typed output schema when the SDK supports it;
- return concise text plus structured content when both human and programmatic
  clients need the result;
- include stable identifiers, paths, line ranges, truncation state, cursors,
  generation/freshness, and evidence/confidence where relevant;
- return an error that says what failed, whether state changed, and what the
  caller can try next;
- mark read-only, destructive, idempotent, and open-world behavior accurately.

Prefer upstream cursor pagination. Never require a full remote count merely to
return the first page. Make large source or documents available through
resources or an explicit detail operation instead of putting everything in
one tool result.

### 4. Implement a thin vertical slice

Set up the project using the selected language reference:

- [TypeScript/Node](./reference/node_mcp_server.md)
- [Python](./reference/python_mcp_server.md)
- [Go](./reference/go_mcp_server.md)
- [Rust](./reference/rust_mcp_server.md)
- [Java](./reference/java_mcp_server.md)
- [Kotlin](./reference/kotlin_mcp_server.md)
- [C#/.NET](./reference/csharp_mcp_server.md)
- [PHP](./reference/php_mcp_server.md)
- [Ruby](./reference/ruby_mcp_server.md)
- [Swift](./reference/swift_mcp_server.md)

Implement one complete read-only path first: initialization, auth, validation,
upstream call, bounded response, error mapping, logging, and a contract test.
Then add mutations and advanced capabilities one at a time.

Shared infrastructure should centralize:

- authenticated HTTP/API clients with timeouts and retry classification;
- request IDs, cancellation, rate-limit handling, and safe structured logging;
- path/URL/identifier validation and output redaction;
- pagination and truncation helpers;
- domain error mapping and response serialization;
- feature/capability detection for optional MCP features.

For stdio, keep protocol output separate from diagnostics; send logs to stderr
according to the MCP transport requirements. For remote deployments, choose
Streamable HTTP deliberately and document state, authentication, origin/SSRF,
proxy, and deployment behavior.

### 5. Integrate with Goose when applicable

Follow [`reference/goose_coding_mcp.md`](./reference/goose_coding_mcp.md) for:

- stdio and Streamable HTTP extension configuration;
- explicit stable extension names and tool prefixes;
- `available_tools` filtering and context budgeting;
- environment variables, secret storage, OAuth, and `AGENT_SESSION_ID`;
- MCP Roots and workspace changes;
- traditional calling versus Code Mode;
- permission modes, adversary/prompt-injection safeguards, sampling,
  elicitation, recipes, skills, and optional MCP Apps.

Do not make Goose the only security boundary. A coding server must enforce its
own workspace, authorization, and mutation policy even when Goose runs in
`approve` or `smart_approve` mode.

### 6. Verify and evaluate

Run the language-specific build, type-check, lint, and test commands. Then run
protocol and client acceptance tests:

- initialize and inspect negotiated capabilities;
- list tools with pagination and verify schemas/descriptions;
- call valid, invalid, unauthorized, timed-out, cancelled, and oversized
  requests;
- verify text, structured, error, and truncation results;
- test reconnect/state behavior for Streamable HTTP;
- test root discovery and root changes for workspace-aware servers;
- test Goose traditional mode and Code Mode when Goose is a target;
- test read-only and mutating tools under the intended permission modes;
- inspect logs without leaking credentials or sensitive source.

Use [`reference/evaluation.md`](./reference/evaluation.md) for task suites, but
separate deterministic protocol tests from model/tool-usability benchmarks and
Goose end-to-end coding tasks. Use frozen fixtures or immutable revisions for
stable answers. Do not reward a server merely because a task takes dozens of
calls; efficient composition is a feature.

## Definition of done

- [ ] The implementation brief and capability matrix are checked in or attached
      to the change.
- [ ] SDK/protocol/runtime versions and validation date are recorded.
- [ ] The server builds reproducibly with the repository's package manager.
- [ ] Tools have typed, bounded schemas and actionable descriptions/errors.
- [ ] Read-only, destructive, idempotent, and open-world behavior is accurate.
- [ ] Pagination, cancellation, timeout, retry, and oversized-output behavior
      are tested.
- [ ] Authentication, authorization, path/URL validation, secret redaction,
      and audit logging are enforced server-side.
- [ ] stdio and/or Streamable HTTP contract tests pass for the supported modes.
- [ ] Goose integration is verified when Goose is a target client.
- [ ] Evaluation fixtures, model/client mode, SDK version, and run date are
      recorded; failures are actionable and reproducible.

## Reference router

| Need | Read |
|---|---|
| Protocol-wide design, security, pagination, transport | [`mcp_best_practices.md`](./reference/mcp_best_practices.md) |
| Goose coding MCP integration and advanced client features | [`goose_coding_mcp.md`](./reference/goose_coding_mcp.md) |
| Model/tool usability evaluation | [`evaluation.md`](./reference/evaluation.md) |
| Language implementation | The selected language guide listed above |

The language references are implementation aids, not substitutes for the
pinned SDK's release documentation. Revalidate copied snippets whenever the SDK
or protocol revision changes.
