package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.core.ToolType
import burp.api.montoya.http.HttpService
import burp.api.montoya.http.handler.HttpHandler
import burp.api.montoya.http.handler.HttpRequestToBeSent
import burp.api.montoya.http.handler.HttpResponseReceived
import burp.api.montoya.http.handler.RequestToBeSentAction
import burp.api.montoya.http.handler.ResponseReceivedAction
import burp.api.montoya.http.message.requests.HttpRequest
import burp.api.montoya.scanner.AuditConfiguration
import burp.api.montoya.scanner.BuiltInAuditConfiguration
import burp.api.montoya.scanner.CrawlConfiguration
import java.net.URI
import java.nio.charset.StandardCharsets
import java.security.MessageDigest
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicInteger


internal class LongOperationFacade(
    private val api: MontoyaApi,
    private val jobs: JobFacade,
    private val taskTimeoutMillis: Long = 30_000,
    private val taskStableMillis: Long = 2_000,
    private val taskPollMillis: Long = 100,
) {
    private val auditRunning = AtomicBoolean()

    fun startRace(request: String, host: String, port: Int, https: Boolean, count: Int): JobSnapshot {
        require(count in 1..100) { "count must be between 1 and 100" }
        return jobs.start("concurrent_request_check") {
            val service = HttpService.httpService(host, port, https)
            val message = HttpRequest.httpRequest(service, request)
            val items = api.http().sendRequests(List(count) { message }).mapIndexed { index, exchange ->
                val response = exchange.response()
                HttpJobItem(index.toString(), response?.statusCode()?.toInt(), response?.body()?.length()?.toInt())
            }
            val uniqueLengths = items.mapNotNull(HttpJobItem::length).toSet().size
            HttpBatchJobOutput(items, uniqueLengths, if (uniqueLengths > 1) "responses differ" else "responses match")
        }
    }

    fun startInlineFuzzer(template: String, host: String, port: Int, https: Boolean, marker: String, wordlist: List<String>): JobSnapshot {
        val substitutionCount = validateInlineFuzzerInput(template, marker, wordlist)
        val requestFingerprint = sha256(template.toByteArray(StandardCharsets.UTF_8))
        return jobs.start("bounded_input_matrix") {
            val service = HttpService.httpService(host, port, https)
            val items = wordlist.map { value ->
                val response = api.http().sendRequest(HttpRequest.httpRequest(service, template.replace(marker, value))).response()
                HttpJobItem(value, response?.statusCode()?.toInt(), response?.body()?.length()?.toInt())
            }
            HttpBatchJobOutput(
                items = items,
                uniqueLengths = items.mapNotNull(HttpJobItem::length).toSet().size,
                verdict = "completed",
                substitutionCount = substitutionCount,
                requestFingerprint = requestFingerprint,
            )
        }
    }

    fun startCrawl(url: String): JobSnapshot {
        require(url.isNotBlank()) { "url must not be blank" }
        return jobs.start("crawl") {
            val wasInScope = api.scope().isInScope(url)
            if (!wasInScope) api.scope().includeInScope(url)
            val observed = AtomicInteger()
            val registration = api.http().registerHttpHandler(scannerHandler(crawlOrigin(url), observed))
            var crawl: burp.api.montoya.scanner.Crawl? = null
            try {
                crawl = api.scanner().startCrawl(CrawlConfiguration.crawlConfiguration(url))
                awaitTaskCompletion(
                    operation = "crawl",
                    snapshot = { TaskJobOutput(crawl!!.requestCount(), crawl!!.errorCount()) },
                    observedRequestCount = observed::get,
                    status = { runCatching { crawl!!.statusMessage() }.getOrNull() },
                    timeoutMillis = taskTimeoutMillis,
                    stableMillis = taskStableMillis,
                    pollMillis = taskPollMillis,
                )
            } finally {
                registration.deregister()
                if (!wasInScope) api.scope().excludeFromScope(url)
                crawl?.delete()
            }
        }
    }

    fun startAudit(url: String, active: Boolean): JobSnapshot {
        require(url.isNotBlank()) { "url must not be blank" }
        return jobs.start("scanner_audit") {
            check(auditRunning.compareAndSet(false, true)) { "another scanner audit is already running" }
            val wasInScope = api.scope().isInScope(url)
            if (!wasInScope) api.scope().includeInScope(url)
            val observed = AtomicInteger()
            val registration = api.http().registerHttpHandler(scannerHandler(crawlOrigin(url), observed))
            var audit: burp.api.montoya.scanner.audit.Audit? = null
            try {
                val mode = if (active) BuiltInAuditConfiguration.LEGACY_ACTIVE_AUDIT_CHECKS else BuiltInAuditConfiguration.LEGACY_PASSIVE_AUDIT_CHECKS
                audit = api.scanner().startAudit(AuditConfiguration.auditConfiguration(mode))
                audit!!.addRequest(HttpRequest.httpRequestFromUrl(url))
                awaitAuditCompletion(
                    snapshot = { AuditJobOutput(runCatching { audit!!.requestCount() }.getOrDefault(observed.get()), runCatching { audit!!.errorCount() }.getOrDefault(0), runCatching { audit!!.issues().size }.getOrDefault(0)) },
                    observedRequestCount = observed::get,
                    status = { runCatching { audit!!.statusMessage() }.getOrNull() },
                    timeoutMillis = taskTimeoutMillis,
                    stableMillis = taskStableMillis,
                    pollMillis = taskPollMillis,
                )
            } finally {
                registration.deregister()
                if (!wasInScope) api.scope().excludeFromScope(url)
                auditRunning.set(false)
            }
        }
    }

    private fun scannerHandler(origin: String, observed: AtomicInteger): HttpHandler = object : HttpHandler {
        override fun handleHttpRequestToBeSent(requestToBeSent: HttpRequestToBeSent): RequestToBeSentAction {
            if (requestToBeSent.toolSource().isFromTool(ToolType.SCANNER) && requestToBeSent.url().startsWith(origin)) observed.incrementAndGet()
            return RequestToBeSentAction.continueWith(requestToBeSent)
        }
        override fun handleHttpResponseReceived(responseReceived: HttpResponseReceived): ResponseReceivedAction = ResponseReceivedAction.continueWith(responseReceived)
    }

    private fun crawlOrigin(url: String): String {
        val parsed = URI(url)
        return "${parsed.scheme}://${parsed.rawAuthority}"
    }
}
internal fun validateInlineFuzzerInput(template: String, marker: String, wordlist: List<String>): Int {
    require(template.isNotBlank()) { "template must be a complete raw HTTP request" }
    require(template.lineSequence().firstOrNull()?.matches(Regex("^[A-Z]+\\s+\\S+\\s+HTTP/\\d(?:\\.\\d)?$")) == true) {
        "template must be a complete raw HTTP request with a request line such as GET / HTTP/1.1"
    }
    require(marker.isNotBlank()) { "marker must not be blank" }
    require(wordlist.isNotEmpty()) { "wordlist must contain at least one entry" }
    require(wordlist.size <= 500) { "wordlist must contain at most 500 entries" }
    val substitutionCount = countMarkerOccurrences(template, marker)
    require(substitutionCount > 0) { "template must contain marker at least once" }
    return substitutionCount
}
private fun countMarkerOccurrences(template: String, marker: String): Int {
    var count = 0
    var start = 0
    while (true) {
        val index = template.indexOf(marker, start)
        if (index < 0) return count
        count++
        start = index + marker.length
    }
}


