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
        val entries = api.siteMap().requestResponses(SiteMapFilter.prefixFilter(query.urlPrefix))
        val total = entries.size
        val items =
            entries
                .drop(query.offset)
                .take(query.limit)
                .map { entry ->
                    val response = entry.response()
                    SitemapItem(
                        url = entry.request().url(),
                        method = entry.request().method(),
                        status = response?.statusCode()?.toInt() ?: 0,
                        contentType = response?.headerValue("Content-Type").orEmpty().take(256),
                        responseBody = response?.body()?.let { it.subArray(0, minOf(it.length(), MAX_GRAPH_BODY_BYTES)).bytes } ?: byteArrayOf(),
                        redirectUrl = response?.headerValue("Location").orEmpty().take(MAX_GRAPH_URL_BYTES),
                        responseLinks = emptyList(),
                        formActions = emptyList(),
                        scriptSources = emptyList(),
                    )
                }
        return SitemapPage(items, total, query.offset.coerceAtMost(total))
    }

    private companion object {
        const val MAX_GRAPH_BODY_BYTES = 1024 * 1024
        const val MAX_GRAPH_URL_BYTES = 8 * 1024
    }
}
