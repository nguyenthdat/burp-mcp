# Swift MCP Server Implementation Guide

## Overview

This document provides Swift-specific best practices and examples for implementing MCP servers using the official MCP Swift SDK (`swift-sdk`, product `MCP`). The SDK is `async`/`await`-first, actor-based, and requires Swift 6.0+ (Xcode 16+); it runs on Apple platforms and Linux. It covers server setup, method-handler registration (`withMethodHandler`), tools/resources/prompts, transports, error handling, graceful shutdown, and a complete example.

For complete SDK documentation, use WebFetch to load:
`https://raw.githubusercontent.com/modelcontextprotocol/swift-sdk/main/README.md`

---

## Quick Reference

### Installation (Swift Package Manager)
```swift
// Package.swift
dependencies: [
    .package(url: "https://github.com/modelcontextprotocol/swift-sdk.git", from: "0.11.0")
]
// target:
.product(name: "MCP", package: "swift-sdk")
```

### Server Initialization
```swift
import MCP

let server = Server(
    name: "service_mcp",
    version: "1.0.0",
    capabilities: .init(tools: .init(listChanged: true))
)
let transport = StdioTransport()
try await server.start(transport: transport)
```

### Tool Registration Pattern
```swift
await server.withMethodHandler(ListTools.self) { _ in
    .init(tools: [Tool(name: "service_search_users", description: "...", inputSchema: schema)])
}
await server.withMethodHandler(CallTool.self) { params in
    // switch on params.name; return .init(content: [.text("...")], isError: false)
}
```

---

## MCP Swift SDK (`MCP`)

Unlike per-tool registration in some SDKs, the Swift server registers **method handlers**: one for `ListTools` (advertise your tools) and one for `CallTool` (dispatch on `params.name`). Handlers are `async` closures on the `Server` actor. Declare capabilities up front so clients know what you support.

## Server Naming Convention

Use the server-info name `{service}_mcp` (the `name:` argument). Use idiomatic Swift type names for supporting types.

## Project Structure

```
service-mcp/
├── Package.swift
└── Sources/service-mcp/
    ├── main.swift        # server setup + handlers + start
    ├── Tools.swift       # tool metadata + dispatch
    ├── ServiceClient.swift  # API client (URLSession) + auth
    └── Format.swift      # shared formatting / error mapping
```

## Tool Implementation

### Tool Naming
snake_case, action-oriented, service-prefixed: `github_create_issue`.

### Tool Structure

Advertise tools in the `ListTools` handler; implement them in the `CallTool` handler by switching on `params.name`. Read arguments from `params.arguments?["key"]?.stringValue` (and `.intValue`, `.boolValue`).

```swift
let searchTool = Tool(
    name: "service_search_users",
    description: "Search users by name or email. Read-only; does not modify data.",
    inputSchema: .object([
        "type": .string("object"),
        "properties": .object([
            "query": .object(["type": .string("string"), "description": .string("Search text")]),
            "limit": .object(["type": .string("integer"), "minimum": .int(1), "maximum": .int(100)]),
        ]),
        "required": .array([.string("query")]),
    ])
)

await server.withMethodHandler(ListTools.self) { _ in
    .init(tools: [searchTool])
}

await server.withMethodHandler(CallTool.self) { params in
    switch params.name {
    case "service_search_users":
        guard let query = params.arguments?["query"]?.stringValue, !query.isEmpty else {
            return .init(content: [.text("Error: 'query' is required.")], isError: true)
        }
        let result = try await ServiceClient.searchUsers(query)  // shared client
        return .init(content: [.text(result)], isError: false)
    default:
        return .init(content: [.text("Unknown tool")], isError: true)
    }
}
```

## Input Validation

Encode constraints (`type`, `minimum`, `maximum`, `enum`, `required`) in the tool `inputSchema`. Because arguments arrive as dynamic `Value`s, read them defensively (`?.stringValue`, `?.intValue`) and return an `isError` result for missing/invalid values.

## Response Format Options

Return `.text(...)` content by default (Markdown). Offer a `format` argument and serialize JSON (`Codable` + `JSONEncoder`) when callers need structured data. Content can also be `.image(...)`, `.audio(...)`, `.resource(...)`. Humanize timestamps, pair display names with IDs, and omit verbose metadata.

## Pagination Implementation

Accept `limit`/`offset` (or a cursor) in the schema and return pagination metadata in the JSON body. `ListTools`/`ListResources` results also carry a `nextCursor` for protocol-level pagination.

## Error Handling

- **Recoverable (tool-level):** return `.init(content: [.text("...")], isError: true)` for validation/domain errors the model can fix.
- **Unrecoverable (protocol-level):** `throw MCPError.invalidParams("...")` (or another `MCPError` case) for genuine faults; it maps to a JSON-RPC error.

