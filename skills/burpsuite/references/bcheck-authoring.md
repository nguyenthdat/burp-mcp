# Writing and Importing BChecks (v2-Beta Specification)

Use this reference when the user asks to create, adapt, review, or import a BCheck via `burp_bcheck_import`. BChecks are declarative custom checks for **Burp Scanner** in Burp Suite Professional.

---

## 1. Choose BCheck vs Bambda

| Feature / Need | Choose BCheck | Choose Bambda |
|---|---|---|
| **Custom Scanner Rule** | **Yes** (Declarative, fast, built-in loops & Collaborator) | Only if complex Java / external dependencies required |
| **Proxy / Logger View Filter** | No | **Yes** (Java boolean expression) |
| **Custom Table Column** | No | **Yes** (Java string expression) |
| **Match & Replace Rules** | No | **Yes** (Java message transformation) |
| **Repeater Custom Action** | No | **Yes** (Java action execution) |

---

## 2. Complete BCheck Language Syntax (v2-Beta)

Every BCheck definition must follow this exact structural hierarchy:

```bcheck
metadata:
    language: v2-beta
    name: "Descriptive Check Name"
    description: "Detailed description of what the check detects."
    author: "Security Engineer"
    tags: "tag1", "tag2", "active", "passive"

define:
    test_var = "constant_value"
    collab = {generate_collaborator_address()}
    rand_token = {random_str(12)}

given <mode> then
    # Conditions, request actions, issue reporting
```

### 2.1 Metadata Block (Mandatory)
- `language: v2-beta` (Always use `v2-beta` for modern Burp Suite versions).
- `name`: Unique, user-visible title of the issue / check.
- `description`: Explanation of the vulnerability and test rationale.
- `author`: Author or team identifier.
- `tags`: Comma-separated categories (e.g. `"xxe"`, `"ssrf"`, `"sqli"`, `"passive"`, `"headers"`).

### 2.2 Define & Variables Block (Optional)
Declare constants, generated strings, or Collaborator domains before execution:
```bcheck
define:
    canary = "xssCanary9821"
    random_int = {random_int(1000, 9999)}
    random_token = {random_str(16)}
    collab = {generate_collaborator_address()}
    encoded_payload = {base64_encode("admin:admin")}
```

#### Built-in Variable Functions:
- `{generate_collaborator_address()}`: Creates a unique Burp Collaborator domain.
- `{random_str(length)}`: Generates a random alphanumeric string of specified length.
- `{random_int(min, max)}`: Generates a random integer between min and max.
- `{base64_encode(string)}` / `{base64_decode(string)}`: Base64 conversion.
- `{url_encode(string)}` / `{url_decode(string)}`: URL encoding / decoding.
- `{hex_encode(string)}` / `{hex_decode(string)}`: Hex conversion.
- `{sha256(string)}` / `{sha1(string)}` / `{md5(string)}`: Cryptographic hashing.
- `{to_lower(string)}` / `{to_upper(string)}`: String case normalization.
- `{regex_replace(subject, pattern, replacement)}`: Regex replacement.

### 2.3 `run for each` Loops (Optional)
To test a list of payloads or paths:
```bcheck
run for each:
    path_suffix =
        ".bak",
        ".old",
        "~",
        ".backup"

given path then
    send request called check_variant:
        replacing path: {regex_replace({base.response.url.path}, "(.)/?$", `$1{path_suffix}`)}
```

---

## 3. Execution Modes (`given ... then`)

Every BCheck must contain **exactly one** `given ... then` block. Select the narrowest applicable mode:

| Mode | Trigger / Surface | Traffic Impact | Best For |
|---|---|---|---|
| `given response then` | Base response inspection | **Zero traffic** (Passive) | Missing headers, leaked secrets, exposed debug info. |
| `given request then` | Base request inspection | Active / Passive | Request-level anomalies or single-shot replays. |
| `given host then` | Once per unique host | Low active traffic | `robots.txt`, `security.txt`, global host misconfigurations. |
| `given path then` | Once per unique path | Low active traffic | Backup files (`.bak`), hidden directories, path traversal. |
| `given any insertion point then` | Every detected parameter | High active traffic | Universal injection checks (SQLi, XSS, SSRF). |
| `given query insertion point then` | URL query parameters (`?q=`) | Targeted active traffic | Query-based injection / XSS. |
| `given body insertion point then` | POST body parameters | Targeted active traffic | Form / JSON body injection. |
| `given header insertion point then` | HTTP request headers | Targeted active traffic | Header injection (`User-Agent`, `Referer`, `X-Forwarded-For`). |
| `given cookie insertion point then` | Cookie values | Targeted active traffic | Cookie-based session / SQL injection. |
| `given json insertion point then` | JSON key values | Targeted active traffic | API JSON injection / type tampering. |
| `given url path insertion point then` | REST URL path segments | Targeted active traffic | Route parameters (`/users/{id}`). |

---

## 4. Request Actions & Payload Insertion

