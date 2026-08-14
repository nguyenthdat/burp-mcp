import { type ChildProcessWithoutNullStreams, spawn } from "node:child_process"
import { createInterface } from "node:readline"
import { fileURLToPath } from "node:url"
import { parseJson } from "./json"
import { isJsonObject, type JsonObject, type JsonValue } from "./types"

type PendingRequest = {
  readonly generation: number
  readonly timeout: ReturnType<typeof setTimeout>
  readonly resolve: (value: JsonValue) => void
  readonly reject: (error: Error) => void
}

type WorkerProcess = {
  readonly child: ChildProcessWithoutNullStreams
  readonly generation: number
}

export class CyberChefRuntimeError extends Error {
  override readonly name = "CyberChefRuntimeError"
}

export class CyberChefRuntime {
  private process: WorkerProcess | undefined
  private generation = 0
  private nextId = 1
  private readonly pending = new Map<number, PendingRequest>()

  constructor(
    private readonly timeoutMs = 30_000,
    private readonly maxMessageBytes = 10 * 1024 * 1024,
    private readonly runtimeCommand = process.execPath,
  ) {}

  async request(method: string, params: JsonValue): Promise<JsonValue> {
    const process = this.ensureProcess()
    const id = this.nextId
    this.nextId += 1
    const payload = `${JSON.stringify({ id, method, params })}\n`
    if (Buffer.byteLength(payload) > this.maxMessageBytes) {
      throw new CyberChefRuntimeError("CyberChef request exceeds the 10 MiB limit")
    }
    const response = new Promise<JsonValue>((resolve, reject) => {
      const timeout = setTimeout(() => {
        const pending = this.pending.get(id)
        if (pending?.generation !== process.generation) return
        this.pending.delete(id)
        reject(new CyberChefRuntimeError(`CyberChef request timed out after ${this.timeoutMs}ms`))
        this.stopGeneration(process.generation)
      }, this.timeoutMs)
      this.pending.set(id, { generation: process.generation, timeout, resolve, reject })
    })
    process.child.stdin.write(payload, (error) => {
      if (error) {
        this.failGeneration(
          process.generation,
          new CyberChefRuntimeError("Cannot write to CyberChef runtime", { cause: error }),
        )
      }
    })
    return response
  }

  close(): void {
    const process = this.process
    this.process = undefined
    if (process === undefined) return
    this.failGeneration(process.generation, new CyberChefRuntimeError("CyberChef runtime closed"))
    process.child.kill()
  }

  private ensureProcess(): WorkerProcess {
    if (this.process !== undefined) return this.process
    this.generation += 1
    const generation = this.generation
    const worker = fileURLToPath(new URL("./cyberchef-worker.mjs", import.meta.url))
    const child = spawn(this.runtimeCommand, [worker], {
      stdio: ["pipe", "pipe", "pipe"],
    })
    createInterface({ input: child.stdout }).on("line", (line) => this.handleLine(line, generation))
    child.stderr.resume()
    child.on("error", (error) => {
      this.failGeneration(
        generation,
        new CyberChefRuntimeError("Cannot start CyberChef runtime", { cause: error }),
      )
      if (this.process?.child === child) this.process = undefined
    })
    child.on("exit", (code, signal) => {
      this.failGeneration(
        generation,
        new CyberChefRuntimeError(
          `CyberChef runtime exited (${signal === null ? `code ${code ?? "unknown"}` : signal})`,
        ),
      )
      if (this.process?.child === child) this.process = undefined
    })
    this.process = { child, generation }
    return this.process
  }

  private handleLine(line: string, generation: number): void {
    if (Buffer.byteLength(line) > this.maxMessageBytes) {
      this.failGeneration(
        generation,
        new CyberChefRuntimeError("CyberChef response exceeds 10 MiB"),
      )
      this.stopGeneration(generation)
      return
    }
    let response: JsonValue
    try {
      response = parseJson(line)
    } catch (error) {
      this.failGeneration(
        generation,
        new CyberChefRuntimeError("CyberChef runtime returned invalid JSON", { cause: error }),
      )
      this.stopGeneration(generation)
      return
    }
    if (!isJsonObject(response) || typeof response["id"] !== "number") {
      this.failGeneration(
        generation,
        new CyberChefRuntimeError("CyberChef runtime returned an invalid response"),
      )
      this.stopGeneration(generation)
      return
    }
    const pending = this.pending.get(response["id"])
    if (pending === undefined || pending.generation !== generation) return
    this.pending.delete(response["id"])
    clearTimeout(pending.timeout)
    if (typeof response["error"] === "string") {
      pending.reject(new CyberChefRuntimeError(response["error"]))
      return
    }
    pending.resolve(response["result"] ?? null)
  }

  private failGeneration(generation: number, error: CyberChefRuntimeError): void {
    for (const [id, request] of this.pending) {
      if (request.generation !== generation) continue
      clearTimeout(request.timeout)
      request.reject(error)
      this.pending.delete(id)
    }
  }

  private stopGeneration(generation: number): void {
    if (this.process?.generation !== generation) return
    const process = this.process
    this.process = undefined
    process.child.kill()
  }
}

export function expectRuntimeObject(value: JsonValue, context: string): JsonObject {
  if (!isJsonObject(value)) {
    throw new CyberChefRuntimeError(`${context} returned a non-object result`)
  }
  return value
}
