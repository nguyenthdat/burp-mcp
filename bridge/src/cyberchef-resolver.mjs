import { access } from "node:fs/promises"

export async function resolve(specifier, context, nextResolve) {
  try {
    return await nextResolve(specifier, context)
  } catch (error) {
    if (error?.code !== "ERR_MODULE_NOT_FOUND" || !specifier.startsWith(".")) {
      throw error
    }
    const candidate = new URL(`${specifier}.mjs`, context.parentURL)
    await access(candidate)
    return nextResolve(candidate.href, context)
  }
}
