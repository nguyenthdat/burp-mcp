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

private data class IndexedOrganizerItem(
    val index: Int,
    val item: burp.api.montoya.organizer.OrganizerItem,
    val request: HttpRequest,
    val url: String,
    val status: String,
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
        val filtered = allItems.mapIndexedNotNull { index, item ->
            val request = runCatching { item.request() }.getOrNull() ?: return@mapIndexedNotNull null
            val status =
                runCatching { item.status()?.name }
                    .getOrNull()
                    .orEmpty()
                    .ifEmpty { "UNKNOWN" }
            val url = runCatching { request.url() }.getOrNull().orEmpty()
            if (url.isEmpty()) return@mapIndexedNotNull null
            if (
                !query.statusFilter.isNullOrBlank() &&
                !query.statusFilter.equals("all", ignoreCase = true) &&
                !status.equals(query.statusFilter, ignoreCase = true)
            ) {
                return@mapIndexedNotNull null
            }
            if (!query.urlFilter.isNullOrBlank() && !url.contains(query.urlFilter)) {
                return@mapIndexedNotNull null
            }
            IndexedOrganizerItem(index, item, request, url, status)
        }

        val start = min(query.offset, filtered.size)
        val end = min(start + query.limit, filtered.size)
        val items = filtered.subList(start, end).map { indexed ->
            val item = indexed.item
            val response = runCatching { item.response() }.getOrNull()
            val annotations = runCatching { item.annotations() }.getOrNull()
            OrganizerItemDto(
                id = runCatching { item.id() }.getOrDefault(indexed.index),
                index = indexed.index,
                url = indexed.url,
                method = runCatching { indexed.request.method() }.getOrNull().orEmpty(),
                statusCode = runCatching { response?.statusCode()?.toInt() }.getOrNull() ?: 0,
                status = indexed.status,
                notes = runCatching { annotations?.notes() }.getOrNull().orEmpty(),
                highlight = runCatching { annotations?.highlightColor()?.name }.getOrNull().orEmpty(),
                hasResponse = response != null,
                contentType = runCatching { response?.statedMimeType()?.name }.getOrNull().orEmpty(),
            )
        }

        return OrganizerPage(items, filtered.size, start)
    }
}
