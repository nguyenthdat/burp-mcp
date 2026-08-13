import { afterAll, beforeAll, describe, expect, test } from "bun:test"
import { CyberChefProvider } from "../src/cyberchef-provider"

describe("CyberChefProvider analysis workflows", () => {
  let provider: CyberChefProvider

  beforeAll(() => {
    provider = new CyberChefProvider()
  })

  afterAll(() => {
    provider.close()
  })

  test("bounds Magic details by default and reports truncation", async () => {
    // Given / When
    const result = await provider.callTool("magic", { input: "hello" })

    // Then
    expect(result).toMatchObject({
      cyberchefType: "JSON",
      output: {
        kind: "json",
        value: expect.any(Array),
        summary: {
          returnedEntryCount: 1,
          totalEntryCount: expect.any(Number),
          truncated: true,
          entryLimit: 20,
          languageScoreLimit: 5,
        },
      },
    })
    expect(JSON.stringify(result).length).toBeLessThan(100_000)
  })

  test("returns complete Magic details only when explicitly requested", async () => {
    // Given / When
    const result = await provider.callTool("magic", { input: "hello", fullDetails: true })

    // Then
    expect(result).toMatchObject({
      cyberchefType: "JSON",
      output: { kind: "json", value: expect.any(Array) },
    })
  })

  test("rejects invalid Base64 when strict mode is enabled", async () => {
    // Given / When
    const result = await provider.callTool("from_base64", {
      input: "%%%",
      arguments: { "Strict mode": true },
    })

    // Then
    expect(result).toMatchObject({
      error: expect.stringContaining("Base64 input contains non-alphabet char"),
    })
  })

  test("decodes valid Base64 when strict mode is enabled", async () => {
    // Given / When
    const result = await provider.callTool("from_base64", {
      input: "aGVsbG8=",
      arguments: { "Strict mode": true },
    })

    // Then
    expect(result).toMatchObject({
      cyberchefType: "byteArray",
      output: { kind: "bytes", base64: "aGVsbG8=", byteLength: 5 },
    })
  })

  test("rejects non-boolean strict mode values", async () => {
    // Given / When
    const result = await provider.callTool("from_base64", {
      input: "%%%",
      arguments: { "Strict mode": "true" },
    })

    // Then
    expect(result).toMatchObject({ error: expect.stringContaining("Strict mode must be boolean") })
  })

  test("keeps strict mode optional for positional arguments", async () => {
    // Given / When
    const result = await provider.callTool("from_base64", {
      input: "aGVsbG8=",
      arguments: ["A-Za-z0-9+/=", true],
    })

    // Then
    expect(result).toMatchObject({
      cyberchefType: "byteArray",
      output: { kind: "bytes", base64: "aGVsbG8=", byteLength: 5 },
    })
  })
})
