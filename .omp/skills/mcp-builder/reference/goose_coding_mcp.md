# Goose Coding MCP Reference

> **Purpose:** design an MCP server that Goose can use as a high-quality coding
> extension. This is a client-integration reference, not another MCP server
> SDK guide.
>
> **Version note:** Goose documentation and draft MCP capabilities can change.
> Record the Goose version, SDK version, protocol revision, and validation date
> when testing an implementation. The examples below reflect the official
> Goose documentation reviewed on 2026-08-21.

## 1. Integration model

Goose extensions are based on MCP, and any MCP server can be added as a Goose
extension even when it is not listed in the extension directory. Goose supports
local command-line extensions and remote extensions over Streamable HTTP.

A coding MCP server should add capabilities that are distinct from Goose's
built-in Developer and Analyze extensions. Good differentiators include:

- semantic or repository-aware symbol search;
- exact definition, reference, caller, callee, and data-flow tracing;
- index freshness, parse coverage, and skipped-file reporting;
- diff blast-radius and dependency impact analysis;
- cross-service route, channel, or API tracing;
- repository-specific architecture, policy, or security intelligence.

Avoid exposing a second generic `read_file`, `write_file`, or unrestricted shell
tool unless isolation, policy enforcement, or a domain-specific workflow makes
that duplication intentional. Smaller, coherent tool surfaces are easier for
Goose and other clients to select correctly.

### Official Goose sources

Use these canonical documents for client-specific behavior:

