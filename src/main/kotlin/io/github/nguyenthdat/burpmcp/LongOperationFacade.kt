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
import java.util.concurrent.atomic.AtomicReference
import java.util.concurrent.ConcurrentHashMap


internal class LongOperationFacade(
    private val api: MontoyaApi,
    private val jobs: JobFacade,
    private val taskTimeoutMillis: Long = 30_000,
    private val taskStableMillis: Long = 2_000,
    private val taskPollMillis: Long = 100,
) {
    private class AuditHandle {
        private val audit = AtomicReference<burp.api.montoya.scanner.audit.Audit?>()
        private val stopped = AtomicBoolean()
        private val deleted = AtomicBoolean()

        fun attach(value: burp.api.montoya.scanner.audit.Audit) {
            check(audit.compareAndSet(null, value)) { "audit handle is already attached" }
            if (stopped.get()) delete()
        }

        fun stop() {
            stopped.set(true)
            delete()
        }

        private fun delete() {
            val current = audit.get() ?: return
            if (deleted.compareAndSet(false, true)) current.delete()
        }
    }

    private val auditRunning = AtomicBoolean()
    private val audits = ConcurrentHashMap<String, AuditHandle>()

    fun startRace(
        request: String,
        host: String,
        port: Int,
        https: Boolean,
        count: Int,
        singlePacketAttack: Boolean = false,
    ): JobSnapshot {
        require(count in 1..100) { "count must be between 1 and 100" }
        return jobs.start("concurrent_request_check") {
            val service = HttpService.httpService(host, port, https)
            val items: List<HttpJobItem>
            if (singlePacketAttack && request.length > 1) {
                val prefix = request.substring(0, request.length - 1)
                val lastByte = request.substring(request.length - 1)
                val readyLatch = java.util.concurrent.CountDownLatch(count)
                val fireLatch = java.util.concurrent.CountDownLatch(1)
                val executor = java.util.concurrent.Executors.newFixedThreadPool(count)
                val futures = (0 until count).map { index ->
                    executor.submit<HttpJobItem> {
                        try {
                            val socket = if (https) {
                                (javax.net.ssl.SSLSocketFactory.getDefault().createSocket(host, port) as javax.net.ssl.SSLSocket).apply {
                                    val params = sslParameters
                                    params.endpointIdentificationAlgorithm = "HTTPS"
                                    sslParameters = params
                                }
                            } else {
                                java.net.Socket(host, port)
                            }
                            socket.soTimeout = 10_000
                            val out = socket.getOutputStream()
                            out.write(prefix.toByteArray(StandardCharsets.UTF_8))
                            out.flush()
                            readyLatch.countDown()
                            fireLatch.await(5, TimeUnit.SECONDS)
                            val startNanos = System.nanoTime()
                            out.write(lastByte.toByteArray(StandardCharsets.UTF_8))
                            out.flush()

                            val inp = socket.getInputStream()
                            val reader = inp.bufferedReader(StandardCharsets.UTF_8)
                            val statusLine = reader.readLine().orEmpty()
                            val elapsedMicros = (System.nanoTime() - startNanos) / 1_000
                            val statusCode = Regex("""HTTP/\d(?:\.\d)?\s+(\d{3})""").find(statusLine)?.groupValues?.get(1)?.toIntOrNull()

                            var contentLength: Int? = null
                            var line = reader.readLine()
                            while (!line.isNullOrBlank()) {
                                if (line.lowercase().startsWith("content-length:")) {
                                    contentLength = line.substringAfter(':').trim().toIntOrNull()
                                }
                                line = reader.readLine()
                            }
                            val length = contentLength ?: 0
                            socket.close()
                            HttpJobItem("${index} [${elapsedMicros}µs]", statusCode, length)
                        } catch (e: Exception) {
                            val resp = api.http().sendRequest(HttpRequest.httpRequest(service, request)).response()
                            HttpJobItem(index.toString(), resp?.statusCode()?.toInt(), resp?.body()?.length()?.toInt())
                        }
                    }
                }
                readyLatch.await(5, TimeUnit.SECONDS)
                fireLatch.countDown()
                items = futures.map { it.get(10, TimeUnit.SECONDS) }
                executor.shutdown()
            } else {
                val message = HttpRequest.httpRequest(service, request)
                items = api.http().sendRequests(List(count) { message }).mapIndexed { index, exchange ->
                    val response = exchange.response()
                    HttpJobItem(index.toString(), response?.statusCode()?.toInt(), response?.body()?.length()?.toInt())
                }
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
        attackMode: String = "pitchfork",
        markerPayloads: Map<String, List<String>> = emptyMap(),
    ): JobSnapshot {
        val markers = if (markerPayloads.isNotEmpty()) markerPayloads else mapOf(marker to wordlist)
        val attackType = attackMode.lowercase().trim()

        val combinations: List<Pair<String, Map<String, String>>> = when (attackType) {
            "cluster_bomb" -> {
                var combos: List<Map<String, String>> = listOf(emptyMap())
                for ((m, list) in markers) {
                    combos = combos.flatMap { existing ->
                        list.map { item -> existing + (m to item) }
                    }
                }
                combos.map { mapping ->
                    val label = mapping.entries.joinToString(", ") { "${it.key}=${it.value}" }
                    label to mapping
                }
            }
            "sniper" -> {
                val list = mutableListOf<Pair<String, Map<String, String>>>()
                for ((targetMarker, payloads) in markers) {
                    for (p in payloads) {
                        list.add("$targetMarker=$p" to mapOf(targetMarker to p))
                    }
                }
                list
            }
            else -> {
                val maxLen = markers.values.maxOfOrNull { it.size } ?: 0
                (0 until maxLen).map { idx ->
                    val mapping = markers.mapValues { (_, list) ->
                        if (idx < list.size) list[idx] else list.lastOrNull().orEmpty()
                    }
                    val label = mapping.values.joinToString(",")
                    label to mapping
                }
            }
        }

        val substitutionCount = combinations.size
        val requestFingerprint = sha256(template.toByteArray(StandardCharsets.UTF_8))
        return jobs.start("bounded_input_matrix") {
            val service = HttpService.httpService(host, port, https)
            val items = combinations.map { (label, mapping) ->
                var req = template
                for ((m, v) in mapping) {
                    req = req.replace(m, v)
                }
                val response = api.http().sendRequest(HttpRequest.httpRequest(service, req)).response()
                HttpJobItem(label, response?.statusCode()?.toInt(), response?.body()?.length()?.toInt())
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

    fun startCrawl(spec: CrawlExecutionSpec): JobSnapshot {
        require(spec.seedUrls.isNotEmpty()) { "crawl must contain at least one seed URL" }
        require(spec.resourcePoolId.isBlank() || spec.resourcePoolId == "built-in-default") {
            "Burp Scanner Montoya API 2026.7 cannot bind a resource pool; select built-in-default"
        }
        val outOfScopeAtSubmission = spec.seedUrls.filterNot { api.scope().isInScope(it) }
        require(spec.includeOutOfScope || outOfScopeAtSubmission.isEmpty()) {
            "crawl seed is out of scope; set include_out_of_scope=true explicitly"
        }
        return jobs.start("crawl") {
            val scopeChanges = spec.seedUrls.filterNot { api.scope().isInScope(it) }
            scopeChanges.forEach(api.scope()::includeInScope)
            val observed = AtomicInteger()
            val origins = spec.seedUrls.map(::crawlOrigin).toSet()
            val registration = api.http().registerHttpHandler(scannerHandler(origins, observed))
            var crawl: burp.api.montoya.scanner.Crawl? = null
            try {
                crawl = api.scanner().startCrawl(CrawlConfiguration.crawlConfiguration(*spec.seedUrls.toTypedArray()))
                awaitTaskCompletion(
                    operation = "crawl",
                    snapshot = { TaskJobOutput(crawl!!.requestCount(), crawl!!.errorCount()) },
                    observedRequestCount = observed::get,
                    status = { runCatching { crawl!!.statusMessage() }.getOrNull() },
                    timeoutMillis = spec.timeoutMillis,
                    stableMillis = spec.stableMillis,
                    pollMillis = taskPollMillis,
                )
            } finally {
                registration.deregister()
                scopeChanges.forEach(api.scope()::excludeFromScope)
                crawl?.delete()
            }
        }
    }

    fun startAudit(spec: AuditExecutionSpec): JobSnapshot {
        require(spec.resourcePoolId.isBlank() || spec.resourcePoolId == "built-in-default") {
            "Burp Scanner Montoya API 2026.7 cannot bind a resource pool; select built-in-default"
        }
        if (spec.auditType == AuditType.PASSIVE) {
            val issues = api.siteMap().issues().count { issue -> issue.baseUrl().startsWith(spec.url) }
            return jobs.completed(
                "scanner_passive_snapshot",
                AuditJobOutput(0, 0, issues, "passive", true, "stateless site map issue snapshot"),
            )
        }
        return jobs.startWithId("scanner_audit") { id ->
            val handle = AuditHandle()
            audits[id] = handle
            if (Thread.currentThread().isInterrupted) throw InterruptedException()
            check(auditRunning.compareAndSet(false, true)) { "another scanner audit is already running" }
            val wasInScope = api.scope().isInScope(spec.url)
            if (!wasInScope) {
                require(spec.includeOutOfScope) { "audit target is out of scope; set include_out_of_scope=true explicitly" }
                api.scope().includeInScope(spec.url)
            }
            val observed = AtomicInteger()
            val registration = api.http().registerHttpHandler(scannerHandler(setOf(crawlOrigin(spec.url)), observed))
            var audit: burp.api.montoya.scanner.audit.Audit? = null
            try {
                audit = api.scanner().startAudit(AuditConfiguration.auditConfiguration(BuiltInAuditConfiguration.LEGACY_ACTIVE_AUDIT_CHECKS))
                handle.attach(audit)
                audit.addRequest(HttpRequest.httpRequestFromUrl(spec.url))
                awaitAuditCompletion(
                    snapshot = {
                        AuditJobOutput(
                            runCatching { audit!!.requestCount() }.getOrDefault(observed.get()),
                            runCatching { audit!!.errorCount() }.getOrDefault(0),
                            runCatching { audit!!.issues().size }.getOrDefault(0),
                            "active",
                            false,
                            runCatching { audit!!.statusMessage() }.getOrDefault(""),
                        )
                    },
                    observedRequestCount = observed::get,
                    status = { runCatching { audit!!.statusMessage() }.getOrNull() },
                    timeoutMillis = spec.timeoutMillis,
                    stableMillis = spec.stableMillis,
                    pollMillis = taskPollMillis,
                )
            } finally {
                registration.deregister()
                if (!wasInScope) api.scope().excludeFromScope(spec.url)
                auditRunning.set(false)
            }
        }
    }
    fun stopAudit(id: String): JobSnapshot? {
        val snapshot = jobs.status(id) ?: return null
        require(snapshot.operation == "scanner_audit" || snapshot.operation == "scanner_passive_snapshot") { "job is not a scanner audit" }
        if (snapshot.state == JobState.QUEUED || snapshot.state == JobState.RUNNING) {
            audits[id]?.stop()
            return jobs.cancel(id)
        }
        return snapshot
    }

    fun removeAudit(id: String): JobSnapshot? {
        val snapshot = jobs.status(id) ?: return null
        require(snapshot.operation == "scanner_audit" || snapshot.operation == "scanner_passive_snapshot" || snapshot.operation == "crawl") {
            "job is not a scanner task"
        }
        check(snapshot.state in setOf(JobState.COMPLETED, JobState.FAILED, JobState.CANCELLED)) {
            "scanner task must be terminal before removal"
        }
        audits.remove(id)?.stop()
        return jobs.remove(id)
    }
    private fun scannerHandler(origins: Set<String>, observed: AtomicInteger): HttpHandler = object : HttpHandler {
        override fun handleHttpRequestToBeSent(requestToBeSent: HttpRequestToBeSent): RequestToBeSentAction {
            if (requestToBeSent.toolSource().isFromTool(ToolType.SCANNER) && origins.any(requestToBeSent.url()::startsWith)) observed.incrementAndGet()
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
        if (effective > 0 && now - changedAt >= stableNanos) return withRequestCount(current, effective)
        if (now - started >= timeoutNanos) error(if (effective == 0) "$operation completed without observing any requests" else "$operation did not settle before timeout")
        Thread.sleep(pollMillis)
    }
}
