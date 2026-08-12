# Burp MCP

Burp MCP exposes Burp Suite operations through a local authenticated HTTP server and a standard MCP stdio bridge.

## Requirements

- Burp Suite with the extension JAR loaded
- Java 25 and Gradle 9+
- Bun 1.3+

## Build

Build the Burp extension:

```sh
gradle jar
```

The JAR is written to `build/libs/burp-mcp.jar`.

Run the published bridge directly with Bun:

```sh
bunx @nguyenthdat/burpmcp
```

For MCP clients, use:

```json
{
  "command": "bunx",
  "args": ["@nguyenthdat/burpmcp"]
}
```

The package exposes the `burpmcp` binary. If an environment requires the package and binary names separately, use:

```sh
bunx --package @nguyenthdat/burpmcp burpmcp
```

There is no generated or checked-in root JavaScript bridge. `bridge/src/main.ts` is published and executed natively by Bun.

## Configuration

The bridge supports:

- `BURP_MCP_HOST`, default `127.0.0.1`
- `BURP_MCP_PORT`, default `9876`
- `BURP_MCP_TOKEN`, otherwise read from `~/.burp-mcp-token`

The Burp extension supports the same port and token environment variables. JVM properties `-Dburp.mcp.port` and `-Dburp.mcp.token` take precedence inside Burp.

## Tool providers

The stdio layer composes tools through `ToolProvider`. Burp is the first provider and publishes names such as `burp_proxy_history`. A future CyberChef provider can be added in `bridge/src/main.ts` with namespace `cyberchef`; the MCP protocol loop and the Kotlin Burp server do not need to change.

Providers own their local tool definitions and execution. `ToolDirectory` adds namespaces, preserves provider order, rejects duplicate namespaces/public names, and routes calls to the owning provider. Unprefixed calls remain a compatibility alias for Burp.

## Verification

Run all bridge checks:

```sh
bun run check
```

Run the Kotlin tests and build:

```sh
gradle test jar
```
