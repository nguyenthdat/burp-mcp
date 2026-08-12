export type JsonPrimitive = string | number | boolean | null
export type JsonValue = JsonPrimitive | JsonObject | readonly JsonValue[]
export type JsonObject = { readonly [key: string]: JsonValue }

export type ProviderTool = {
  readonly localName: string
  readonly description: string
  readonly inputSchema: JsonObject
}

export type McpTool = {
  readonly name: string
  readonly description: string
  readonly inputSchema: JsonObject
}

export interface ToolProvider {
  readonly namespace: string
  listTools(): Promise<readonly ProviderTool[]>
  callTool(localName: string, arguments_: JsonValue): Promise<JsonValue>
  invalidate(): void
}

export class ProviderUnavailableError extends Error {
  override readonly name: string = "ProviderUnavailableError"

  constructor(
    readonly namespace: string,
    readonly publicMessage: string,
    options?: ErrorOptions,
  ) {
    super(publicMessage, options)
  }
}

export function isJsonObject(value: JsonValue | undefined): value is JsonObject {
  return value !== null && typeof value === "object" && !Array.isArray(value)
}

export function hasOwnError(value: JsonValue): boolean {
  return isJsonObject(value) && Object.hasOwn(value, "error")
}
