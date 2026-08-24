# Go MCP Server Implementation Guide

## Overview

This document provides Go-specific best practices and examples for implementing MCP servers using the official MCP Go SDK (`github.com/modelcontextprotocol/go-sdk`). The SDK derives JSON input/output schemas from Go structs (via `json` and `jsonschema` struct tags) and uses generics for type-safe tool handlers. It covers server setup, `mcp.AddTool`, transports, error handling, and a complete example.

For complete SDK documentation, use WebFetch to load:
`https://raw.githubusercontent.com/modelcontextprotocol/go-sdk/main/README.md` (and see the `docs/` directory / `https://pkg.go.dev/github.com/modelcontextprotocol/go-sdk/mcp`).

---

## Quick Reference

### Key Imports
```go
import (
    "context"
    "github.com/modelcontextprotocol/go-sdk/mcp"
)
```

### Server Definition
```go
server := mcp.NewServer(&mcp.Implementation{Name: "service_mcp", Version: "v1.0.0"}, nil)
```

### Tool Registration Pattern
```go
type SearchInput struct {
    Query string `json:"query" jsonschema:"the search text"`
}
type SearchOutput struct {
    Results []string `json:"results" jsonschema:"matching user names"`
}

func SearchUsers(ctx context.Context, req *mcp.CallToolRequest, in SearchInput) (
    *mcp.CallToolResult, SearchOutput, error) {
    // ...
    return nil, SearchOutput{Results: names}, nil
}

mcp.AddTool(server, &mcp.Tool{Name: "service_search_users", Description: "Search users."}, SearchUsers)
```

---

## MCP Go SDK (`github.com/modelcontextprotocol/go-sdk/mcp`)

Key packages:
- `.../mcp` — primary APIs for building servers and clients.
- `.../jsonrpc` — for custom transports.
- `.../auth`, `.../oauthex` — OAuth primitives/extensions.