### 4.1 Insertion Point Modes (`given ... insertion point then`)
In insertion point modes, use `send payload:` to mutate parameters:
```bcheck
given any insertion point then
    send payload:
        replacing: "payload_string"
        # or appending: "payload_string"
        # or prepending: "payload_string"
        # or leaving: unaltered
```

Named payload requests for differential testing:
```bcheck
given any insertion point then
    send payload called probe:
        replacing: `' AND 1=1--`

    send payload called negative_probe:
        replacing: `' AND 1=2--`

    if {probe.response.status_code} is "200" and {negative_probe.response.status_code} is not "200" then
        report issue:
            severity: high
            confidence: certain
            detail: `Boolean SQL injection detected at parameter {insertion_point.name}.`
            remediation: "Use parameterized queries."
    end if
```

### 4.2 Path & Host Modes (`given path then` / `given host then`)
Use `send request called <name>:` to mutate full request components:
```bcheck
given path then
    send request called candidate:
        replacing method: "POST"
        replacing path: `/admin/backup.zip`
        replacing headers:
            "X-Custom-Header": "TestValue",
            "Content-Type": "application/json"
        replacing body: `{"admin": true}`
```

---

## 5. Conditions & Logic

### Condition Operators:
- `is` / `is not`: Exact equality check (`if {base.response.status_code} is "200" then`).
- `in` / `not in`: Substring or array containment (`if "root:x:0:0:" in {latest.response.body} then`).
- `matches` / `not matches`: Regex pattern matching (`if {latest.response.headers} matches "(?i)server:\s*apache" then`).
- `differs from`: Full HTTP response diff (`if {candidate} differs from {negative_control} then`).
- `and` / `or` / `not`: Boolean logic operators.

### Referencing Request/Response Properties:
- `{base.request.url}` / `{base.response.status_code}` / `{base.response.body}` / `{base.response.headers}`
- `{latest.request.url}` / `{latest.response.status_code}` / `{latest.response.body}` / `{latest.response.headers}`
- `{<request_name>.response.status_code}` / `{<request_name>.response.body}`
- String interpolation: Use backticks: `` `Found issue at {candidate.request.url}` ``.

---

## 6. Collaborator / OAST in BChecks

```bcheck
metadata:
    language: v2-beta
    name: "Out-of-Band SSRF via Header"
    description: "Detects blind SSRF by injecting Collaborator payload into headers."
    author: "Security Team"
    tags: "oast", "ssrf", "active"

define:
    collab = {generate_collaborator_address()}

given header insertion point then
    send payload:
        replacing: {collab}

    if dns_interaction in {collab.interactions} or http_interaction in {collab.interactions} then
        report issue:
            severity: high
            confidence: certain
            detail: `Out-of-band interaction received from target server via Collaborator domain: {collab}.`
            remediation: "Validate and restrict outbound network connections."
    end if
```

---

## 7. Reporting Issues

```bcheck
report issue:
    severity: high | medium | low | info
    confidence: certain | firm | tentative
    detail: `Exploit successful at {candidate.request.url}. Decisive evidence: {canary}.`
    remediation: "Apply input validation and parameterized controls."
```
Use `report issue and continue:` when continuing an iteration loop across multiple test cases.

---

## 8. Complete Production Templates

### 8.1 Passive Check Template (Missing Security Headers)
```bcheck
metadata:
    language: v2-beta
    name: "Missing Content-Security-Policy"
    description: "Flags responses missing Content-Security-Policy."
    author: "Security Team"
    tags: "passive", "headers"

given response then
    if {base.response.status_code} is "200" and not("Content-Security-Policy" in {base.response.headers}) then
        report issue:
            severity: low
            confidence: firm
            detail: "The response omitted a Content-Security-Policy header."
            remediation: "Define and enforce a restrictive Content-Security-Policy."
    end if
```

### 8.2 Active Insertion Point Check Template (Blind Command Injection)
```bcheck
metadata:
    language: v2-beta
    name: "Blind OS Command Injection (OAST)"
    description: "Tests for command injection using out-of-band DNS lookups."
    author: "Security Team"
    tags: "active", "rce", "oast"

define:
    collab = {generate_collaborator_address()}

given any insertion point then
    send payload:
        replacing: `$(nslookup {collab})`

    if dns_interaction in {collab.interactions} then
        report issue:
            severity: high
            confidence: certain
            detail: `Target executed blind command injection triggering DNS resolution at {collab}.`
            remediation: "Avoid passing user input directly into system shells. Use parameterized APIs."
    end if
```

---

## 9. Safe Import & Review Protocol

1. **Review**:
   - `language: v2-beta` specified.
   - Exactly one `given ... then` block.
   - All loops (`run for each`) and insertion points statically bounded.
   - Negative controls included to prevent false positives.
2. **Import via MCP**:
   Always import initially disabled (`enabled: false`):
   ```json
   {
     "script": "<complete_bcheck_content>",
     "enabled": false
   }
   ```
   Execute with `burp_bcheck_import`.
3. **Verification**:
   - Verify tool returns `success: true`, status `LOADED_WITHOUT_ERRORS`, and `errors: []`.
