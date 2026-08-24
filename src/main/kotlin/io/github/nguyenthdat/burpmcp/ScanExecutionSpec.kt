package io.github.nguyenthdat.burpmcp

internal enum class AuditType {
    PASSIVE,
    ACTIVE,
}

internal data class CrawlExecutionSpec(
    val seedUrls: List<String>,
    val configurationId: String,
    val resourcePoolId: String,
    val timeoutMillis: Long,
    val stableMillis: Long,
    val includeOutOfScope: Boolean,
)

internal data class AuditExecutionSpec(
    val url: String,
    val auditType: AuditType,
    val configurationId: String,
    val resourcePoolId: String,
    val timeoutMillis: Long,
    val stableMillis: Long,
    val includeOutOfScope: Boolean,
)

internal fun ScanCatalogFacade.resolveCrawl(
    seedUrls: List<String>,
    configurationId: String,
    resourcePoolId: String,
    timeoutSeconds: Long,
    stableSeconds: Long,
    includeOutOfScope: Boolean,
): CrawlExecutionSpec {
    val seeds = seedUrls.map(String::trim).filter(String::isNotEmpty).distinct()
    require(seeds.isNotEmpty()) { "seed_urls must contain at least one URL" }
    require(seeds.size <= 100) { "seed_urls must contain at most 100 URLs" }
    seeds.forEach(::requireHttpUrl)
    val configuration = configurationId.takeIf(String::isNotBlank)?.let(::configuration)
    require(configuration == null || configuration.scanType in setOf("crawl", "crawl_and_audit")) {
        "scan configuration does not support crawl"
    }
    val timeout = timeoutSeconds.takeIf { it > 0 } ?: configuration?.timeoutSeconds ?: 900
    val stable = stableSeconds.takeIf { it > 0 } ?: configuration?.stableSeconds ?: 2
    validateTiming(timeout, stable)
    return CrawlExecutionSpec(
        seedUrls = seeds,
        configurationId = configuration?.id.orEmpty(),
        resourcePoolId = resourcePoolId.ifBlank { configuration?.resourcePoolId.orEmpty() },
        timeoutMillis = Math.multiplyExact(timeout, 1_000),
        stableMillis = Math.multiplyExact(stable, 1_000),
        includeOutOfScope = includeOutOfScope || configuration?.includeOutOfScope == true,
    )
}

internal fun ScanCatalogFacade.resolveAudit(
    url: String,
    auditType: String,
    configurationId: String,
    resourcePoolId: String,
    timeoutSeconds: Long,
    stableSeconds: Long,
    includeOutOfScope: Boolean,
): AuditExecutionSpec {
    val target = url.trim()
    requireHttpUrl(target)
    val configuration = configurationId.takeIf(String::isNotBlank)?.let(::configuration)
    require(configuration == null || configuration.scanType in setOf("audit", "crawl_and_audit")) {
        "scan configuration does not support audit"
    }
    val normalizedType = auditType.ifBlank { configuration?.auditType ?: "passive" }.lowercase()
    require(normalizedType in setOf("passive", "active")) { "audit_type must be passive or active" }
    val type = AuditType.valueOf(normalizedType.uppercase())
    val defaultTimeout = if (type == AuditType.PASSIVE) 30L else 3_600L
    val defaultStable = if (type == AuditType.PASSIVE) 0L else 2L
    val timeout = timeoutSeconds.takeIf { it > 0 } ?: configuration?.timeoutSeconds ?: defaultTimeout
    val stable = stableSeconds.takeIf { it > 0 } ?: configuration?.stableSeconds ?: defaultStable
    validateTiming(timeout, stable)
    return AuditExecutionSpec(
        url = target,
        auditType = type,
        configurationId = configuration?.id.orEmpty(),
        resourcePoolId = resourcePoolId.ifBlank { configuration?.resourcePoolId.orEmpty() },
        timeoutMillis = Math.multiplyExact(timeout, 1_000),
        stableMillis = Math.multiplyExact(stable, 1_000),
        includeOutOfScope = includeOutOfScope || configuration?.includeOutOfScope == true,
    )
}

private fun requireHttpUrl(value: String) {
    val uri = runCatching { java.net.URI(value) }.getOrElse { throw IllegalArgumentException("invalid URL: $value") }
    require(uri.scheme.equals("http", true) || uri.scheme.equals("https", true)) { "URL scheme must be http or https" }
    require(!uri.host.isNullOrBlank()) { "URL host must not be blank" }
}

private fun validateTiming(timeoutSeconds: Long, stableSeconds: Long) {
    require(timeoutSeconds in 1..86_400) { "timeout_seconds must be between 1 and 86400" }
    require(stableSeconds in 0..3_600) { "stable_seconds must be between 0 and 3600" }
    require(stableSeconds < timeoutSeconds) { "stable_seconds must be less than timeout_seconds" }
}
