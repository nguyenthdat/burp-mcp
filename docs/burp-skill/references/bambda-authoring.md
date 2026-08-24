# Writing and Importing Bambdas

Use this reference when the user asks to create, adapt, review, or import a
Bambda through `burp_bambda_import`. A Bambda is a reusable Java-based Burp
script packaged as YAML. It can execute arbitrary Java and can slow or mutate
Burp, so treat it as trusted extension code rather than data.

The live Burp template for the selected function/location is authoritative.
Prefer adapting a current built-in template or a current official PortSwigger
example over recalling Montoya APIs from memory.

## Choose Bambda rather than BCheck when

- The task is a Proxy/Site map/Logger/WebSocket table filter.
- The task is a custom table column.
- The task is a Repeater custom action.
- The task is a Proxy HTTP/WebSocket match-and-replace script.
- The task needs Java/Montoya APIs or a Java-based custom scan check.

Use a BCheck instead for a declarative Scanner-only check that can be expressed
with BCheck control flow, request mutations, comparisons, and Collaborator.

## Exported file format

Pass the complete YAML document—not only the Java body—to
`burp_bambda_import`:

```yaml
id: 2f4d6f7a-6ea3-4fd9-b2ec-e5f5f8b3fd0e
name: Filter successful JSON responses
function: VIEW_FILTER
location: PROXY_HTTP_HISTORY
source: |+
  /**
   * Keeps successful HTTP responses whose Content-Type is JSON.
   * @author security-team
   **/
  return requestResponse.hasResponse()
      && requestResponse.response().statusCode() >= 200
      && requestResponse.response().statusCode() < 300
      && requestResponse.response().hasHeader("Content-Type")
      && requestResponse.response().headerValue("Content-Type")
          .toLowerCase(Locale.ROOT)
          .contains("application/json");
```

Rules:

1. Generate one stable UUID for a new script. Preserve `id` when updating it:
   Burp replaces an existing library script with the same ID.
2. Give `name` a specific user-visible purpose.
3. Select `function` and `location` from a current Burp template. Do not invent
   enum values from a desired behavior.
4. Put only valid Java statements in the YAML block scalar under `source`.
5. Use spaces, preserve YAML indentation, and do not add Markdown fences to the
   string passed to the import tool.
6. Add a Javadoc header with purpose and author. Explain security-sensitive
   behavior or network/filesystem use in the header.

## Function contracts

The selected template defines variables, return type, and available helper
methods. Keep that contract intact.

### View filter

Typical exported header:

```yaml
function: VIEW_FILTER
location: PROXY_HTTP_HISTORY
```

The source returns a boolean. Guard missing responses before dereferencing:

```java
return requestResponse.hasResponse()
    && requestResponse.response().hasHeader("Server");
```

A view filter runs over many rows. Keep it pure, deterministic, and cheap. Do
not send requests, write files, call remote services, mutate annotations, or
log once per row unless the user explicitly requested that side effect.

### Custom column

Typical exported header:

```yaml
function: CUSTOM_COLUMN
location: PROXY_HTTP_HISTORY
```

Typical source returns a compact display value:

```java
return requestResponse.hasResponse()
    && requestResponse.response().hasHeader("Server")
        ? requestResponse.response().headerValue("Server")
        : "";
```

Return a stable scalar/string, not a large body or sensitive credential. Avoid
expensive regexes because sorting/filtering can evaluate the column repeatedly.

### Repeater custom action

Typical exported header:

```yaml
function: CUSTOM_ACTION
location: REPEATER
```

The source operates on the current `requestResponse`. Use the built-in template
to determine whether the action should return a modified message, create a tab,
or only log output. Do not assume the contract is identical across Burp
versions. A custom action can generate network traffic or process external
data; make such behavior explicit in its name and source header.

### Match and replace

Use the exact function/location pair emitted by Burp for HTTP versus WebSocket,
request versus response, and literal versus regex-style tasks. Ensure the
script only transforms the intended component and preserves unrelated bytes.
Avoid recursive/self-triggering behavior and add a cheap guard so already
transformed messages are left unchanged.

### Java custom scan check

Prefer a passive per-request check when the issue is demonstrable from an
existing response. A passive check must not send new requests. Return an empty
`AuditResult` when there is no issue:

