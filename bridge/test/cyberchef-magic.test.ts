import { expect, test } from "bun:test"
import { summarizeMagicResult } from "../src/cyberchef-magic"
import { isJsonObject } from "../src/types"

test("limits Magic entries and language scores deterministically", () => {
  // Given
  const value = Array.from({ length: 25 }, (_, entryIndex) => ({
    data: `candidate-${entryIndex}`,
    languageScores: Array.from({ length: 8 }, (_, scoreIndex) => ({
      lang: `lang-${scoreIndex}`,
      score: scoreIndex,
    })),
  }))

  // When
  const result = summarizeMagicResult({ cyberchefType: "JSON", output: { kind: "json", value } })

  // Then
  if (!isJsonObject(result)) throw new Error("Expected Magic result object")
  const output = result["output"]
  if (!isJsonObject(output)) throw new Error("Expected Magic output object")
  const summary = output["value"]
  expect(Array.isArray(summary)).toBe(true)
  expect(summary).toHaveLength(20)
  const metadata = output["summary"]
  if (!isJsonObject(metadata)) throw new Error("Expected Magic summary metadata")
  expect(metadata["returnedEntryCount"]).toBe(20)
  expect(metadata["totalEntryCount"]).toBe(25)
  expect(metadata["truncated"]).toBe(true)
  expect(metadata["entryLimit"]).toBe(20)
  expect(metadata["languageScoreLimit"]).toBe(5)
  const serialized = JSON.stringify(result)
  expect(serialized).toContain('"languageScoreCount":8')
  expect(serialized).not.toContain('"lang":"lang-5"')
  expect(serialized).not.toContain("candidate-20")
})
