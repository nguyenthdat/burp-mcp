import { Dish, expandAlphabetRange, magic } from "cyberchef"
import { getOperationDescriptor } from "./cyberchef-catalog.mjs"

export async function bake(input, recipe) {
  try {
    const ingredients = Array.isArray(recipe) ? recipe : [recipe]
    let dish = new Dish(parseInput(input))
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
    return dishResult(
      await magic(parseInput(params.input), [
        params.depth,
        params.intensiveMode,
        params.extensiveLanguageSupport,
        params.crib,
      ]),
    )
  } catch (error) {
    throw new TypeError(
      `CyberChef Magic failed: ${error instanceof Error ? error.message : "unknown error"}`,
    )
  }
}

async function runLoadedOperation(descriptor, input, args) {
  const dish = input instanceof Dish ? input : new Dish(input)
  if (descriptor.name === "From Base64") {
    const strict = operationArgument(args, "Strict mode", 2)
    if (strict !== undefined && typeof strict !== "boolean") {
      throw new TypeError("Strict mode must be boolean")
    }
    if (strict === true) {
      const configuredAlphabet = operationArgument(args, "Alphabet", 0)
      const alphabet =
        typeof configuredAlphabet === "string" ? configuredAlphabet : "A-Za-z0-9+/="
      const expandedAlphabet = expandAlphabetRange(alphabet).toString()
      const hasInvalidCharacter = [...dish.get("string")].some(
        (character) => !expandedAlphabet.includes(character),
      )
      if (hasInvalidCharacter) throw new TypeError("Base64 input contains non-alphabet char(s)")
    }
  }
  return await descriptor.execute(dish, args)
}

function operationArgument(args, name, position) {
  if (Array.isArray(args)) return args[position]
  if (!args || typeof args !== "object") return undefined
  const entry = Object.entries(args).find(
    ([key]) => normalizeArgument(key) === normalizeArgument(name),
  )
  return entry?.[1]
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
