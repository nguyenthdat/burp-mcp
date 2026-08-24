# Java MCP Server Implementation Guide

## Overview

This document provides Java-specific best practices and examples for implementing MCP servers using the official MCP Java SDK (`io.modelcontextprotocol.sdk:mcp`). It covers server setup with the `McpServer` builder, tool/resource/prompt specifications, input validation, the two-tier error model, transports, and a complete working example. The SDK targets Java 17+ and exposes both a synchronous facade (`McpSyncServer`) and a reactive async API (`McpAsyncServer`, built on Project Reactor).

For complete SDK documentation, use WebFetch to load:
- SDK README: `https://raw.githubusercontent.com/modelcontextprotocol/java-sdk/main/README.md`
- Server guide: `https://java.sdk.modelcontextprotocol.io/latest/server/`

---

## Quick Reference

### Key Imports
```java
import io.modelcontextprotocol.server.McpServer;
import io.modelcontextprotocol.server.McpSyncServer;
import io.modelcontextprotocol.server.McpServerFeatures.SyncToolSpecification;
import io.modelcontextprotocol.server.transport.StdioServerTransportProvider;
import io.modelcontextprotocol.spec.McpSchema;
import io.modelcontextprotocol.spec.McpSchema.CallToolResult;
import io.modelcontextprotocol.spec.McpSchema.ServerCapabilities;
import io.modelcontextprotocol.spec.McpSchema.Tool;
import io.modelcontextprotocol.json.McpJsonDefaults;
import java.util.List;
```

### Server Initialization
```java
var transportProvider = new StdioServerTransportProvider(McpJsonDefaults.getMapper());

McpSyncServer server = McpServer.sync(transportProvider)
    .serverInfo("service_mcp", "1.0.0")
    .capabilities(ServerCapabilities.builder()
        .tools(true)      // enable tools + listChanged notifications
        .build())
    .build();
```

### Tool Registration Pattern
```java
var spec = SyncToolSpecification.builder()
    .tool(Tool.builder("service_tool_name", schema).description("...").build())
    .callHandler((exchange, request) -> CallToolResult.builder()
        .content(List.of(new McpSchema.TextContent("result")))
        .build())
    .build();

server.addTool(spec); // or register at build time via .tools(spec)
```

---

## MCP Java SDK (`io.modelcontextprotocol.sdk`)

The core `io.modelcontextprotocol.sdk:mcp` module (a convenience bundle of `mcp-core` + Jackson) provides STDIO, SSE, and Streamable HTTP transports without any external web framework. Spring integrations (`mcp-spring-webflux`, `mcp-spring-webmvc`) ship separately in Spring AI 2.0+.

- Sync API (`McpServer.sync(...)` → `McpSyncServer`): blocking handlers that return values directly. Prefer this unless you need reactive streaming.
- Async API (`McpServer.async(...)` → `McpAsyncServer`): handlers return `Mono<T>`; registration returns `Mono<Void>` you must `.subscribe()`.

### Dependencies

Use the BOM (`mcp-bom`) to align versions. Maven:
```xml
<dependencyManagement>
  <dependencies>
    <dependency>
      <groupId>io.modelcontextprotocol.sdk</groupId>
      <artifactId>mcp-bom</artifactId>
      <version>0.14.0</version> <!-- check Maven Central for the latest -->
      <type>pom</type>
      <scope>import</scope>
    </dependency>
  </dependencies>
</dependencyManagement>
<dependencies>
  <dependency>
    <groupId>io.modelcontextprotocol.sdk</groupId>
    <artifactId>mcp</artifactId>
  </dependency>
</dependencies>
```
Gradle: `implementation(platform("io.modelcontextprotocol.sdk:mcp-bom:0.14.0"))` then `implementation("io.modelcontextprotocol.sdk:mcp")`.

## Server Naming Convention

Use the MCP server-info name `{service}_mcp` (lowercase, no version/date), e.g. `github_mcp`. Use idiomatic Java type names for classes (`GithubMcpServer`) and packages (`com.example.githubmcp`).

## Project Structure

```
service-mcp/
├── pom.xml (or build.gradle.kts)
└── src/main/java/com/example/servicemcp/
    ├── ServiceMcpServer.java   # main() + server wiring
    ├── tools/                  # one class per tool or grouped tool specs
    ├── client/                 # API client + auth
    └── util/                   # shared formatting, error mapping
```

## Tool Implementation

### Tool Naming
Use snake_case, action-oriented names with a service prefix to avoid collisions: `github_create_issue`, not `create_issue`.

### Tool Structure

