# PHP MCP Server Implementation Guide

## Overview

This document provides PHP-specific best practices and examples for implementing MCP servers using the official MCP PHP SDK (`mcp/sdk` on Packagist), a framework-agnostic SDK built as a collaboration between the PHP Foundation and the Symfony project. It covers server setup with `Server::builder()`, attribute-based capability discovery (`#[McpTool]`, `#[McpResource]`, `#[McpPrompt]`), transports, session management, and a complete example.

> Note: until its first major release the SDK is **experimental** (Symfony BC policy) — pin a version and re-check the API against the docs.

For complete SDK documentation, use WebFetch to load:
`https://raw.githubusercontent.com/modelcontextprotocol/php-sdk/main/README.md` (and the `docs/` directory).

---

## Quick Reference

### Installation
```bash
composer require mcp/sdk
```

### Server Initialization (stdio)
```php
use Mcp\Server;
use Mcp\Server\Transport\StdioTransport;

$server = Server::builder()
    ->setServerInfo('service_mcp', '1.0.0')
    ->setDiscovery(__DIR__, ['.'])   // auto-discover attributes
    ->build();

$server->run(new StdioTransport());
```

### Tool Registration Pattern
```php
use Mcp\Capability\Attribute\McpTool;

class UserCapabilities
{
    #[McpTool(name: 'service_search_users')]
    public function searchUsers(string $query, int $limit = 20): array
    {
        // ...
        return ['total' => $total, 'users' => $users];
    }
}
```

---

## MCP PHP SDK (`mcp/sdk`)

Capabilities can be registered three ways:
1. **Attribute-based discovery** — annotate methods with `#[McpTool]`, `#[McpResource]`, `#[McpResourceTemplate]`, `#[McpPrompt]`; call `->setDiscovery(__DIR__, ['.'])` to scan.
2. **Manual registration** — `->addTool([Calculator::class, 'add'], 'add_numbers')` without attributes.
3. **Hybrid** — combine discovery and manual registration.

Parameter types and defaults on the annotated method drive the tool's input schema, and the return type informs the output. Requires a modern PHP (see the Packagist `php-v` badge).

## Server Naming Convention

Use the server-info name `{service}_mcp` (first arg to `setServerInfo`). Use idiomatic PHP class names (`UserCapabilities`, `GithubServer`) and PSR-4 namespaces.

## Project Structure

```
service-mcp/
├── composer.json
├── bin/server.php            # builds + runs the server
└── src/
    ├── Capabilities/         # classes with #[McpTool] / #[McpResource]
    ├── Client/ServiceClient.php   # API client + auth
    └── Format/Formatter.php       # shared formatting / error mapping
```

## Tool Implementation

### Tool Naming
snake_case, action-oriented, service-prefixed. Set it explicitly with `#[McpTool(name: 'github_create_issue')]`; otherwise it derives from the method name.

### Tool Structure

```php
use Mcp\Capability\Attribute\McpTool;

class UserCapabilities
{
    public function __construct(private ServiceClient $client) {}

    /**
     * Search users by name or email. Read-only; does not modify data.
     *
     * @param string $query Search text, e.g. "john" or "team:marketing"
     * @param int    $limit Max results (1-100)
     * @return array{total:int, users:array<int, array{id:string, name:string}>}
     */
    #[McpTool(name: 'service_search_users')]
    public function searchUsers(string $query, int $limit = 20): array
    {
        if (trim($query) === '') {
            throw new \InvalidArgumentException("'query' is required.");
        }
        return $this->client->searchUsers($query, $limit);
    }
}
```

The method's typed parameters become the input schema; the docblock supplies descriptions. Use PHP types (`string`, `int`, `?string`, enums, arrays) so the SDK generates accurate schemas.

## Input Validation

Typed parameters are validated during argument binding (a non-integer `limit` is rejected before your method runs). Use PHP enums for constrained choices and nullable types for optional values. Throw `\InvalidArgumentException` (or return an error structure) for domain validation that types can't express.

## Response Format Options

Return arrays/objects (serialized to JSON) for structured data, or strings for Markdown. Offer a `$format` enum parameter when callers need both. Humanize timestamps, pair display names with IDs, and omit verbose metadata.

## Pagination Implementation

Accept `$limit`/`$offset` (or a cursor) and return pagination metadata:
```php
return [
    'total' => $total,
    'count' => count($items),
    'offset' => $offset,
    'has_more' => $total > $offset + count($items),
    'items' => $items,
];
```

## Error Handling

- **Recoverable (tool-level):** throw `\InvalidArgumentException` (or return an error array) for validation/domain errors the model can correct.
- **Unrecoverable (protocol-level):** let genuine infrastructure exceptions propagate. Map upstream HTTP status codes to actionable messages in one helper:
```php
function apiError(int $status): string {
    return match ($status) {
        404 => "Error: Resource not found. Check the ID.",
        403 => "Error: Permission denied.",
        429 => "Error: Rate limit exceeded. Retry later.",
        default => "Error: API request failed ($status).",
    };
}
```

