# Kotlin MCP Server Implementation Guide

## Overview

This document provides Kotlin-specific best practices and examples for implementing MCP servers using the official MCP Kotlin SDK (`io.modelcontextprotocol:kotlin-sdk`). The SDK is Kotlin Multiplatform (JVM, Native, JS, Wasm), coroutine-first, and uses Ktor for HTTP transports and kotlinx.serialization for JSON. It covers server setup, `addTool` registration, transports (STDIO / Streamable HTTP over Ktor), error handling, and a complete example.

For complete SDK documentation, use WebFetch to load:
`https://raw.githubusercontent.com/modelcontextprotocol/kotlin-sdk/main/README.md`

---

## Quick Reference

### Key Imports
```kotlin
import io.modelcontextprotocol.kotlin.sdk.server.Server
import io.modelcontextprotocol.kotlin.sdk.server.ServerOptions
import io.modelcontextprotocol.kotlin.sdk.server.mcpStreamableHttp
import io.modelcontextprotocol.kotlin.sdk.types.CallToolResult
import io.modelcontextprotocol.kotlin.sdk.types.Implementation
import io.modelcontextprotocol.kotlin.sdk.types.ServerCapabilities
import io.modelcontextprotocol.kotlin.sdk.types.TextContent
import io.modelcontextprotocol.kotlin.sdk.types.ToolSchema
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put
```

### Server Initialization
```kotlin
val server = Server(
    serverInfo = Implementation(name = "service_mcp", version = "1.0.0"),
    options = ServerOptions(
        capabilities = ServerCapabilities(
            tools = ServerCapabilities.Tools(listChanged = true),
        ),
    ),
)
```

### Tool Registration Pattern
```kotlin
server.addTool(
    name = "service_tool_name",
    description = "...",
    inputSchema = ToolSchema(properties = buildJsonObject {
        put("query", buildJsonObject { put("type", "string") })
    }),
) { request ->
    CallToolResult(content = listOf(TextContent("result")))
}
```

---

## MCP Kotlin SDK (`io.modelcontextprotocol:kotlin-sdk`)

Artifacts:
- `io.modelcontextprotocol:kotlin-sdk` — umbrella (client + server)
- `io.modelcontextprotocol:kotlin-sdk-server` — server-only
- `io.modelcontextprotocol:kotlin-sdk-client` — client-only

The SDK uses Ktor but does **not** pull Ktor engine dependencies transitively — declare your own engine (e.g. `ktor-server-cio` / `ktor-server-netty`). Tool/resource/prompt handlers are `suspend` functions, so all I/O uses coroutines.

Gradle (JVM):
```kotlin
repositories { mavenCentral() }
dependencies {
    implementation("io.modelcontextprotocol:kotlin-sdk:0.8.0") // check Maven Central for latest
    implementation("io.ktor:ktor-server-cio:3.0.0")            // only for Streamable HTTP
}
```

## Server Naming Convention

Use the server-info name `{service}_mcp` (lowercase, no version/date), e.g. `github_mcp`. Use idiomatic Kotlin names for classes/files (`GithubMcpServer.kt`).

## Project Structure

```
service-mcp/
├── build.gradle.kts
└── src/main/kotlin/com/example/servicemcp/
    ├── Main.kt          # server wiring + transport
    ├── Tools.kt         # addTool registrations
    ├── ServiceClient.kt # API client (Ktor client) + auth
    └── Format.kt        # shared formatting / error mapping
```

## Tool Implementation

### Tool Naming
snake_case, action-oriented, service-prefixed: `slack_send_message`, not `send_message`.

### Tool Structure

`addTool` takes a name, description, and `ToolSchema` (JSON Schema via `buildJsonObject`). The trailing lambda is a `suspend` handler receiving a `CallToolRequest`; read arguments as JSON with `request.arguments?.get("key")?.jsonPrimitive?.content`.

