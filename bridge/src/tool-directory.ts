import { type JsonValue, type McpTool, ProviderUnavailableError, type ToolProvider } from "./types"

export class ToolDirectoryError extends Error {
  readonly name = "ToolDirectoryError"
}

export class ToolDirectory {
  private readonly providersByNamespace: ReadonlyMap<string, ToolProvider>
  private readonly providersByPrefixLength: readonly ToolProvider[]

  constructor(
    private readonly providers: readonly ToolProvider[],
    private readonly compatibilityNamespace: string,
  ) {
    const providersByNamespace = new Map<string, ToolProvider>()
    for (const provider of providers) {
      if (providersByNamespace.has(provider.namespace)) {
        throw new ToolDirectoryError(`Duplicate tool provider namespace: ${provider.namespace}`)
      }
      providersByNamespace.set(provider.namespace, provider)
    }
    if (!providersByNamespace.has(compatibilityNamespace)) {
      throw new ToolDirectoryError(
        `Missing compatibility provider namespace: ${compatibilityNamespace}`,
      )
    }
    this.providersByNamespace = providersByNamespace
    this.providersByPrefixLength = Array.from(providers).sort(
      (left, right) => right.namespace.length - left.namespace.length,
    )
  }

  async listTools(): Promise<readonly McpTool[]> {
    const settledProviderTools = await Promise.allSettled(
      this.providers.map(async (provider) => ({ provider, tools: await provider.listTools() })),
    )
    const providerTools: {
      readonly provider: ToolProvider
      readonly tools: Awaited<ReturnType<ToolProvider["listTools"]>>
    }[] = []
    const unavailableProviders: ProviderUnavailableError[] = []
    for (const result of settledProviderTools) {
      if (result.status === "fulfilled") {
        providerTools.push(result.value)
      } else if (result.reason instanceof ProviderUnavailableError) {
        unavailableProviders.push(result.reason)
      } else {
        throw result.reason
      }
    }
    if (providerTools.length === 0) {
      const firstUnavailable = unavailableProviders[0]
      if (firstUnavailable !== undefined) {
        throw firstUnavailable
      }
    }
    const names = new Set<string>()
    return providerTools.flatMap(({ provider, tools }) =>
      tools.map((tool) => {
        const name = `${provider.namespace}_${tool.localName}`
        if (names.has(name)) {
          throw new ToolDirectoryError(`Duplicate public tool name: ${name}`)
        }
        names.add(name)
        return { name, description: tool.description, inputSchema: tool.inputSchema }
      }),
    )
  }

  async callTool(publicName: string, arguments_: JsonValue): Promise<JsonValue> {
    const resolved = this.resolve(publicName)
    await resolved.provider.listTools()
    return resolved.provider.callTool(resolved.localName, arguments_)
  }

  invalidate(publicName: string): void {
    this.resolve(publicName).provider.invalidate()
  }

  private resolve(publicName: string): {
    readonly provider: ToolProvider
    readonly localName: string
  } {
    for (const provider of this.providersByPrefixLength) {
      const prefix = `${provider.namespace}_`
      if (publicName.startsWith(prefix)) {
        return { provider, localName: publicName.slice(prefix.length) }
      }
    }
    const provider = this.providersByNamespace.get(this.compatibilityNamespace)
    if (provider === undefined) {
      throw new ToolDirectoryError(
        `Missing compatibility provider namespace: ${this.compatibilityNamespace}`,
      )
    }
    return { provider, localName: publicName }
  }
}
