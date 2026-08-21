#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONTROL="$(mktemp -d "${TMPDIR:-/tmp}/burp-mcp-grpc.XXXXXX")"
SERVER_PID=""
cleanup() {
  touch "$CONTROL/exit" 2>/dev/null || true
  if [[ -n "$SERVER_PID" ]]; then
    wait "$SERVER_PID" 2>/dev/null || true
  fi
  rm -rf "$CONTROL"
}
trap cleanup EXIT INT TERM

cd "$ROOT"
gradle --no-daemon testClasses >/dev/null
RUNTIME_CLASSPATH="$(gradle --no-daemon -q printTestRuntimeClasspath)"

PORT="$(python3 - <<'PY'
import socket
with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
)"

java -cp "$RUNTIME_CLASSPATH" io.github.nguyenthdat.burpmcp.GrpcInteropServerMain "$PORT" "$CONTROL" &
SERVER_PID=$!
for _ in {1..250}; do
  [[ -f "$CONTROL/ready" ]] && break
  kill -0 "$SERVER_PID" 2>/dev/null || { echo "Kotlin gRPC server exited before becoming ready" >&2; exit 1; }
  sleep 0.02
done
[[ -f "$CONTROL/ready" ]] || { echo "Kotlin gRPC server did not become ready" >&2; exit 1; }

export BURP_MCP_INTEROP_ENDPOINT="http://127.0.0.1:$PORT"
export BURP_MCP_INTEROP_CONTROL="$CONTROL"
cargo test -p burp-grpc --test interop kotlin_server_echoes_binary_payloads_and_handles_concurrency -- --nocapture
cargo run -p burp-mcp --locked -- probe --endpoint "$BURP_MCP_INTEROP_ENDPOINT"

# The reconnect test controls an in-process shutdown/restart so the same
# lifecycle code used on extension unload runs deterministically.
cargo test -p burp-grpc --test interop kotlin_server_honors_deadlines_and_reconnects_after_restart -- --nocapture
