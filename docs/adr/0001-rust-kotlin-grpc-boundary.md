# ADR-0001: Rust/Kotlin loopback gRPC boundary

- **Status:** Accepted and implemented for v3 production.
- **Date:** 2026-08-21
- **Decision owners:** Burp MCP maintainers

## Context

The v3 system serves MCP JSON-RPC over stdio from Rust and keeps Montoya access
inside the Kotlin extension. Protobuf is the only cross-language contract; the
retired HTTP/JSON bridge, Bun runtime, and CyberChef runtime are no longer part
of production.

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
- The production typed surface is capability-oriented at the Montoya facade and
  Rust module seams. The current wire contract remains one `BurpService` during
  the v3 alpha compatibility window; generated tonic types stay inside
  `burp-protocol` except for explicitly feature-gated interoperability tests.
  Every call carries a client deadline no greater than 30 seconds.

## Consequences

This creates a typed, local-only production seam. The Rust actor reports an
actionable `Unavailable` error while Burp is offline and recreates the channel
after transport failures. Production probes use typed Rust DTOs; raw generated
types are reserved for interoperability fixtures.

Host-level validation on JDK 25 and cross-platform packaging remain release
verification work, not an architectural gate.

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
