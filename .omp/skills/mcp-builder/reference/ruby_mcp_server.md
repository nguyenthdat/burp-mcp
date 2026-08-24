# Ruby MCP Server Implementation Guide

## Overview

This document provides Ruby-specific best practices and examples for implementing MCP servers using the official MCP Ruby SDK (the `mcp` gem). It covers `MCP::Server` setup, defining tools as `MCP::Tool` subclasses (or via `MCP::Tool.define` / `server.define_tool`), input/output schemas, transports (stdio and Streamable HTTP as a Rack app), error reporting, and a complete example.

For complete SDK documentation, use WebFetch to load:
`https://raw.githubusercontent.com/modelcontextprotocol/ruby-sdk/main/README.md`

---

## Quick Reference

### Installation
```ruby
# Gemfile
gem 'mcp'
```

### Server Initialization
```ruby
require "mcp"

server = MCP::Server.new(
  name: "service_mcp",
  version: "1.0.0",
  tools: [SearchUsersTool],
)
```

### Tool Registration Pattern
```ruby
class SearchUsersTool < MCP::Tool
  description "Search users by name or email."
  input_schema(
    properties: { query: { type: "string" } },
    required: ["query"],
  )

  def self.call(query:, server_context:)
    MCP::Tool::Response.new([{ type: "text", text: result }])
  end
end
```

---

## MCP Ruby SDK (the `mcp` gem)

`MCP::Server` handles JSON-RPC 2.0, capability negotiation, and dispatch. Tools can be created three ways:
1. **Class definition** — subclass `MCP::Tool` (recommended for real tools).
2. **`MCP::Tool.define(name:, description:) { |args, server_context:| ... }`** — inline block.
3. **`server.define_tool(name:, description:) { |args, server_context:| ... }`** — register on an existing server.

Tool arguments arrive as a `Hash` with **symbol keys** at every level (transports parse JSON with `symbolize_names: true`). Read nested values with symbols (`payload[:subject]`).

## Server Naming Convention

Use the server-info name `{service}_mcp` (the `name:` argument). Use idiomatic Ruby class names for tools (`SearchUsersTool`, `GithubCreateIssueTool`).

## Project Structure

```
service-mcp/
├── Gemfile
├── bin/server.rb          # builds + runs the server
└── lib/
    ├── tools/             # MCP::Tool subclasses
    ├── service_client.rb  # API client + auth
    └── formatter.rb       # shared formatting / error mapping
```

## Tool Implementation

### Tool Naming
Tool name defaults to the underscored class name; override with `tool_name "service_search_users"`. Use snake_case, action-oriented, service-prefixed names.

### Tool Structure

```ruby
class SearchUsersTool < MCP::Tool
  tool_name "service_search_users"
  title "Search Users"
  description "Search users by name or email. Read-only; does not modify data."

  input_schema(
    properties: {
      query: { type: "string", description: "Search text" },
      limit: { type: "integer", minimum: 1, maximum: 100 },
    },
    required: ["query"],
  )

  annotations(
    read_only_hint: true,
    destructive_hint: false,
    idempotent_hint: true,
    open_world_hint: true,
  )

  def self.call(query:, limit: 20, server_context:)
    if query.to_s.strip.empty?
      return MCP::Tool::Response.new(
        [{ type: "text", text: "Error: 'query' is required." }], error: true)
    end
    result = ServiceClient.search_users(query, limit) # shared client
    MCP::Tool::Response.new([{ type: "text", text: result }])
  end
end
```

The `server_context:` keyword carries per-request data (auth state, user id) that you pass into `MCP::Server.new(server_context: {...})`.

## Input Validation

Declare `input_schema(properties:, required:)` with JSON-Schema constraints (`type`, `minimum`, `enum`). The SDK validates arguments against it. Add domain checks inside `self.call` and return an error response for anything the schema can't express.

## Response Format Options

Build `MCP::Tool::Response.new([{ type: "text", text: ... }])`. Return Markdown by default; add a `format` argument and return JSON (`data.to_json`) when callers need structured output. Humanize timestamps, pair display names with IDs, and omit verbose metadata.

## Pagination Implementation

Accept `limit`/`offset` (or a cursor) and include pagination metadata in the JSON body:
```ruby
{ total:, count: items.size, offset:, has_more: total > offset + items.size, items: }.to_json
```

## Error Handling

- **Recoverable (tool-level):** `MCP::Tool::Response.new([...], error: true)` for validation/domain errors the model can fix.
- **Unrecoverable:** raise; configure a reporter to capture it. For tool calls the SDK returns a generic `{ error: "Internal error occurred", isError: true }` to the client and reports the exception:
```ruby
MCP.configure do |config|
  config.exception_reporter = ->(exception, server_context) {
    # e.g. Bugsnag.notify(exception)
  }
  config.around_request = ->(data, &req) {
    logger.info("Start: #{data[:method]}"); req.call
  }
end
```

