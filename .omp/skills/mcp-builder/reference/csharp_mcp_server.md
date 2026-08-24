# C# MCP Server Implementation Guide

## Overview

This document provides C#/.NET-specific best practices and examples for implementing MCP servers using the official MCP C# SDK (`ModelContextProtocol` on NuGet). The SDK integrates with `Microsoft.Extensions.Hosting` (generic host + dependency injection) and discovers tools/prompts/resources from attributes. It covers server setup over stdio and ASP.NET Core HTTP, attribute-based tool registration, DI, error handling, and a complete example.

For complete SDK documentation, use WebFetch to load:
- SDK README: `https://raw.githubusercontent.com/modelcontextprotocol/csharp-sdk/main/README.md`
- Getting started: `https://csharp.sdk.modelcontextprotocol.io/concepts/getting-started.html`

---

## Quick Reference

### Packages
- **ModelContextProtocol** — main package: hosting, DI, attribute discovery, stdio. Start here for most servers.
- **ModelContextProtocol.Core** — minimal dependencies; client or low-level server APIs only.
- **ModelContextProtocol.AspNetCore** — HTTP-based servers hosted in ASP.NET Core.

### Server Initialization (stdio)
```csharp
var builder = Host.CreateApplicationBuilder(args);
builder.Logging.AddConsole(o => o.LogToStandardErrorThreshold = LogLevel.Trace); // logs -> stderr
builder.Services
    .AddMcpServer()
    .WithStdioServerTransport()
    .WithToolsFromAssembly();
await builder.Build().RunAsync();
```

### Tool Registration Pattern
```csharp
[McpServerToolType]
public static class ServiceTools
{
    [McpServerTool, Description("Search users by name or email.")]
    public static async Task<string> SearchUsers(
        [Description("Search text")] string query) => await ServiceClient.SearchUsersAsync(query);
}
```

---

## MCP C# SDK (`ModelContextProtocol`)

Tools, prompts, and resources are plain methods annotated with attributes; `WithToolsFromAssembly()` scans the assembly for `[McpServerToolType]` classes and registers every `[McpServerTool]` method. Method parameters are bound from the JSON arguments, and `[Description]` (from `System.ComponentModel`) becomes the tool/parameter description. Handlers may be sync or `async Task<T>`, and may inject DI services and a `CancellationToken`.

Install:
```bash
dotnet add package ModelContextProtocol
dotnet add package Microsoft.Extensions.Hosting
# HTTP servers also:
dotnet add package ModelContextProtocol.AspNetCore
```

## Server Naming Convention

Use the server-info name `{service}_mcp` (configurable via `AddMcpServer` options / assembly metadata). Use idiomatic C# type names for classes (`ServiceTools`, `GithubMcpServer`).

## Project Structure

```
ServiceMcp/
├── ServiceMcp.csproj
├── Program.cs            # host builder + transport
├── Tools/                # [McpServerToolType] classes
│   └── UserTools.cs
├── Client/ServiceClient.cs   # typed HttpClient + auth
└── Formatting/Format.cs      # shared response/error formatting
```

## Tool Implementation

### Tool Naming
Tool names derive from the method name; use service-prefixed, action-oriented names (`GithubCreateIssue`) or set an explicit name via `[McpServerTool(Name = "github_create_issue")]`.

### Tool Structure

```csharp
using System.ComponentModel;
using ModelContextProtocol.Server;

[McpServerToolType]
public class UserTools
{
    private readonly ServiceClient _client;
    public UserTools(ServiceClient client) => _client = client; // injected via DI

    [McpServerTool(Name = "service_search_users"),
     Description("Search users by name or email. Read-only; does not modify data.")]
    public async Task<string> SearchUsers(
        [Description("Search text, e.g. 'john' or 'team:marketing'")] string query,
        [Description("Max results, 1-100")] int limit = 20,
        CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(query))
            return "Error: 'query' is required.";
        return await _client.SearchUsersAsync(query, limit, cancellationToken);
    }
}
```

For rich input validation and structured output, accept a parameter object annotated with `System.ComponentModel.DataAnnotations` and/or return a typed object (the SDK serializes it and can emit an output schema).

## Input Validation

Parameter types are enforced during argument binding (a non-numeric `limit` is rejected before your method runs). Add `[Description]` for guidance and `DataAnnotations` (`[Range]`, `[Required]`, `[StringLength]`) on parameter objects. Return a clear error string for domain validation the binder can't express.

## Response Format Options

Return Markdown strings by default; add a `format` enum parameter and return JSON (via `System.Text.Json`) when callers need structured data. Humanize timestamps, pair display names with IDs, and omit verbose metadata. Returning a typed object (record/class) lets the SDK provide structured content automatically.

## Pagination Implementation

Accept `limit`/`offset` (or a cursor) parameters and include pagination metadata in the response so the agent can page:
```csharp
var body = new { total, count = items.Count, offset,
                 hasMore = total > offset + items.Count, items };
return JsonSerializer.Serialize(body);
```

## Error Handling

