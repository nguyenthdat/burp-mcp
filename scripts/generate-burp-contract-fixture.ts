import { readFileSync } from "node:fs"
import { getBurpToolDescription } from "../bridge/src/burp-tool-descriptions"
import { getBurpToolInputSchema } from "../bridge/src/burp-tool-schemas"

const namesFixture = JSON.parse(
  readFileSync(new URL("../src/test/resources/contracts/burp-tool-names.json", import.meta.url), "utf8"),
) as { version: string; tools: string[] }

const fixture = {
  version: namesFixture.version,
  namespace: "burp_",
  tools: namesFixture.tools.map((backendName) => ({
    name: `burp_${backendName}`,
    backendName,
    description: getBurpToolDescription(backendName),
    inputSchema: getBurpToolInputSchema(backendName),
  })),
}
const rendered = `${JSON.stringify(fixture, null, 2)}\n`
const outputUrl = new URL("../test-fixtures/contracts/burp-tools-v2.json", import.meta.url)

if (process.argv.includes("--check")) {
  const current = readFileSync(outputUrl, "utf8")
  if (current !== rendered) {
    console.error("v2 Burp tool contract fixture is stale; run bun scripts/generate-burp-contract-fixture.ts")
    process.exitCode = 1
  }
} else {
  await Bun.write(outputUrl, rendered)
}
