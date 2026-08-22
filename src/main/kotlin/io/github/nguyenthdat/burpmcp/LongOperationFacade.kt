package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.http.HttpService
import burp.api.montoya.http.message.requests.HttpRequest

internal class LongOperationFacade(
    private val api: MontoyaApi,
    private val jobs: JobFacade,
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
            TaskJobOutput(crawl.requestCount(), crawl.errorCount())
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
            AuditJobOutput(audit.requestCount(), audit.errorCount(), audit.issues().size)
        }
    }
}
