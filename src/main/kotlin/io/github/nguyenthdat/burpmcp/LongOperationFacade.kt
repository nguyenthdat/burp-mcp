package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.http.HttpService
import burp.api.montoya.http.message.requests.HttpRequest

internal class LongOperationFacade(
    private val api: MontoyaApi,
    private val jobs: JobFacade,
    private val taskTimeoutMillis: Long = 30_000,
    private val taskStableMillis: Long = 2_000,
    private val taskPollMillis: Long = 100,
) {
    fun startRace(request: String, host: String, port: Int, https: Boolean, count: Int): JobSnapshot {
        require(count in 1..100) { "count must be between 1 and 100" }
        return jobs.start("concurrent_request_check") {
            val service = HttpService.httpService(host, port, https)
            val message = HttpRequest.httpRequest(service, request)
            val items =
                api.http().sendRequests(List(count) { message }).mapIndexed { index, exchange ->
                    val response = exchange.response()
                    HttpJobItem(index.toString(), response?.statusCode()?.toInt(), response?.body()?.length()?.toInt())
                }
            val uniqueLengths = items.mapNotNull(HttpJobItem::length).toSet().size
            HttpBatchJobOutput(items, uniqueLengths, if (uniqueLengths > 1) "responses differ" else "responses match")
        }
    }

    fun startInlineFuzzer(
        template: String,
        host: String,
        port: Int,
        https: Boolean,
        marker: String,
        wordlist: List<String>,
    ): JobSnapshot {
        require(wordlist.size <= 500) { "wordlist must contain at most 500 entries" }
        return jobs.start("bounded_input_matrix") {
            val service = HttpService.httpService(host, port, https)
            val items =
                wordlist.map { value ->
                    val response = api.http().sendRequest(HttpRequest.httpRequest(service, template.replace(marker, value))).response()
                    HttpJobItem(value, response?.statusCode()?.toInt(), response?.body()?.length()?.toInt())
                }
            HttpBatchJobOutput(items, items.mapNotNull(HttpJobItem::length).toSet().size, "completed")
        }
    }

    fun startCrawl(url: String): JobSnapshot {
        require(url.isNotBlank()) { "url must not be blank" }
        return jobs.start("crawl") {
            api.scope().includeInScope(url)
            val crawl =
                api.scanner().startCrawl(
                    burp.api.montoya.scanner.CrawlConfiguration.crawlConfiguration(url),
                )
            awaitTaskCompletion(
                operation = "crawl",
                snapshot = { TaskJobOutput(crawl.requestCount(), crawl.errorCount()) },
                status = { runCatching { crawl.statusMessage() }.getOrNull() },
                timeoutMillis = taskTimeoutMillis,
                stableMillis = taskStableMillis,
                pollMillis = taskPollMillis,
            )
        }
    }
    fun startAudit(url: String, active: Boolean): JobSnapshot {
        require(url.isNotBlank()) { "url must not be blank" }
        return jobs.start("scanner_audit") {
            api.scope().includeInScope(url)
            val mode =
                if (active) {
                    burp.api.montoya.scanner.BuiltInAuditConfiguration.LEGACY_ACTIVE_AUDIT_CHECKS
                } else {
                    burp.api.montoya.scanner.BuiltInAuditConfiguration.LEGACY_PASSIVE_AUDIT_CHECKS
                }
            val audit =
                api.scanner().startAudit(
                    burp.api.montoya.scanner.AuditConfiguration.auditConfiguration(mode),
                )
            audit.addRequest(HttpRequest.httpRequestFromUrl(url))
            awaitAuditCompletion(
                snapshot = { AuditJobOutput(audit.requestCount(), audit.errorCount(), audit.issues().size) },
                status = { runCatching { audit.statusMessage() }.getOrNull() },
                timeoutMillis = taskTimeoutMillis,
                stableMillis = taskStableMillis,
                pollMillis = taskPollMillis,
            )
        }
    }
}

internal fun awaitTaskCompletion(
    operation: String,
    snapshot: () -> TaskJobOutput,
    status: () -> String?,
    timeoutMillis: Long,
    stableMillis: Long,
    pollMillis: Long,
): TaskJobOutput =
    awaitScannerProgress(operation, snapshot, TaskJobOutput::requestCount, status, timeoutMillis, stableMillis, pollMillis)

internal fun awaitAuditCompletion(
    snapshot: () -> AuditJobOutput,
    status: () -> String?,
    timeoutMillis: Long,
    stableMillis: Long,
    pollMillis: Long,
): AuditJobOutput =
    awaitScannerProgress("scanner audit", snapshot, AuditJobOutput::requestCount, status, timeoutMillis, stableMillis, pollMillis)

private fun <T> awaitScannerProgress(
    operation: String,
    snapshot: () -> T,
    requestCount: (T) -> Int,
    status: () -> String?,
    timeoutMillis: Long,
    stableMillis: Long,
    pollMillis: Long,
): T {
    require(timeoutMillis > 0 && stableMillis >= 0 && pollMillis > 0)
    val started = System.nanoTime()
    val timeoutNanos = java.util.concurrent.TimeUnit.MILLISECONDS.toNanos(timeoutMillis)
    val stableNanos = java.util.concurrent.TimeUnit.MILLISECONDS.toNanos(stableMillis)
    var last = snapshot()
    var changedAt = started
    while (true) {
        val message = status()?.trim().orEmpty()
        if (message.contains("unsupported", ignoreCase = true)) error(message)
        val now = System.nanoTime()
        val current = snapshot()
        if (current != last) {
            last = current
            changedAt = now
        }
        if (requestCount(current) > 0 && now - changedAt >= stableNanos) return current
        if (now - started >= timeoutNanos) {
            error(if (requestCount(current) == 0) "$operation issued no requests before timeout" else "$operation did not settle before timeout")
        }
        Thread.sleep(pollMillis)
    }
}
