#!/usr/bin/env bun

import { BurpDiscoveryError, BurpHttpProvider } from "./burp-provider"
import { loadConfig } from "./config"
import { CyberChefProvider } from "./cyberchef-provider"
import { RpcDispatcher } from "./rpc"
import { runStdio } from "./stdio"
import { ToolDirectory } from "./tool-directory"

export async function main(): Promise<void> {
  const config = loadConfig()
  const burp = new BurpHttpProvider(config)
  const cyberchef = new CyberChefProvider()
  const directory = new ToolDirectory([burp, cyberchef], "burp")
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
  try {
    await runStdio(new RpcDispatcher(directory))
  } finally {
    cyberchef.close()
  }
}

void main()
