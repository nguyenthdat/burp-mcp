import { afterAll, beforeAll, describe, expect, test } from "bun:test"
import { CyberChefProvider } from "../src/cyberchef-provider"

describe("CyberChefProvider", () => {
  let provider: CyberChefProvider

  beforeAll(() => {
    provider = new CyberChefProvider()
  })

  afterAll(() => {
    provider.close()
  })

  test("advertises workflow tools and every supported Node operation", async () => {
    // Given / When
    const tools = await provider.listTools()
    const names = tools.map((tool) => tool.localName)

    // Then
    expect(names).toContain("bake")
    expect(names).toContain("search_operations")
    expect(names).toContain("batch_bake")
    expect(names).toContain("magic")
    expect(names).toContain("transform_http_request")
    expect(names).toContain("transform_http_response")
    expect(names).toContain("to_base64")
    expect(names).toContain("aes_decrypt")
    expect(names.length).toBeGreaterThan(380)
    expect(new Set(names).size).toBe(names.length)
  })

  test("runs a CyberChef recipe on text data", async () => {
    // Given / When
    const result = await provider.callTool("bake", {
      input: "hello",
      recipe: ["To Base64"],
    })

    // Then
    expect(result).toMatchObject({
      cyberchefType: "string",
      output: { kind: "text", value: "aGVsbG8=" },
    })
  })

  test("runs a generated operation tool with operation arguments", async () => {
    // Given / When
    const result = await provider.callTool("to_base64", {
      input: "hello",
      arguments: {},
    })

    // Then
    expect(result).toMatchObject({ output: { kind: "text", value: "aGVsbG8=" } })
  })

  test("returns binary output as lossless base64", async () => {
    // Given / When
    const result = await provider.callTool("from_base64", {
      input: "/wA=",
      arguments: {},
    })

    // Then
    expect(result).toMatchObject({
      cyberchefType: "byteArray",
      output: { kind: "bytes", base64: "/wA=", byteLength: 2 },
    })
  })

  test("searches operation metadata", async () => {
    // Given / When
    const result = await provider.callTool("search_operations", {
      query: "base64",
      limit: 3,
    })

    // Then
    expect(result).toMatchObject({
      matches: expect.arrayContaining([
        expect.objectContaining({ name: "To Base64", toolName: "to_base64" }),
      ]),
    })
  })

  test("bakes multiple inputs independently", async () => {
    // Given / When
    const result = await provider.callTool("batch_bake", {
      inputs: ["hello", "world"],
      recipe: ["To Base64"],
    })

    // Then
    expect(result).toMatchObject({
      results: [
        { success: true, output: { kind: "text", value: "aGVsbG8=" } },
        { success: true, output: { kind: "text", value: "d29ybGQ=" } },
      ],
    })
  })

  test("loads crypto operations lazily", async () => {
    // Given / When
    const result = await provider.callTool("sha2", {
      input: "hello",
      arguments: { size: 256, rounds: 64 },
    })

    // Then
    expect(result).toMatchObject({
      output: {
        kind: "text",
        value: "e378c29a879c765ab711ec0a800b899ccdb78f7ed9a2a1563ed768ff7f94b7b7",
      },
    })
  })

  test("loads accented operation names through their actual files", async () => {
    // Given / When
    const result = await provider.callTool("vigenere_encode", {
      input: "ATTACKATDAWN",
      arguments: { key: "LEMON" },
    })

    // Then
    expect(result).toMatchObject({ output: { kind: "text", value: "LXFOPVEFRNHR" } })
  })

  test("awaits promise-returning operations", async () => {
    // Given / When
    const result = await provider.callTool("bzip2_compress", {
      input: "hello",
      arguments: { "block size (100s of kb)": 9, "work factor": 30 },
    })

    // Then
    expect(result).toMatchObject({
      cyberchefType: "ArrayBuffer",
      output: { kind: "bytes" },
    })
    expect(JSON.stringify(result)).not.toContain("{}")
  })

  test("serializes File-producing operations", async () => {
    // Given / When
    const result = await provider.callTool("zip", {
      input: "hello",
      arguments: { filename: "hello.txt" },
    })

    // Then
    expect(result).toMatchObject({
      cyberchefType: "File",
      output: {
        kind: "json",
        value: [expect.objectContaining({ name: "hello.txt", base64: expect.any(String) })],
      },
    })
  })

  test("does not advertise unsafe or browser-only operations", async () => {
    // Given / When
    const tools = await provider.listTools()
    const names = tools.map((tool) => tool.localName)

    // Then
    expect(names).not.toContain("http_request")
    expect(names).not.toContain("dns_over_https")
    expect(names).not.toContain("optical_character_recognition")
  })

  test("marks filtered operations unsupported in search", async () => {
    // Given / When
    const result = await provider.callTool("search_operations", {
      query: "HTTP request",
      limit: 5,
    })

    // Then
    expect(result).toMatchObject({
      matches: expect.arrayContaining([
        expect.objectContaining({ name: "HTTP request", supported: false }),
      ]),
    })
  })

  test("rejects unsafe operations inside recipes", async () => {
    // Given / When
    const result = await provider.callTool("bake", {
      input: "hello",
      recipe: ["HTTP request"],
    })

    // Then
    expect(result).toEqual({
      error: "CyberChef recipe failed: Unsupported CyberChef operation: HTTP request",
    })
  })

  test("returns actionable tool errors for invalid recipes", async () => {
    // Given / When
    const result = await provider.callTool("bake", {
      input: "hello",
      recipe: ["Not A CyberChef Operation"],
    })

    // Then
    expect(result).toEqual({
      error: "CyberChef recipe failed: Unsupported CyberChef operation: Not A CyberChef Operation",
    })
  })
})
