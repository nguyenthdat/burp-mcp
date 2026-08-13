import type { JsonObject } from "./types"

const DATA_SCHEMA = {
  oneOf: [
    { type: "string", description: "UTF-8 text input" },
    {
      type: "object",
      properties: {
        kind: { const: "text" },
        value: { type: "string" },
      },
      required: ["kind", "value"],
      additionalProperties: false,
    },
    {
      type: "object",
      properties: {
        kind: { const: "bytes" },
        base64: {
          type: "string",
          pattern: "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$",
          description: "Canonical base64-encoded arbitrary bytes",
        },
      },
      required: ["kind", "base64"],
      additionalProperties: false,
    },
    {
      type: "object",
      properties: {
        kind: { const: "json" },
        value: {},
      },
      required: ["kind", "value"],
      additionalProperties: false,
    },
  ],
} as const

const RECIPE_SCHEMA = {
  oneOf: [
    { type: "string" },
    {
      type: "object",
      properties: { op: { type: "string" }, args: {} },
      required: ["op"],
      additionalProperties: false,
    },
    {
      type: "array",
      minItems: 1,
      items: {
        oneOf: [
          { type: "string" },
          {
            type: "object",
            properties: { op: { type: "string" }, args: {} },
            required: ["op"],
            additionalProperties: false,
          },
        ],
      },
    },
  ],
} as const

export const CYBERCHEF_WORKFLOW_SCHEMAS = {
  bake: {
    type: "object",
    properties: { input: DATA_SCHEMA, recipe: RECIPE_SCHEMA },
    required: ["input", "recipe"],
    additionalProperties: false,
  },
  search_operations: {
    type: "object",
    properties: {
      query: { type: "string", minLength: 1 },
      limit: { type: "integer", minimum: 1, maximum: 100, default: 20 },
    },
    required: ["query"],
    additionalProperties: false,
  },
  batch_bake: {
    type: "object",
    properties: {
      inputs: { type: "array", minItems: 1, maxItems: 100, items: DATA_SCHEMA },
      recipe: RECIPE_SCHEMA,
    },
    required: ["inputs", "recipe"],
    additionalProperties: false,
  },
  magic: {
    type: "object",
    properties: {
      input: DATA_SCHEMA,
      depth: { type: "integer", minimum: 1, maximum: 10, default: 3 },
      intensiveMode: { type: "boolean", default: false },
      extensiveLanguageSupport: { type: "boolean", default: false },
      crib: { type: "string", default: "" },
    },
    required: ["input"],
    additionalProperties: false,
  },
  transform_http_request: httpTransformSchema("request"),
  transform_http_response: httpTransformSchema("response"),
} as const satisfies Readonly<Record<string, JsonObject>>

export const CYBERCHEF_WORKFLOW_DESCRIPTIONS = {
  bake: "Run any CyberChef Node-compatible recipe on text, JSON, or lossless base64 bytes.",
  search_operations:
    "Search CyberChef operations and inspect their input, output, and argument metadata.",
  batch_bake: "Run one CyberChef recipe over multiple independent inputs.",
  magic: "Use CyberChef Magic to detect likely encodings and useful follow-up operations.",
  transform_http_request:
    "Apply a CyberChef recipe to a raw HTTP request body, selected header values, or the complete message.",
  transform_http_response:
    "Apply a CyberChef recipe to a raw HTTP response body, selected header values, or the complete message.",
} as const satisfies Readonly<Record<string, string>>

function httpTransformSchema(messageKey: "request" | "response"): JsonObject {
  return {
    type: "object",
    properties: {
      [messageKey]: { type: "string" },
      target: { type: "string", enum: ["body", "header", "message"], default: "body" },
      headerName: { type: "string" },
      recipe: RECIPE_SCHEMA,
      updateContentLength: { type: "boolean", default: true },
    },
    required: [messageKey, "recipe"],
    additionalProperties: false,
  }
}
