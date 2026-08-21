# MCP Server Evaluation Guide

Evaluation answers three different questions. Keep them separate:

1. **Protocol/contract:** does the server implement MCP correctly and
   deterministically?
2. **Tool usability:** can a model discover and compose the tools to solve
   representative tasks?
3. **Target-client acceptance:** does the server work in the actual client and
   mode, such as Goose traditional tool calling and Goose Code Mode?

The included `scripts/evaluation.py` harness addresses only the second layer. It
uses Anthropic's API and XML question/answer fixtures. It is not a protocol
conformance suite, a security boundary, or a replacement for Goose end-to-end
testing.

## 1. Build an evaluation plan

Record the run context before creating tasks:

```yaml
fixture_snapshot: immutable revision or frozen dataset ID
server_version: x.y.z
sdk_version: x.y.z
protocol_revision: YYYY-MM-DD
transport: stdio | streamable_http | legacy_sse
client: harness | goose-traditional | goose-code-mode
client_version: x.y.z
provider: provider name
model: exact model ID
run_date: YYYY-MM-DD
max_turns: 20
task_timeout_seconds: 300
```

Without this manifest, a score can change because the data, server, model,
client, or prompt changed. Historical data is not automatically stable; prefer
frozen fixtures or immutable object revisions.

### Capability and risk matrix

Select tasks that cover the server's intended capabilities and risks:

| Category | Examples |
|---|---|
| Discovery | Search, filtering, ranking, ambiguity resolution |
| Read/detail | IDs, metadata, timestamps, source/evidence retrieval |
| Composition | Search → resolve ID → retrieve → aggregate |
| Pagination | Multiple pages, final page, stale cursor, truncation |
| Errors | Invalid input, not found, auth, rate limit, timeout |
| Freshness/coverage | Stale, partial, skipped, or incomplete data |
| Mutation | Preview, revision conflict, apply, idempotency, rollback |
| Advanced MCP | Roots, resources, prompts, sampling, elicitation |

Use deterministic tests for mutation semantics. The model-usability fixtures in
this guide should remain read-only unless the environment is isolated and reset
between tasks.

## 2. Design high-signal tasks

A good task is:

- **realistic:** based on work a user actually wants to complete;
- **independent:** not dependent on another task's result or side effects;
- **bounded:** has a clear scope and completion condition;
- **compositional:** exercises meaningful discovery or synthesis rather than an
  exact-title lookup;
- **verifiable:** has a typed grader or one unambiguous requested value;
- **stable:** based on a frozen snapshot or immutable revision;
- **diagnostic:** a failure points to a tool/schema/result problem rather than
  arbitrary puzzle difficulty.

Do not use “dozens of tool calls” as a quality criterion. A server that solves a
real task in fewer calls with less context is often better. Include simple
single-capability tasks, realistic multi-step tasks, and boundary/error cases.

### Avoid weak tasks

Avoid:

- current counts or mutable state without a fixed snapshot;
- questions containing the exact title/ID to search for;
- answers that can be ordered or formatted many ways without a custom grader;
- trivia that no real user would request;
- tasks that require unbounded scanning;
- destructive operations in a shared environment;
- questions whose “correct” answer depends on model opinion.

### Prefer explicit answer formats

For the included exact-match harness, make the required format part of the
question:

- “Return the integer only.”
- “Return `true` or `false` in lowercase.”
- “Return the date as `YYYY-MM-DD`.”
- “Return the repository slug only.”

For numbers, sets/lists, dates, or structured output where formatting can vary,
use a task-specific typed grader in a more complete evaluation system rather
than weakening the answer into a fragile string.

## 3. Explore safely before writing fixtures

1. Read the target API and MCP tool documentation.
2. Inspect the server's advertised tool names, descriptions, and schemas.
3. Use only read-only operations against the selected snapshot.
4. Keep exploratory calls small with explicit limits and cursors.
5. Independently solve every candidate task and preserve evidence.
6. Reject any task that is ambiguous, unstable, destructive, or not supported.

Evaluate the public MCP contract, not private implementation details. Reading
the implementation while designing tasks can accidentally reward undocumented
behavior that a real client cannot discover.

## 4. XML fixture format

The included harness accepts a non-empty `<evaluation>` document with one or
more non-empty `<qa_pair>` entries:

```xml
<evaluation>
  <qa_pair>
    <question>Within fixture snapshot 2024-Q1, find critical bugs opened in January and resolved within 48 hours. Which assignee has the highest qualifying count? Return the username only.</question>
    <answer>alex_eng</answer>
  </qa_pair>
  <qa_pair>
    <question>In immutable repository revision abc123, trace inbound calls to authenticate. How many production functions call it directly? Return the integer only.</question>
    <answer>2</answer>
  </qa_pair>
</evaluation>
```

The harness rejects malformed XML, the wrong root element, empty tasks, and
missing questions/answers with a nonzero exit.

## 5. Included harness

### Reproducible setup

From `skills/mcp-builder`:

```bash
uv sync --dev
```

Dependencies are declared in `pyproject.toml` and pinned in `uv.lock`. Set the
provider credentials and exact model explicitly:

```bash
export ANTHROPIC_API_KEY='<secret>'
export ANTHROPIC_MODEL='<exact-model-id>'
```

