# Burp MCP

[![CI](https://github.com/nguyenthdat/burp-mcp/actions/workflows/ci.yml/badge.svg)](https://github.com/nguyenthdat/burp-mcp/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Burp MCP connects MCP-compatible clients to Burp Suite through a native Rust stdio server and a Kotlin extension built on the Montoya API. The Kotlin/Rust boundary is typed protobuf over loopback gRPC; pure transformations and persistent site metadata stay in Rust.

## Capabilities

- Inspect, search, annotate, highlight, compare, and clear Proxy history.
- Send and modify HTTP requests through Burp Repeater.
- Run bounded background operations and inspect their status and paginated results.
- Start bounded crawls and audits, inspect issues, and work with Burp Collaborator.
- Query the site map, target information, scope, and cookie jar.
- Create and interact with managed WebSocket connections.
- Convert, export, and analyze text, JSON, binary data, and raw HTTP messages.
- Import Bambdas and BChecks without executing them automatically.
- Apply HTTP handlers, proxy rules, and session rules.
- Persist a privacy-preserving SQLite site graph containing endpoint metadata and parameter names, never parameter values or message bodies.
- Run deterministic, binary-safe utility recipes without network, filesystem, browser, or arbitrary-code capabilities.

Some capabilities require Burp Suite Professional or a Burp feature that is available only in specific editions.

`PLAN.md` defines the v3 architecture. Rust serves MCP over stdio and owns the
bounded reconnecting gRPC actor, typed protocol client, local utility engine,
and persistent sitegraph facade. Kotlin owns Montoya state and exposes only the
loopback gRPC adapter on `127.0.0.1:9877`; the retired HTTP/JSON/NanoHTTPD path,
Bearer-token file, Bun bridge, and CyberChef runtime are not production
dependencies.

Override the fixed-loopback gRPC port before starting Burp when required:

```sh
BURP_MCP_GRPC_PORT=10077
# JVM alternative: -Dburp.mcp.grpc.port=10077
```

The endpoint deliberately has no application-level authentication and listens
only on `127.0.0.1`; any local process that can reach the port can call it. Run
the cross-language fixture with `scripts/run-grpc-interop.sh`. The remaining
Burp/JDK 25 interactive unload/reload gate is recorded in
`docs/phase0-burp-jdk25-verification.md`.

## Architecture

```text
MCP client
    │ MCP JSON-RPC / stdio
    ▼
burp-mcp native Rust binary
    │ typed gRPC / HTTP/2 on 127.0.0.1
    ▼
Kotlin Burp extension ──► Montoya API ──► Burp Suite
```

JavaScript, Bun, npm, the legacy HTTP/JSON transport, and the external utility runtime are not part of the v3 production path.

## Requirements

- Burp Suite with support for Montoya API extensions.
- Java 25 for building the extension.
- Rust 1.88 or newer for the native MCP server.

Utility inputs are bounded to 16 MiB, recipes to 64 steps, and batches to 100 items. Binary values use tagged base64 at the MCP boundary and remain raw bytes internally.

## Install

### 1. Load the Burp extension

Download `burp-mcp.jar` from the latest [GitHub Release](https://github.com/nguyenthdat/burp-mcp/releases), then add it in Burp Suite:

1. Open **Extensions**.
2. Select **Installed**.
3. Click **Add**.
4. Choose **Java** and select `burp-mcp.jar`.

The extension starts the typed loopback gRPC service on `127.0.0.1:9877` by default.

### 2. Configure an MCP client

Run the downloaded native `burp-mcp` binary. Example MCP server configuration on macOS or Linux:

```json
{
  "command": "/absolute/path/to/burp-mcp",
  "args": ["serve"]
}
```

On Windows, set `command` to the absolute path of `burp-mcp.exe`. The binary accepts `--endpoint` and `--graph-path`; environment equivalents are listed below.

## Configuration

| Setting | Default | Description |
| --- | --- | --- |
| `BURP_MCP_GRPC_PORT` | `9877` | Loopback gRPC port used by Kotlin and Rust. |
| `-Dburp.mcp.grpc.port=<port>` | `9877` | JVM override for the extension gRPC port. |
| `BURP_MCP_GRPC_ENDPOINT` | `http://127.0.0.1:9877` | Rust endpoint; only IPv4 loopback is accepted. |

The gRPC host is fixed to IPv4 loopback and cannot be configured. Every gRPC
call must include a deadline of at most 30 seconds. `BURP_MCP_PORT`,
`BURP_MCP_TOKEN`, `BURP_MCP_TRANSPORT` and their JVM equivalents are removed or
ignored by the v3 Kotlin extension.

## Security

Burp MCP is dual-use security software intended only for systems you own or are explicitly authorized to test.

- The extension listens on localhost by default.
- High-impact tools retain the capabilities and side effects of the underlying Burp APIs.


## Build and test

Run the native and extension checks:

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

Published GitHub Releases contain the native MCP binary, the extension JAR, checksums, and an SBOM generated from the locked Rust dependency graph.

## License

Burp MCP is available under the [MIT License](LICENSE).