`mcp.AddTool` is generic: it infers the tool's input and output JSON schemas from the handler's input and output struct types, so you rarely hand-write schemas. Install:
```bash
go get github.com/modelcontextprotocol/go-sdk@latest
```
Target only supported Go versions (see the module's release policy).

## Server Naming Convention

Use the server-info name `{service}_mcp` (lowercase). Module path is idiomatic Go (`github.com/you/service-mcp`).

## Project Structure

```
service-mcp/
├── go.mod
├── main.go            # NewServer + AddTool + Run
└── internal/
    ├── tools/         # tool handlers + input/output structs
    ├── client/        # API client + auth
    └── format/        # shared formatting / error mapping
```

## Tool Implementation

### Tool Naming
snake_case, action-oriented, service-prefixed: `github_create_issue`.

### Tool Structure

Define input and output structs with `json` (field name) and `jsonschema` (description/constraints) tags. The handler signature is `func(ctx, *mcp.CallToolRequest, In) (*mcp.CallToolResult, Out, error)`.

```go
type SearchInput struct {
    Query string `json:"query" jsonschema:"search text, e.g. 'john'"`
    Limit int    `json:"limit,omitempty" jsonschema:"max results 1-100"`
}

type SearchOutput struct {
    Total int      `json:"total"`
    Users []string `json:"users"`
}

func SearchUsers(ctx context.Context, req *mcp.CallToolRequest, in SearchInput) (
    *mcp.CallToolResult, SearchOutput, error) {

    if in.Query == "" {
        // Tool-level error the model can correct:
        return &mcp.CallToolResult{
            IsError: true,
            Content: []mcp.Content{&mcp.TextContent{Text: "Error: 'query' is required."}},
        }, SearchOutput{}, nil
    }

    users, total, err := client.SearchUsers(ctx, in.Query, in.Limit)
    if err != nil {
        return nil, SearchOutput{}, err // protocol-level (JSON-RPC) error
    }
    return nil, SearchOutput{Total: total, Users: users}, nil
}
```

Returning a `nil` result with a populated output struct makes the SDK emit structured content automatically. Return a non-nil `*mcp.CallToolResult` when you need custom text content or `IsError`.

## Input Validation

Encode constraints in `jsonschema` tags (`jsonschema:"minimum=1,maximum=100"`, required-ness via non-pointer fields, enums via `enum=...`). The SDK validates arguments against the generated schema before your handler runs. Add domain checks (like empty-after-trim) inside the handler and return a tool-level error.

## Response Format Options

Prefer returning a typed output struct for structured content. When you also need human-readable text, return a `*mcp.CallToolResult` with a `&mcp.TextContent{Text: markdown}` block. Humanize timestamps, pair display names with IDs, and omit verbose metadata.

## Pagination Implementation

Add `Limit`/`Offset` (or a cursor) to the input struct and pagination metadata to the output struct:
```go
type ListOutput struct {
    Total   int      `json:"total"`
    Count   int      `json:"count"`
    Offset  int      `json:"offset"`
    HasMore bool     `json:"has_more"`
    Items   []string `json:"items"`
}
```

## Error Handling

Two tiers, decided by the return values:
- **Tool-level (recoverable):** return `&mcp.CallToolResult{IsError: true, Content: [...]}` with `nil` error — the model reads the message and retries.
- **Protocol-level (unrecoverable):** return a non-nil `error` — mapped to a JSON-RPC error. Use for infrastructure faults, not input validation.

Wrap upstream errors with actionable context: `fmt.Errorf("search users: %w", err)`.

## Shared Utilities

Put the API client (with auth + `*http.Client` reuse), formatting, and error mapping in `internal/`. Reuse one `*http.Client`; never construct a client per call and never duplicate formatting logic.

## Concurrency

Handlers may run concurrently — keep them stateless or guard shared state with a mutex. Always propagate the incoming `context.Context` to upstream calls for cancellation and deadlines.

## Package Configuration

`go build -o service-mcp .` produces a single static binary launched over stdio. Send logs to **stderr** (`log` package writes to stderr by default) — stdout carries JSON-RPC on the stdio transport.

## Complete Example

```go
package main

import (
    "context"
    "log"

    "github.com/modelcontextprotocol/go-sdk/mcp"
)

type SearchInput struct {
    Query string `json:"query" jsonschema:"search text"`
    Limit int    `json:"limit,omitempty" jsonschema:"max results 1-100"`
}

type SearchOutput struct {
    Total int      `json:"total"`
    Users []string `json:"users"`
}

func searchUsers(ctx context.Context, req *mcp.CallToolRequest, in SearchInput) (
    *mcp.CallToolResult, SearchOutput, error) {
    if in.Query == "" {
        return &mcp.CallToolResult{
            IsError: true,
            Content: []mcp.Content{&mcp.TextContent{Text: "Error: 'query' is required."}},
        }, SearchOutput{}, nil
    }
    // users, total, err := client.SearchUsers(ctx, in.Query, in.Limit) ...
    return nil, SearchOutput{Total: 0, Users: []string{}}, nil
}

func main() {
    server := mcp.NewServer(&mcp.Implementation{Name: "service_mcp", Version: "v1.0.0"}, nil)

    mcp.AddTool(server, &mcp.Tool{
        Name:        "service_search_users",
        Description: "Search users by name or email. Read-only.",
    }, searchUsers)

    // Run over stdin/stdout until the client disconnects.
    if err := server.Run(context.Background(), &mcp.StdioTransport{}); err != nil {
        log.Fatal(err)
    }
}
```

---

## Advanced MCP Features

### Resources & Prompts
Register with `server.AddResource(&mcp.Resource{...}, handler)` and `server.AddPrompt(&mcp.Prompt{...}, handler)` (see the `mcp` package docs for exact handler signatures).

### Transports
- **STDIO:** `&mcp.StdioTransport{}` — local subprocess servers.
- **Streamable HTTP (remote):** wrap the server in the SDK's streamable HTTP handler (e.g. `mcp.NewStreamableHTTPHandler(func(*http.Request) *mcp.Server { return server }, nil)`) and serve it with `net/http`. Verify the exact constructor/options against the `mcp` package docs for your SDK version.

### Version Compatibility
The SDK supports multiple MCP spec versions per release; check the compatibility table in the README and pin a version in `go.mod`.

## Building and Running

```bash
go mod tidy
go build -o service-mcp .
./service-mcp

# Test with MCP Inspector
npx @modelcontextprotocol/inspector ./service-mcp
```

## Quality Checklist

### Strategic Design
- [ ] Tools model complete workflows, not thin endpoint wrappers
- [ ] Tool names are service-prefixed and action-oriented
- [ ] Response formats optimize for agent context efficiency
- [ ] Error messages guide the agent toward correct usage

### Implementation Quality
- [ ] Server-info name follows `{service}_mcp`
- [ ] Input/output structs carry `json` + `jsonschema` tags
- [ ] Shared `*http.Client`, formatting, and error mapping (no duplication)
- [ ] `context.Context` propagated to all upstream calls

### Go Quality
- [ ] Tool-level errors use `CallToolResult{IsError:true}`; faults return `error`
- [ ] Errors wrapped with `%w` and actionable context
- [ ] Logging goes to stderr, never stdout on stdio transport
- [ ] Handlers are concurrency-safe

### Advanced Features (where applicable)
- [ ] Resources / prompts registered where useful
- [ ] Structured output via typed output structs
- [ ] Appropriate transport (STDIO local, Streamable HTTP remote)

### Testing and Build
- [ ] `go build` succeeds and `go vet` is clean
- [ ] Server starts and lists tools in MCP Inspector
- [ ] Sample tool calls and error scenarios behave as expected
```
