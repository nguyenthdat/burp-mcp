# v3 migration inventory

## Baseline and cutover state

- Starting revision: `204248fc22f58ee6d9c806a77f82b9e333395b50`.
- Kotlin now has one production transport: typed gRPC over HTTP/2 on
  `127.0.0.1:9877`.
- The Kotlin HTTP/JSON/NanoHTTPD adapter, Gson transport DTOs, Bearer-token
  authentication and `~/.burp-mcp-token` management are deleted.
- The native Rust binary serves MCP over stdio with `rmcp` and reaches Kotlin
  through a bounded, reconnecting tonic actor.
- Phase 0 deterministic interop, reconnect, cancellation, binary round-trip and
  lifecycle results are recorded in `docs/phase0-interop-results.md`.
- The interactive Burp/JDK 25 unload/reload gate remains recorded in
  `docs/phase0-burp-jdk25-verification.md`.
- Historical v2 tool fixtures remain under `test-fixtures/contracts/` to drive
  parity work. They are migration evidence, not Kotlin runtime dependencies.

## Implemented migration seams

| Area | Files | Boundary |
| --- | --- | --- |
| Protobuf source | `proto/common.proto`, `proto/burp.proto` | Shared typed source of truth |
| Rust client | `crates/burp-protocol/` | Typed tonic client, bounded queue, deadlines, reconnect |
| Rust MCP composition | `crates/burp-mcp/src/main.rs`, `cli.rs`, `tools.rs` | `rmcp` stdio server, CLI/probe, typed tool adapters |
| Kotlin lifecycle | `TransportLifecycle.kt`, `BurpMcpExtension.kt` | Explicit gRPC start/close owner |
| Kotlin typed facades | `ProxyFacade.kt`, `SitemapFacade.kt`, `TargetFacade.kt`, `ScannerFacade.kt` | Transport-independent Montoya seams |
| Kotlin RPC adapter | `rpc/BurpRpcServer.kt` | Loopback server, bounded worker pool, deadlines and structured errors |
| Kotlin process fixture | `GrpcInteropServerMain.kt` | Test-only restart/unload harness |
| Interop tests | `crates/burp-protocol/tests/interop.rs` | Real Kotlin-process gate |
| Version generation | `build.gradle.kts`, `Cargo.toml`, `Cargo.lock` | Pinned JVM/protobuf/gRPC/Rust dependencies |

## Current Rust MCP tools

- `burp_server_info`
- `burp_proxy_history`
- `burp_proxy_detail`
- `burp_sitemap`
- `burp_target_info`
- `burp_get_scope`

## Remaining migration work

- Port scan issues and the remaining read-only Burp tools through typed protobuf
  services.
- Port mutation and long-running Burp operations; long-running calls must use
  the job lifecycle.
- Port pure utilities to Rust, then delete the Bun/CyberChef runtime.
- Build the persistent SQLite sitegraph.
- Replace legacy npm release/CI paths with Cargo/Gradle native artifacts.

No untyped `InvokeLegacy` method exists. New Burp capabilities must cross the
boundary through typed protobuf messages and transport-independent Kotlin
facades.