Build a `Tool` (name + description + JSON input schema) and attach a call handler. The handler receives an `McpSyncServerExchange` (for sampling/elicitation/logging back to the client) and a `CallToolRequest` (arguments via `request.arguments()`).

```java
String schema = """
    {
      "type": "object",
      "properties": {
        "query":  { "type": "string", "description": "Search text", "minLength": 1 },
        "limit":  { "type": "integer", "description": "Max results 1-100", "minimum": 1, "maximum": 100 }
      },
      "required": ["query"]
    }
    """;

var searchUsers = SyncToolSpecification.builder()
    .tool(Tool.builder("service_search_users", schema)
        .description("Search users by name or email. Read-only; does not modify data.")
        .annotations(new McpSchema.ToolAnnotations(
            "Search Users", true, false, true, true)) // title, readOnly, destructive, idempotent, openWorld
        .build())
    .callHandler((exchange, request) -> {
        String query = (String) request.arguments().get("query");
        // ... call the API, format the result ...
        return CallToolResult.builder()
            .content(List.of(new McpSchema.TextContent(formatted)))
            .build();
    })
    .build();
```

## Input Validation

The server validates incoming arguments against the tool `inputSchema` **before** invoking your handler; on failure it returns a `CallToolResult` with `isError` set instead of calling you. Define constraints (`minLength`, `minimum`, `enum`, `required`) in the schema. You can disable this with `.validateToolInputs(false)` on the builder, but keep it on. Embedded schemas are validated against JSON Schema 2020-12 at build time — malformed schemas throw `IllegalArgumentException`.

## Response Format Options

Return human-readable Markdown by default and offer JSON when callers need structured data (add a `response_format` enum to the schema). Convert epoch timestamps to readable form; show display names with IDs (`@john (U123)`); omit verbose metadata. For structured data, prefer defining an output schema and returning `structuredContent` (see Advanced Features).

## Pagination Implementation

For list tools, accept `limit`/`offset` (or a cursor) in the schema and return pagination metadata in the response body so the agent can request the next page:
```java
Map<String, Object> body = Map.of(
    "total", total, "count", items.size(), "offset", offset,
    "items", items, "has_more", total > offset + items.size());
```

## Error Handling

MCP distinguishes two tiers — choose deliberately:

1. **Tool-level (recoverable by the LLM):** validation failures, missing arguments, domain errors. Return a normal result with `isError(true)` so the model can read the message and self-correct:
   ```java
   return CallToolResult.builder()
       .content(List.of(new McpSchema.TextContent("Invalid argument: 'email' must be a valid email.")))
       .isError(true)
       .build();
   ```
2. **Protocol-level (unrecoverable):** infrastructure failures (DB timeout, auth misconfig). Throw `McpError` (or let an exception propagate); it maps to a JSON-RPC error:
   ```java
   throw new McpError(McpSchema.ErrorCodes.INTERNAL_ERROR, "Upstream API unavailable");
   ```
Use tier 1 for anything the agent could fix by changing inputs; reserve tier 2 for genuine faults.

## Shared Utilities

Centralize the API client, auth headers, response formatting, and error mapping in a `util`/`client` package. Never duplicate HTTP or formatting logic across tools. Use the JDK `HttpClient` (built into `mcp-core`) for upstream calls.

## Concurrency (Sync vs Async)

- **Sync:** handlers run on the calling thread and return values directly. Simplest; fine for blocking I/O.
- **Async:** handlers return `Mono<T>`; compose non-blocking pipelines with Reactor. Use for high-concurrency streaming servers. Async registrations return a `Mono<Void>` — remember to `.subscribe()`.

## Package Configuration

Produce an executable jar (Maven Shade / Gradle `application` + `shadowJar`) so the server can be launched as `java -jar service-mcp.jar` over stdio. Send all logging to **stderr** (SLF4J) — stdout is reserved for the JSON-RPC protocol on the stdio transport.

## Complete Example