## Shared Utilities

Inject a PSR-18 HTTP client / API wrapper into capability classes via the constructor. Centralize auth, formatting, and error mapping — never duplicate request/format logic across tools.

## Concurrency & Sessions

PHP servers are typically request-scoped. Configure session storage to persist state across HTTP requests:
- **In-memory** (default, good for stdio): `->setSession(ttl: 7200)`
- **File-based** (single-server HTTP): `->setSession(new FileSessionStore(__DIR__.'/sessions'))`
- **PSR-16 cache** (scaled HTTP, e.g. Redis): `->setSession(new Psr16SessionStore(cache: $cache, prefix: 'mcp-', ttl: 3600))`

## Package Configuration

`composer.json` with PSR-4 autoloading and a `bin/server.php` entrypoint. For stdio, write logs to **stderr** (`fwrite(STDERR, ...)` or a PSR-3 logger targeting stderr) — stdout carries JSON-RPC on the stdio transport.

## Complete Example

`bin/server.php`:
```php
#!/usr/bin/env php
<?php
require __DIR__ . '/../vendor/autoload.php';

use Mcp\Server;
use Mcp\Server\Transport\StdioTransport;

$server = Server::builder()
    ->setServerInfo('service_mcp', '1.0.0')
    ->setDiscovery(__DIR__ . '/../src', ['.'])   // scan src/ for #[McpTool] etc.
    ->build();

$server->run(new StdioTransport());
```

`src/Capabilities/UserCapabilities.php`:
```php
<?php
namespace App\Capabilities;

use Mcp\Capability\Attribute\McpTool;
use App\Client\ServiceClient;

class UserCapabilities
{
    public function __construct(private ServiceClient $client) {}

    /**
     * Search users by name or email. Read-only.
     * @param string $query Search text
     * @return array{total:int, users:array}
     */
    #[McpTool(name: 'service_search_users')]
    public function searchUsers(string $query, int $limit = 20): array
    {
        if (trim($query) === '') {
            throw new \InvalidArgumentException("'query' is required.");
        }
        return $this->client->searchUsers($query, $limit);
    }
}
```

For an **HTTP** server, run a `StreamableHttpTransport` built from your PSR-7 request/response/stream factories instead of `StdioTransport`:
```php
use Mcp\Server\Transport\StreamableHttpTransport;
$response = $server->run(new StreamableHttpTransport($request, $responseFactory, $streamFactory));
```

---

## Advanced MCP Features

### Resources & Prompts
```php
use Mcp\Capability\Attribute\McpResource;

#[McpResource(uri: 'config://service/settings')]
public function getSettings(): array { return ['precision' => 2]; }
```
Use `#[McpResourceTemplate]` for parameterized URIs and `#[McpPrompt]` for prompt templates.

### Server-Initiated Communication
The SDK supports elicitations, sampling, logging, and progress notifications; wire handlers via the builder (see `docs/server-client-communication.md`).

### Transports
- **STDIO:** `new StdioTransport()` — CLI / local processes.
- **Streamable HTTP:** `new StreamableHttpTransport($request, $responseFactory, $streamFactory)` — web/distributed; mount in any PSR-7 framework (Symfony, Laravel, etc.).

## Building and Running

```bash
composer install
php bin/server.php

# Test with MCP Inspector
npx @modelcontextprotocol/inspector php bin/server.php
```

## Quality Checklist

### Strategic Design
- [ ] Tools model complete workflows, not thin endpoint wrappers
- [ ] Tool names are service-prefixed and action-oriented
- [ ] Response formats optimize for agent context efficiency
- [ ] Error messages guide the agent toward correct usage

### Implementation Quality
- [ ] Server-info name follows `{service}_mcp`
- [ ] Tools are typed methods with `#[McpTool]` and docblock descriptions
- [ ] API client injected via constructor; shared formatting/error mapping (no duplication)
- [ ] Session store chosen to match deployment (memory/file/PSR-16)

### PHP Quality
- [ ] Parameters use precise PHP types (enums, nullables, arrays)
- [ ] Recoverable errors throw `\InvalidArgumentException`; faults propagate
- [ ] Logging goes to stderr, never stdout on stdio transport
- [ ] PSR-4 autoloading configured

### Advanced Features (where applicable)
- [ ] Resources / resource templates / prompts registered where useful
- [ ] Appropriate transport (STDIO local, Streamable HTTP remote)
- [ ] Session storage configured for HTTP deployments

### Testing and Build
- [ ] `composer install` resolves dependencies
- [ ] Server starts and lists tools in MCP Inspector
- [ ] Sample tool calls and error scenarios behave as expected
```
