# Writing and Importing BChecks

Use this reference when the user asks to create, adapt, review, or import a
BCheck with `burp_bcheck_import`. BChecks are declarative Burp Scanner checks;
they can still send requests, mutate insertion points, use Collaborator, and
report persistent issues.

Use the current PortSwigger BCheck definition reference as the syntax authority.
This reference standardizes the workflow and safe defaults; it is not a
replacement for the live language reference.

## Choose BCheck rather than Bambda when

- The deliverable is a Scanner custom check.
- The logic fits BCheck metadata, `define`/`run for each`, one `given ... then`
  block, conditions, request actions, comparisons, and issue reporting.
- Java/Montoya APIs, Burp UI customization, or arbitrary transformations are not
  required.

Choose a Bambda for table filters/columns, Repeater actions, match-and-replace,
or Java-based scan checks.

## Minimal passive definition

New definitions should use the currently documented language version:

```yaml
metadata:
    language: v2-beta
    name: "Missing example security header"
    description: "Reports responses missing X-Example-Security"
    author: "security-team"
    tags: "passive", "headers"

given response then
    if not("X-Example-Security" in {latest.response.headers}) then
        report issue:
            severity: info
            confidence: firm
            detail: "The response did not include X-Example-Security."
            remediation: "Return X-Example-Security with the required policy."
    end if
```

This is a structural template, not a recommendation to report every absent
header. Only create a check when the condition has a defensible security impact
and low false-positive rate.

## Minimal active path definition

```yaml
metadata:
    language: v2-beta
    name: "Exposed backup variant"
    description: "Checks one bounded backup suffix using differential evidence"
    author: "security-team"
    tags: "active", "exposure"

define:
    backup_suffix = ".bak"

given path then
    if not({base.response.status_code} is "404") then
        send request called candidate:
            replacing path: {regex_replace({base.response.url.path}, "(.)/?$", `$1{backup_suffix}`)}

        send request called negative_control:
            replacing path: {regex_replace({base.response.url.path}, "(.)/?$", `$1.{random_str(12)}`)}

        if {candidate.response.status_code} is {base.response.status_code}
            and {negative_control} differs from {candidate} then
            report issue:
                severity: info
                confidence: firm
                detail: `A backup-like resource was returned at {candidate.request.url}.`
                remediation: "Remove backup artifacts from the deployed web root."
        end if
    end if
```

An active definition is only a starting point. Adapt method/session behavior,
content validation, false-positive controls, and path handling to the target.

## Required structure

1. `metadata` is mandatory and must be first.
2. Use `language: v2-beta` for new checks unless the current Burp reference
   names a newer supported version.
3. Give every check a specific `name`; complete `description`, `author`, and
   `tags` for maintainability.
4. Optional `define` declares single constants/variables.
5. Optional `run for each` declares an outer-scope array and repeats the check.
   Use it only for a genuinely bounded payload set; for one item use `define`.
6. Every BCheck has exactly one `given ... then` block.
7. Close every conditional with `end if`.
8. Indent with four spaces, never tabs.

## Select the narrowest execution mode

- `given response then`: passive response evidence; sends no request.
- `given request then`: request-level logic without relying on insertion points.
- `given host then`: once per host; suitable only for bounded host-level checks.
- `given path then`: once per path; good for controlled alternative-path checks.
- `given any insertion point then`: broadest active mode; avoid when a specific
  insertion point works.
- `given query|header|body|cookie insertion point then`: restrict active checks
  to the relevant parameter class. Combine only the classes required.

Do not use an active mode for a condition already provable from the base
response.

## Variables, strings, and evidence

- Use `{base...}` for the original pair, `{latest...}` for the most recent pair,
  and a `called` name for a specific sent request.
- Double quotes are literal strings. Backticks allow interpolation such as
  `` `Found at {candidate.request.url}` ``.
- Prefer exact status/header/body predicates before expensive regex matching.
- A response difference alone is not vulnerability proof. Use a negative
  control, stable marker, expected semantic evidence, or Collaborator
  interaction as appropriate.
- Inspect the base response before generating any request or payload.

## Request actions

`send request called name:` can replace, append, or remove method, path, body,
headers, and query parameters as allowed by the selected mode. Rules:

