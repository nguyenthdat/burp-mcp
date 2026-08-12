import { describe, expect, test } from "bun:test"
import { ToolDirectory } from "../src/tool-directory"
import {
  isJsonObject,
  type JsonObject,
  type JsonValue,
  type ProviderTool,
  ProviderUnavailableError,
  type ToolProvider,
} from "../src/types"

class FakeProvider implements ToolProvider {
  readonly calls: { readonly name: string; readonly arguments: JsonObject }[] = []

  constructor(
    readonly namespace: string,
    private readonly tools: readonly ProviderTool[],
  ) {}

  async listTools(): Promise<readonly ProviderTool[]> {
    return this.tools
  }

  async callTool(localName: string, arguments_: JsonValue): Promise<JsonValue> {
    if (!isJsonObject(arguments_)) {
      throw new Error("FakeProvider expects object arguments")
    }
    this.calls.push({ name: localName, arguments: arguments_ })
    return { provider: this.namespace, localName }
  }

  invalidate(): void {}
}

class UnavailableProvider implements ToolProvider {
  readonly namespace = "burp"

  async listTools(): Promise<readonly ProviderTool[]> {
    throw new ProviderUnavailableError("burp", "Burp unavailable")
  }

  async callTool(): Promise<JsonValue> {
    throw new ProviderUnavailableError("burp", "Burp unavailable")
  }

  invalidate(): void {}
}

const EMPTY_SCHEMA = { type: "object", properties: {} } as const

describe("ToolDirectory", () => {
  test("lists providers in registration order and routes namespaced calls", async () => {
    // Given
    const burp = new FakeProvider("burp", [
      { localName: "proxy_history", description: "Burp history", inputSchema: EMPTY_SCHEMA },
    ])
    const cyberchef = new FakeProvider("cyberchef", [
      { localName: "bake", description: "Run a recipe", inputSchema: EMPTY_SCHEMA },
    ])
    const directory = new ToolDirectory([burp, cyberchef], "burp")

    // When
    const tools = await directory.listTools()
    const result = await directory.callTool("cyberchef_bake", { recipe: "To Base64" })

    // Then
    expect(tools.map((tool) => tool.name)).toEqual(["burp_proxy_history", "cyberchef_bake"])
    expect(result).toEqual({ provider: "cyberchef", localName: "bake" })
    expect(cyberchef.calls).toEqual([{ name: "bake", arguments: { recipe: "To Base64" } }])
  })

  test("routes unprefixed names through the compatibility provider", async () => {
    // Given
    const burp = new FakeProvider("burp", [])
    const directory = new ToolDirectory([burp], "burp")

    // When
    await directory.callTool("proxy_history", { limit: 1 })

    // Then
    expect(burp.calls).toEqual([{ name: "proxy_history", arguments: { limit: 1 } }])
  })

  test("rejects duplicate provider namespaces", () => {
    // Given
    const first = new FakeProvider("burp", [])
    const second = new FakeProvider("burp", [])

    // When / Then
    expect(() => new ToolDirectory([first, second], "burp")).toThrow(
      "Duplicate tool provider namespace: burp",
    )
  })

  test("keeps a healthy provider usable when Burp is unavailable", async () => {
    // Given
    const cyberchef = new FakeProvider("cyberchef", [
      { localName: "bake", description: "Run a recipe", inputSchema: EMPTY_SCHEMA },
    ])
    const directory = new ToolDirectory([new UnavailableProvider(), cyberchef], "burp")

    // When
    const tools = await directory.listTools()
    const result = await directory.callTool("cyberchef_bake", { recipe: "To Base64" })

    // Then
    expect(tools.map((tool) => tool.name)).toEqual(["cyberchef_bake"])
    expect(result).toEqual({ provider: "cyberchef", localName: "bake" })
  })
})
