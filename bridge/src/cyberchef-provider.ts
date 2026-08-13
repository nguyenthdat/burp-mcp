import { z } from "zod"
import { transformHttpMessage } from "./cyberchef-http"
import { CyberChefRuntime, CyberChefRuntimeError, expectRuntimeObject } from "./cyberchef-runtime"
import { CYBERCHEF_WORKFLOW_DESCRIPTIONS, CYBERCHEF_WORKFLOW_SCHEMAS } from "./cyberchef-schemas"
import {
  isJsonObject,
  type JsonObject,
  type JsonValue,
  type ProviderTool,
  type ToolProvider,
} from "./types"

const OperationSchema = z.object({
  localName: z.string().min(1),
  name: z.string().min(1),
  description: z.string(),
  inputType: z.string(),
  outputType: z.string(),
  flowControl: z.boolean(),
  args: z.array(z.custom<JsonValue>()),
  inputSchema: z.custom<JsonObject>(),
})
const OperationsSchema = z.array(OperationSchema)
const Base64Schema = z
  .string()
  .refine(
    (value) =>
      /^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/.test(value) &&
      Buffer.from(value, "base64").toString("base64") === value,
    { message: "bytes.base64 must be canonical base64" },
  )
const DataInputSchema = z.union([
  z.string(),
  z.object({ kind: z.literal("text"), value: z.string() }).strict(),
  z.object({ kind: z.literal("bytes"), base64: Base64Schema }).strict(),
  z.object({ kind: z.literal("json"), value: z.custom<JsonValue>() }).strict(),
])
const RecipeIngredientSchema = z.union([
  z.string().min(1),
  z
    .object({
      op: z.string().min(1),
      args: z
        .union([
          z.array(z.custom<JsonValue>()),
          z.record(z.string(), z.custom<JsonValue>()),
          z.null(),
        ])
        .default(null),
    })
    .strict(),
])
const RecipeSchema = z.union([RecipeIngredientSchema, z.array(RecipeIngredientSchema).min(1)])
const BakeInputSchema = z.object({ input: DataInputSchema, recipe: RecipeSchema }).strict()
const SearchInputSchema = z
  .object({ query: z.string().min(1), limit: z.number().int().min(1).max(100).default(20) })
  .strict()
const BatchInputSchema = z
  .object({ inputs: z.array(DataInputSchema).min(1).max(100), recipe: RecipeSchema })
  .strict()
const MagicInputSchema = z
  .object({
    input: DataInputSchema,
    depth: z.number().int().min(1).max(10).default(3),
    intensiveMode: z.boolean().default(false),
    extensiveLanguageSupport: z.boolean().default(false),
    crib: z.string().default(""),
  })
  .strict()
const OperationInputSchema = z
  .object({
    input: DataInputSchema,
    arguments: z
      .union([z.array(z.custom<JsonValue>()), z.record(z.string(), z.custom<JsonValue>())])
      .default({}),
  })
  .strict()

export class CyberChefProvider implements ToolProvider {
  readonly namespace = "cyberchef"
  private tools: readonly ProviderTool[] | undefined
  private operations: ReadonlyMap<string, string> | undefined

  constructor(private readonly runtime = new CyberChefRuntime()) {}

  async listTools(): Promise<readonly ProviderTool[]> {
    if (this.tools !== undefined) {
      return this.tools
    }
    const value = await this.runtime.request("list", {})
    const operations = OperationsSchema.parse(value)
    const operationNames = new Map<string, string>()
    const operationTools = operations.map((operation) => {
      operationNames.set(operation.localName, operation.name)
      return {
        localName: operation.localName,
        description: operation.description,
        inputSchema: operation.inputSchema,
      }
    })
    const workflowTools = Object.entries(CYBERCHEF_WORKFLOW_SCHEMAS).map(
      ([localName, inputSchema]) => ({
        localName,
        description:
          CYBERCHEF_WORKFLOW_DESCRIPTIONS[
            localName as keyof typeof CYBERCHEF_WORKFLOW_DESCRIPTIONS
          ],
        inputSchema,
      }),
    )
    this.operations = operationNames
    this.tools = [...workflowTools, ...operationTools]
    return this.tools
  }

  async callTool(localName: string, arguments_: JsonValue): Promise<JsonValue> {
    try {
      await this.listTools()
      switch (localName) {
        case "bake":
          return await this.bake(arguments_)
        case "search_operations":
          return await this.search(arguments_)
        case "batch_bake":
          return await this.batchBake(arguments_)
        case "magic":
          return await this.runtime.request("magic", MagicInputSchema.parse(arguments_))
        case "transform_http_request":
          return await transformHttpMessage(arguments_, "request", (input, recipe) =>
            this.bakeObject(input, recipe),
          )
        case "transform_http_response":
          return await transformHttpMessage(arguments_, "response", (input, recipe) =>
            this.bakeObject(input, recipe),
          )
        default:
          return await this.callOperation(localName, arguments_)
      }
    } catch (error) {
      if (error instanceof z.ZodError) {
        return { error: `Invalid CyberChef input: ${error.issues[0]?.message ?? "invalid input"}` }
      }
      if (error instanceof CyberChefRuntimeError) {
        return { error: error.message }
      }
      throw error
    }
  }

  invalidate(): void {
    this.tools = undefined
    this.operations = undefined
    this.runtime.close()
  }

  close(): void {
    this.runtime.close()
  }

  private async bake(arguments_: JsonValue): Promise<JsonValue> {
    const input = BakeInputSchema.parse(arguments_)
    return this.runtime.request("bake", { input: input.input, recipe: input.recipe })
  }

  private async bakeObject(input: string, recipe: JsonValue): Promise<JsonObject> {
    return expectRuntimeObject(
      await this.runtime.request("bake", { input, recipe }),
      "CyberChef bake",
    )
  }

  private async search(arguments_: JsonValue): Promise<JsonValue> {
    const input = SearchInputSchema.parse(arguments_)
    return this.runtime.request("search", input)
  }

  private async batchBake(arguments_: JsonValue): Promise<JsonValue> {
    const input = BatchInputSchema.parse(arguments_)
    return this.runtime.request("batch", { inputs: input.inputs, recipe: input.recipe })
  }

  private async callOperation(localName: string, arguments_: JsonValue): Promise<JsonValue> {
    const operationName = this.operations?.get(localName)
    if (operationName === undefined) {
      return { error: `Unknown CyberChef tool: ${localName}` }
    }
    if (!isJsonObject(arguments_)) {
      return { error: "CyberChef operation arguments must be an object" }
    }
    const input = OperationInputSchema.parse(arguments_)
    return this.runtime.request("operation", { ...input, operationName })
  }
}
