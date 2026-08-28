package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.core.Annotations
import burp.api.montoya.core.ByteArray as MontoyaByteArray
import burp.api.montoya.core.HighlightColor
import burp.api.montoya.http.HttpService
import burp.api.montoya.http.message.HttpRequestResponse
import burp.api.montoya.http.message.requests.HttpRequest
import burp.api.montoya.http.message.responses.HttpResponse
import kotlin.math.min

internal data class OrganizerQuery(
    val limit: Int = 100,
    val offset: Int = 0,
    val statusFilter: String? = null,
    val urlFilter: String? = null,
)

internal data class OrganizerItemDto(
    val id: Int,
    val index: Int,
    val url: String,
    val method: String,
    val statusCode: Int,
    val status: String,
    val notes: String,
    val highlight: String,
    val hasResponse: Boolean,
    val contentType: String,
)

internal data class OrganizerPage(
    val items: List<OrganizerItemDto>,
    val total: Int,
    val offset: Int,
)

internal class OrganizerFacade(
    private val api: MontoyaApi,
) {
    fun sendToOrganizer(
        request: ByteArray,
        response: ByteArray?,
        host: String,
        port: Int,
        https: Boolean,
        notes: String?,
        highlight: String?,
    ) {
        val service = HttpService.httpService(host, port, https)
        val req = HttpRequest.httpRequest(service, MontoyaByteArray.byteArray(*request))
        val resp = response?.let { HttpResponse.httpResponse(MontoyaByteArray.byteArray(*it)) }
        val annotations = Annotations.annotations(
            notes.orEmpty(),
            highlight?.let { runCatching { HighlightColor.valueOf(it.trim().uppercase()) }.getOrNull() } ?: HighlightColor.NONE,
        )
        val message = HttpRequestResponse.httpRequestResponse(req, resp, annotations)
        api.organizer().sendToOrganizer(message)
    }

    fun list(query: OrganizerQuery): OrganizerPage {
        require(query.limit >= 0) { "limit must be non-negative" }
        require(query.offset >= 0) { "offset must be non-negative" }

        val allItems = api.organizer().items()
        val filtered = allItems.indices.filter { idx ->
            val item = allItems[idx]
            val itemStatus = item.status().name
            val itemUrl = item.url()
            (query.statusFilter.isNullOrBlank() || query.statusFilter.equals("all", ignoreCase = true) || itemStatus.equals(query.statusFilter, ignoreCase = true)) &&
                (query.urlFilter.isNullOrBlank() || itemUrl.contains(query.urlFilter))
        }

        val start = min(query.offset, filtered.size)
        val end = min(start + query.limit, filtered.size)
        val items = filtered.subList(start, end).map { idx ->
            val item = allItems[idx]
            OrganizerItemDto(
                id = item.id(),
                index = idx,
                url = item.url(),
                method = runCatching { item.request().method() }.getOrDefault(""),
                statusCode = item.statusCode().toInt(),
                status = item.status().name,
                notes = item.annotations().notes().orEmpty(),
                highlight = item.annotations().highlightColor().name,
                hasResponse = item.hasResponse(),
                contentType = item.contentType()?.name.orEmpty(),
            )
        }

        return OrganizerPage(items, filtered.size, start)
    }
}
