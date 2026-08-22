package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi

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
                    val detail = issue.detail().orEmpty()
                    ScanIssueItem(
                        index = query.offset + position,
                        name = issue.name(),
                        severity = issue.severity().name,
                        confidence = issue.confidence().name,
                        url = issue.baseUrl(),
                        detail = detail.take(200),
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
        require(url.isNotBlank()) { "issue URL must not be blank" }
        val parsedSeverity = runCatching {
            burp.api.montoya.scanner.audit.issues.AuditIssueSeverity.valueOf(severity.uppercase())
        }.getOrDefault(burp.api.montoya.scanner.audit.issues.AuditIssueSeverity.INFORMATION)
        val parsedConfidence = runCatching {
            burp.api.montoya.scanner.audit.issues.AuditIssueConfidence.valueOf(confidence.uppercase())
        }.getOrDefault(burp.api.montoya.scanner.audit.issues.AuditIssueConfidence.TENTATIVE)
        api.siteMap().add(
            burp.api.montoya.scanner.audit.issues.AuditIssue.auditIssue(
                name,
                detail,
                remediation,
                url,
                parsedSeverity,
                parsedConfidence,
                "",
                "",
                parsedSeverity,
            ),
        )
    }
}
