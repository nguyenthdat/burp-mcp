package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import kotlin.math.min

internal data class ProxyHistoryQuery(
    val limit: Int = 100,
    val offset: Int = 0,
    val urlFilter: String? = null,
    val methodFilter: String? = null,
    val statusFilter: Int? = null,
    val hasNotes: Boolean = false,
    val colorFilter: String? = null,
)

internal data class ProxyHistoryItem(
    val index: Int,
    val method: String,
    val url: String,
    val status: Int?,
    val length: Int?,
    val hasResponse: Boolean,
    val notes: String?,
    val highlight: String?,
)

internal data class ProxyHistoryPage(
    val items: List<ProxyHistoryItem>,
    val total: Int,
    val offset: Int,
)

internal data class ProxyDetail(
    val index: Int,
    val request: String,
    val response: String?,
    val notes: String?,
    val highlight: String?,
)

/** Typed Montoya seam shared by the compatibility HTTP and gRPC adapters. */
internal class ProxyFacade(
    private val api: MontoyaApi,
) {
    fun history(query: ProxyHistoryQuery): ProxyHistoryPage {
        require(query.limit >= 0) { "limit must be non-negative" }
        require(query.offset >= 0) { "offset must be non-negative" }
        val history = api.proxy().history()
        val filteredIndices =
            history.indices.reversed().filter { index ->
                val entry = history[index]
                (query.urlFilter == null || entry.finalRequest().url().contains(query.urlFilter)) &&
                    (query.methodFilter == null || entry.finalRequest().method().equals(query.methodFilter, ignoreCase = true)) &&
                    (query.statusFilter == null || entry.response()?.statusCode()?.toInt() == query.statusFilter) &&
                    (!query.hasNotes || !entry.annotations().notes().isNullOrBlank()) &&
                    (query.colorFilter == null || entry.annotations().highlightColor().name.equals(query.colorFilter, ignoreCase = true))
            }
        val start = min(query.offset, filteredIndices.size)
        val end = min(start + query.limit, filteredIndices.size)
        val items =
            filteredIndices.subList(start, end).map { index ->
                val entry = history[index]
                val response = entry.response()
                ProxyHistoryItem(
                    index = index,
                    method = entry.finalRequest().method(),
                    url = entry.finalRequest().url(),
                    status = response?.statusCode()?.toInt(),
                    length = response?.body()?.length(),
                    hasResponse = response != null,
                    notes = entry.annotations().notes(),
                    highlight = entry.annotations().highlightColor().name,
                )
            }
        return ProxyHistoryPage(items, filteredIndices.size, start)
    }

    fun detail(index: Int): ProxyDetail? {
        if (index < 0) return null
        val history = api.proxy().history()
        if (index >= history.size) return null
        val entry = history[index]
        return ProxyDetail(
            index = index,
            request = entry.finalRequest().toString(),
            response = entry.response()?.toString(),
            notes = entry.annotations().notes(),
            highlight = entry.annotations().highlightColor().name,
        )
    }
}