```swift
do {
    try await server.start(transport: transport)
} catch let error as MCPError {
    // handle protocol error
}
```

## Shared Utilities

Wrap `URLSession` (auth headers, base URL, decoding) in a `ServiceClient` and share formatting/error mapping. Never duplicate request/format logic across tool cases.

## Concurrency

The SDK is actor-based and `async`/`await`-first. Handlers are `async` closures — `await` all I/O and never block. Task-locals are **not** inherited by `Task.detached`; capture `Server.currentHandlerContext` up front if you spawn detached work.

## Package Configuration

`Package.swift` with the `MCP` product dependency and an executable target. Build a release binary launched over stdio. Send logs to **stderr** (swift-log) — stdout carries JSON-RPC on the stdio transport.

## Complete Example

`Sources/service-mcp/main.swift`:
```swift
import MCP

let server = Server(
    name: "service_mcp",
    version: "1.0.0",
    capabilities: .init(tools: .init(listChanged: true))
)

let searchTool = Tool(
    name: "service_search_users",
    description: "Search users by name or email. Read-only.",
    inputSchema: .object([
        "type": .string("object"),
        "properties": .object([
            "query": .object(["type": .string("string")]),
        ]),
        "required": .array([.string("query")]),
    ])
)

await server.withMethodHandler(ListTools.self) { _ in .init(tools: [searchTool]) }

await server.withMethodHandler(CallTool.self) { params in
    switch params.name {
    case "service_search_users":
        guard let query = params.arguments?["query"]?.stringValue, !query.isEmpty else {
            return .init(content: [.text("Error: 'query' is required.")], isError: true)
        }
        let result = try await ServiceClient.searchUsers(query)
        return .init(content: [.text(result)], isError: false)
    default:
        return .init(content: [.text("Unknown tool")], isError: true)
    }
}

let transport = StdioTransport()
try await server.start(transport: transport)
try await Task.sleep(for: .days(365 * 100)) // keep running until terminated
```

---

## Advanced MCP Features

### Resources & Prompts
Register `ListResources`/`ReadResource` and `ListPrompts`/`GetPrompt` handlers the same way. Resource content: `Resource.Content.text(body, uri:, mimeType:)`.

### Structured / Rich Content
`CallTool` results accept multiple content items: `.text`, `.image(data:mimeType:metadata:)`, `.audio`, `.resource`.

### Transports
- **StdioTransport** — local subprocesses / CLI (Apple & Linux).
- **HTTPClientTransport** — client-side streaming HTTP.
- **StatelessHTTPServerTransport** — simple request/response HTTP servers (serverless/edge).
- **StatefulHTTPServerTransport** — full session management + SSE for server-initiated messages.
- **InMemoryTransport** — in-process testing.

### Graceful Shutdown
Prefer `swift-service-lifecycle`: implement the server as a `Service`, add it to a `ServiceGroup` with `gracefulShutdownSignals: [.sigterm, .sigint]`, and `await server.stop()` in `shutdown()`.

### Initialize Hook & HTTP Context
`server.start(transport:) { clientInfo, clientCapabilities in ... }` validates/inspects connecting clients. Over HTTP transports, read the originating request via `Server.currentHandlerContext?.httpContext` (headers, path).

## Building and Running

```bash
swift build
swift run

# Test with MCP Inspector
npx @modelcontextprotocol/inspector swift run
```

## Quality Checklist

### Strategic Design
- [ ] Tools model complete workflows, not thin endpoint wrappers
- [ ] Tool names are service-prefixed and action-oriented
- [ ] Response formats optimize for agent context efficiency
- [ ] Error messages guide the agent toward correct usage

### Implementation Quality
- [ ] Server-info name follows `{service}_mcp`
- [ ] `ListTools` advertises tools; `CallTool` dispatches on `params.name`
- [ ] Each tool has an `inputSchema` with types, constraints, and `required`
- [ ] Arguments read defensively from `Value`; shared client/format (no duplication)

### Swift Quality
- [ ] Handlers are `async`; all I/O `await`ed, nothing blocks
- [ ] Recoverable errors use `isError: true`; faults `throw MCPError`
- [ ] Logging goes to stderr, never stdout on stdio transport
- [ ] Capabilities declared to match implemented handlers

### Advanced Features (where applicable)
- [ ] Resources / prompts handlers registered where useful
- [ ] Graceful shutdown via swift-service-lifecycle
- [ ] Appropriate transport (Stdio local; Stateless/Stateful HTTP remote)

### Testing and Build
- [ ] `swift build` succeeds
- [ ] Server starts and lists tools in MCP Inspector
- [ ] Sample tool calls and error scenarios behave as expected
```