```kotlin
server.addTool(
    name = "service_search_users",
    description = "Search users by name or email. Read-only; does not modify data.",
    inputSchema = ToolSchema(
        properties = buildJsonObject {
            put("query", buildJsonObject {
                put("type", "string")
                put("description", "Search text")
            })
            put("limit", buildJsonObject {
                put("type", "integer"); put("minimum", 1); put("maximum", 100)
            })
        },
        required = listOf("query"),
    ),
) { request ->
    val query = request.arguments?.get("query")?.jsonPrimitive?.content
    if (query.isNullOrBlank()) {
        return@addTool CallToolResult(
            content = listOf(TextContent("Error: 'query' is required.")),
            isError = true,
        )
    }
    val result = ServiceClient.searchUsers(query) // suspend call
    CallToolResult(content = listOf(TextContent(result)))
}
```

## Input Validation

Express constraints (`type`, `minimum`, `maximum`, `enum`, `required`) in the `ToolSchema`. Because arguments arrive as `JsonElement`s, read them defensively (`?.jsonPrimitive?.content`, `?.jsonPrimitive?.intOrNull`) and return an `isError` result for missing/malformed values rather than throwing. For richer validation, deserialize the arguments into a `@Serializable` data class with kotlinx.serialization.

## Response Format Options

Default to Markdown; offer a `response_format` enum (`markdown`/`json`) when callers need structured output. Humanize timestamps, pair display names with IDs, and drop verbose metadata. Serialize JSON responses with kotlinx.serialization.

## Pagination Implementation

Accept `limit`/`offset` (or a cursor) and return pagination metadata so the agent can page:
```kotlin
val body = buildJsonObject {
    put("total", total); put("count", items.size); put("offset", offset)
    put("has_more", total > offset + items.size)
}
```

## Error Handling

- **Recoverable (tool-level):** return `CallToolResult(content = ..., isError = true)` for validation/domain errors the model can fix.
- **Unrecoverable (protocol-level):** let exceptions propagate (or throw) for genuine infrastructure faults; they map to a JSON-RPC error.

Centralize upstream error mapping (HTTP status → actionable message) in one helper:
```kotlin
fun apiError(status: Int): String = when (status) {
    404 -> "Error: Resource not found. Check the ID."
    403 -> "Error: Permission denied."
    429 -> "Error: Rate limit exceeded. Retry later."
    else -> "Error: API request failed ($status)."
}
```

## Shared Utilities

Use a single Ktor `HttpClient` for upstream calls and share auth, formatting, and error mapping. Never duplicate request/format logic across tools.

## Coroutines & Concurrency

All handlers are `suspend` — call suspending APIs directly and never block the thread. Use `withContext(Dispatchers.IO)` only for unavoidable blocking libraries. The SDK manages concurrent client requests; keep handlers stateless or guard shared state.

## Package Configuration

For a stdio server, build an executable/`application` distribution (`application` plugin + `installDist`/`shadowJar`) launched as a subprocess. Send logs to **stderr** — stdout carries JSON-RPC on the stdio transport.

## Complete Example

```kotlin
package com.example.servicemcp

import io.ktor.server.cio.CIO
import io.ktor.server.engine.embeddedServer
import io.modelcontextprotocol.kotlin.sdk.server.Server
import io.modelcontextprotocol.kotlin.sdk.server.ServerOptions
import io.modelcontextprotocol.kotlin.sdk.server.mcpStreamableHttp
import io.modelcontextprotocol.kotlin.sdk.types.*
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put

fun buildServer(): Server {
    val server = Server(
        serverInfo = Implementation(name = "service_mcp", version = "1.0.0"),
        options = ServerOptions(
            capabilities = ServerCapabilities(tools = ServerCapabilities.Tools(listChanged = true)),
        ),
    )

    server.addTool(
        name = "service_search_users",
        description = "Search users by name or email. Read-only.",
        inputSchema = ToolSchema(
            properties = buildJsonObject {
                put("query", buildJsonObject { put("type", "string") })
            },
            required = listOf("query"),
        ),
    ) { request ->
        val query = request.arguments?.get("query")?.jsonPrimitive?.content
        if (query.isNullOrBlank()) {
            CallToolResult(content = listOf(TextContent("Error: 'query' is required.")), isError = true)
        } else {
            val result = ServiceClient.searchUsers(query)
            CallToolResult(content = listOf(TextContent(result)))
        }
    }
    return server
}

// Streamable HTTP (recommended for remote servers)
fun main() {
    embeddedServer(CIO, host = "127.0.0.1", port = 3000) {
        mcpStreamableHttp { buildServer() }
    }.start(wait = true)
}
```

