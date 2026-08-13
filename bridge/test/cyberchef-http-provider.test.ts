import { afterAll, beforeAll, describe, expect, test } from "bun:test"
import { CyberChefProvider } from "../src/cyberchef-provider"
import { CyberChefRuntime } from "../src/cyberchef-runtime"

describe("CyberChefProvider HTTP and validation", () => {
  let provider: CyberChefProvider

  beforeAll(() => {
    provider = new CyberChefProvider()
  })

  afterAll(() => {
    provider.close()
  })

  test("transforms an HTTP request body and updates Content-Length", async () => {
    // Given
    const request =
      "POST /submit HTTP/1.1\r\nHost: example.test\r\nContent-Length: 5\r\nX-Test: keep\r\n\r\nhello"

    // When
    const result = await provider.callTool("transform_http_request", {
      request,
      target: "body",
      recipe: ["To Base64"],
    })

    // Then
    expect(result).toMatchObject({
      request:
        "POST /submit HTTP/1.1\r\nHost: example.test\r\nContent-Length: 8\r\nX-Test: keep\r\n\r\naGVsbG8=",
      changed: { target: "body", contentLengthUpdated: true },
    })
  })

  test("transforms an HTTP response header without changing the body", async () => {
    // Given
    const response =
      "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nX-Token: hello\r\nContent-Length: 4\r\n\r\nbody"

    // When
    const result = await provider.callTool("transform_http_response", {
      response,
      target: "header",
      headerName: "X-Token",
      recipe: ["To Base64"],
    })

    // Then
    expect(result).toMatchObject({
      response:
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nX-Token: aGVsbG8=\r\nContent-Length: 4\r\n\r\nbody",
      changed: { target: "header", contentLengthUpdated: false },
    })
  })

  test("rejects body transforms with Transfer-Encoding", async () => {
    // Given
    const request =
      "POST / HTTP/1.1\r\nHost: example.test\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhello\r\n0\r\n\r\n"

    // When
    const result = await provider.callTool("transform_http_request", {
      request,
      target: "body",
      recipe: ["To Base64"],
    })

    // Then
    expect(result).toEqual({
      error: "Cannot transform an HTTP body with Transfer-Encoding; decode the framing first",
    })
  })

  test("rejects conflicting duplicate Content-Length headers", async () => {
    // Given
    const request =
      "POST / HTTP/1.1\r\nHost: example.test\r\nContent-Length: 5\r\nContent-Length: 4\r\n\r\nhello"

    // When
    const result = await provider.callTool("transform_http_request", {
      request,
      target: "body",
      recipe: ["To Base64"],
    })

    // Then
    expect(result).toEqual({ error: "Conflicting Content-Length headers in raw HTTP request" })
  })

  test("validates Magic depth and canonical base64", async () => {
    // Given / When
    const magic = await provider.callTool("magic", { input: "hello", depth: 11 })
    const bytes = await provider.callTool("bake", {
      input: { kind: "bytes", base64: "%%%" },
      recipe: ["To Base64"],
    })

    // Then
    expect(magic).toMatchObject({ error: expect.stringContaining("Invalid CyberChef input") })
    expect(bytes).toMatchObject({ error: expect.stringContaining("Invalid CyberChef input") })
  })

  test("validates generated operation input", async () => {
    // Given / When
    const result = await provider.callTool("to_base64", {
      input: { kind: "bytes", base64: "%%%" },
    })

    // Then
    expect(result).toMatchObject({ error: expect.stringContaining("Invalid CyberChef input") })
  })

  test("restarts cleanly after invalidation", async () => {
    // Given
    const runtime = new CyberChefRuntime()
    const isolated = new CyberChefProvider(runtime)

    // When
    await isolated.listTools()
    isolated.invalidate()
    const result = await isolated.callTool("bake", { input: "hello", recipe: ["To Base64"] })
    isolated.close()

    // Then
    expect(result).toMatchObject({ output: { kind: "text", value: "aGVsbG8=" } })
  })
})
