# Install Burp MCP

Burp MCP has two runtime parts:

1. `burp-mcp.jar`, loaded into Burp Suite.
2. The native `burp-mcp` MCP stdio server, launched by your MCP client.

The optional `burp-skill` teaches compatible coding agents how to operate the exposed Burp tools safely. It is not required by the server.

## Requirements

- Burp Suite with Montoya API extension support.
- macOS or Linux for the one-line native installer. Windows users should download the release binary manually when available.
- `curl` and either `sha256sum` or `shasum`.
- Node.js and `npx` only when installing the optional agent skill.

Java 25, Rust 1.88, and Gradle 9.7 are build requirements, not requirements for running published release artifacts.

## One-line install on macOS or Linux

Review [`install.sh`](../install.sh) before piping it to a shell, then run:

```sh
curl -fsSL https://raw.githubusercontent.com/nguyenthdat/burp-mcp/main/install.sh | bash
```

This installs the native binary to `~/.local/bin/burp-mcp`, verifies it against the release `SHA256SUMS`, and prints the remaining Burp extension steps. Supported release assets are:

- `burp-mcp-linux-x86_64`
- `burp-mcp-macos-aarch64`
- `burp-mcp-macos-x86_64`

Install somewhere else:

```sh
curl -fsSL https://raw.githubusercontent.com/nguyenthdat/burp-mcp/main/install.sh \
  | bash -s -- --dir "$HOME/bin"
```

Install the native binary and global agent skill together:

```sh
curl -fsSL https://raw.githubusercontent.com/nguyenthdat/burp-mcp/main/install.sh \
  | bash -s -- --with-skill
```

Target one agent supported by the [`skills` CLI](https://github.com/vercel-labs/skills):

```sh
curl -fsSL https://raw.githubusercontent.com/nguyenthdat/burp-mcp/main/install.sh \
  | bash -s -- --with-skill --agent codex
```

The installer refuses non-HTTPS release URLs, uses a temporary directory, validates the selected asset's SHA-256 digest, checks the candidate with `burp-mcp --version`, and stages it before replacing the installed binary. It does not use `sudo` or modify shell startup files.

## Load the Burp extension

1. Download `burp-mcp.jar` from the [latest GitHub Release](https://github.com/nguyenthdat/burp-mcp/releases/latest).
2. In Burp Suite, open **Extensions > Installed**.
3. Click **Add**.
4. Select extension type **Java** and choose `burp-mcp.jar`.
5. Confirm the Burp MCP extension starts its loopback gRPC listener on `127.0.0.1:9877`.

## Configure an MCP client

Use the installed absolute path:

```json
{
  "command": "/Users/you/.local/bin/burp-mcp",
  "args": ["serve"]
}
```

On Linux the common path is `/home/you/.local/bin/burp-mcp`. Run `command -v burp-mcp` to resolve the installed path.

Keep sitegraph disabled for the normal v3 setup. See [Sitegraph](sitegraph.md) for the explicit advanced opt-in.

## Install or update only the agent skill

The skill package lives at [`docs/burp-skill`](burp-skill/SKILL.md). Install it globally with:

```sh
npx --yes skills add \
  https://github.com/nguyenthdat/burp-mcp/tree/main/docs/burp-skill \
  --global --yes
```

Target one supported agent when needed:

```sh
npx --yes skills add \
  https://github.com/nguyenthdat/burp-mcp/tree/main/docs/burp-skill \
  --global --yes --agent codex
```

Run the same command again to update the installed skill after repository changes.

## Verify the connection

Load the extension first, then run:

```sh
burp-mcp probe --endpoint http://127.0.0.1:9877
```

The probe checks server information and byte-exact binary round trips. The MCP client can then launch `burp-mcp serve` over stdio.

## Manual installation

1. Download the binary for your platform, `burp-mcp.jar`, and `SHA256SUMS` from the same release.
2. Verify the downloaded files with `sha256sum --check SHA256SUMS` on Linux or compare `shasum -a 256 FILE` on macOS.
3. Mark the binary executable and move it to a directory on `PATH`.
4. Load the JAR and configure the MCP client as described above.

## Uninstall

Remove the native binary:

```sh
rm "$HOME/.local/bin/burp-mcp"
```

Remove the extension from **Extensions > Installed** in Burp Suite. Use the uninstall command printed by the `skills` CLI for your selected agent to remove the optional skill.
