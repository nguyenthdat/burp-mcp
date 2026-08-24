package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import java.nio.file.Files
import java.nio.file.Path

internal data class ScanIssueQuery(
    val limit: Int = 50,
    val offset: Int = 0,
)

internal data class ScanIssueItem(
    val index: Int,
    val name: String,
    val severity: String,
    val confidence: String,
    val url: String,
    val detail: String,
)

internal data class ScanIssuePage(
    val items: List<ScanIssueItem>,
    val total: Int,
    val offset: Int,
)

internal data class ScannerReportResult(
    val path: String,
    val format: String,
    val issueCount: Int,
    val sizeBytes: Long,
)

internal class ScannerFacade(
    private val api: MontoyaApi,
) {
    fun issues(query: ScanIssueQuery): ScanIssuePage {
        require(query.limit >= 0) { "limit must be non-negative" }
        require(query.offset >= 0) { "offset must be non-negative" }
        val issues = api.siteMap().issues()
        val items =
            issues
                .drop(query.offset)
                .take(query.limit)
                .mapIndexed { position, issue ->
                    ScanIssueItem(
                        index = query.offset + position,
                        name = issue.name(),
                        severity = issue.severity().name,
                        confidence = issue.confidence().name,
                        url = issue.baseUrl(),
                        detail = issue.detail().orEmpty().take(200),
                    )
                }
        return ScanIssuePage(items, issues.size, query.offset.coerceAtMost(issues.size))
    }

    fun issueDetail(index: Int): ScanIssueItem {
        require(index >= 0) { "index must be non-negative" }
        val issue = api.siteMap().issues().getOrNull(index) ?: error("scan issue index out of range: $index")
        return ScanIssueItem(
            index = index,
            name = issue.name(),
            severity = issue.severity().name,
            confidence = issue.confidence().name,
            url = issue.baseUrl(),
            detail = issue.detail().orEmpty(),
        )
    }

    fun addIssue(
        name: String,
        url: String,
        detail: String,
        remediation: String,
        severity: String,
        confidence: String,
    ) {
        require(name.isNotBlank()) { "issue name must not be blank" }
        require(url.isNotBlank()) { "issue url must not be blank" }
        val normalizedSeverity = severity.trim().uppercase()
        val normalizedConfidence = confidence.trim().uppercase()
        require(normalizedSeverity in SEVERITIES) { "severity must be high, medium, low, information, or false_positive" }
        require(normalizedConfidence in CONFIDENCES) { "confidence must be certain, firm, or tentative" }
        val normalizedName = name.trim()
        val normalizedUrl = normalizeIssueUrl(url)
        val duplicate = api.siteMap().issues().any { existing ->
            existing.name().trim() == normalizedName && normalizeIssueUrl(existing.baseUrl()) == normalizedUrl
        }
        require(!duplicate) { "issue already exists for name and URL" }
        val issue =
            burp.api.montoya.scanner.audit.issues.AuditIssue.auditIssue(
                normalizedName,
                detail,
                remediation,
                normalizedUrl,
                burp.api.montoya.scanner.audit.issues.AuditIssueSeverity.valueOf(normalizedSeverity),
                burp.api.montoya.scanner.audit.issues.AuditIssueConfidence.valueOf(normalizedConfidence),
                null,
                null,
                burp.api.montoya.scanner.audit.issues.AuditIssueSeverity.valueOf(normalizedSeverity),
                burp.api.montoya.http.message.HttpRequestResponse.httpRequestResponse(
                    burp.api.montoya.http.message.requests.HttpRequest.httpRequestFromUrl(normalizedUrl),
                    null,
                ),
            )
        api.siteMap().add(issue)
    }


    private fun normalizeIssueUrl(value: String): String =
        runCatching {
            java.net.URI(value.trim()).normalize().toASCIIString().trimEnd('/').ifEmpty { value.trim() }
        }.getOrElse { value.trim() }
    fun generateReport(
        format: String,
        path: String,
        issueIndexes: List<Int>,
    ): ScannerReportResult {
        val reportFormat =
            when (format.lowercase()) {
                "html" -> burp.api.montoya.scanner.ReportFormat.HTML
                "xml" -> burp.api.montoya.scanner.ReportFormat.XML
                else -> throw IllegalArgumentException("format must be html or xml")
            }
        require(path.isNotBlank()) { "path must not be blank" }
        val reportPath = Path.of(path).toAbsolutePath().normalize()
        val parent = requireNotNull(reportPath.parent) { "path must include a parent directory" }
        require(Files.isDirectory(parent)) { "report parent directory does not exist" }
        require(!Files.exists(reportPath)) { "report path already exists" }

        val allIssues = api.siteMap().issues()
        val selected =
            if (issueIndexes.isEmpty()) {
                allIssues
            } else {
                require(issueIndexes.size <= MAX_REPORT_ISSUES) { "issue_indexes must contain at most $MAX_REPORT_ISSUES entries" }
                require(issueIndexes.distinct().size == issueIndexes.size) { "issue_indexes must not contain duplicates" }
                issueIndexes.map { index ->
                    require(index in allIssues.indices) { "scanner issue index out of range: $index" }
                    allIssues[index]
                }
            }
        require(selected.isNotEmpty()) { "no scanner issues selected" }
        api.scanner().generateReport(selected, reportFormat, reportPath)
        require(Files.isRegularFile(reportPath)) { "Burp did not create the scanner report" }
        return ScannerReportResult(
            path = reportPath.toString(),
            format = reportFormat.name.lowercase(),
            issueCount = selected.size,
            sizeBytes = Files.size(reportPath),
        )
    }

    private companion object {
        const val MAX_REPORT_ISSUES = 10_000
        val SEVERITIES = setOf("HIGH", "MEDIUM", "LOW", "INFORMATION", "FALSE_POSITIVE")
        val CONFIDENCES = setOf("CERTAIN", "FIRM", "TENTATIVE")
    }
}
