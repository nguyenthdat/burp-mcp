#!/usr/bin/env bash
set -euo pipefail

# Usage:
#   curl -fsSL https://raw.githubusercontent.com/nguyenthdat/burp-mcp/main/install.sh | bash
#   curl -fsSL .../install.sh | bash -s -- --with-skill --agent codex

main() {
  REPO="${BURP_MCP_REPO:-nguyenthdat/burp-mcp}"
  DOWNLOAD_BASE="${BURP_MCP_DOWNLOAD_BASE:-https://github.com/${REPO}/releases/latest/download}"
  INSTALL_DIR="${BURP_MCP_INSTALL_DIR:-$HOME/.local/bin}"
  SKILL_SOURCE="${BURP_MCP_SKILL_SOURCE:-https://github.com/${REPO}/tree/main/docs/burp-skill}"
  WITH_SKILL=false
  SKILL_AGENT=""

  while [ "$#" -gt 0 ]; do
    case "$1" in
      --dir)
        [ "$#" -ge 2 ] || fail "--dir requires a path"
        INSTALL_DIR="$2"
        shift 2
        ;;
      --with-skill)
        WITH_SKILL=true
        shift
        ;;
      --agent)
        [ "$#" -ge 2 ] || fail "--agent requires a skills CLI agent name"
        SKILL_AGENT="$2"
        shift 2
        ;;
      -h|--help)
        usage
        return 0
        ;;
      *)
        fail "unknown option: $1"
        ;;
    esac
  done

  case "$DOWNLOAD_BASE" in
    https://*) ;;
    *) fail "BURP_MCP_DOWNLOAD_BASE must use HTTPS" ;;
  esac

  require_command curl
  require_command uname

  os="$(uname -s)"
  arch="$(uname -m)"
  case "$os/$arch" in
    Linux/x86_64|Linux/amd64) asset="burp-mcp-linux-x86_64"; daemon_asset="sitegraph-daemon-linux-x86_64" ;;
    Darwin/arm64|Darwin/aarch64) asset="burp-mcp-macos-aarch64"; daemon_asset="sitegraph-daemon-macos-aarch64" ;;
    Darwin/x86_64|Darwin/amd64) asset="burp-mcp-macos-x86_64"; daemon_asset="sitegraph-daemon-macos-x86_64" ;;
    *) fail "unsupported platform: $os/$arch" ;;
  esac

  tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/burp-mcp.XXXXXX")"
  trap 'rm -rf "$tmp_dir"' EXIT

  echo "Installing burp-mcp from $DOWNLOAD_BASE"
  download "$DOWNLOAD_BASE/$asset" "$tmp_dir/$asset"
  download "$DOWNLOAD_BASE/$daemon_asset" "$tmp_dir/$daemon_asset"
  download "$DOWNLOAD_BASE/SHA256SUMS" "$tmp_dir/SHA256SUMS"
  verify_checksum "$tmp_dir" "$asset"
  verify_checksum "$tmp_dir" "$daemon_asset"

  chmod 0755 "$tmp_dir/$asset" "$tmp_dir/$daemon_asset"
  "$tmp_dir/$asset" --version >/dev/null
  mkdir -p "$INSTALL_DIR"
  staged="$INSTALL_DIR/.burp-mcp.new.$$"
  daemon_staged="$INSTALL_DIR/.sitegraph-daemon.new.$$"
  install -m 0755 "$tmp_dir/$asset" "$staged"
  install -m 0755 "$tmp_dir/$daemon_asset" "$daemon_staged"
  mv -f "$staged" "$INSTALL_DIR/burp-mcp"
  mv -f "$daemon_staged" "$INSTALL_DIR/sitegraph-daemon"
  echo "Installed $INSTALL_DIR/burp-mcp and $INSTALL_DIR/sitegraph-daemon"

  case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *) echo "Add $INSTALL_DIR to PATH before configuring your MCP client." ;;
  esac

  if [ "$WITH_SKILL" = true ]; then
    install_skill
  fi

  cat <<EOF
Next:
  1. Download burp-mcp.jar from https://github.com/${REPO}/releases/latest
  2. Load it in Burp Suite: Extensions > Installed > Add > Java
  3. Configure your MCP client to run: $INSTALL_DIR/burp-mcp serve
EOF
}

usage() {
  cat <<'EOF'
Install burp-mcp on macOS or Linux.

Options:
  --dir PATH       Install binary to PATH (default: ~/.local/bin)
  --with-skill     Install docs/burp-skill with the skills CLI
  --agent NAME     Target one skills CLI agent (for example codex)
  -h, --help       Show this help

Environment:
  BURP_MCP_INSTALL_DIR   Default install directory
  BURP_MCP_DOWNLOAD_BASE HTTPS release download base (for mirrors/tests)
  BURP_MCP_SKILL_SOURCE Skill URL passed to `npx skills add`
EOF
}

require_command() {
  command -v "$1" >/dev/null 2>&1 || fail "required command not found: $1"
}

download() {
  curl --fail --show-error --silent --location \
    --proto '=https' --tlsv1.2 --retry 3 --retry-all-errors \
    "$1" --output "$2"
}

verify_checksum() {
  directory="$1"
  asset="$2"
  expected="$(awk -v name="$asset" 'NF >= 2 && ($2 == name || $2 == "*" name) { print $1; exit }' "$directory/SHA256SUMS")"
  case "$expected" in
    ''|*[!0-9a-fA-F]*) fail "SHA256SUMS has no valid entry for $asset" ;;
  esac
  [ "${#expected}" -eq 64 ] || fail "invalid SHA-256 entry for $asset"

  if command -v sha256sum >/dev/null 2>&1; then
    actual="$(sha256sum "$directory/$asset" | awk '{print $1}')"
  elif command -v shasum >/dev/null 2>&1; then
    actual="$(shasum -a 256 "$directory/$asset" | awk '{print $1}')"
  else
    fail "sha256sum or shasum is required to verify the release"
  fi
  [ "$actual" = "$expected" ] || fail "checksum mismatch for $asset"
}

install_skill() {
  require_command node
  require_command npx
  echo "Installing Burp skill from $SKILL_SOURCE"
  if [ -n "$SKILL_AGENT" ]; then
    npx --yes skills add "$SKILL_SOURCE" --global --yes --agent "$SKILL_AGENT"
  else
    npx --yes skills add "$SKILL_SOURCE" --global --yes
  fi
}

fail() {
  echo "error: $*" >&2
  exit 1
}

main "$@"