```java
if (!requestResponse.hasResponse())
{
    return AuditResult.auditResult();
}

if (!requestResponse.response().hasHeader("Content-Security-Policy"))
{
    return AuditResult.auditResult(
        AuditIssue.auditIssue(
            "Content Security Policy header missing",
            "The response omitted Content-Security-Policy.",
            "Return an application-specific restrictive policy.",
            requestResponse.request().url(),
            AuditIssueSeverity.LOW,
            AuditIssueConfidence.FIRM,
            "A restrictive CSP reduces the impact of content injection.",
            "Deploy in report-only mode before enforcement.",
            AuditIssueSeverity.LOW,
            requestResponse
        )
    );
}

return AuditResult.auditResult();
```

Use the current Burp template for the exact `AuditIssue.auditIssue` overload;
Montoya signatures can change. For active Java checks, require explicit testing
authorization, bound requests/payloads, and avoid duplicate traffic.

## Authoring rules

1. **Start from the exact context.** Identify function, location, Burp edition,
   and expected return/effect before writing Java.
2. **Keep imports implicit unless the template requires them.** Bambda exposes a
   context-specific API; ordinary extension scaffolding does not belong in the
   script body.
3. **Handle absent data.** Guard `hasResponse`, missing headers, empty bodies,
   malformed encodings, and nullable/optional values.
4. **Stay fast on hot paths.** Compile constant regexes once when the template
   permits, short-circuit cheap predicates first, bound loops, and avoid full
   body copies when headers/status suffice.
5. **Avoid secrets and exfiltration.** Never log or transmit cookies, tokens,
   or bodies unless that is the reviewed purpose. Never embed credentials.
6. **Avoid unrestricted capabilities.** No arbitrary process execution,
   filesystem traversal, reflection, dynamic class loading, or external network
   calls unless explicitly requested and reviewed.
7. **Minimize mutation.** Filters and columns should be pure. For actions and
   match/replace scripts, alter only the declared message component.
8. **Make repeat application safe.** Prefer idempotent mutations and detect an
   already-applied marker/header before adding it again.
9. **Avoid false findings.** For scan checks, require evidence specific enough
   to distinguish a vulnerability from a generic status, timeout, or reflection.
10. **Keep one responsibility.** One Bambda, one function/location/purpose.

## Review before import

Review the full YAML and answer:

- Does the UUID intentionally create or update a script?
- Do `function` and `location` come from a current template/example?
- Does every possible path satisfy the expected return contract?
- Can malformed or missing HTTP data throw?
- Could the code block Burp, allocate/copy large bodies, or run per-row I/O?
- Does it send traffic, mutate messages/state, log secrets, access disk/network,
  or execute arbitrary code?
- Is a passive scan check actually passive?
- Is the effect bounded, target-specific, and reversible?

Do not import until these questions have concrete answers.

## Import and verification

1. Tell the user what function/location will be added and whether it can send
   traffic or mutate state.
2. Call `burp_bambda_import` with `{script: <complete YAML>}`.
3. Require all of:
   - `success: true`;
   - `status: LOADED_WITHOUT_ERRORS`;
   - empty `errors`.
4. If import reports errors, preserve the error text, repair the source, and
   re-import with the same ID. Never claim a loaded script is valid merely
   because it appears in the library.
5. Import does not execute the Bambda. Verification must occur in Burp on a
   controlled fixture appropriate to the function:
   - filter: matching and non-matching rows;
   - column: response with and without the target value;
   - action: expected transformed output and unchanged unrelated data;
   - match/replace: matching, non-matching, and already-transformed messages;
   - scan check: vulnerable and negative fixtures, plus duplicate behavior.
6. The current burp-mcp exposes import but no list, enable/disable, execute, or
   delete tool for Bambdas. If UI verification/removal is unavailable, state
   that limitation exactly; do not fabricate cleanup.

## Primary references

- PortSwigger: https://portswigger.net/burp/documentation/desktop/extend-burp/bambdas
- Creating scripts: https://portswigger.net/burp/documentation/desktop/extend-burp/bambdas/creating
- Official examples: https://github.com/PortSwigger/bambdas
