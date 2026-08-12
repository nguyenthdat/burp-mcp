# Burp MCP

[![CI](https://github.com/nguyenthdat/burp-mcp/actions/workflows/ci.yml/badge.svg)](https://github.com/nguyenthdat/burp-mcp/actions/workflows/ci.yml)
[![npm](https://img.shields.io/npm/v/%40nguyenthdat%2Fburpmcp)](https://www.npmjs.com/package/@nguyenthdat/burpmcp)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Burp MCP connects AI agents and MCP-compatible clients to Burp Suite. It combines a Kotlin extension built on the Montoya API with a Bun-native stdio bridge and exposes more than 80 tools for authorized application-security testing.

## Capabilities

- Inspect, search, annotate, highlight, compare, and clear Proxy history.
- Send and modify HTTP requests through Burp Repeater.
- Run Intruder attacks, inline fuzzing, race-condition tests, and access-control sweeps.
- Start crawls and scans, inspect issues, and work with Burp Collaborator.
- Query the site map, target information, scope, cookies, and loaded extensions.
- Create and interact with WebSocket connections.
- Encode, decode, transform, export, and analyze payloads, requests, JWTs, and tokens.
- Import Bambdas and BChecks without executing them automatically.
- Apply HTTP handlers, proxy rules, session rules, DNS overrides, upstream proxies, and HTTP/2 settings.

Some capabilities require Burp Suite Professional or a Burp feature that is available only in specific editions.

## Architecture

```text
MCP client
    │ stdio
    ▼
@nguyenthdat/burpmcp (Bun bridge)
    │ authenticated HTTP on 127.0.0.1:9876
    ▼
Burp MCP extension (Kotlin + Montoya API)
    │
    ▼
Burp Suite
```

The bridge discovers tools from the running extension, publishes them under the `burp_` namespace, and forwards tool calls without exposing Burp directly to the MCP client process.

## Requirements

- Burp Suite with support for Montoya API extensions.
- Java 25 for building the extension.
- Gradle 9.1 or newer.
- Bun 1.3 or newer for the MCP bridge.

## Install

### 1. Load the Burp extension

Download `burp-mcp.jar` from the latest [GitHub Release](https://github.com/nguyenthdat/burp-mcp/releases), then add it in Burp Suite:

1. Open **Extensions**.
2. Select **Installed**.
3. Click **Add**.
4. Choose **Java** and select `burp-mcp.jar`.

The extension starts a local authenticated server on `127.0.0.1:9876` by default and writes its generated token to `~/.burp-mcp-token`.

### 2. Configure an MCP client

Run the published bridge with Bun:

```sh
bunx @nguyenthdat/burpmcp
```

Example MCP server configuration:

```json
{
  "command": "bunx",
  "args": ["@nguyenthdat/burpmcp"]
}
```

If a client requires separate package and binary names:

```sh
bunx --package @nguyenthdat/burpmcp burpmcp
```

## Configuration

| Setting | Default | Description |
| --- | --- | --- |
| `BURP_MCP_HOST` | `127.0.0.1` | Host used by the Bun bridge. |
| `BURP_MCP_PORT` | `9876` | Port shared by the bridge and Burp extension. |
| `BURP_MCP_TOKEN` | `~/.burp-mcp-token` | Bearer token shared by the bridge and extension. |
| `-Dburp.mcp.port=<port>` | `9876` | JVM override for the extension port. |
| `-Dburp.mcp.token=<token>` | generated token | JVM override for the extension token. |

JVM properties take precedence inside Burp. When overriding the port or token, configure matching values for the bridge.

## Security

Burp MCP is dual-use security software intended only for systems you own or are explicitly authorized to test.

- The extension listens on localhost by default.
- Every extension request requires the generated or configured bearer token.
- The bridge reads the token from the environment or `~/.burp-mcp-token`.
- The npm package contains no telemetry and does not connect to a maintainer-operated service.
- High-impact tools retain the capabilities and side effects of the underlying Burp APIs.

See [DISCLOSURE](DISCLOSURE) for the npm dual-use content declaration.

## Build and test

Install bridge dependencies and run all TypeScript checks:

```sh
bun install --frozen-lockfile
bun run check
```

Build and test the Burp extension:

```sh
gradle clean test jar
```

The extension JAR is written to `build/libs/burp-mcp.jar`.

## Releases

Published GitHub Releases build both the extension JAR and npm tarball from the same stable version. The npm package is staged through Trusted Publishing with provenance and becomes public only after maintainer inspection and 2FA approval.

## License

Burp MCP is available under the [MIT License](LICENSE).
