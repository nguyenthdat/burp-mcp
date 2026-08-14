import { plugin } from "bun"

const getBuiltinModule = process.getBuiltinModule.bind(process)
const v8Module = getBuiltinModule("v8")
const v8Compat = new Proxy(v8Module, {
  get(target, property, receiver) {
    return property === "startupSnapshot" ? undefined : Reflect.get(target, property, receiver)
  },
})

Object.defineProperty(process, "getBuiltinModule", {
  configurable: true,
  value(name) {
    return name === "v8" ? v8Compat : getBuiltinModule(name)
  },
})

plugin({
  name: "cyberchef-bun-compat",
  setup(build) {
    build.onLoad(
      {
        filter: /cyberchef[\\/]src[\\/]core[\\/]operations[\\/](Jq|DisassembleARM)\.mjs$/,
      },
      async ({ path }) => ({
        contents: (await Bun.file(path).text())
          .replace(/^import jq from "jq-web";\n/m, "")
          .replace(/^import cs from "@alexaltea\/capstone-js\/dist\/capstone\.min\.js";\n/m, ""),
        loader: "js",
      }),
    )
    build.onLoad(
      {
        filter:
          /cyberchef[\\/]src[\\/]core[\\/]vendor[\\/]gost[\\/]gost(Cipher|Crypto|Digest|Sign)\.mjs$/,
      },
      async ({ path }) => {
        const source = await Bun.file(path).text()
        const renamedImport = source.replace(
          /import GostRandom from ([^;]+);/,
          "import ImportedGostRandom from $1;",
        )
        if (renamedImport === source) {
          throw new Error(`CyberChef Bun compatibility import not found in ${path}`)
        }
        const contents = renamedImport.replace(
          /(var rootCrypto = crypto;?)/,
          "$1\nvar GostRandom = ImportedGostRandom;",
        )
        if (contents === renamedImport) {
          throw new Error(`CyberChef Bun compatibility insertion point not found in ${path}`)
        }
        return { contents, loader: "js" }
      },
    )
  },
})
