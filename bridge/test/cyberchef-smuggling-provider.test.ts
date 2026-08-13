import { afterAll, beforeAll, describe, expect, test } from "bun:test"
import { CyberChefProvider } from "../src/cyberchef-provider"

describe("CyberChefProvider request-smuggling framing", () => {
  let provider: CyberChefProvider

  beforeAll(() => {
    provider = new CyberChefProvider()
  })

  afterAll(() => {
    provider.close()
  })

  async function transform(request: string) {
    return provider.callTool("transform_http_request", {
      request,
      target: "body",
      recipe: ["To Base64"],
    })
  }

  test("preserves Transfer-Encoding framing", async () => {
    const request =
      "POST / HTTP/1.1\r\nHost: example.test\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhello\r\n0\r\n\r\n"
    expect(await transform(request)).toEqual({
      request:
        "POST / HTTP/1.1\r\nHost: example.test\r\nTransfer-Encoding: chunked\r\n\r\nNQ0KaGVsbG8NCjANCg0K",
      changed: {
        target: "body",
        bodyLengthBefore: 15,
        bodyLengthAfter: 20,
        contentLengthUpdated: false,
      },
    })
  })

  test("preserves conflicting Content-Length framing", async () => {
    const request =
      "POST / HTTP/1.1\r\nHost: example.test\r\nContent-Length: 5\r\nContent-Length: 4\r\n\r\nhello"
    expect(await transform(request)).toMatchObject({
      request: expect.stringContaining("Content-Length: 5\r\nContent-Length: 4\r\n\r\naGVsbG8="),
      changed: { contentLengthUpdated: false },
    })
  })

  test("preserves equal duplicate Content-Length framing", async () => {
    const request =
      "POST / HTTP/1.1\r\nHost: example.test\r\nContent-Length: 5\r\nContent-Length: 5\r\n\r\nhello"
    expect(await transform(request)).toMatchObject({
      request: expect.stringContaining("Content-Length: 5\r\nContent-Length: 5\r\n\r\naGVsbG8="),
      changed: { contentLengthUpdated: false },
    })
  })

  test("preserves CL TE header order and values", async () => {
    const request =
      "POST / HTTP/1.1\r\nHost: example.test\r\nContent-Length: 5\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhello\r\n0\r\n\r\n"
    expect(await transform(request)).toMatchObject({
      request: expect.stringContaining(
        "Content-Length: 5\r\nTransfer-Encoding: chunked\r\n\r\nNQ0KaGVsbG8NCjANCg0K",
      ),
      changed: { contentLengthUpdated: false },
    })
  })

  test("preserves TE CL header order and values", async () => {
    const request =
      "POST / HTTP/1.1\r\nHost: example.test\r\nTransfer-Encoding: chunked\r\nContent-Length: 5\r\n\r\n5\r\nhello\r\n0\r\n\r\n"
    expect(await transform(request)).toMatchObject({
      request: expect.stringContaining(
        "Transfer-Encoding: chunked\r\nContent-Length: 5\r\n\r\nNQ0KaGVsbG8NCjANCg0K",
      ),
      changed: { contentLengthUpdated: false },
    })
  })

  test("preserves comma-separated Content-Length values", async () => {
    const request = "POST / HTTP/1.1\r\nHost: example.test\r\nContent-Length: 5, 4\r\n\r\nhello"
    expect(await transform(request)).toMatchObject({
      request: expect.stringContaining("Content-Length: 5, 4\r\n\r\naGVsbG8="),
      changed: { contentLengthUpdated: false },
    })
  })

  test("preserves folded Transfer-Encoding framing", async () => {
    const request =
      "POST / HTTP/1.1\r\nHost: example.test\r\n Transfer-Encoding: chunked\r\nContent-Length: 5\r\n\r\nhello"
    expect(await transform(request)).toMatchObject({
      request: expect.stringContaining(
        " Transfer-Encoding: chunked\r\nContent-Length: 5\r\n\r\naGVsbG8=",
      ),
      changed: { contentLengthUpdated: false },
    })
  })

  test("preserves malformed Transfer-Encoding field names", async () => {
    const request =
      "POST / HTTP/1.1\r\nHost: example.test\r\nTransfer-Encoding : chunked\r\nContent-Length: 5\r\n\r\nhello"
    expect(await transform(request)).toMatchObject({
      request: expect.stringContaining(
        "Transfer-Encoding : chunked\r\nContent-Length: 5\r\n\r\naGVsbG8=",
      ),
      changed: { contentLengthUpdated: false },
    })
  })

  test("preserves malformed Content-Length field names", async () => {
    const request = "POST / HTTP/1.1\r\nHost: example.test\r\nContent-Length : 5\r\n\r\nhello"
    expect(await transform(request)).toMatchObject({
      request: expect.stringContaining("Content-Length : 5\r\n\r\naGVsbG8="),
      changed: { contentLengthUpdated: false },
    })
  })

  test("adds Content-Length only when framing is unambiguous", async () => {
    const request = "POST / HTTP/1.1\r\nHost: example.test\r\n\r\nhello"
    expect(await transform(request)).toMatchObject({
      request: "POST / HTTP/1.1\r\nHost: example.test\r\nContent-Length: 8\r\n\r\naGVsbG8=",
      changed: { contentLengthUpdated: true },
    })
  })
})
