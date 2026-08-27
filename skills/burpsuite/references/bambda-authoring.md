# Writing and Importing Bambdas (Montoya Java API Specification)

Use this reference when the user asks to create, adapt, review, or import a Bambda through `burp_bambda_import`. A Bambda is a reusable Java snippet packaged inside a YAML document that executes natively inside Burp Suite via the **Montoya API**.

---

## 1. Bambda YAML Document Structure

Every Bambda passed to `burp_bambda_import` must be a complete YAML document:

```yaml
id: 3c9b1e84-7a2f-4e9b-8d1c-5a6b7c8d9e0f
name: Descriptive Bambda Name
function: FUNCTION_ENUM
location: LOCATION_ENUM
source: |+
  /**
   * Javadoc explaining the script purpose and behavior.
   * @author Security Engineer
   */
  // Java code body executed by Burp Suite
```

### Required Fields:
- `id`: A valid, stable UUID string (e.g. `2f4d6f7a-6ea3-4fd9-b2ec-e5f5f8b3fd0e`). Keep `id` unchanged when updating an existing Bambda.
- `name`: Human-readable title displayed in Burp Suite UI.
- `function`: The operational role (e.g. `VIEW_FILTER`, `CUSTOM_COLUMN`, `CUSTOM_ACTION`, `MATCH_AND_REPLACE`, `SCAN_CHECK`).
- `location`: The Burp Suite UI surface where the Bambda runs (e.g. `PROXY_HTTP_HISTORY`, `PROXY_WEBSOCKET_HISTORY`, `SITE_MAP`, `LOGGER_CAPTURE`, `LOGGER_DISPLAY`, `REPEATER`, `SCANNER`).
- `source`: Valid Java code (indented under `|+` block scalar) fulfilling the function's return contract.

---

## 2. Supported Function & Location Contracts

| Function | Location | Context Variable(s) | Return Type | Purpose |
|---|---|---|---|---|
| `VIEW_FILTER` | `PROXY_HTTP_HISTORY`<br>`SITE_MAP`<br>`LOGGER_CAPTURE`<br>`LOGGER_DISPLAY` | `HttpRequestResponse requestResponse` | `boolean` | Filter HTTP history table rows. |
| `VIEW_FILTER` | `PROXY_WEBSOCKET_HISTORY` | `WebSocketMessage webSocketMessage` | `boolean` | Filter WebSocket message rows. |
| `CUSTOM_COLUMN` | `PROXY_HTTP_HISTORY`<br>`LOGGER` | `HttpRequestResponse requestResponse` | `String` / `Object` | Render a custom table column value. |
| `CUSTOM_ACTION` | `REPEATER`<br>`PROXY_HTTP_HISTORY` | `HttpRequestResponse requestResponse` | `HttpRequest` / `void` | Execute custom action on request. |
| `MATCH_AND_REPLACE` | `PROXY_HTTP_REQUEST`<br>`PROXY_HTTP_RESPONSE` | `HttpRequestToBeSent request`<br>`HttpResponseReceived response` | `HttpRequest` / `HttpResponse` | Transform requests/responses in proxy pipeline. |
| `SCAN_CHECK` | `SCANNER` | `HttpRequestResponse requestResponse` | `AuditResult` | Custom passive or active audit check. |

---

## 3. Core Montoya API Cheat Sheet (Java in Bambda)

The following Montoya API methods are natively available inside Bambda execution scopes:

### Request Inspection (`requestResponse.request()`)
- `request.url()` $\rightarrow$ `String`
- `request.method()` $\rightarrow$ `String` (`"GET"`, `"POST"`, etc.)
- `request.path()` $\rightarrow$ `String`
- `request.query()` $\rightarrow$ `String`
- `request.hasHeader(String name)` $\rightarrow$ `boolean`
- `request.headerValue(String name)` $\rightarrow$ `String` (or `null`)
- `request.headers()` $\rightarrow$ `List<HttpHeader>`
- `request.bodyToString()` $\rightarrow$ `String`
- `request.body()` $\rightarrow$ `ByteArray`
- `request.hasParameters()` $\rightarrow$ `boolean`
- `request.parameters()` $\rightarrow$ `List<ParsedHttpParameter>`
- `request.withHeader(String name, String value)` $\rightarrow$ `HttpRequest`
- `request.withBody(String body)` $\rightarrow$ `HttpRequest`

### Response Inspection (`requestResponse.response()`)
- `requestResponse.hasResponse()` $\rightarrow$ `boolean` *(Always check this before accessing response!)*
- `response.statusCode()` $\rightarrow$ `short` (e.g. `200`, `302`, `403`, `500`)
- `response.hasHeader(String name)` $\rightarrow$ `boolean`
- `response.headerValue(String name)` $\rightarrow$ `String`
- `response.headers()` $\rightarrow$ `List<HttpHeader>`
- `response.bodyToString()` $\rightarrow$ `String`
- `response.body()` $\rightarrow$ `ByteArray`
- `response.mimeType()` $\rightarrow$ `MimeType`

### Built-in Helpers
- `utilities().base64Utils().encodeToString(...)`
- `utilities().urlUtils().decode(...)`

---

## 4. Production Bambda Templates

