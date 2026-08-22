# Phase 0 Rust/Kotlin gRPC interoperability results

- **Date:** 2026-08-21
- **Host:** macOS 26.6.2, Apple arm64
- **JDK:** JetBrains Runtime 25.0.4+1
- **Gradle:** 9.7.0
- **Rust gate:** workspace MSRV 1.88; local verification also ran on Rust 1.100.0-nightly
- **Kotlin/gRPC:** Kotlin 2.4.10, grpc-java 1.73.0, protobuf Java/protoc 4.31.1, Gradle protobuf plugin 0.9.6
- **Rust/gRPC:** tonic/tonic-prost 0.14.6, prost 0.14.4, protoc-bin-vendored 3.2.0
- **Extension JAR SHA-256:** `700dc288ef7c60b24d21c140278a832360ace827a6c5f69d5b37a402f38ca01a`

## Deterministic process fixture

`scripts/run-grpc-interop.sh` passed in 17.23 seconds. The fixture started the Kotlin/JVM gRPC server on an ephemeral IPv4 loopback port and exercised it from the real Rust tonic client.

Observed results:

- `Ping`, `ServerInfo`, and bounded `ProxyHistory` calls succeeded.
- 0-byte, 1-byte, and 10 MiB payloads returned byte-exact.
- 32 concurrent unary echo calls returned their original 4 KiB payloads without corruption.
- A delayed call with a 25 ms deadline terminated with cancellation/deadline status.
- The server released and rebound the same listener during restart.
- The long-lived Rust actor reported the offline interval, discarded its failed channel, and succeeded after restart.
- The server advertised and enforced 16 MiB request/response limits, page limit 500, 32 concurrent calls per connection, and a 30-second maximum deadline.

Representative probe output:

```text
PASS endpoint=http://127.0.0.1:<ephemeral-port>
server=burp-mcp-kotlin version=development
capabilities=proxy.read,transport.echo,lifecycle.restart
limits: message=16777216 response=16777216 page=500 concurrency=32 timeout=30s
byte-exact payloads: 0, 1, and 10485760 bytes
```

The observed 0.29-second concurrent/binary test and 0.10-second deadline/reconnect test are smoke timings, not stable performance benchmarks. The gate establishes interoperability and bounded lifecycle behavior; it does not claim throughput or latency targets.

## Remaining production-host gate

Burp Suite 2026.7.3 and JDK 25 are installed on the macOS host. The deterministic fixture cannot prove Montoya extension classloading or UI-driven unload/reload behavior. Complete and record the live Burp steps in `docs/phase0-burp-jdk25-verification.md` before treating Phase 0 as accepted or starting the behavior-changing Kotlin migration.

Linux and Windows Burp-hosted runs remain unavailable on this workstation and must be collected on their release hosts where possible.
