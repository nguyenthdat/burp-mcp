import { expect, test } from "bun:test"
import { parseJson } from "../src/json"

test("parses deeply nested JSON without overflowing the call stack", () => {
  // Given
  const depth = 10_000
  const input = `${"[".repeat(depth)}null${"]".repeat(depth)}`

  // When
  const value = parseJson(input)

  // Then
  expect(Array.isArray(value)).toBe(true)
})