- [Using Extensions](https://goose-docs.ai/docs/getting-started/using-extensions)
- [Configuration Files](https://goose-docs.ai/docs/guides/config-files)
- [CLI Commands](https://goose-docs.ai/docs/guides/goose-cli-commands)
- [Running Tasks](https://goose-docs.ai/docs/guides/running-tasks)
- [MCP Roots](https://goose-docs.ai/docs/guides/mcp-roots)
- [Managing Tool Permissions](https://goose-docs.ai/docs/guides/managing-tools/tool-permissions)
- [Goose Permission Modes](https://goose-docs.ai/docs/guides/managing-tools/goose-permissions)
- [Code Mode](https://goose-docs.ai/docs/guides/managing-tools/code-mode)
- [MCP Sampling](https://goose-docs.ai/docs/guides/mcp-sampling)
- [MCP Elicitation](https://goose-docs.ai/docs/guides/mcp-elicitation)
- [Recipes](https://goose-docs.ai/docs/guides/recipes/recipe-reference)
- [Agent Skills](https://goose-docs.ai/docs/guides/context-engineering/using-skills)
- [Logs](https://goose-docs.ai/docs/guides/logs)
- [Prompt Injection Detection](https://goose-docs.ai/docs/guides/security/prompt-injection-detection)
- [Adversary Mode](https://goose-docs.ai/docs/guides/security/adversary-mode)
- [MCP Apps](https://goose-docs.ai/docs/guides/interactive-chat/mcp-ui)

## 2. Choose the transport

### stdio: local coding server

Use `stdio` when Goose should launch a local process in the active development
environment. It is a good fit for a repository indexer, local language-server
adapter, test orchestrator, or tool that needs local credentials and files.

Design requirements:

- communicate MCP protocol frames only on stdout;
- send diagnostics to stderr;
- use explicit process timeouts and cancellation;
- handle the process being restarted or terminated;
- resolve the workspace from MCP Roots rather than assuming the launch
  directory is the only source of truth;
- use `AGENT_SESSION_ID` when session-isolated temporary files, worktrees, or
  correlation IDs are useful.

Goose's documentation confirms that stdio extensions receive
`AGENT_SESSION_ID`; it does not define this as a security boundary. Apply your
own path and authorization policy.

Example one-session launch:

```bash
goose session \
  --with-extension "codeintel:uvx my-codeintel-mcp" \
  -t "Trace callers of the authentication entry point and summarize the risk"
```

The documented CLI format is `[name:]ENV1=val1 command args...`. Use an explicit
name such as `codeintel:` so the tool prefix is stable (`codeintel__...`) even
when the launcher is `uvx`, `python`, `npx`, or `python -m`.

For a project-local Python server, prefer a locked project command such as:

```bash
goose session \
  --with-extension "codeintel:uv run python -m codeintel_mcp" \
  -t "Find the definition and inbound callers of authenticate"
```

For a JavaScript/TypeScript server, use the project's authoritative package
manager and lockfile. For a new Bun project, that may be:

```bash
goose session \
  --with-extension "codeintel:bun run start" \
  -t "Find the most affected modules in the current diff"
```

Do not copy these commands blindly: verify the server entry point and package
manager in the target repository first.

### Streamable HTTP: remote or multi-client server

Use `streamable_http` when the server is remote, shared by multiple clients, or
managed independently from Goose. Define authentication, TLS, authorization,
request size limits, rate limits, origin/SSRF controls, session state, and
reconnect behavior before deployment.

One-session launch:

```bash
goose session \
  --with-streamable-http-extension "https://codeintel.example.com/mcp" \
  -t "Search the repository for insecure deserialization and show evidence"
```

Legacy `sse` may occur in older Goose configurations, but new integrations
should target Streamable HTTP. Keep SSE only when compatibility with an
existing deployment is a requirement and test it separately.

### Persistent extension configuration

Goose uses `~/.config/goose/config.yaml` on macOS/Linux and
`%APPDATA%\\Block\\goose\\config\\config.yaml` on Windows. The documented
extension shapes are:

```yaml
extensions:
  codeintel:
    type: stdio
    name: codeintel
    display_name: Code Intelligence
    enabled: true
    cmd: uvx
    args: [my-codeintel-mcp]
    env_keys: [CODEINTEL_INDEX]
    envs: {}
    timeout: 300
    available_tools:
      - repo_search_symbols
      - repo_get_definition
      - repo_trace_callers

  remote-codeintel:
    type: streamable_http
    name: remote-codeintel
    display_name: Remote Code Intelligence
    enabled: true
    uri: https://codeintel.example.com/mcp
    headers: {}
    env_keys: []
    envs: {}
    timeout: 300
    available_tools: []
```

`available_tools` is an allowlist. Only the listed tools are loaded from that
extension; omitted or empty means all tools. Use it to keep a coding session's
context focused. Check the installed Goose release if using fields beyond the
documented shapes above.

After direct config edits, restart Goose or the session as appropriate and
verify with:

```bash
goose info -v
goose mcp codeintel
```

Environment variables take precedence over config-file values, which take
precedence over defaults.

## 3. Tool-surface design for coding workflows

### Recommended capability groups

A useful coding MCP server can expose a small set of composable tools:

| Group | Example tools | Default response |
|---|---|---|
| Discovery | `repo_search_symbols`, `repo_search_code` | Ranked compact matches |
| Source | `repo_get_definition`, `repo_get_source` | Exact source with path/lines |
| Graph | `repo_trace_callers`, `repo_trace_callees`, `repo_trace_data_flow` | Bounded hops and evidence |
| Impact | `repo_diff_blast_radius` | Changed symbols and impacted modules |
| Health | `repo_index_status`, `repo_check_coverage` | Generation, freshness, gaps |
| Context | `repo_get_architecture`, `repo_get_resource` | Focused summary or resource |
| Mutation | `repo_apply_patch` or domain-specific write | Preview first; explicit apply |

Do not create one tool per internal function or per API endpoint when a typed
search/trace tool can cover the same workflow. Conversely, do not make one
ambiguous `repo_do_everything` tool: clients need predictable schemas and
side-effect boundaries.

### Tool descriptions and schemas

Every tool description should state:

- what it does and does not do;
- whether it reads, writes, deletes, executes, or contacts the network;
- path/workspace scope and how the scope is selected;
- pagination, truncation, maximum depth, and maximum result behavior;
- whether results are from a fresh, stale, partial, or skipped index;
- how errors, cancellation, and partial success are represented.

Use action-oriented names with a stable server prefix, for example:

```text
codeintel__repo_search_symbols
codeintel__repo_get_definition
codeintel__repo_trace_callers
codeintel__repo_check_coverage
```

Keep the tool's default result compact. Offer explicit options such as
`detail: "compact" | "full"`, `max_depth`, `limit`, and `cursor` rather than
always returning a large source window or an entire call graph.

### Results that work in both Goose modes

For structured results, define an output schema and return structured content
when the selected SDK supports it. Also include a concise text representation
when CLI or Code Mode compatibility matters. A good coding result contains:

```json
{
  "items": [
    {
      "symbol": "authenticate",
      "qualified_name": "auth.AuthService.authenticate",
      "path": "src/auth.py",
      "start_line": 15,
      "end_line": 42,
      "kind": "function"
    }
  ],
  "has_more": true,
  "next_cursor": "opaque-cursor",
  "index_generation": "2026-08-21T08:24:29Z",
  "coverage": "complete"
}
```

The exact field names are a server contract; keep them stable. Do not include a
full `total_count` if calculating it requires an expensive scan. Use an opaque
cursor from the upstream/index query, and make cursors invalid after a
reindex if necessary with an actionable stale-cursor error.

For source retrieval, return a path, line range, language, and whether the
window was truncated. For graph traversal, return hop distance and evidence
strategy/confidence when the relationship can be heuristic or unresolved.

## 4. Workspace context with MCP Roots

Goose automatically advertises MCP Roots to MCP extensions that support it. The
current Goose root list contains one entry: the current session working
directory. If the session directory changes, Goose updates the root and notifies
connected extensions. Goose currently exposes a single root rather than a
multi-folder workspace.

A roots-aware coding server should:

1. request and validate the active root during initialization;
2. canonicalize it before using it as a repository/cache key;
3. resolve relative paths under the root and reject traversal;
4. decide explicitly how symlinks, submodules, generated directories, and
   external worktrees are handled;
5. invalidate or switch index state when the root changes;
6. include root identity and index generation in responses;
7. never claim that Roots alone is an OS sandbox.

Acceptance test:

- start Goose in repository A;
- call a workspace-aware tool and verify all paths are under A;
- switch the Goose session directory to repository B;
- call the same tool and verify the server observes B rather than serving A's
  cached result;
- attempt `../outside` and symlink escape paths and verify rejection.

## 5. Context budget and Code Mode

Traditional Goose tool calling loads the enabled tool definitions directly.
Goose Code Mode instead discovers enabled tools on demand, batches calls, and
chains intermediate results in a JavaScript execution environment. The Goose
documentation describes Code Mode as exposing `list_functions`,
`get_function_details`, and `execute_typescript` meta-tools when enabled.

Code Mode is useful for large multi-step coding tasks and many extensions, but
it has an important compatibility constraint: only text content from tool
results is supported; images, binary data, and other content types are ignored.

To work well in both modes:

- make tool names and schemas deterministic;
- return useful text even when returning structured content;
- avoid requiring the model to parse decorative Markdown;
- use compact defaults and explicit detail expansion;
- make independent calls composable and safe to batch;
- return machine-readable IDs, paths, lines, cursors, and status fields;
- do not rely on a hidden server-side conversation state unless the transport
  contract makes it explicit;
- test the same task in traditional mode and with Code Mode enabled.

A server should not require Code Mode. It is a Goose client strategy and is
available only when the relevant built-in extension is included and enabled.

## 6. Permissions, mutations, and safety

Goose documents four global modes:

| Config/CLI value | Meaning |
|---|---|
| `auto` | Completely autonomous tool use and file changes |
| `approve` | Manual approval |
| `smart_approve` | Risk-based approval |
| `chat` | No tool or extension use |

Autonomous mode is the documented default. Per-tool levels are **Always Allow**,
**Ask Before**, and **Never Allow**. These client controls are useful, but Goose
classifies read/write behavior on a best-effort basis and tool annotations are
not authorization.

### Server-side mutation requirements

For any write or execute tool:

- separate read-only inspection, preview, and apply operations;
- accurately set read-only, destructive, idempotent, and open-world hints;
- enforce authorization and allowed roots on every request;
- require an expected file/index revision or hash for stale-write protection;
- return a patch/diff preview before applying when practical;
- cap changed files, bytes, commands, and execution time;
- refuse credentials, system paths, and out-of-scope network destinations by
  default;
- log the actor, request ID, scope, outcome, and reason without secrets;
- make retries safe or explicitly report that the operation may have partially
  completed.

Suggested coding tools:

```text
repo_propose_patch   # read-only; returns a patch and validation plan
repo_apply_patch     # mutating; requires patch ID and expected revision
repo_run_check       # bounded command/check allowlist, not arbitrary shell
```

Test mutating tools under `auto`, `approve`, and `smart_approve`; also verify
that `chat` mode does not execute them. Never treat an approval prompt as a
substitute for server authorization.

### Additional Goose defenses

For development environments, consider enabling Goose's prompt-injection
detection and configuring an `adversary.md` policy. These are defense-in-depth:

```yaml
SECURITY_PROMPT_ENABLED: true
SECURITY_PROMPT_THRESHOLD: 0.8
```

Prompt-injection detection is a safeguard, not a guarantee. Adversary mode can
block suspicious tool calls, but the documented reviewer is fail-open if it
fails. The server must still validate and authorize requests independently.
Avoid returning secrets, untrusted executable instructions, or unnecessary
large source content because tool arguments/results can appear in Goose and
provider logs.

## 7. Credentials, environment, and OAuth

Never put API keys in shared `config.yaml`. Prefer Goose's system keyring; on
headless/CI/container systems, understand that file-based `secrets.yaml` may be
used and is plain text.

For local/recipe extensions, declare names with `env_keys` and resolve values at
startup. Goose checks the process environment before its secret storage. Recipe
loading does not prompt for missing values; initialization fails if a required
value is unavailable.

Example:

```yaml
extensions:
  codeintel:
    type: stdio
    name: codeintel
    cmd: uvx
    args: [my-codeintel-mcp]
    env_keys:
      - CODEINTEL_API_URL
      - CODEINTEL_TOKEN
    timeout: 300
```

For Streamable HTTP OAuth where the authorization server requires an
out-of-band registered client, Goose documents:

```yaml
extensions:
  remote-codeintel:
    type: streamable_http
    name: remote-codeintel
    uri: https://codeintel.example.com/mcp
    client_id: ${CODEINTEL_OAUTH_CLIENT_ID}
    client_secret_key: CODEINTEL_OAUTH_SECRET
    scopes:
      - code.read
    enabled: true
    timeout: 300
```

`client_secret_key` is the name of the secret, not the secret value. Public
PKCE clients can omit it. Request the narrowest scopes needed. The OAuth
callback is bound to `127.0.0.1` on an ephemeral port by default; use
`GOOSE_OAUTH_CALLBACK_PORT` only when the identity provider requires a fixed
registered port.

For remote servers, do not assume that a local environment variable is sent as
an HTTP header. Configure and test authentication explicitly using Goose's
supported extension configuration and the installed release.

## 8. Optional advanced MCP capabilities

Implement these only when they materially improve a coding workflow, and
negotiate capabilities rather than assuming every client supports them.

### Roots

Use for active repository/workspace discovery. See [MCP Roots](https://goose-docs.ai/docs/guides/mcp-roots).
Handle a single root and root-change notifications as described above.

### Sampling

Goose documents MCP Sampling as automatically enabled: an MCP extension that
supports sampling can request help from the LLM configured in Goose. Useful
coding cases include contextual documentation, ranking search results, and
synthesizing diagnostics.

Sampling is not deterministic and the linked protocol documentation is marked
draft. Set bounded prompts, input size, latency, and fallback behavior. Never
make a security decision depend only on a sampled response, and do not assume a
specific model, token budget, or availability.

### Elicitation

Goose supports form-mode MCP Elicitation automatically when a supporting server
requests it. It can ask users for missing repository selection, a deployment
 target, or confirmation data without forcing the server to guess. Goose
 documents a five-minute elicitation timeout and cancellation behavior.

Handle all of:

- submitted values and defaults;
- user cancellation;
- timeout/no response;
- invalid or incomplete values;
- clients that do not advertise elicitation.

Do not use elicitation as the only authorization step for a destructive action.

### Resources and prompts

Use Resources for large, addressable source snapshots, architecture documents,
reports, or generated patches. Use Prompts for reusable, domain-specific
instructions. Keep a concise tool fallback for clients that do not present
resources or prompts as expected.

### MCP Apps

Goose Desktop supports MCP Apps in standalone sandboxed windows and inline chat,
but the official Goose documentation labels this support experimental. Apps
may call MCP tools/read resources when configured for it, but cannot directly
communicate with Goose chat. Apps are optional for a coding server; textual
results remain mandatory for CLI, automation, and Code Mode compatibility.

### Tool Shim

Goose Tool Shim is an experimental client compatibility feature for models that
emit text-based tool calls. A normal MCP server does not need a shim-specific
API. Test with native tool calling first; treat shim behavior and configuration
as version-dependent.

## 9. Recipes, skills, and repeatable coding workflows

A Goose recipe can package instructions, parameters, extension declarations,
model settings, retries, subrecipes, and structured output. This is useful for
shipping a repeatable coding workflow with a pinned MCP launcher and a small
allowlist of tools.

Example recipe extension block:

```yaml
extensions:
  - type: stdio
    name: codeintel
    cmd: uvx
    args:
      - my-codeintel-mcp
    env_keys:
      - CODEINTEL_TOKEN
    available_tools:
      - repo_search_symbols
      - repo_get_definition
      - repo_trace_callers
    timeout: 300
    description: "Workspace-aware code intelligence"
```

When a recipe declares an explicit `extensions` block, only the listed
extensions are available. If it needs Goose subagent delegation, explicitly
include the `summon` platform extension unless the recipe's subrecipe behavior
injects it.

Use a Goose Skill for workflow instructions that should not be repeated in
MCP tool definitions, such as:

1. check index status;
2. search symbols;
3. retrieve exact definitions;
4. trace callers and callees;
5. check coverage before making a negative claim;
6. inspect the diff blast radius;
7. run focused tests;
8. summarize evidence and unresolved gaps.

MCP provides capabilities and data; Skills and Recipes provide reusable
composition and policy. Keep each layer focused.

## 10. Acceptance-test matrix

### Protocol and server tests

- [ ] `initialize` succeeds and negotiated capabilities are correct.
- [ ] `tools/list` returns stable names, descriptions, input schemas, and
      output schemas; pagination works.
- [ ] `tools/call` handles valid, malformed, unauthorized, oversized, timed-out,
      and cancelled requests.
- [ ] Text, structured content, tool errors, truncation, and partial results
      have documented representations.
- [ ] Native cursors survive normal paging and fail clearly after a reindex when
      invalidated.
- [ ] stdio framing/logging is clean, or Streamable HTTP auth/TLS/reconnect/state
      behavior is verified.
- [ ] Secrets, tokens, and unnecessary source are absent from returned errors
      and logs.

### Goose integration tests

- [ ] Add the server through `goose configure` as a command-line extension.
- [ ] Add it through Streamable HTTP when supported.
- [ ] Verify the explicit extension name and stable `server__tool` prefix.
- [ ] Verify `available_tools` limits the exposed surface.
- [ ] Start in the intended session root and test a root change.
- [ ] Run a search → source → trace multi-step task in traditional mode.
- [ ] Repeat the task with Code Mode enabled; ensure text results are sufficient.
- [ ] Run read-only tools under `auto`, `approve`, `smart_approve`, and `chat`.
- [ ] For mutations, verify preview, approval, revision checks, and rejection.
- [ ] Test sampling/elicitation only when the server implements them.
- [ ] Run with `--debug` and inspect CLI/server logs for initialization,
      capabilities, schemas, tool calls, and errors.

Useful diagnostic commands:

```bash
goose info -v
goose run --debug -t "Inspect the active code intelligence extension"
goose run --output-format json -t "Run a bounded symbol search and report the result"
goose run --output-format stream-json -t "Trace the authentication call path"
```

Use debug output carefully: it can expose full parameters and paths. Redact
captured logs before sharing them.

## 11. Troubleshooting

| Symptom | Checks |
|---|---|
| Extension does not start | Verify `cmd`/`args`, executable search paths, required `env_keys`, timeout, and stderr logs. Run `goose info -v`. |
| Tools have an unexpected prefix | Add an explicit `[name:]` in the session launch or set the extension `name`; launcher-derived names can vary. |
| Tool is missing | Check `enabled`, `available_tools`, extension type, capability negotiation, and whether Code Mode has discovered it. |
| Goose chooses the wrong tool | Reduce the exposed surface, improve descriptions/parameter constraints, use stable prefixes, and return compact results. |
| Code Mode loses results | Put essential information in text content; Code Mode ignores non-text result content. |
| Paths point to another repository | Implement Roots and invalidate root-scoped caches on root changes; never trust a stale process working directory. |
| Search result is stale or incomplete | Return index generation and coverage state; expose a status/coverage tool and explain skipped/partial files. |
| Remote auth fails | Verify URI, TLS, OAuth registration, scopes, `client_secret_key` resolution, callback port, and server-side token audience/authorization. |
| Calls time out | Bound upstream requests, page sizes, graph depth, and server work; align server deadlines with Goose extension timeout. |
| Security alert appears | Read the Goose finding, inspect the exact tool parameters, deny unexpected operations, and review prompt-injection/adversary configuration. |
| Debugging data is too large | Use compact detail, `available_tools`, pagination, `GOOSE_MAX_TOOL_RESPONSE_SIZE`, and targeted queries instead of disabling all truncation. |

## 12. Version and documentation checklist

Before releasing or updating the server:

- [ ] Record `validated_on`, Goose version, SDK package/version, protocol
      revision, runtime version, and test client mode.
- [ ] Re-check Goose extension field names against the current configuration
      reference.
- [ ] Re-check draft features (Sampling and Elicitation) against the installed
      Goose and MCP protocol versions.
- [ ] Pin dependencies and update lockfiles intentionally.
- [ ] Re-run protocol tests, Goose acceptance tests, and representative coding
      tasks after SDK or Goose upgrades.
- [ ] Treat clean tests or Goose permissions as evidence for that configuration,
      not as a general security guarantee.
