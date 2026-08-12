import { expect, test } from "bun:test"
import { Readable, Writable } from "node:stream"
import { runStdio } from "../src/stdio"
import { isJsonObject, type JsonObject, type JsonValue } from "../src/types"

class InitializeDispatcher {
  async handle(message: JsonValue): Promise<JsonObject> {
    const id = isJsonObject(message) ? (message["id"] ?? null) : null
    return { jsonrpc: "2.0", id, result: { ok: true } }
  }
}

test("drains every response before completing after stdin closes", async () => {
  // Given
  const requestCount = 2_000
  const input = Readable.from(
    Array.from({ length: requestCount }, (_, id) => `${JSON.stringify({ id })}\n`),
  )
  let output = ""
  const slowOutput = new Writable({
    highWaterMark: 1,
    write(chunk, _encoding, callback) {
      setImmediate(() => {
        output += chunk.toString()
        callback()
      })
    },
  })

  // When
  await runStdio(new InitializeDispatcher(), input, slowOutput)

  // Then
  const responses = output.trim().split("\n")
  expect(responses).toHaveLength(requestCount)
  expect(JSON.parse(responses[0] ?? "null").id).toBe(0)
  expect(JSON.parse(responses.at(-1) ?? "null").id).toBe(requestCount - 1)
})
