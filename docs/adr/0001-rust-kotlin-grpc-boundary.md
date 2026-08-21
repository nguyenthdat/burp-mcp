# ADR-0001: Rust/Kotlin loopback gRPC boundary

- **Status:** Accepted for Phase 0 spike; production adoption is gated.
- **Date:** 2026-08-21
- **Decision owners:** Burp MCP maintainers

## Context

The v2 system serves MCP JSON-RPC through a Kotlin HTTP/JSON server and a Bun
bridge. PLAN.md requires v3 to keep MCP stdio in Rust while moving the
Burp/Montoya boundary to a typed, local-only protocol. The current HTTP bridge
must remain available until parity and cutover gates pass.

## Decision

Use Protocol Buffers as the only cross-language contract and gRPC over HTTP/2
on an IPv4 loopback TCP listener for the Phase 0 interoperability spike and
subsequent v3 transport.

- Kotlin/JVM uses `grpc-java` 1.73.0, protobuf Java 4.31.1, and the Gradle
  protobuf plugin 0.9.6.
- Rust uses `tonic`/`tonic-prost` 0.14.6, `prost` 0.14.4, and
  `protoc-bin-vendored` 3.2.0 (which packages `protoc` 31.1). Rust pins the
  workspace MSRV to 1.88.
- The listener is constructed from `127.0.0.1` and rejects invalid ports. It
  has no application-level authentication; the machine is the trust boundary.
- Both sides enforce a 16 MiB message limit. Every Rust request has a bounded
  queue and a per-call timeout. The generated tonic client is owned by a
  reconnecting actor and is not exposed to future MCP handlers.
- The initial typed spike surface is `Ping`, `EchoBytes`, `ProxyHistory`, and
  `ServerInfo`. `ProxyHistory` uses bounded cursor pagination. `EchoBytes` is
  the binary round-trip probe; its delay field exists only to test deadline and
  cancellation interoperability and will be reserved/removed before the
  production contract is frozen. Every call must carry a client deadline no
  greater than 30 seconds.

## Consequences

This creates a small, testable seam without deleting or changing the existing
NanoHTTPD server, Bun package, CyberChef runtime, or release workflow. The
Rust actor reports an actionable `Unavailable` error while Burp is offline and
recreates the channel after transport failures. Generated Rust code remains
internal to `burp-grpc`; future MCP APIs must use Rust DTOs.

The spike still needs evidence from a Burp-hosted extension on JDK 25 and
cross-platform runs. Mocked Montoya tests prove lifecycle and protocol behavior
but cannot satisfy the Burp-hosted portion of the Phase 0 gate.

## Rejected alternatives

- **HTTP/JSON:** existing v2 transport; it loses typed binary/framing guarantees
  and is the migration boundary being replaced.
- **JNI:** rejected because Rust must remain a separate native process and the
  JVM lifecycle/packaging model would be more fragile.
- **Stringly `invoke(toolName, jsonParams)`:** permitted only as a temporary
  compatibility adapter; it is not the final typed seam.
- **Externally reachable TCP or application token auth:** rejected for the
  first version; binding is strictly local and the trust limitation is
  documented rather than hidden.
