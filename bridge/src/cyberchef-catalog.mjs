import { createRequire } from "node:module"
import { readdirSync } from "node:fs"
import { dirname, join } from "node:path"

const require = createRequire(import.meta.url)
const cyberchefRoot = dirname(require.resolve("cyberchef-node/package.json"))
const operationConfig = require(join(cyberchefRoot, "src/core/config/OperationConfig.json"))
const unsupportedOperations = new Set([
  "DNS over HTTPS",
  "HTTP request",
  "Optical Character Recognition",
])
const operationFiles = new Map(
  readdirSync(join(cyberchefRoot, "src/core/operations"))
    .filter((file) => file.endsWith(".js"))
    .map((file) => [sanitize(file.slice(0, -3)), file]),
)

export function listOperations() {
  return Object.entries(operationConfig)
    .filter(([name, operation]) => isSupported(name, operation))
    .map(([name, operation]) => ({
      localName: toToolName(name),
      name,
      description: stripHtml(operation.description ?? name),
      inputType: operation.inputType ?? "unknown",
      outputType: operation.outputType ?? "unknown",
      flowControl: Boolean(operation.flowControl),
      args: operation.args ?? [],
      inputSchema: operationInputSchema(operation.args ?? []),
    }))
}

export function searchOperations(query, limit) {
  const sanitizedQuery = sanitize(query)
  const matches = Object.entries(operationConfig)
    .filter(
      ([name, operation]) =>
        sanitize(name).includes(sanitizedQuery) ||
        sanitize(operation.description ?? "").includes(sanitizedQuery),
    )
    .sort(([leftName], [rightName]) => {
      const leftMatches = sanitize(leftName).includes(sanitizedQuery) ? 1 : 0
      const rightMatches = sanitize(rightName).includes(sanitizedQuery) ? 1 : 0
      return rightMatches - leftMatches
    })
  return {
    matches: matches.slice(0, limit).map(([name, operation]) => ({
      name,
      toolName: toToolName(name),
      description: stripHtml(operation.description ?? ""),
      inputType: operation.inputType,
      outputType: operation.outputType,
      flowControl: Boolean(operation.flowControl),
      supported: isSupported(name, operation),
      args: operation.args ?? [],
    })),
  }
}

export function getOperationDescriptor(name) {
  const entry = Object.entries(operationConfig).find(
    ([candidate]) => sanitize(candidate) === sanitize(name),
  )
  if (!entry) return null
  const [canonicalName, operation] = entry
  if (!isSupported(canonicalName, operation)) return null
  const file = operationFiles.get(sanitize(canonicalName))
  return file ? { name: canonicalName, operation, file } : null
}

function operationInputSchema(args) {
  return {
    type: "object",
    properties: {
      input: {
        oneOf: [
          { type: "string" },
          {
            type: "object",
            properties: { kind: { const: "text" }, value: { type: "string" } },
            required: ["kind", "value"],
            additionalProperties: false,
          },
          {
            type: "object",
            properties: { kind: { const: "bytes" }, base64: { type: "string" } },
            required: ["kind", "base64"],
            additionalProperties: false,
          },
          {
            type: "object",
            properties: { kind: { const: "json" }, value: {} },
            required: ["kind", "value"],
            additionalProperties: false,
          },
        ],
      },
      arguments: {
        oneOf: [
          { type: "object", additionalProperties: true },
          { type: "array", items: {} },
        ],
        description: "Named arguments from cyberchef_search_operations or positional values",
      },
    },
    required: ["input"],
    additionalProperties: false,
    $comment: args.length === 0 ? "This operation has no configurable arguments" : undefined,
  }
}

function toToolName(name) {
  return name
    .normalize("NFKD")
    .replace(/\p{M}/gu, "")
    .replace(/([a-z0-9])([A-Z])/g, "$1_$2")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "_")
    .replace(/^_+|_+$/g, "")
}

function stripHtml(value) {
  return value.replace(/<[^>]*>/g, " ").replace(/\s+/g, " ").trim()
}

function sanitize(value) {
  return value.normalize("NFKD").toLowerCase().replace(/[^a-z0-9]/g, "")
}

function isSupported(name, operation) {
  return (
    !operation.flowControl &&
    !operation.manualBake &&
    !unsupportedOperations.has(name) &&
    operationFiles.has(sanitize(name))
  )
}
