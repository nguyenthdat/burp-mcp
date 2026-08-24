package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi

internal data class ScriptImportResult(
    val success: Boolean,
    val status: String,
    val errors: List<String>,
)

internal class ScriptImportFacade(
    private val api: MontoyaApi,
) {
    fun importBambda(script: String): ScriptImportResult {
        require(script.isNotBlank()) { "script must not be blank" }
        val result = api.bambda().importBambda(script)
        return ScriptImportResult(result.importErrors().isEmpty(), result.status().name, result.importErrors())
    }

    fun importBCheck(script: String, enabled: Boolean): ScriptImportResult {
        require(script.isNotBlank()) { "script must not be blank" }
        val result = api.scanner().bChecks().importBCheck(script, enabled)
        return ScriptImportResult(result.importErrors().isEmpty(), result.status().name, result.importErrors())
    }
}
