import { isJsonObject, type JsonObject, type JsonValue } from "./types"

const ENTRY_LIMIT = 20
const LANGUAGE_SCORE_LIMIT = 5

export function summarizeMagicResult(result: JsonValue): JsonValue {
  if (!isJsonObject(result)) return result
  const output = result["output"]
  if (!isJsonObject(output)) return result
  const values = output["value"]
  if (!Array.isArray(values)) return result

  let hasTruncatedLanguageScores = false
  const entries = values.slice(0, ENTRY_LIMIT).map((value: JsonValue): JsonValue => {
    if (!isJsonObject(value)) return value
    const languageScores = value["languageScores"]
    if (!Array.isArray(languageScores)) return value
    if (languageScores.length > LANGUAGE_SCORE_LIMIT) hasTruncatedLanguageScores = true
    return {
      ...value,
      languageScores: languageScores.slice(0, LANGUAGE_SCORE_LIMIT),
      languageScoreCount: languageScores.length,
    }
  })
  const summary: JsonObject = {
    returnedEntryCount: entries.length,
    totalEntryCount: values.length,
    truncated: values.length > ENTRY_LIMIT || hasTruncatedLanguageScores,
    entryLimit: ENTRY_LIMIT,
    languageScoreLimit: LANGUAGE_SCORE_LIMIT,
  }
  return { ...result, output: { ...output, value: entries, summary } }
}
