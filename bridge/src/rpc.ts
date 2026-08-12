import type { ToolDirectory } from "./tool-directory"
import {
  hasOwnError,
  isJsonObject,
  type JsonObject,
  type JsonValue,
  ProviderUnavailableError,
} from "./types"

export type RpcResponse = JsonObject | null

export class RpcDispatcher {
  constructor(private readonly directory: ToolDirectory) {}

  async handle(message: JsonValue): Promise<RpcResponse> {
    const request = isJsonObject(message) ? message : {}
    const method = request["method"]
    const id = request["id"] ?? null
    switch (method) {
      case "initialize":
        return {
          jsonrpc: "2.0",
          id,
          result: {
            protocolVersion: "2024-11-05",
            capabilities: { tools: {} },
            serverInfo: { name: "burpsuite-mcp", version: "2.0.0" },
          },
        }
      case "notifications/initialized":
        return null
      case "tools/list":
        return this.listTools(id)
      case "tools/call":
        return this.callTool(id, request["params"])
      default:
        return rpcError(id, -32601, `Method not found: ${String(method)}`)
    }
  }

  private async listTools(id: JsonValue): Promise<JsonObject> {
    try {
      return { jsonrpc: "2.0", id, result: { tools: await this.directory.listTools() } }
    } catch (error) {
      if (error instanceof ProviderUnavailableError) {
        return rpcError(id, -32000, error.publicMessage)
      }
      throw error
    }
  }

  private async callTool(id: JsonValue, rawParams: JsonValue | undefined): Promise<JsonObject> {
    if (!isJsonObject(rawParams) || typeof rawParams["name"] !== "string") {
      return rpcError(id, -32602, "tools/call requires params.name")
    }
    const name = rawParams["name"]
    const rawArguments = rawParams["arguments"]
    const arguments_ = rawArguments ? rawArguments : {}
    try {
      const result = await this.directory.callTool(name, arguments_)
      const toolResult: JsonObject = {
        content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
        ...(hasOwnError(result) ? { isError: true } : {}),
      }
      return { jsonrpc: "2.0", id, result: toolResult }
    } catch (error) {
      if (error instanceof ProviderUnavailableError) {
        return rpcError(id, -32000, error.publicMessage)
      }
      this.directory.invalidate(name)
      return rpcError(id, -1, error instanceof Error ? error.message : "Tool call failed")
    }
  }
}

export function rpcError(id: JsonValue, code: number, message: string): JsonObject {
  return { jsonrpc: "2.0", id, error: { code, message } }
}
