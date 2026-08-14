import chef, { operations } from "cyberchef"

const unsupportedOperations = new Set([
  "DNS over HTTPS",
  "Disassemble ARM",
  "HTTP request",
  "Jq",
  "Optical Character Recognition",
])
const operationEntries = [
  ...new Map(
    operations
      .flatMap((execute) => {
        const operation = chef.help(execute)?.[0]
        return operation ? [{ name: operation.name, operation, execute }] : []
      })
      .map((entry) => [sanitize(entry.name), entry]),
  ).values(),
]

export function listOperations() {
  return operationEntries
    .filter(isSupported)
    .map(({ name, operation }) => ({
      localName: toToolName(name),
      name,
      description: stripHtml(operation.description ?? name),
      inputType: operation.inputType ?? "unknown",
      outputType: operation.outputType ?? "unknown",
      flowControl: Boolean(operation.flowControl),
      args: operation.args ?? [],
      inputSchema: operationInputSchema(operation.args ?? []),
    }))
}

export function searchOperations(query, limit) {
  const queryTokens = [...new Set(tokenize(query))]
  const queryPhrase = sanitize(query)
  const nameTokenFrequencies = new Map(
    queryTokens.map((queryToken) => [
      queryToken,
      operationEntries.filter(({ name }) => tokenize(name).includes(queryToken)).length,
    ]),
  )
  const matches = operationEntries
    .map((entry) => {
      const { name, operation } = entry
      const nameTokens = tokenize(name)
      const descriptionTokens = tokenize(operation.description ?? "")
      let coverage = 0
      let nameScore = 0
      let totalScore = 0
      for (const queryToken of queryTokens) {
        const exactName = nameTokens.includes(queryToken)
        const prefixName = nameTokens.some((token) => token.startsWith(queryToken))
        const exactDescription = descriptionTokens.includes(queryToken)
        const prefixDescription = descriptionTokens.some((token) => token.startsWith(queryToken))
        const rarity = operationEntries.length / (nameTokenFrequencies.get(queryToken) || 1)
        let score = 0
        if (exactName) score = 8 + rarity
        else if (prefixName) score = 6 + rarity / 2
        else if (exactDescription) score = 3
        else if (prefixDescription) score = 2
        if (score > 0) coverage += 1
        if (exactName || prefixName) nameScore += score
        totalScore += score
      }
      return {
        entry,
        name,
        coverage,
        nameScore,
        totalScore,
        phraseMatchesName: sanitize(name).includes(queryPhrase),
      }
    })
    .filter(({ coverage }) => coverage > 0)
    .sort(
      (left, right) =>
        Number(right.phraseMatchesName) - Number(left.phraseMatchesName) ||
        right.coverage - left.coverage ||
        right.nameScore - left.nameScore ||
        right.totalScore - left.totalScore ||
        left.name.localeCompare(right.name),
    )
  return {
    matches: matches.slice(0, limit).map(({ entry, name }) => ({
      name,
      toolName: toToolName(name),
      description: stripHtml(entry.operation.description ?? ""),
      inputType: entry.operation.inputType,
      outputType: entry.operation.outputType,
      flowControl: Boolean(entry.operation.flowControl),
      supported: isSupported(entry),
      args: entry.operation.args ?? [],
    })),
  }
}

export function getOperationDescriptor(name) {
  const entry = operationEntries.find(
    ({ name: candidate }) => sanitize(candidate) === sanitize(name),
  )
  if (!entry) return null
  return isSupported(entry) ? entry : null
}

function operationInputSchema(args) {
  return {
    type: "object",
    properties: {
      input: {
        oneOf: [
          { type: "string" },
          {
            type: "object",
            properties: { kind: { const: "text" }, value: { type: "string" } },
            required: ["kind", "value"],
            additionalProperties: false,
          },
          {
            type: "object",
            properties: { kind: { const: "bytes" }, base64: { type: "string" } },
            required: ["kind", "base64"],
            additionalProperties: false,
          },
          {
            type: "object",
            properties: { kind: { const: "json" }, value: {} },
            required: ["kind", "value"],
            additionalProperties: false,
          },
        ],
      },
      arguments: {
        oneOf: [
          { type: "object", additionalProperties: true },
          { type: "array", items: {} },
        ],
        description: "Named arguments from cyberchef_search_operations or positional values",
      },
    },
    required: ["input"],
    additionalProperties: false,
    $comment: args.length === 0 ? "This operation has no configurable arguments" : undefined,
  }
}

function toToolName(name) {
  return name
    .normalize("NFKD")
    .replace(/\p{M}/gu, "")
    .replace(/([a-z0-9])([A-Z])/g, "$1_$2")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "_")
    .replace(/^_+|_+$/g, "")
}

function stripHtml(value) {
  return value.replace(/<[^>]*>/g, " ").replace(/\s+/g, " ").trim()
}

function sanitize(value) {
  return value.normalize("NFKD").toLowerCase().replace(/[^a-z0-9]/g, "")
}

function tokenize(value) {
  return (
    value
      .normalize("NFKD")
      .replace(/\p{M}/gu, "")
      .toLowerCase()
      .match(/[a-z0-9]+/g) ?? []
  )
}

function isSupported({ execute, name, operation }) {
  return (
    execute.args !== undefined &&
    typeof execute.flowControl === "boolean" &&
    !operation.flowControl &&
    !operation.manualBake &&
    !unsupportedOperations.has(name)
  )
}