- **Recoverable (tool-level):** return an error message string (or a result with `IsError`) for validation/domain errors the model can fix.
- **Unrecoverable (protocol-level):** throw an exception for genuine infrastructure faults; the SDK maps it to a JSON-RPC error. Use a typed `HttpClient` and translate upstream status codes into actionable messages:
```csharp
static string ApiError(HttpStatusCode s) => s switch {
    HttpStatusCode.NotFound   => "Error: Resource not found. Check the ID.",
    HttpStatusCode.Forbidden  => "Error: Permission denied.",
    HttpStatusCode.TooManyRequests => "Error: Rate limit exceeded. Retry later.",
    _ => $"Error: API request failed ({(int)s})."
};
```

## Shared Utilities

Register a typed `HttpClient` (`builder.Services.AddHttpClient<ServiceClient>()`) and inject it into tool classes. Centralize auth, formatting, and error mapping — never duplicate HTTP/formatting logic across tools.

## Async/Await Best Practices

Prefer `async Task<T>` handlers and `await` all I/O; accept and honor the injected `CancellationToken`. Never block with `.Result`/`.Wait()`.

## Package Configuration

`Program.cs` uses the generic host; publish a self-contained executable (`dotnet publish -c Release`) launched over stdio. **Critical:** route logs to stderr (`LogToStandardErrorThreshold = LogLevel.Trace`) — stdout carries JSON-RPC on the stdio transport.

## Complete Example

**stdio server** (`Program.cs`):
```csharp
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;
using ModelContextProtocol.Server;
using System.ComponentModel;

var builder = Host.CreateApplicationBuilder(args);
builder.Logging.AddConsole(o => o.LogToStandardErrorThreshold = LogLevel.Trace);

builder.Services.AddHttpClient<ServiceClient>();
builder.Services
    .AddMcpServer()
    .WithStdioServerTransport()
    .WithToolsFromAssembly();

await builder.Build().RunAsync();

[McpServerToolType]
public class UserTools
{
    private readonly ServiceClient _client;
    public UserTools(ServiceClient client) => _client = client;

    [McpServerTool(Name = "service_search_users"),
     Description("Search users by name or email. Read-only.")]
    public async Task<string> SearchUsers(
        [Description("Search text")] string query,
        CancellationToken ct = default)
    {
        if (string.IsNullOrWhiteSpace(query)) return "Error: 'query' is required.";
        return await _client.SearchUsersAsync(query, ct);
    }
}
```

**HTTP server** (ASP.NET Core):
```csharp
using ModelContextProtocol.Server;

var builder = WebApplication.CreateBuilder(args);
builder.Services.AddMcpServer()
    .WithHttpTransport(o => o.Stateless = true) // stateless unless you need sampling/elicitation
    .WithToolsFromAssembly();

var app = builder.Build();
app.MapMcp();
app.Run("http://localhost:3001");
```

---

## Advanced MCP Features

### Prompts & Resources
Mirror tools: `[McpServerPromptType]` + `[McpServerPrompt]`, and `[McpServerResourceType]` + `[McpServerResource]`, discovered via `WithPromptsFromAssembly()` / `WithResourcesFromAssembly()`.

### Structured Output
Return a typed record/class instead of a string; the SDK serializes it and can generate an output schema for clients.

### Transports
- **stdio:** `WithStdioServerTransport()` — local subprocess servers.
- **Streamable HTTP:** `WithHttpTransport()` + `app.MapMcp()` in ASP.NET Core. Prefer `Stateless = true` unless you need server→client requests (sampling/elicitation).

### HTTP Security
For local HTTP servers, restrict Kestrel `AllowedHosts` to loopback names (guards against DNS rebinding); do not use `"*"`. Enable CORS **only** if you intentionally allow browser cross-origin access, with the most restrictive policy possible.

## Building and Running

```bash
dotnet build
dotnet run                       # stdio (or the HTTP host)

# Test with MCP Inspector
npx @modelcontextprotocol/inspector dotnet run
```

## Quality Checklist

### Strategic Design
- [ ] Tools model complete workflows, not thin endpoint wrappers
- [ ] Tool names are service-prefixed and action-oriented
- [ ] Response formats optimize for agent context efficiency
- [ ] Error messages guide the agent toward correct usage

### Implementation Quality
- [ ] Server-info name follows `{service}_mcp`
- [ ] Tools are `[McpServerTool]` methods in `[McpServerToolType]` classes
- [ ] `[Description]` on every tool and parameter
- [ ] Typed `HttpClient` injected via DI; shared formatting/error mapping (no duplication)

### C# Quality
- [ ] Handlers are `async Task<T>` and honor `CancellationToken`
- [ ] Logging routed to stderr, never stdout on stdio transport
- [ ] Recoverable errors return a message; faults throw exceptions
- [ ] No blocking (`.Result`/`.Wait()`)

### Advanced Features (where applicable)
- [ ] Prompts/resources registered where useful
- [ ] Typed return objects for structured output
- [ ] Appropriate transport (stdio local, Streamable HTTP remote)
- [ ] HTTP servers: `AllowedHosts` restricted; CORS only if intended

### Testing and Build
- [ ] `dotnet build` succeeds
- [ ] Server starts and lists tools in MCP Inspector
- [ ] Sample tool calls and error scenarios behave as expected
```
