# Phase 0 real Burp/JDK 25 verification

This is the remaining AC-011 manual gate. The deterministic Kotlin process
fixture is supporting evidence; only this real Burp-hosted run validates
extension loading and unloading.

## Build

```sh
gradle --no-daemon clean test jar
cargo build -p burp-mcp --locked
shasum -a 256 build/libs/burp-mcp.jar
```

Expected JAR: `build/libs/burp-mcp.jar`. It contains the Montoya extension
descriptor, Kotlin implementation, generated protobuf classes, grpc-java,
protobuf-java, and shaded Netty. The JAR does not contain test classes.

## Start Burp on JDK 25

The default mode is `dual`: HTTP remains on `127.0.0.1:9876` and gRPC starts on
`127.0.0.1:9877`. To isolate the spike, launch Burp with either environment
variables or JVM properties:

```sh
BURP_MCP_TRANSPORT=grpc BURP_MCP_GRPC_PORT=9877 <start-burp-command>
# equivalent JVM properties:
# -Dburp.mcp.transport=grpc -Dburp.mcp.grpc.port=9877
```

In Burp, open **Extensions → Installed → Add**, choose **Java**, and select
`build/libs/burp-mcp.jar`.

Expected extension output:

```text
[MCP] Transport mode=grpc, HTTP=127.0.0.1:9876, gRPC=127.0.0.1:9877
[MCP] gRPC server ready on 127.0.0.1:9877 (...)
[MCP] gRPC has no application token; any local process can connect
```

Failure to bind is logged with the exception and does not leave the worker pool
running. The server address is compiled as IPv4 loopback; no host setting exists.

## Interoperability probe

With the extension loaded:

```sh
cargo run -p burp-mcp --locked -- probe --endpoint http://127.0.0.1:9877
```

Expected result begins with `PASS` and reports:

- server/version and capabilities;
- 16 MiB request and response bounds;
- maximum page size 500;
- 32 concurrent calls per connection;
- maximum accepted call deadline 30 seconds;
- byte-exact 0-byte, 1-byte and 10 MiB payloads.

Run it several times. If using the default `dual` mode, also run `bun run check`
or the configured v2 MCP bridge to confirm the HTTP compatibility path remains
available.

## Unload and reconnect gate

1. Remove/unload **Burp MCP** from Burp.
2. Confirm Burp logs `[MCP] HTTP and gRPC servers stopped`.
3. Run the probe again; it must fail to connect.
4. Add the same JAR again without restarting the Rust terminal.
5. Run the probe again; it must pass with a fresh gRPC channel.
6. Repeat once while a short-deadline call is in flight if practical; Burp must
   stay responsive and the call must terminate with cancellation/unavailable.

Record OS, architecture, Burp version, JDK version, JAR SHA-256, all probe
outputs, and extension output/error logs. Repeat on macOS, Linux, and Windows
where available. Phase 1 remains blocked until the real Burp/JDK 25 run passes.
