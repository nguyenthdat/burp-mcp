import { existsSync, readFileSync } from "node:fs"
import { homedir } from "node:os"
import { join } from "node:path"

export type BridgeConfig = {
  readonly host: string
  readonly port: number
  readonly authHeaders: Readonly<Record<string, string>>
}

export function loadConfig(env: NodeJS.ProcessEnv = process.env): BridgeConfig {
  const host = env["BURP_MCP_HOST"] || "127.0.0.1"
  const port = Number.parseInt(env["BURP_MCP_PORT"] || "9876", 10)
  const token = resolveToken(env)
  return {
    host,
    port,
    authHeaders: token === null || token.length === 0 ? {} : { Authorization: `Bearer ${token}` },
  }
}

function resolveToken(env: NodeJS.ProcessEnv): string | null {
  const environmentToken = env["BURP_MCP_TOKEN"]
  if (environmentToken) {
    return environmentToken
  }
  try {
    const tokenFile = join(homedir(), ".burp-mcp-token")
    return existsSync(tokenFile) ? readFileSync(tokenFile, "utf8").trim() : null
  } catch (error) {
    if (error instanceof Error) {
      return null
    }
    throw error
  }
}
