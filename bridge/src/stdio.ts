import { createInterface } from "node:readline"
import { InvalidJsonError, parseJson } from "./json"
import { type RpcResponse, rpcError } from "./rpc"
import { isJsonObject, type JsonValue } from "./types"

export interface RpcHandler {
  handle(message: JsonValue): Promise<RpcResponse>
}

export function runStdio(
  dispatcher: RpcHandler,
  input: NodeJS.ReadableStream = process.stdin,
  output: NodeJS.WritableStream = process.stdout,
): Promise<void> {
  const lines = createInterface({ input, terminal: false })
  const writer = new LineWriter(output)
  let pending = 0
  let inputClosed = false
  let resolveCompletion: (() => void) | undefined
  const completion = new Promise<void>((resolve) => {
    resolveCompletion = resolve
  })

  const finishWhenDrained = (): void => {
    if (!inputClosed || pending !== 0) {
      return
    }
    writer.drain().then(() => resolveCompletion?.())
  }

  lines.on("line", (line) => {
    if (!line.trim()) {
      return
    }
    let message: JsonValue
    try {
      message = parseJson(line)
    } catch (error) {
      if (error instanceof InvalidJsonError) {
        writer.write(`${JSON.stringify(rpcError(null, -32700, "Parse error"))}\n`)
        return
      }
      throw error
    }

    pending += 1
    dispatcher
      .handle(message)
      .then((response) => {
        if (response !== null) {
          writer.write(`${JSON.stringify(response)}\n`)
        }
      })
      .catch((error) => {
        const id = isJsonObject(message) ? (message["id"] ?? null) : null
        const errorMessage = error instanceof Error ? error.message : "Handler error"
        writer.write(`${JSON.stringify(rpcError(id, -1, errorMessage))}\n`)
      })
      .finally(() => {
        pending -= 1
        finishWhenDrained()
      })
  })

  lines.on("close", () => {
    inputClosed = true
    finishWhenDrained()
  })
  return completion
}

class LineWriter {
  private queue: Promise<void> = Promise.resolve()

  constructor(private readonly output: NodeJS.WritableStream) {}

  write(line: string): void {
    this.queue = this.queue.then(
      () =>
        new Promise<void>((resolve) => {
          if (this.output.write(line)) {
            resolve()
          } else {
            this.output.once("drain", resolve)
          }
        }),
    )
  }

  async drain(): Promise<void> {
    await this.queue
  }
}
