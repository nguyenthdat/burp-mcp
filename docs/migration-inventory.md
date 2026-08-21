# v3 migration inventory

## Baseline

- Starting revision: `204248fc22f58ee6d9c806a77f82b9e333395b50`
- Current production path: Kotlin `McpHttpServer` (NanoHTTPD + Gson) and
  `bridge/` Bun/TypeScript MCP stdio + CyberChef worker.
- Phase 0 status: the typed gRPC spike is enabled with the v2 HTTP server in
  default `dual` mode on `127.0.0.1:9877`; `BURP_MCP_TRANSPORT` /
  `-Dburp.mcp.transport` can select `http`, `grpc`, or `dual`. v2 HTTP remains
  available and is not removed by this change.
- Contract fixtures: `src/test/resources/contracts/burp-tool-names.json` contains
  the 80 currently advertised v2 `burp_*` backend names and
  `test-fixtures/contracts/burp-tools-v2.json` captures each public description
  and input schema. Run `bun run check:contract` to detect drift; regenerate
  with `bun scripts/generate-burp-contract-fixture.ts`. These are fixtures, not
  runtime dependencies.

## Phase 0 ownership

| Area | Files | Boundary |
| --- | --- | --- |
| Protobuf source | `proto/common.proto`, `proto/burp.proto` | Shared source of truth |
| Rust client/actor | `crates/burp-grpc/` | Typed tonic client, bounded queue, timeouts, reconnect |
| Kotlin spike | `src/main/kotlin/.../GrpcSpikeServer.kt` | Loopback server and Montoya read adapter |
| Kotlin process fixture | `src/test/kotlin/.../GrpcInteropServerMain.kt` | Test-only restart/unload harness |
| Interop tests | `crates/burp-grpc/tests/interop.rs` | Optional real Kotlin process gate |
| Kotlin unit/integration tests | `src/test/kotlin/.../GrpcSpikeServerTest.kt` | Lifecycle, bytes, cancellation |
| Version generation | `build.gradle.kts` | Pinned JVM/protobuf/gRPC plugins |

## Tool classification for later phases

- **Rust/pure:** `encode`, `decode`, `payload_process`, `jwt_decode`, and
  future `utility_*` operations. These must be ported before removing CyberChef
  or the current bridge.
- **Kotlin/Burp read:** proxy history/detail/search, sitemap, target info,
  scope reads, cookie reads, scanner results, extension/version information.
- **Kotlin/Burp write:** request sending, Repeater/Intruder, annotations,
  scope/configuration/handler changes, WebSocket, Collaborator, imports, and
  other stateful operations.
- **Long-running/job candidates:** scans, crawls, Intruder/fuzzer/race
  operations, Sequencer, and Collaborator polling. They must not become
  unbounded unary calls.

This classification is preliminary. Each tool requires a parity test or
migration note before v2 removal, as required by PLAN.md AC-013.
