import { createInterface } from "node:readline"

createInterface({ input: process.stdin }).on("line", (line) => {
  const request = JSON.parse(line)
  if (request.method === "magic" && !process.argv.includes("--replacement")) return
  process.stdout.write(
    `${JSON.stringify({
      id: request.id,
      result: {
        cyberchefType: "string",
        output: { kind: "text", value: "aGVsbG8=" },
      },
    })}\n`,
  )
})
