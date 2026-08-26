package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import kotlin.math.min

internal data class ProxyHistoryQuery(
    val limit: Int = 100,
    val offset: Int = 0,
    val afterId: Int? = null,
    val urlFilter: String? = null,
    val methodFilter: String? = null,
    val statusFilter: Int? = null,
    val hasNotes: Boolean = false,
    val colorFilter: String? = null,
)

internal data class ProxyHistoryItem(
    val id: Int,
    val index: Int,
    val method: String,
    val url: String,
    val status: Int?,
    val length: Int?,
    val hasResponse: Boolean,
    val request: ByteArray,
    val response: ByteArray?,
    val notes: String?,
    val highlight: String?,
    val time: String,
    val contentType: String,
)

internal data class ProxyHistoryPage(
    val items: List<ProxyHistoryItem>,
    val total: Int,
    val offset: Int,
)

internal data class ProxyDetail(
    val index: Int,
    val request: ByteArray,
    val response: ByteArray?,
    val notes: String?,
    val highlight: String?,
)

internal data class ProxyWebSocketItem(
    val index: Int,
    val id: Int,
    val webSocketId: Int,
    val direction: String,
    val payload: ByteArray,
    val editedPayload: ByteArray,
    val time: String,
    val listenerPort: Int,
    val upgradeUrl: String,
)

internal data class ProxyWebSocketPage(
    val items: List<ProxyWebSocketItem>,
    val total: Int,
    val offset: Int,
)

/** Typed Montoya seam owned by the gRPC adapter. */
internal class ProxyFacade(
    private val api: MontoyaApi,
) {
    fun history(query: ProxyHistoryQuery): ProxyHistoryPage {
        require(query.limit >= 0) { "limit must be non-negative" }
        require(query.offset >= 0) { "offset must be non-negative" }
        val history = query.afterId?.let { afterId ->
            api.proxy().history { entry -> runCatching { entry.id() > afterId }.getOrDefault(false) }
        } ?: api.proxy().history()
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
                val request = runCatching { entry.finalRequest().toByteArray().getBytes() }.getOrDefault(byteArrayOf())
                val response = runCatching { entry.response()?.toByteArray()?.getBytes() }.getOrNull()
                ProxyHistoryItem(
                    id = runCatching { entry.id() }.getOrDefault(index),
                    index = index,
                    method = entry.finalRequest().method(),
                    url = entry.finalRequest().url(),
                    status = response?.let { runCatching { entry.response()?.statusCode()?.toInt() }.getOrNull() },
                    length = response?.size,
                    hasResponse = response != null,
                    request = request,
                    response = response,
                    notes = entry.annotations().notes(),
                    highlight = entry.annotations().highlightColor().name,
                    time = runCatching { entry.time().toString() }.getOrDefault(""),
                    contentType = runCatching { entry.response()?.statedMimeType()?.description() }.getOrNull().orEmpty(),
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
            request = runCatching { entry.finalRequest().toByteArray().getBytes() }.getOrDefault(byteArrayOf()),
            response = runCatching { entry.response()?.toByteArray()?.getBytes() }.getOrNull(),
            notes = entry.annotations().notes(),
            highlight = entry.annotations().highlightColor().name,
        )
    }

    fun interceptState(enabled: Boolean?): Boolean {
        when (enabled) {
            true -> api.proxy().enableIntercept()
            false -> api.proxy().disableIntercept()
            null -> Unit
        }
        return api.proxy().isInterceptEnabled
    }

    fun webSocketHistory(limit: Int, offset: Int, afterId: Int? = null): ProxyWebSocketPage {
        require(limit in 0..500) { "limit must be between 0 and 500" }
        require(offset >= 0) { "offset must be non-negative" }
        val history = afterId?.let { id ->
            api.proxy().webSocketHistory { message -> runCatching { message.id() > id }.getOrDefault(false) }
        } ?: api.proxy().webSocketHistory()
        val start = offset.coerceAtMost(history.size)
        val end = min(start + limit, history.size)
        val items = history.subList(start, end).mapIndexed { position, message ->
            ProxyWebSocketItem(
                index = start + position,
                id = runCatching { message.id() }.getOrDefault(start + position),
                webSocketId = runCatching { message.webSocketId() }.getOrDefault(0),
                direction = runCatching { message.direction().name }.getOrDefault(""),
                payload = boundedPayload { message.payload()?.bytes },
                editedPayload = boundedPayload { message.editedPayload()?.bytes },
                time = runCatching { message.time().toString() }.getOrDefault(""),
                listenerPort = runCatching { message.listenerPort() }.getOrDefault(0),
                upgradeUrl = runCatching { message.upgradeRequest()?.url() }.getOrNull().orEmpty(),
            )
        }
        return ProxyWebSocketPage(items, history.size, start)
    }

    private fun boundedPayload(payload: () -> ByteArray?): ByteArray =
        runCatching { payload()?.let { if (it.size <= MAX_WEBSOCKET_PAYLOAD_BYTES) it else it.copyOf(MAX_WEBSOCKET_PAYLOAD_BYTES) } ?: byteArrayOf() }
            .getOrElse { byteArrayOf() }

    private companion object {
        const val MAX_WEBSOCKET_PAYLOAD_BYTES = 1024 * 1024
    }
}
