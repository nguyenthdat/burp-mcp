import { createRequire } from "node:module"
import { dirname, join } from "node:path"
import { getOperationDescriptor } from "./cyberchef-catalog.mjs"

const require = createRequire(import.meta.url)
const cyberchefRoot = dirname(require.resolve("cyberchef-node/package.json"))
const Dish = require(join(cyberchefRoot, "src/core/Dish.js"))
const NodeDish = require(join(cyberchefRoot, "src/node/NodeDish.js"))
globalThis.File = require(join(cyberchefRoot, "src/node/File.js"))

export async function bake(input, recipe) {
  try {
    const ingredients = Array.isArray(recipe) ? recipe : [recipe]
    let dish = parseInput(input)
    for (const ingredient of ingredients) {
      const name = typeof ingredient === "string" ? ingredient : ingredient?.op
      if (typeof name !== "string") {
        throw new TypeError("Recipe can only contain operation names or { op, args } objects")
      }
      const descriptor = getOperationDescriptor(name)
      if (!descriptor) throw new TypeError(`Unsupported CyberChef operation: ${name}`)
      dish = await runLoadedOperation(
        descriptor,
        dish,
        typeof ingredient === "string" ? null : ingredient.args ?? null,
      )
    }
    return dishResult(dish)
  } catch (error) {
    throw new TypeError(
      `CyberChef recipe failed: ${error instanceof Error ? error.message : "unknown error"}`,
    )
  }
}

export async function runOperation(params) {
  const descriptor = getOperationDescriptor(params.operationName)
  if (!descriptor) {
    throw new TypeError(`Unsupported CyberChef operation: ${params.operationName}`)
  }
  try {
    return dishResult(
      await runLoadedOperation(descriptor, parseInput(params.input), params.arguments ?? null),
    )
  } catch (error) {
    throw new TypeError(
      `CyberChef operation ${params.operationName} failed: ${error instanceof Error ? error.message : "unknown error"}`,
    )
  }
}

export async function runMagic(params) {
  try {
    const Operation = require(join(cyberchefRoot, "src/core/operations/Magic.js"))
    const operation = new Operation()
    operation.ingValues = [
      params.depth,
      params.intensiveMode,
      params.extensiveLanguageSupport,
      params.crib,
    ]
    const dish = new NodeDish(parseInput(params.input))
    const state = await operation.run({ progress: 0, dish, opList: [operation] })
    const result = new NodeDish({ value: state.dish.value, type: operation.outputType })
    return dishResult(result)
  } catch (error) {
    throw new TypeError(
      `CyberChef Magic failed: ${error instanceof Error ? error.message : "unknown error"}`,
    )
  }
}

async function runLoadedOperation(descriptor, input, args) {
  const Operation = require(join(cyberchefRoot, `src/core/operations/${descriptor.file}`))
  const operation = new Operation()
  const transformedArgs = transformArgs(operation.args, args)
  const dish = input instanceof NodeDish ? input : new NodeDish(input)
  const transformedInput = dish.get(operation.inputType)
  const result = await Promise.resolve(operation.run(transformedInput, transformedArgs))
  return new NodeDish({ value: result, type: operation.outputType })
}

function transformArgs(originalArgs, newArgs) {
  if (Array.isArray(newArgs)) return newArgs
  const args = structuredClone(originalArgs)
  if (newArgs) {
    for (const [key, value] of Object.entries(newArgs)) {
      const argument = args.find(
        (candidate) => normalizeArgument(candidate.name) === normalizeArgument(key),
      )
      if (!argument) continue
      if (argument.type === "toggleString") {
        if (typeof value === "string") argument.string = value
        else if (value) Object.assign(argument, value)
      } else if (argument.type === "editableOption" || argument.type === "editableOptionShort") {
        argument.value = typeof value === "string" ? value : value?.value
      } else {
        argument.value = value
      }
    }
  }
  return args.map(argumentValue)
}

function argumentValue(argument) {
  if (argument.type === "option" || argument.type === "argSelector") {
    const selected = Array.isArray(argument.value)
      ? argument.value[argument.defaultIndex ?? 0]
      : argument.value
    return typeof selected === "object" ? selected?.name ?? "" : selected
  }
  if (argument.type === "editableOption" || argument.type === "editableOptionShort") {
    const selected = Array.isArray(argument.value)
      ? argument.value[argument.defaultIndex ?? 0]
      : argument.value
    return typeof selected === "object" ? selected?.value ?? "" : selected
  }
  if (argument.type === "toggleString") {
    return {
      string: argument.string ?? argument.value ?? "",
      option: argument.option ?? argument.toggleValues?.[0],
    }
  }
  return argument.value
}

function normalizeArgument(value) {
  return value.toLowerCase().replace(/[^a-z0-9]/g, "")
}

function parseInput(input) {
  if (typeof input === "string") return input
  if (input?.kind === "text") return input.value
  if (input?.kind === "bytes") return Buffer.from(input.base64, "base64")
  if (input?.kind === "json") return input.value
  throw new TypeError("CyberChef input must be text, tagged bytes, or tagged JSON")
}

function dishResult(dish) {
  const cyberchefType = Dish.enumLookup(dish.type)
  return { cyberchefType, output: serializeOutput(cyberchefType, dish.value) }
}

function serializeOutput(type, value) {
  if (type === "byteArray" || type === "ArrayBuffer") {
    const bytes = type === "ArrayBuffer" ? new Uint8Array(value) : Uint8Array.from(value)
    return {
      kind: "bytes",
      base64: Buffer.from(bytes).toString("base64"),
      byteLength: bytes.byteLength,
    }
  }
  if (type === "JSON") return { kind: "json", value }
  if (type === "File" || type === "List<File>") {
    const files = Array.isArray(value) ? value : [value]
    return {
      kind: "json",
      value: files.map((file) => ({
        name: file.name,
        type: file.type,
        base64: Buffer.from(file.data).toString("base64"),
      })),
    }
  }
  return { kind: "text", value: String(value) }
}
