#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/burp-mcp-install-test.XXXXXX")"
SERVER_PID=""
cleanup() {
  [ -z "$SERVER_PID" ] || kill "$SERVER_PID" 2>/dev/null || true
  rm -rf "$TMP"
}
trap cleanup EXIT

case "$(uname -s)/$(uname -m)" in
  Darwin/arm64|Darwin/aarch64) asset="burp-mcp-macos-aarch64" ;;
  Darwin/x86_64|Darwin/amd64) asset="burp-mcp-macos-x86_64" ;;
  Linux/x86_64|Linux/amd64) asset="burp-mcp-linux-x86_64" ;;
  *) echo "unsupported smoke-test platform" >&2; exit 0 ;;
esac

cat >"$TMP/$asset" <<'EOF'
#!/usr/bin/env bash
[ "${1:-}" = "--version" ] || exit 1
echo "burp-mcp 3.0.0"
EOF
chmod +x "$TMP/$asset"
if command -v sha256sum >/dev/null 2>&1; then
  digest="$(sha256sum "$TMP/$asset" | awk '{print $1}')"
else
  digest="$(shasum -a 256 "$TMP/$asset" | awk '{print $1}')"
fi
printf '%s  %s\n' "$digest" "$asset" >"$TMP/SHA256SUMS"

python3 -m http.server 18473 --bind 127.0.0.1 --directory "$TMP" >"$TMP/http.log" 2>&1 &
SERVER_PID="$!"
for _ in 1 2 3 4 5; do
  curl -fsS "http://127.0.0.1:18473/SHA256SUMS" >/dev/null 2>&1 && break
  sleep 1
done

# The production installer rejects HTTP. Use a temporary HTTPS-to-loopback
# substitution only to exercise platform selection, checksum validation, and
# installation without hitting a real GitHub release.
TEST_INSTALLER="$TMP/install.sh"
sed -e 's#case "$DOWNLOAD_BASE" in#case "$DOWNLOAD_BASE" in\n    http://127.0.0.1:18473) ;;#' \
    -e "s/--proto '=https' --tlsv1.2/--proto '=http' --proto-redir '=http'/" \
    "$ROOT/install.sh" >"$TEST_INSTALLER"
BURP_MCP_DOWNLOAD_BASE="http://127.0.0.1:18473" \
  bash "$TEST_INSTALLER" --dir "$TMP/bin" >"$TMP/install.log"
"$TMP/bin/burp-mcp" --version | grep -q '3.0.0'

echo "installer smoke test passed"
