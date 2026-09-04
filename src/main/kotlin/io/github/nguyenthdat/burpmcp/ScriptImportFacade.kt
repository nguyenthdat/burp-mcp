package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import java.nio.charset.StandardCharsets

internal data class ScriptImportResult(
    val success: Boolean,
    val status: String,
    val errors: List<String>,
)

internal data class TestBCheckResult(
    val valid: Boolean,
    val matched: Boolean,
    val status: String,
    val errors: List<String>,
    val findings: List<String>,
)

internal class ScriptImportFacade(
    private val api: MontoyaApi,
) {
    fun importBambda(script: String): ScriptImportResult {
        require(script.isNotBlank()) { "script must not be blank" }
        val result = api.bambda().importBambda(script)
        val errors = result.importErrors().map(::explainBambdaImportError)
        return ScriptImportResult(errors.isEmpty(), result.status().name, errors)
    }

    fun importBCheck(script: String, enabled: Boolean): ScriptImportResult {
        require(script.isNotBlank()) { "script must not be blank" }
        val result = api.scanner().bChecks().importBCheck(script, enabled)
        return ScriptImportResult(result.importErrors().isEmpty(), result.status().name, result.importErrors())
    }

    fun testBCheck(
        script: String,
        request: ByteArray,
        response: ByteArray,
        host: String,
        port: Int,
        https: Boolean,
    ): TestBCheckResult {
        require(script.isNotBlank()) { "BCheck script must not be blank" }

        // Test import against Burp BCheck engine
        val importRes = api.scanner().bChecks().importBCheck(script, false)
        val errors = importRes.importErrors()
        val isValid = errors.isEmpty()

        val findings = mutableListOf<String>()
        var matched = false

        if (isValid) {
            val reqStr = String(request, StandardCharsets.UTF_8)
            val respStr = String(response, StandardCharsets.UTF_8)

            // Extract pattern matchers from BCheck then / if statements
            val patternRegex = Regex("""(?:matches|contains|equals)\s+["']([^"']+)["']""", RegexOption.IGNORE_CASE)
            for (match in patternRegex.findAll(script)) {
                val targetPattern = match.groupValues[1]
                if (respStr.contains(targetPattern, ignoreCase = true) || reqStr.contains(targetPattern, ignoreCase = true)) {
                    findings.add("Pattern matched in exchange: '$targetPattern'")
                    matched = true
                }
            }

            // Extract status code checks
            val statusRegex = Regex("""(?:status-code|response\.status)\s*==\s*(\d{3})""", RegexOption.IGNORE_CASE)
            for (match in statusRegex.findAll(script)) {
                val expectedCode = match.groupValues[1]
                if (respStr.startsWith("HTTP/") && respStr.contains(" $expectedCode ")) {
                    findings.add("Status code condition matched: $expectedCode")
                    matched = true
                }
            }

            // Extract issue title if reported
            val issueRegex = Regex("""report\s+issue:\s*\n\s*name:\s*["']([^"']+)["']""", RegexOption.IGNORE_CASE)
            issueRegex.find(script)?.let {
                val issueName = it.groupValues[1]
                findings.add("Target issue definition: '$issueName'")
            }
        }

        return TestBCheckResult(
            valid = isValid,
            matched = matched,
            status = if (isValid) (if (matched) "MATCHED" else "NO_MATCH") else "SYNTAX_ERROR",
            errors = errors,
            findings = findings,
        )
    }
}

private fun explainBambdaImportError(error: String): String {
    val lower = error.lowercase()
    return if (
        "utf8" in lower ||
        "constant string" in lower ||
        "constant pool" in lower ||
        "65535" in lower
    ) {
        "$error. Bambda compiles to JVM bytecode, whose CONSTANT_Utf8 entries are limited to 65,535 bytes; do not embed large bundles. Use burp_settings register_proxy_rule for bounded match/replace or an external streaming proxy."
    } else {
        error
    }
}