### 4.1 Proxy HTTP View Filter (`VIEW_FILTER` on `PROXY_HTTP_HISTORY`)
```yaml
id: 2f4d6f7a-6ea3-4fd9-b2ec-e5f5f8b3fd0e
name: Filter Successful JSON Responses
function: VIEW_FILTER
location: PROXY_HTTP_HISTORY
source: |+
  /**
   * Keeps only 2xx/3xx HTTP responses whose Content-Type is JSON.
   */
  return requestResponse.hasResponse()
      && requestResponse.response().statusCode() >= 200
      && requestResponse.response().statusCode() < 400
      && requestResponse.response().hasHeader("Content-Type")
      && requestResponse.response().headerValue("Content-Type")
          .toLowerCase()
          .contains("application/json");
```

### 4.2 Proxy WebSocket View Filter (`VIEW_FILTER` on `PROXY_WEBSOCKET_HISTORY`)
```yaml
id: a1b2c3d4-e5f6-7a8b-9c0d-1e2f3a4b5c6d
name: Filter Client Outbound Auth Frames
function: VIEW_FILTER
location: PROXY_WEBSOCKET_HISTORY
source: |+
  /**
   * Filter client-to-server WebSocket messages containing sensitive auth fields.
   */
  return webSocketMessage.direction() == Direction.CLIENT_TO_SERVER
      && (webSocketMessage.payload().contains("Bearer") || webSocketMessage.payload().contains("token"));
```

### 4.3 Custom Table Column (`CUSTOM_COLUMN` on `PROXY_HTTP_HISTORY`)
```yaml
id: b2c3d4e5-f6a7-8b9c-0d1e-2f3a4b5c6d7e
name: Extract Server & Technology Headers
function: CUSTOM_COLUMN
location: PROXY_HTTP_HISTORY
source: |+
  /**
   * Displays Server or X-Powered-By header in a custom history column.
   */
  if (!requestResponse.hasResponse()) {
      return "";
  }
  if (requestResponse.response().hasHeader("Server")) {
      return requestResponse.response().headerValue("Server");
  }
  if (requestResponse.response().hasHeader("X-Powered-By")) {
      return requestResponse.response().headerValue("X-Powered-By");
  }
  return "-";
```

### 4.4 Repeater Custom Action (`CUSTOM_ACTION` on `REPEATER`)
```yaml
id: c3d4e5f6-a7b8-9c0d-1e2f-3a4b5c6d7e8f
name: Add Testing Canary Header
function: CUSTOM_ACTION
location: REPEATER
source: |+
  /**
   * Repeater action: adds X-Security-Canary header to current request.
   */
  HttpRequest req = requestResponse.request();
  return req.withHeader("X-Security-Canary", "AuthTest-" + System.currentTimeMillis());
```

### 4.5 Match and Replace Script (`MATCH_AND_REPLACE` on `PROXY_HTTP_REQUEST`)
```yaml
id: d4e5f6a7-b8c9-0d1e-2f3a-4b5c6d7e8f9a
name: Remove Anti-Clickjacking Frame Restrictions
function: MATCH_AND_REPLACE
location: PROXY_HTTP_RESPONSE
source: |+
  /**
   * Strips X-Frame-Options and Content-Security-Policy frame-ancestors for clickjacking testing.
   */
  HttpResponse resp = requestResponse.response();
  if (resp == null) {
      return resp;
  }
  HttpResponse modified = resp;
  if (modified.hasHeader("X-Frame-Options")) {
      modified = modified.withoutHeader("X-Frame-Options");
  }
  return modified;
```

### 4.6 Custom Passive Scan Check (`SCAN_CHECK` on `SCANNER`)
```yaml
id: e5f6a7b8-c9d0-1e2f-3a4b-5c6d7e8f9a0b
name: Audit Missing Strict-Transport-Security
function: SCAN_CHECK
location: SCANNER
source: |+
  /**
   * Passive custom audit check reporting missing HSTS on HTTPS targets.
   */
  if (!requestResponse.hasResponse() || !requestResponse.request().url().startsWith("https://")) {
      return AuditResult.auditResult();
  }

  if (!requestResponse.response().hasHeader("Strict-Transport-Security")) {
      return AuditResult.auditResult(
          AuditIssue.auditIssue(
              "Strict-Transport-Security Header Missing",
              "The HTTPS endpoint did not include a Strict-Transport-Security (HSTS) header.",
              "Configure HSTS with max-age >= 31536000 and includeSubDomains.",
              requestResponse.request().url(),
              AuditIssueSeverity.LOW,
              AuditIssueConfidence.CERTAIN,
              "HSTS ensures browsers only communicate via HTTPS, preventing SSL stripping.",
              "Ensure all subdomains support HTTPS before enabling includeSubDomains.",
              AuditIssueSeverity.LOW,
              requestResponse
          )
      );
  }

  return AuditResult.auditResult();
```

---

## 5. Safety & Performance Best Practices

1. **Guard Missing Responses**: Always verify `requestResponse.hasResponse()` before calling `requestResponse.response()`.
2. **Deterministic & Fast Execution**: View filters and custom columns evaluate over thousands of rows. Avoid disk I/O, network requests, or excessive string copying.
3. **No Embedded Credentials**: Never hardcode production secrets or tokens.
4. **Preserve UUIDs**: Preserve the `id` field when updating a script so Burp updates the existing entry rather than duplicating it.
5. **Import Verification**:
   - Send complete YAML via `burp_bambda_import`.
   - Verify `success: true`, `status: LOADED_WITHOUT_ERRORS`, and `errors: []`.
