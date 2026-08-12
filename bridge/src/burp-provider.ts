import { request } from "node:http"
import { getBurpToolDescription } from "./burp-tool-descriptions"
import { getBurpToolInputSchema } from "./burp-tool-schemas"
import type { BridgeConfig } from "./config"
import { InvalidJsonError, parseJson } from "./json"
import {
  hasOwnError,
  type JsonValue,
  type ProviderTool,
  ProviderUnavailableError,
  type ToolProvider,
} from "./types"

export class BurpConnectionError extends Error {
  override readonly name = "BurpConnectionError"
}

export class BurpDiscoveryError extends ProviderUnavailableError {
  override readonly name = "BurpDiscoveryError"

  constructor(config: BridgeConfig, options?: ErrorOptions) {
    super(
      "burp",
      `Burp MCP not connected at ${config.host}:${config.port}. Start Burp with the "Burp MCP" extension loaded, then retry.`,
      options,
    )
  }
}

export class BurpHttpProvider implements ToolProvider {
  readonly namespace = "burp"
  private tools: readonly ProviderTool[] | undefined
  private toolsRequest: Promise<readonly ProviderTool[]> | undefined

  constructor(
    private readonly config: BridgeConfig,
    private readonly diagnostics: NodeJS.WritableStream = process.stderr,
  ) {}

  async listTools(): Promise<readonly ProviderTool[]> {
    if (this.tools !== undefined) {
      return this.tools
    }
    if (this.toolsRequest === undefined) {
      this.toolsRequest = this.fetchToolNames()
        .then((names) => {
          const tools = names.map((localName) => ({
            localName,
            description: getBurpToolDescription(localName),
            inputSchema: getBurpToolInputSchema(localName),
          }))
          this.tools = tools
          this.diagnostics.write(
            `[burp-mcp-bridge] Connected to Burp. ${tools.length} tools available.\n`,
          )
          return tools
        })
        .finally(() => {
          this.toolsRequest = undefined
        })
    }
    return this.toolsRequest
  }

  async callTool(localName: string, arguments_: JsonValue): Promise<JsonValue> {
    const body = JSON.stringify({ tool: localName, params: arguments_ })
    return new Promise((resolve, reject) => {
      const outgoing = request(
        {
          hostname: this.config.host,
          port: this.config.port,
          path: "/",
          method: "POST",
          headers: {
            "Content-Type": "application/json; charset=utf-8",
            "Content-Length": Buffer.byteLength(body),
            ...this.config.authHeaders,
          },
        },
        (response) => {
          let data = ""
          response.setEncoding("utf8")
          response.on("data", (chunk: string) => {
            data += chunk
          })
          response.on("end", () => {
            const statusCode = response.statusCode ?? 0
            if (statusCode === 403) {
              resolve({
                error:
                  "Burp MCP rejected the request: missing or invalid BURP_MCP_TOKEN. Set BURP_MCP_TOKEN to match the token in ~/.burp-mcp-token.",
              })
              return
            }
            const parsed = parseBackendResponse(data)
            if (statusCode < 200 || statusCode >= 300) {
              if (parsed.isJson && hasOwnError(parsed.value)) {
                resolve(parsed.value)
                return
              }
              resolve({ error: `Burp MCP returned HTTP ${statusCode}: ${data.slice(0, 200)}` })
              return
            }
            resolve(parsed.value)
          })
        },
      )
      outgoing.on("error", (error: Error) => {
        reject(
          new BurpConnectionError(
            `Cannot reach Burp MCP at ${this.config.host}:${this.config.port} (${getErrorCode(error) ?? error.message}). ` +
              'Ensure Burp Suite is running with the "Burp MCP" extension loaded. ' +
              "If the port differs, set BURP_MCP_PORT and -Dburp.mcp.port=<same> on Burp.",
            { cause: error },
          ),
        )
      })
      outgoing.write(body)
      outgoing.end()
    })
  }

  invalidate(): void {
    this.tools = undefined
  }

  private async fetchToolNames(): Promise<readonly string[]> {
    const value = await new Promise<JsonValue>((resolve, reject) => {
      const outgoing = request(
        {
          hostname: this.config.host,
          port: this.config.port,
          path: "/tools",
          method: "GET",
          headers: this.config.authHeaders,
        },
        (response) => {
          let data = ""
          response.setEncoding("utf8")
          response.on("data", (chunk: string) => {
            data += chunk
          })
          response.on("end", () => {
            if (response.statusCode !== 200) {
              reject(
                new BurpDiscoveryError(this.config, {
                  cause: new Error(
                    `Cannot fetch tools: HTTP ${response.statusCode ?? 0} ${data.slice(0, 200)}`,
                  ),
                }),
              )
              return
            }
            try {
              resolve(parseJson(data))
            } catch (error) {
              reject(new BurpDiscoveryError(this.config, { cause: error }))
            }
          })
        },
      )
      outgoing.on("error", (error: Error) => {
        reject(new BurpDiscoveryError(this.config, { cause: error }))
      })
      outgoing.end()
    })
    if (!Array.isArray(value) || !value.every((name) => typeof name === "string")) {
      throw new BurpDiscoveryError(this.config, {
        cause: new InvalidJsonError("Burp /tools must return an array of strings"),
      })
    }
    return value
  }
}

type ParsedBackendResponse = {
  readonly isJson: boolean
  readonly value: JsonValue
}

function parseBackendResponse(data: string): ParsedBackendResponse {
  try {
    return { isJson: true, value: parseJson(data) }
  } catch (error) {
    if (error instanceof InvalidJsonError) {
      return { isJson: false, value: { error: data } }
    }
    throw error
  }
}

function getErrorCode(error: Error): string | undefined {
  const code = Reflect.get(error, "code")
  return typeof code === "string" ? code : undefined
}
