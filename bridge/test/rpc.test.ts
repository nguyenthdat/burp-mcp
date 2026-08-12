import { expect, test } from "bun:test"
import { RpcDispatcher } from "../src/rpc"
import { ToolDirectory } from "../src/tool-directory"
import {
  type JsonValue,
  type ProviderTool,
  ProviderUnavailableError,
  type ToolProvider,
} from "../src/types"

class UnavailableBurpProvider implements ToolProvider {
  readonly namespace = "burp"

  async listTools(): Promise<readonly ProviderTool[]> {
    throw new ProviderUnavailableError("burp", "Burp unavailable")
  }

  async callTool(): Promise<JsonValue> {
    throw new ProviderUnavailableError("burp", "Burp unavailable")
  }

  invalidate(): void {}
}

class HangingBurpProvider implements ToolProvider {
  readonly namespace = "burp"

  async listTools(): Promise<readonly ProviderTool[]> {
    return new Promise(() => {})
  }

  async callTool(): Promise<JsonValue> {
    return new Promise(() => {})
  }

  invalidate(): void {}
}

class CyberChefProvider implements ToolProvider {
  readonly namespace = "cyberchef"

  async listTools(): Promise<readonly ProviderTool[]> {
    return [
      {
        localName: "bake",
        description: "Run a recipe",
        inputSchema: { type: "object", properties: {} },
      },
    ]
  }

  async callTool(localName: string): Promise<JsonValue> {
    return { baked: localName }
  }

  invalidate(): void {}
}

test("calls CyberChef through MCP when Burp is unavailable", async () => {
  // Given
  const directory = new ToolDirectory(
    [new UnavailableBurpProvider(), new CyberChefProvider()],
    "burp",
  )
  const dispatcher = new RpcDispatcher(directory)

  // When
  const response = await dispatcher.handle({
    jsonrpc: "2.0",
    id: 9,
    method: "tools/call",
    params: { name: "cyberchef_bake", arguments: { input: "hello" } },
  })

  // Then
  expect(response?.["error"]).toBeUndefined()
  expect(JSON.stringify(response?.["result"])).toContain('\\"baked\\": \\"bake\\"')
})

test("calls CyberChef without waiting for hanging Burp discovery", async () => {
  // Given
  const directory = new ToolDirectory([new HangingBurpProvider(), new CyberChefProvider()], "burp")
  const dispatcher = new RpcDispatcher(directory)

  // When
  const response = await Promise.race([
    dispatcher.handle({
      jsonrpc: "2.0",
      id: 10,
      method: "tools/call",
      params: { name: "cyberchef_bake", arguments: { input: "hello" } },
    }),
    Bun.sleep(100).then(() => "timeout" as const),
  ])

  // Then
  expect(response).not.toBe("timeout")
  expect(JSON.stringify(response)).toContain('\\"baked\\": \\"bake\\"')
})
