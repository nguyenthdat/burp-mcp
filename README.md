# Burp MCP

[![CI](https://github.com/nguyenthdat/burp-mcp/actions/workflows/ci.yml/badge.svg)](https://github.com/nguyenthdat/burp-mcp/actions/workflows/ci.yml)
[![npm](https://img.shields.io/npm/v/%40nguyenthdat%2Fburpmcp)](https://www.npmjs.com/package/@nguyenthdat/burpmcp)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Burp MCP connects AI agents and MCP-compatible clients to Burp Suite and CyberChef. It combines a Kotlin extension built on the Montoya API with a Bun-native stdio bridge, exposes more than 80 Burp tools for authorized application-security testing, and adds local CyberChef transformations.

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
- Run CyberChef recipes, individual operations, batch transformations, and Magic detection.
- Transform arbitrary text, JSON, binary data, and raw HTTP request or response bodies and headers.

Some capabilities require Burp Suite Professional or a Burp feature that is available only in specific editions.

## v3 migration status

`PLAN.md` defines a staged v3 migration. Phase 0 now includes a
loopback-only Kotlin gRPC interoperability spike and a Rust `tonic` client.
The v2 Bun/HTTP/CyberChef path remains the production default until the
migration's parity and cutover gates pass.

The extension now starts in dual mode by default: the v2 HTTP compatibility
server uses `127.0.0.1:9876`, while the typed gRPC spike uses
`127.0.0.1:9877`. Override the transport or gRPC port before starting Burp:

```sh
# gRPC only
BURP_MCP_TRANSPORT=grpc

# Custom loopback gRPC port
BURP_MCP_GRPC_PORT=10077
# JVM alternatives: -Dburp.mcp.transport=grpc -Dburp.mcp.grpc.port=10077
```

The spike deliberately has no bearer-token authentication and listens only on
`127.0.0.1`; any local process that can reach the port can call it. Run the
cross-language fixture with `scripts/run-grpc-interop.sh`. This fixture is
supporting evidence only: Phase 0 is not accepted until the same lifecycle is
validated in Burp on JDK 25 and on the intended release platforms.

## Architecture

```text
MCP client
    │ stdio
    ▼
@nguyenthdat/burpmcp (Bun bridge)
    ├── authenticated HTTP on 127.0.0.1:9876 ──► Burp MCP extension ──► Burp Suite
    └── local Bun worker ──► CyberChef operations
```

The bridge discovers tools from the running extension, publishes them under the `burp_` namespace, and forwards tool calls without exposing Burp directly to the MCP client process. It also publishes local CyberChef tools under `cyberchef_`; these remain usable when Burp is stopped.

## Requirements

- Burp Suite with support for Montoya API extensions.
- Java 25 for building the extension.
- Gradle 9.1 or newer.
- Bun 1.3 or newer for the MCP bridge.

CyberChef exposes `cyberchef_bake`, operation discovery, batch and Magic workflows, HTTP request/response transforms, and one generated MCP tool for every supported non-flow-control operation. The bridge runs the official `cyberchef@11.3.0` public API in a Bun worker; a local runtime adapter handles Bun-incompatible upstream modules, while `Jq` and `Disassemble ARM` remain discoverable but unsupported. Binary inputs and outputs use tagged base64 values so arbitrary bytes remain lossless.

Browser-only, flow-control, and network-capable CyberChef operations are intentionally not executable in the bridge. `cyberchef_search_operations` still reports them with `supported: false`. Requests are limited to 10 MiB and 30 seconds. Body transforms update only clearly unambiguous `Content-Length` framing; `Transfer-Encoding`, duplicate or comma-separated lengths, folded headers, and malformed framing are preserved verbatim for request-smuggling tests.

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
| `BURP_MCP_TRANSPORT` | `dual` | `http`, `grpc`, or `dual`; controls which extension listeners start. |
| `-Dburp.mcp.transport=<mode>` | `dual` | JVM override for the extension transport mode. |
| `BURP_MCP_GRPC_PORT` | `9877` | Phase 0 loopback gRPC port; must differ from HTTP in dual mode. |
| `-Dburp.mcp.grpc.port=<port>` | `9877` | JVM override for the loopback gRPC port. |

JVM properties take precedence inside Burp. HTTP settings must still match the
v2 bridge. The gRPC host is fixed to IPv4 loopback and cannot be configured.
Every gRPC call must include a deadline of at most 30 seconds.

## Security

Burp MCP is dual-use security software intended only for systems you own or are explicitly authorized to test.

- The extension listens on localhost by default.
- Every extension request requires the generated or configured bearer token.
- The bridge reads the token from the environment or `~/.burp-mcp-token`.
- The npm package contains no telemetry and does not connect to a maintainer-operated service.
- Local CyberChef execution blocks operations that can initiate HTTP or DNS traffic.
- High-impact tools retain the capabilities and side effects of the underlying Burp APIs.

See [DISCLOSURE](DISCLOSURE) for the npm dual-use content declaration.

## Build and test

Install bridge dependencies and run all TypeScript checks:

```sh
bun install --frozen-lockfile
bun run check
```

Build and test the Burp extension and Rust Phase 0 workspace:

```sh
gradle clean test jar
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --locked
scripts/run-grpc-interop.sh
```

The extension JAR is written to `build/libs/burp-mcp.jar`. After loading it in
Burp, run the live Phase 0 probe:

```sh
cargo run -p burp-mcp --locked -- probe --endpoint http://127.0.0.1:9877
```

The probe verifies server information and byte-exact 0-byte, 1-byte, and 10 MiB
round trips. To complete the lifecycle gate, unload the extension, confirm the
probe fails, reload the same JAR, and confirm the probe passes again.

## Releases

Published GitHub Releases build both the extension JAR and npm tarball from the same stable version. The npm package is published directly through npm Trusted Publishing with GitHub Actions OIDC and provenance, without a long-lived npm token.

## License

Burp MCP is available under the [MIT License](LICENSE).
