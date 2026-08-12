import type { JsonValue } from "./types"

export class InvalidJsonError extends Error {
  readonly name = "InvalidJsonError"
}

export function parseJson(text: string): JsonValue {
  let value: unknown
  try {
    value = JSON.parse(text)
  } catch (error) {
    throw new InvalidJsonError("Invalid JSON", { cause: error })
  }
  if (!isJsonValue(value)) {
    throw new InvalidJsonError("JSON contains an unsupported value")
  }
  return value
}

function isJsonValue(value: unknown): value is JsonValue {
  const pending: unknown[] = [value]
  while (pending.length > 0) {
    const current = pending.pop()
    if (current === null || typeof current === "string" || typeof current === "boolean") {
      continue
    }
    if (typeof current === "number") {
      if (!Number.isFinite(current)) {
        return false
      }
      continue
    }
    if (Array.isArray(current)) {
      for (const item of current) {
        pending.push(item)
      }
      continue
    }
    if (typeof current !== "object") {
      return false
    }
    for (const item of Object.values(current)) {
      pending.push(item)
    }
  }
  return true
}
