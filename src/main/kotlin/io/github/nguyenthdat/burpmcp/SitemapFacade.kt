package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.sitemap.SiteMapFilter

internal data class SitemapQuery(
    val urlPrefix: String = "",
    val limit: Int = 100,
    val offset: Int = 0,
)

internal data class SitemapItem(
    val url: String,
    val method: String,
    val status: Int,
    val contentType: String,
    val responseBody: ByteArray,
    val requestBytes: ByteArray,
    val responseBytes: ByteArray,
    val redirectUrl: String,
    val responseLinks: List<String>,
    val formActions: List<String>,
    val scriptSources: List<String>,
)

internal data class SitemapPage(
    val items: List<SitemapItem>,
    val total: Int,
    val offset: Int,
)

internal class SitemapFacade(
    private val api: MontoyaApi,
) {
    fun snapshot(query: SitemapQuery): SitemapPage {
        require(query.limit >= 0) { "limit must be non-negative" }
        require(query.offset >= 0) { "offset must be non-negative" }
        val entries = if (query.urlPrefix.isEmpty()) {
            api.siteMap().requestResponses()
        } else {
            api.siteMap().requestResponses(SiteMapFilter.prefixFilter(query.urlPrefix))
        }
        val total = entries.size
        val items =
            entries
                .drop(query.offset)
                .take(query.limit)
                .map { entry ->
                    val request = runCatching { entry.request() }.getOrNull()
                    val response = runCatching { entry.response() }.getOrNull()
                    SitemapItem(
                        url = runCatching { request?.url() }.getOrNull().orEmpty(),
                        method = runCatching { request?.method() }.getOrNull().orEmpty(),
                        status = runCatching { response?.statusCode()?.toInt() }.getOrNull() ?: 0,
                        contentType = runCatching { response?.headerValue("Content-Type") }.getOrNull().orEmpty().take(256),
                        responseBody = boundedBody(response),
                        requestBytes = runCatching { request?.toByteArray()?.bytes }.getOrNull() ?: byteArrayOf(),
                        responseBytes = runCatching { response?.toByteArray()?.bytes }.getOrNull() ?: byteArrayOf(),
                        redirectUrl = runCatching { response?.headerValue("Location") }.getOrNull().orEmpty().take(MAX_GRAPH_URL_BYTES),
                        responseLinks = emptyList(),
                        formActions = emptyList(),
                        scriptSources = emptyList(),
                    )
                }
        return SitemapPage(items, total, query.offset.coerceAtMost(total))
    }

    private fun boundedBody(response: burp.api.montoya.http.message.responses.HttpResponse?): ByteArray =
        runCatching { response?.body()?.bytes?: byteArrayOf() }
            .getOrElse { byteArrayOf() }

    private companion object {
        const val MAX_GRAPH_URL_BYTES = 8 * 1024
    }
}