internal fun sha256(value: ByteArray): String =
    MessageDigest.getInstance("SHA-256").digest(value).joinToString("") { byte -> "%02x".format(byte.toInt() and 0xff) }


internal fun awaitTaskCompletion(operation: String, snapshot: () -> TaskJobOutput, observedRequestCount: () -> Int = { 0 }, status: () -> String?, timeoutMillis: Long, stableMillis: Long, pollMillis: Long): TaskJobOutput =
    awaitScannerProgress(operation, snapshot, { output -> maxOf(output.requestCount, observedRequestCount()) }, { output, count -> output.copy(requestCount = count) }, status, timeoutMillis, stableMillis, pollMillis)

internal fun awaitAuditCompletion(snapshot: () -> AuditJobOutput, observedRequestCount: () -> Int = { 0 }, status: () -> String?, timeoutMillis: Long, stableMillis: Long, pollMillis: Long): AuditJobOutput =
    awaitScannerProgress("scanner audit", snapshot, { output -> maxOf(output.requestCount, observedRequestCount()) }, { output, count -> output.copy(requestCount = count) }, status, timeoutMillis, stableMillis, pollMillis)

private fun <T> awaitScannerProgress(operation: String, snapshot: () -> T, requestCount: (T) -> Int, withRequestCount: (T, Int) -> T, status: () -> String?, timeoutMillis: Long, stableMillis: Long, pollMillis: Long): T {
    require(timeoutMillis > 0 && stableMillis >= 0 && pollMillis > 0)
    val started = System.nanoTime()
    val timeoutNanos = TimeUnit.MILLISECONDS.toNanos(timeoutMillis)
    val stableNanos = TimeUnit.MILLISECONDS.toNanos(stableMillis)
    var last = snapshot()
    var changedAt = started
    while (true) {
        val message = runCatching { status() }.getOrNull()?.trim().orEmpty()
        if (message.contains("unsupported", true)) error(message)
        if (message.contains("failed", true) || message.contains("error", true)) error("$operation task failed: $message")
        val now = System.nanoTime()
        val current = snapshot()
        if (current != last) {
            last = current
            changedAt = now
        }
        val effective = requestCount(current)
        val completed = message.contains("finished", true) || message.contains("complete", true) || message.contains("succeeded", true)
        if ((effective > 0 || completed) && now - changedAt >= stableNanos) return withRequestCount(current, effective)
        if (now - started >= timeoutNanos) error(if (effective == 0) "$operation issued no requests before timeout" else "$operation did not settle before timeout")
        Thread.sleep(pollMillis)
    }
}