```java
package com.example.servicemcp;

import io.modelcontextprotocol.server.McpServer;
import io.modelcontextprotocol.server.McpSyncServer;
import io.modelcontextprotocol.server.McpServerFeatures.SyncToolSpecification;
import io.modelcontextprotocol.server.transport.StdioServerTransportProvider;
import io.modelcontextprotocol.spec.McpSchema;
import io.modelcontextprotocol.spec.McpSchema.CallToolResult;
import io.modelcontextprotocol.spec.McpSchema.ServerCapabilities;
import io.modelcontextprotocol.spec.McpSchema.Tool;
import io.modelcontextprotocol.json.McpJsonDefaults;
import java.util.List;

public class ServiceMcpServer {

    static final String SEARCH_SCHEMA = """
        { "type":"object",
          "properties": { "query": {"type":"string","minLength":1} },
          "required": ["query"] }
        """;

    public static void main(String[] args) {
        var transport = new StdioServerTransportProvider(McpJsonDefaults.getMapper());

        var searchUsers = SyncToolSpecification.builder()
            .tool(Tool.builder("service_search_users", SEARCH_SCHEMA)
                .description("Search users by name or email. Read-only.")
                .build())
            .callHandler((exchange, request) -> {
                String query = (String) request.arguments().get("query");
                if (query == null || query.isBlank()) {
                    return CallToolResult.builder()
                        .content(List.of(new McpSchema.TextContent("Error: 'query' is required.")))
                        .isError(true).build();
                }
                try {
                    String result = ServiceClient.searchUsers(query); // shared client
                    return CallToolResult.builder()
                        .content(List.of(new McpSchema.TextContent(result))).build();
                } catch (Exception e) {
                    throw new McpError(McpSchema.ErrorCodes.INTERNAL_ERROR, "Search failed: " + e.getMessage());
                }
            })
            .build();

        McpSyncServer server = McpServer.sync(transport)
            .serverInfo("service_mcp", "1.0.0")
            .capabilities(ServerCapabilities.builder().tools(true).build())
            .tools(searchUsers)
            .build();

        // Server runs over stdio until the client disconnects.
        Runtime.getRuntime().addShutdownHook(new Thread(server::close));
    }
}
```

---

## Advanced MCP Features

### Structured Output
Define an output schema on the `Tool` and return `structuredContent` in the `CallToolResult` so clients get typed data alongside the text block.

### Resource Registration
```java
var resourceSpec = new io.modelcontextprotocol.server.McpServerFeatures.SyncResourceSpecification(
    McpSchema.Resource.builder("custom://resource", "name").mimeType("text/plain").build(),
    (exchange, request) -> McpSchema.ReadResourceResult.builder(contents).build());
server.addResource(resourceSpec);
```
Use `SyncResourceTemplateSpecification` with a `ResourceTemplate` (`file://{path}`) for parameterized resources. Enable `resources(subscribe, listChanged)` in capabilities; notify subscribers with `server.notifyResourcesUpdated(...)`.

### Transports
- **STDIO:** `new StdioServerTransportProvider(McpJsonDefaults.getMapper())` — local subprocess servers.
- **Streamable HTTP (recommended for remote):** `HttpServletStreamableServerTransportProvider.builder().jsonMapper(jsonMapper).mcpEndpoint("/mcp").build()` in any Servlet container.
- **Spring:** WebFlux/WebMVC transport providers via Spring AI 2.0+.

### Sampling, Elicitation & Logging
Inside a handler, use the `exchange`: check `exchange.getClientCapabilities().sampling()`/`.elicitation()` before calling `exchange.createMessage(...)` or `exchange.elicit(...)`; send `exchange.loggingNotification(...)` for structured logs (levels DEBUG→EMERGENCY).

## Building and Running

```bash
# Build an executable jar
./mvnw -q clean package        # or: ./gradlew shadowJar

# Run over stdio
java -jar target/service-mcp.jar

# Test with MCP Inspector
npx @modelcontextprotocol/inspector java -jar target/service-mcp.jar
```

## Quality Checklist

### Strategic Design
- [ ] Tools enable complete workflows, not just thin endpoint wrappers
- [ ] Tool names are prefixed and action-oriented
- [ ] Response formats optimize for agent context efficiency
- [ ] Error messages guide the agent toward correct usage

### Implementation Quality
- [ ] Server-info name follows `{service}_mcp`
- [ ] Each tool has a name, description, and JSON `inputSchema` with constraints
- [ ] Annotations set (`readOnly`, `destructive`, `idempotent`, `openWorld`)
- [ ] Shared API client / formatting / error mapping extracted (no duplication)
- [ ] Upstream I/O uses the JDK `HttpClient` (or Reactor for async)

### Java Quality
- [ ] Sync vs async chosen deliberately; async registrations `.subscribe()`d
- [ ] Logging goes to stderr (SLF4J), never stdout on stdio transport
- [ ] Two-tier errors: `isError(true)` for recoverable, `McpError` for faults
- [ ] Input-schema validation left enabled

### Advanced Features (where applicable)
- [ ] Output schema + `structuredContent` for structured results
- [ ] Resources / resource templates registered where useful
- [ ] Appropriate transport (STDIO local, Streamable HTTP remote)

### Testing and Build
- [ ] `./mvnw package` (or `gradlew shadowJar`) builds a runnable jar
- [ ] Server starts over stdio and responds in MCP Inspector
- [ ] Sample tool calls and error scenarios behave as expected
```