For a **STDIO** server, create a session over stdin/stdout instead of the Ktor block:
```kotlin
import io.modelcontextprotocol.kotlin.sdk.server.StdioServerTransport
import kotlinx.io.asSink; import kotlinx.io.asSource; import kotlinx.io.buffered

suspend fun main() {
    val server = buildServer()
    server.createSession(
        StdioServerTransport(
            inputStream = System.`in`.asSource().buffered(),
            outputStream = System.out.asSink().buffered(),
        )
    )
}
```

---

## Advanced MCP Features

### Resource Registration
```kotlin
server.addResource(
    uri = "note://release/latest", name = "Release notes",
    description = "Last deploy summary", mimeType = "text/markdown",
) { request ->
    ReadResourceResult(contents = listOf(
        TextResourceContents(text = "…", uri = request.uri, mimeType = "text/markdown")))
}
```
Enable `resources = ServerCapabilities.Resources(subscribe = true, listChanged = true)` in capabilities.

### Prompts
`server.addPrompt(name, description, arguments) { request -> GetPromptResult(...) }` with `prompts = ServerCapabilities.Prompts(listChanged = true)`.

### Transports
- **STDIO** — `StdioServerTransport` via `server.createSession(...)`; local subprocesses.
- **Streamable HTTP (recommended remote)** — Ktor `mcpStreamableHttp(path = "/mcp") { server }`; installs `ContentNegotiation` automatically (don't add it yourself). Add the Ktor CORS plugin (allow/expose `Mcp-Session-Id`, `Mcp-Protocol-Version`) for browser clients.
- **SSE** (legacy), **WebSocket**, and **ChannelTransport** (in-process testing) are also available.

## Building and Running

```bash
# Build & run (Streamable HTTP on :3000)
./gradlew run

# Test with MCP Inspector (connect to http://localhost:3000/mcp)
npx -y @modelcontextprotocol/inspector
```

## Quality Checklist

### Strategic Design
- [ ] Tools model complete workflows, not thin endpoint wrappers
- [ ] Tool names are service-prefixed and action-oriented
- [ ] Response formats optimize for agent context efficiency
- [ ] Error messages guide the agent toward correct usage

### Implementation Quality
- [ ] Server-info name follows `{service}_mcp`
- [ ] Each tool has a `ToolSchema` with types, constraints, and `required`
- [ ] Arguments read defensively from `JsonElement` (or a `@Serializable` model)
- [ ] Shared Ktor `HttpClient`, formatting, and error mapping (no duplication)

### Kotlin Quality
- [ ] Handlers are `suspend`; no thread blocking (use `Dispatchers.IO` only when forced)
- [ ] Logging goes to stderr, never stdout on stdio transport
- [ ] Recoverable errors use `isError = true`; faults propagate as exceptions
- [ ] A Ktor engine dependency is declared for HTTP transport

### Advanced Features (where applicable)
- [ ] Resources / prompts registered where useful
- [ ] CORS configured for browser-based Streamable HTTP clients
- [ ] Appropriate transport (STDIO local, Streamable HTTP remote)

### Testing and Build
- [ ] `./gradlew run` starts the server
- [ ] Connects and lists tools in MCP Inspector
- [ ] Sample tool calls and error scenarios behave as expected
```
