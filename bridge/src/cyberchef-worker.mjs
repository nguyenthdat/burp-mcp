import { createInterface } from "node:readline"
import "./cyberchef-bun-compat.mjs"

const [{ listOperations, searchOperations }, { bake, runMagic, runOperation }] = await Promise.all([
  import("./cyberchef-catalog.mjs"),
  import("./cyberchef-engine.mjs"),
])

let queue = Promise.resolve()

createInterface({ input: process.stdin }).on("line", (line) => {
  queue = queue.then(() => handleLine(line))
})

async function handleLine(line) {
  let request
  try {
    request = JSON.parse(line)
    const result = await dispatch(request.method, request.params)
    process.stdout.write(`${JSON.stringify({ id: request.id, result })}\n`)
  } catch (error) {
    const message = error instanceof Error ? error.message : "CyberChef worker failed"
    process.stdout.write(`${JSON.stringify({ id: request?.id ?? null, error: message })}\n`)
  }
}

async function dispatch(method, params) {
  switch (method) {
    case "list":
      return listOperations()
    case "bake":
      return bake(params.input, params.recipe)
    case "operation":
      return runOperation(params)
    case "search":
      return searchOperations(params.query, params.limit)
    case "batch":
      return batch(params.inputs, params.recipe)
    case "magic":
      return runMagic(params)
    default:
      throw new TypeError(`Unknown CyberChef worker method: ${method}`)
  }
}

async function batch(inputs, recipe) {
  const results = []
  for (const input of inputs) {
    try {
      results.push({ success: true, ...(await bake(input, recipe)) })
    } catch (error) {
      results.push({
        success: false,
        error: error instanceof Error ? error.message : "CyberChef batch item failed",
      })
    }
  }
  return { results }
}