1. Name each request by purpose: `candidate`, `control`, `confirm`, not `r1`.
2. Change the minimum component needed for the hypothesis.
3. Keep request counts statically obvious and bounded.
4. Avoid destructive methods and state-changing endpoints unless the user
   explicitly authorized them.
5. Preserve authentication/session context only when authorized and necessary.
6. Use unique markers to correlate responses and Collaborator interactions.
7. Do not retry automatically inside multiple nested payload loops.

## Reporting rules

A `report issue` block should include:

- calibrated `severity` based on demonstrated impact;
- calibrated `confidence` based on evidence strength;
- `detail` naming the decisive observation without exposing secrets;
- actionable `remediation` tied to the root cause.

Prefer `report issue` unless continued payload iteration is intentional and
bounded. Use `report issue and continue` only when multiple independent findings
are useful and duplicate volume is controlled.

Do not report from:

- status code or response length alone;
- generic reflection without executable/security impact;
- timeout alone;
- one error page without a negative control;
- an uncorrelated Collaborator interaction;
- a framework-specific heuristic without fingerprinting the framework.

## Collaborator checks

Generate a Collaborator address only when an out-of-band channel is necessary.
Embed a unique per-request marker, then report only on a correlated interaction
type (`dns`, `http`, `smtp`, or `any`) that supports the claim. Avoid generating
multiple addresses when one suffices. No interaction is not proof of safety.

## Performance and safety rules

1. Passive first; active only with explicit authorization.
2. Base-response gates before payloads.
3. Simple predicates before headers, then body regex/response comparisons.
4. No Cartesian explosion: bound `run for each`, insertion points, and requests
   per iteration. Calculate the maximum request count before import.
5. Avoid unnecessary repeat requests and deeply nested `if` blocks.
6. Use the narrowest insertion point and shortest discriminating payload set.
7. Do not encode destructive, persistence-changing, credential-stuffing,
   denial-of-service, or broad brute-force behavior.
8. Minimize false positives and false negatives; include negative controls and
   application/framework fingerprints when relevant.
9. Never embed credentials, target-specific secrets, or production tokens.
10. Cite primary research in comments/metadata when the check implements a
    named vulnerability or product-specific behavior.

## Review before import

Answer these questions from the complete definition:

- Is the language version currently supported?
- Is metadata first and complete?
- Is there exactly one `given ... then` block?
- Does the mode match passive versus active behavior?
- What is the maximum requests per base item and across every loop/insertion
  point?
- Are all request changes in the authorized boundary and non-destructive?
- Does the base response gate unnecessary requests?
- What negative control or unique evidence prevents false positives?
- Are severity and confidence justified by demonstrated impact?
- Could issue reporting duplicate excessively?

Do not import until active request count and reporting evidence are explicit.

## Import safely

Always import a newly authored or active BCheck disabled first:

```json
{
  "script": "<complete BCheck definition>",
  "enabled": false
}
```

Workflow:

1. Review the complete definition and calculate its traffic bound.
2. Call `burp_bcheck_import` with `enabled: false`.
3. Require `success: true`, `status: LOADED_WITHOUT_ERRORS`, and empty `errors`.
4. On an import error, preserve the exact diagnostics, fix syntax/semantics, and
   import again disabled.
5. Test inside Burp against an authorized, controlled positive fixture and at
   least one negative fixture. Confirm request count and duplicate behavior.
6. Only after successful tests may the user intentionally enable/import it with
   `enabled: true`. An active check must not be enabled merely because it
   compiled.
7. The current burp-mcp exposes import but no list, test, enable/disable after
   import, or delete tool for BChecks. If those UI operations cannot be
   performed, leave the check disabled and report the limitation.

`burp_bcheck_import` imports into Burp; it does not itself run a scan. Import
success proves syntax/load status, not correctness, safety, or detection quality.

## Primary references

- Definition reference: https://portswigger.net/burp/documentation/scanner/bchecks/bcheck-definition-reference
- BChecks overview: https://portswigger.net/burp/documentation/scanner/bchecks
- Testing custom checks: https://portswigger.net/burp/documentation/desktop/extend-burp/custom-scan-checks/testing
- Official definitions: https://github.com/PortSwigger/BChecks