Do not commit credentials or place them in fixture questions/answers.

### stdio

The harness starts and owns the server process. Put the full server command
after `--` so server arguments cannot consume the evaluation file or evaluator
options:

```bash
uv run python3 scripts/evaluation.py evaluation.xml \
  --transport stdio \
  --env API_URL=https://fixture.example.test \
  --env API_TOKEN='<secret>' \
  -- \
  uv run python3 -m my_mcp_server
```

For a Bun project:

```bash
uv run python3 scripts/evaluation.py evaluation.xml \
  --transport stdio \
  -- \
  bun run start
```

`--env KEY=VALUE` is repeatable. The compatibility options `--command` and
repeatable `--arg` still exist, but the `--` form is preferred because it
preserves the server command exactly.

### Streamable HTTP

Start the remote server separately, then connect to its MCP endpoint:

```bash
uv run python3 scripts/evaluation.py evaluation.xml \
  --transport http \
  --url https://example.test/mcp \
  --header "Authorization: Bearer <token>" \
  --header "X-Fixture-Snapshot: 2024-Q1"
```

`--header "Key: Value"` is repeatable.

### Legacy SSE compatibility

Use only when validating an older deployment that cannot use Streamable HTTP:

```bash
uv run python3 scripts/evaluation.py evaluation.xml \
  --transport sse \
  --url https://example.test/sse
```

Treat this as a compatibility lane, not the default remote transport.

### Limits and output

```bash
uv run python3 scripts/evaluation.py evaluation.xml \
  --transport http \
  --url https://example.test/mcp \
  --model '<exact-model-id>' \
  --max-turns 20 \
  --task-timeout 300 \
  --output evaluation-report.md
```

The harness:

- follows pagination for `tools/list`;
- executes every tool-use block returned in a model turn;
- preserves MCP text, structured content, and `isError` state;
- applies basic credential-field redaction before sending results back to the
  model or recording metrics;
- caps model turns and per-task wall-clock time;
- returns exit `0` only when all expected answers match, `1` for task failures,
  and `2` for setup/connection failures.

Redaction is defense-in-depth, not a guarantee. Use fixture credentials with
minimal privileges and review reports before sharing them.

### Harness limitations

- Anthropic provider only.
- Exact string grading only.
- Provider/model behavior is non-deterministic.
- No automatic dataset reset between tasks.
- No enforcement that advertised/called tools are read-only.
- No MCP protocol conformance or security testing.
- Tool calls from the same model turn may execute concurrently; use this
  harness only with read-only, concurrency-safe evaluation tasks.

For provider-neutral or mutation-heavy evaluation, build a dedicated runner
with typed graders, fixture lifecycle hooks, retries, cost accounting, and an
explicit tool policy.

## 6. Interpret results

Track more than accuracy:

- completion rate and answer correctness;
- tool calls and model turns per task;
- latency and timeouts;
- invalid argument rate;
- repeated/redundant calls;
- pagination/truncation mistakes;
- wrong-tool selection;
- unrecoverable versus successfully recovered tool errors;
- token/context usage where available;
- safety-policy or authorization violations;
- qualitative feedback tied to exact schemas/results.

Compare runs only when fixture, server, client mode, model, limits, and grading
are comparable. Review each failure trace and classify the root cause:

1. tool missing;
2. tool name/description not discoverable;
3. schema ambiguous or invalid;
4. output lacks IDs/evidence/pagination state;
5. error not actionable;
6. model reasoning/tool selection failure;
7. fixture/grader defect;
8. transport/auth/timeout failure;
9. stale/partial data not disclosed.

Fix the contract or fixture before tuning prompts around a systemic issue.

## 7. Goose acceptance suite

When Goose is a target, run a separate end-to-end matrix using
[`goose_coding_mcp.md`](./goose_coding_mcp.md):

- stdio and Streamable HTTP (when both are supported);
- explicit extension name and stable tool prefix;
- `available_tools` filtering;
- traditional tool calling and Code Mode;
- session root initialization and root change;
- compact and paginated results on a large repository;
- stale/partial index behavior;
- `auto`, `approve`, `smart_approve`, and `chat` modes;
- preview and guarded apply for mutations;
- timeout, cancellation, reconnect, and malformed input;
- sampling, elicitation, resources, prompts, and Apps only when implemented;
- debug logs and redaction.

Examples for a headless Goose task are documented in the Goose reference. Use
`goose run --output-format json` or `stream-json` when capturing automation
results, and record the exact Goose version and mode.

## 8. Pre-release checklist

- [ ] Protocol/contract tests pass independently of a model.
- [ ] Security and authorization tests pass.
- [ ] Every model-usability task is independent, read-only, bounded, and solved
      against a frozen fixture or immutable revision.
- [ ] The answer format and grader are unambiguous.
- [ ] The run manifest records server, SDK, protocol, client, provider/model,
      limits, fixture, and date.
- [ ] stdio/Streamable HTTP behavior matches the claimed support.
- [ ] Goose acceptance passes when Goose is a target.
- [ ] Reports and logs are reviewed for credentials and sensitive data.
- [ ] Failures are classified and converted into contract, implementation, or
      fixture improvements.
