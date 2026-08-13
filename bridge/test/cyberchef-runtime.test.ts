import { afterEach, describe, expect, test } from "bun:test"
import { chmodSync, rmSync } from "node:fs"
import { fileURLToPath } from "node:url"
import { CyberChefRuntime } from "../src/cyberchef-runtime"

const COUNT_FILE = fileURLToPath(
  new URL("../../test/fixtures/.stalled-cyberchef-count", import.meta.url),
)

describe("CyberChefRuntime", () => {
  let runtime: CyberChefRuntime | undefined

  afterEach(() => {
    runtime?.close()
    runtime = undefined
    rmSync(COUNT_FILE, { force: true })
  })

  test("replaces a timed-out worker without stale exit races", async () => {
    // Given
    const command = fileURLToPath(
      new URL("../../test/fixtures/stalled-cyberchef-node", import.meta.url),
    )
    rmSync(COUNT_FILE, { force: true })
    chmodSync(command, 0o755)
    runtime = new CyberChefRuntime(500, 10 * 1024 * 1024, command)

    // When
    await expect(runtime.request("magic", {})).rejects.toThrow("CyberChef request timed out")
    const retry = await runtime.request("bake", {})

    // Then
    expect(retry).toMatchObject({ output: { kind: "text", value: "aGVsbG8=" } })
  })
})