## Shared Utilities

Put the API client (auth + connection reuse), formatting, and error mapping in `lib/`. Never duplicate request/format logic across tools.

## Concurrency

`StreamableHTTPTransport` stores session/SSE state in memory and must run in a **single process** (e.g. Puma with `workers 0`), or use `stateless: true`, or configure sticky sessions behind a load balancer. Keep tool state out of globals.

## Package Configuration

A `Gemfile` with `gem 'mcp'` and a `bin/server.rb` entrypoint. For stdio, write logs to **stderr** (`$stderr.puts` / a `Logger.new($stderr)`) — stdout carries JSON-RPC on the stdio transport.

## Complete Example

`bin/server.rb` (stdio):
```ruby
#!/usr/bin/env ruby
require "mcp"
require_relative "../lib/tools/search_users_tool"

server = MCP::Server.new(
  name: "service_mcp",
  version: "1.0.0",
  tools: [SearchUsersTool],
)

transport = MCP::Server::Transports::StdioTransport.new(server)
transport.open
```

`lib/tools/search_users_tool.rb`:
```ruby
class SearchUsersTool < MCP::Tool
  tool_name "service_search_users"
  description "Search users by name or email. Read-only."
  input_schema(
    properties: { query: { type: "string" }, limit: { type: "integer", minimum: 1, maximum: 100 } },
    required: ["query"],
  )
  annotations(read_only_hint: true, destructive_hint: false, idempotent_hint: true)

  def self.call(query:, limit: 20, server_context:)
    if query.to_s.strip.empty?
      return MCP::Tool::Response.new([{ type: "text", text: "Error: 'query' is required." }], error: true)
    end
    result = ServiceClient.search_users(query, limit)
    MCP::Tool::Response.new([{ type: "text", text: result }])
  end
end
```

**Streamable HTTP** (Rails mount) — the transport is a Rack app:
```ruby
# config/routes.rb
server = MCP::Server.new(name: "service_mcp", version: "1.0.0", tools: [SearchUsersTool])
transport = MCP::Server::Transports::StreamableHTTPTransport.new(server)
Rails.application.routes.draw { mount transport => "/mcp" }
```

---

## Advanced MCP Features

### Structured Output
Declare `output_schema(...)` and pass `structured_content:` to `MCP::Tool::Response.new([...], structured_content: data)`. Enable server-side validation with `MCP::Configuration.new(validate_tool_call_results: true)`.

### Prompts & Resources
Subclass `MCP::Prompt` (or `MCP::Prompt.define`), and register `MCP::Resource` / `MCP::ResourceTemplate` instances plus a `server.resources_read_handler { |params| ... }`.

### Sampling, Roots, Progress
Inside a tool, use `server_context.create_sampling_message(messages:, max_tokens:)`, `server_context.list_roots`, and progress/logging notifications (session-scoped on Streamable HTTP).

### Transports
- **STDIO:** `MCP::Server::Transports::StdioTransport.new(server)` — local/CLI.
- **Streamable HTTP:** `MCP::Server::Transports::StreamableHTTPTransport.new(server)` — a Rack app; `mount` in Rails or use per-request with `stateless: true`.

## Building and Running

```bash
bundle install
ruby bin/server.rb

# Test with MCP Inspector
npx @modelcontextprotocol/inspector ruby bin/server.rb
```

## Quality Checklist

### Strategic Design
- [ ] Tools model complete workflows, not thin endpoint wrappers
- [ ] Tool names are service-prefixed and action-oriented
- [ ] Response formats optimize for agent context efficiency
- [ ] Error messages guide the agent toward correct usage

### Implementation Quality
- [ ] Server-info name follows `{service}_mcp`
- [ ] Each tool declares `input_schema` with constraints and `annotations`
- [ ] Arguments read with **symbol** keys
- [ ] Shared API client, formatting, and error mapping (no duplication)

### Ruby Quality
- [ ] Recoverable errors use `Response.new([...], error: true)`; faults raise + reporter
- [ ] `exception_reporter` / `around_request` configured for observability
- [ ] Logging goes to stderr, never stdout on stdio transport
- [ ] Streamable HTTP runs single-process or `stateless: true`

### Advanced Features (where applicable)
- [ ] `output_schema` + `structured_content` for structured results
- [ ] Prompts / resources / resource templates registered where useful
- [ ] Appropriate transport (STDIO local, Streamable HTTP remote)

### Testing and Build
- [ ] `bundle install` resolves dependencies
- [ ] Server starts and lists tools in MCP Inspector
- [ ] Sample tool calls and error scenarios behave as expected
```
