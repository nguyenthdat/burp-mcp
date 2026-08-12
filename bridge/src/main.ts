#!/usr/bin/env node

import { BurpDiscoveryError, BurpHttpProvider } from "./burp-provider"
import { loadConfig } from "./config"
import { RpcDispatcher } from "./rpc"
import { runStdio } from "./stdio"
import { ToolDirectory } from "./tool-directory"

export async function main(): Promise<void> {
  const config = loadConfig()
  const burp = new BurpHttpProvider(config)
  const directory = new ToolDirectory([burp], "burp")
  try {
    await burp.listTools()
  } catch (error) {
    if (error instanceof BurpDiscoveryError) {
      process.stderr.write(
        `[burp-mcp-bridge] WARNING: Cannot connect to Burp at ${config.host}:${config.port}. Start Burp first.\n`,
      )
    } else {
      throw error
    }
  }
  await runStdio(new RpcDispatcher(directory))
}

void main()
