import { z } from "zod"
import { isJsonObject, type JsonObject, type JsonValue } from "./types"

const HttpTransformInputSchema = z
  .object({
    message: z.string(),
    target: z.enum(["body", "header", "message"]).default("body"),
    headerName: z.string().min(1).optional(),
    recipe: z.custom<JsonValue>(),
    updateContentLength: z.boolean().default(true),
  })
  .strict()

type HttpMessage = {
  readonly startLine: string
  readonly headers: readonly string[]
  readonly body: string
  readonly newline: "\r\n" | "\n"
}

type Bake = (input: string, recipe: JsonValue) => Promise<JsonObject>

export async function transformHttpMessage(
  value: JsonValue,
  messageKey: "request" | "response",
  bake: Bake,
): Promise<JsonObject> {
  const source = z.record(z.string(), z.custom<JsonValue>()).parse(value)
  const parsed = HttpTransformInputSchema.safeParse({
    message: source[messageKey],
    target: source["target"],
    headerName: source["headerName"],
    recipe: source["recipe"],
    updateContentLength: source["updateContentLength"],
  })
  if (!parsed.success) {
    return {
      error: `Invalid CyberChef HTTP transform input: ${parsed.error.issues[0]?.message ?? "invalid input"}`,
    }
  }
  const message = parseHttpMessage(parsed.data.message)
  if (message === null) {
    return {
      error: `Invalid raw HTTP ${messageKey}: missing a start line and header/body separator`,
    }
  }
  if (parsed.data.target === "message") {
    const baked = await bake(parsed.data.message, parsed.data.recipe)
    const transformed = getTextOutput(baked)
    return transformed === null
      ? { error: "CyberChef HTTP message transforms must produce text" }
      : { [messageKey]: transformed, changed: { target: "message", contentLengthUpdated: false } }
  }
  if (parsed.data.target === "header") {
    return transformHeader(message, messageKey, parsed.data.headerName, parsed.data.recipe, bake)
  }
  const baked = await bake(message.body, parsed.data.recipe)
  const transformedBody = getTextOutput(baked)
  if (transformedBody === null) {
    return {
      error: "CyberChef HTTP body transforms must produce text; use cyberchef_bake for bytes",
    }
  }
  const canUpdateContentLength = parsed.data.updateContentLength && hasUnambiguousFraming(message)
  const updated = canUpdateContentLength
    ? updateContentLength(message, Buffer.byteLength(transformedBody))
    : message
  return {
    [messageKey]: serializeHttpMessage({ ...updated, body: transformedBody }),
    changed: {
      target: "body",
      bodyLengthBefore: Buffer.byteLength(message.body),
      bodyLengthAfter: Buffer.byteLength(transformedBody),
      contentLengthUpdated: canUpdateContentLength,
    },
  }
}

function parseHttpMessage(raw: string): HttpMessage | null {
  const newline = raw.includes("\r\n") ? "\r\n" : "\n"
  const separator = `${newline}${newline}`
  const boundary = raw.indexOf(separator)
  if (boundary < 0) {
    return null
  }
  const head = raw.slice(0, boundary).split(newline)
  const startLine = head[0]
  if (startLine === undefined || startLine.length === 0) {
    return null
  }
  return {
    startLine,
    headers: head.slice(1),
    body: raw.slice(boundary + separator.length),
    newline,
  }
}

async function transformHeader(
  message: HttpMessage,
  messageKey: "request" | "response",
  headerName: string | undefined,
  recipe: JsonValue,
  bake: Bake,
): Promise<JsonObject> {
  if (headerName === undefined) {
    return { error: "headerName is required when target is header" }
  }
  let matches = 0
  const headers: string[] = []
  for (const header of message.headers) {
    const colon = header.indexOf(":")
    if (colon < 1 || header.slice(0, colon).trim().toLowerCase() !== headerName.toLowerCase()) {
      headers.push(header)
      continue
    }
    matches += 1
    const baked = await bake(header.slice(colon + 1).trimStart(), recipe)
    const transformed = getTextOutput(baked)
    if (transformed === null) {
      return { error: "CyberChef HTTP header transforms must produce text" }
    }
    headers.push(`${header.slice(0, colon)}: ${transformed}`)
  }
  if (matches === 0) {
    return { error: `HTTP header not found: ${headerName}` }
  }
  return {
    [messageKey]: serializeHttpMessage({ ...message, headers }),
    changed: { target: "header", headerName, headersChanged: matches, contentLengthUpdated: false },
  }
}

function updateContentLength(message: HttpMessage, byteLength: number): HttpMessage {
  const firstIndex = message.headers.findIndex((header) => /^content-length\s*:/i.test(header))
  const headers = message.headers.filter((header) => !/^content-length\s*:/i.test(header))
  headers.splice(firstIndex < 0 ? headers.length : firstIndex, 0, `Content-Length: ${byteLength}`)
  return { ...message, headers }
}

function hasUnambiguousFraming(message: HttpMessage): boolean {
  if (message.headers.some((header) => /^\s|^transfer-encoding\s*:/i.test(header))) return false
  if (message.headers.some((header) => !/^[!#$%&'*+.^_`|~0-9A-Za-z-]+:\s*.*$/.test(header))) {
    return false
  }
  const lengths = message.headers.filter((header) => /^content-length\s*:/i.test(header))
  if (lengths.length > 1) return false
  if (lengths.length === 0) return true
  return /^\s*\d+\s*$/.test(lengths[0]?.slice(lengths[0].indexOf(":") + 1) ?? "")
}

function serializeHttpMessage(message: HttpMessage): string {
  return [message.startLine, ...message.headers, "", message.body].join(message.newline)
}

function getTextOutput(result: JsonObject): string | null {
  const output = result["output"]
  if (!isJsonObject(output)) {
    return null
  }
  return output["kind"] === "text" && typeof output["value"] === "string" ? output["value"] : null
}
