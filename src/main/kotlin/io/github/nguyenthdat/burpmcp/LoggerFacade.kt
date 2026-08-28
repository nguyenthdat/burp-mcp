package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.core.Registration
import burp.api.montoya.core.ToolType
import burp.api.montoya.http.handler.HttpHandler
import burp.api.montoya.http.handler.HttpRequestToBeSent
import burp.api.montoya.http.handler.HttpResponseReceived
import burp.api.montoya.http.handler.RequestToBeSentAction
import burp.api.montoya.http.handler.ResponseReceivedAction
import java.time.Instant
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.atomic.AtomicInteger
import java.util.concurrent.atomic.AtomicLong
import kotlin.math.min

internal data class LoggerHistoryQuery(
    val limit: Int = 100,
    val offset: Int = 0,
    val afterId: Long? = null,
    val sourceFilter: String? = null,
    val urlFilter: String? = null,
    val methodFilter: String? = null,
    val statusFilter: Int? = null,
    val hasNotes: Boolean = false,
    val colorFilter: String? = null,
)

internal data class LoggerHistoryItem(
    val id: Long,
    val index: Int,
    val source: String,
    val method: String,
    val url: String,
    val status: Int?,
    val length: Long?,
    val hasResponse: Boolean,
    val request: ByteArray,
    val response: ByteArray?,
    val notes: String?,
    val highlight: String?,
    val time: String,
    val contentType: String,
)

internal data class LoggerHistoryPage(
    val items: List<LoggerHistoryItem>,
    val total: Int,
    val offset: Int,
)

internal data class LoggerDetail(
    val index: Int,
    val source: String,
    val request: ByteArray,
    val response: ByteArray?,
    val notes: String?,
    val highlight: String?,
)

internal class LoggerFacade(
    private val api: MontoyaApi,
    private val maxCapacity: Int = 10_000,
) : AutoCloseable {
    private data class RecordedExchange(
        val id: Long,
        val source: String,
        val method: String,
        val url: String,
        var status: Int?,
        var length: Long?,
        var hasResponse: Boolean,
        val request: ByteArray,
        var response: ByteArray?,
        var notes: String?,
        var highlight: String?,
        val time: String,
        var contentType: String,
    )

    private val idCounter = AtomicLong(1L)
    private val inFlightRequests = ConcurrentHashMap<Int, RecordedExchange>()
    private val history = ArrayList<RecordedExchange>(maxCapacity)
    private val lock = Any()

    private val registration: Registration = api.http().registerHttpHandler(
        object : HttpHandler {
            override fun handleHttpRequestToBeSent(requestToBeSent: HttpRequestToBeSent): RequestToBeSentAction {
                val toolSource = resolveSource(requestToBeSent.toolSource().toolType())
                val id = idCounter.getAndIncrement()
                val reqBytes = runCatching { requestToBeSent.toByteArray().getBytes() }.getOrDefault(byteArrayOf())
                val recorded = RecordedExchange(
                    id = id,
                    source = toolSource,
                    method = requestToBeSent.method(),
                    url = requestToBeSent.url(),
                    status = null,
                    length = null,
                    hasResponse = false,
                    request = reqBytes,
                    response = null,
                    notes = requestToBeSent.annotations().notes(),
                    highlight = requestToBeSent.annotations().highlightColor().name,
                    time = Instant.now().toString(),
                    contentType = "",
                )
                inFlightRequests[requestToBeSent.messageId()] = recorded
                synchronized(lock) {
                    if (history.size >= maxCapacity) {
                        history.removeAt(0)
                    }
                    history.add(recorded)
                }
                return RequestToBeSentAction.continueWith(requestToBeSent)
            }

            override fun handleHttpResponseReceived(responseReceived: HttpResponseReceived): ResponseReceivedAction {
                val recorded = inFlightRequests.remove(responseReceived.messageId())
                val respBytes = runCatching { responseReceived.toByteArray().getBytes() }.getOrNull()
                val statusCode = responseReceived.statusCode().toInt()
                val mime = runCatching { responseReceived.statedMimeType().description() }.getOrNull().orEmpty()
                
                if (recorded != null) {
                    recorded.status = statusCode
                    recorded.length = respBytes?.size?.toLong() ?: 0L
                    recorded.hasResponse = respBytes != null
                    recorded.response = respBytes
                    recorded.contentType = mime
                    if (recorded.notes.isNullOrBlank()) {
                        recorded.notes = responseReceived.annotations().notes()
                    }
                    if (recorded.highlight.isNullOrBlank() || recorded.highlight == "NONE") {
                        recorded.highlight = responseReceived.annotations().highlightColor().name
                    }
                }
                return ResponseReceivedAction.continueWith(responseReceived)
            }
        }
    )

    fun history(query: LoggerHistoryQuery): LoggerHistoryPage {
        require(query.limit >= 0) { "limit must be non-negative" }
        require(query.offset >= 0) { "offset must be non-negative" }

        val snapshot: List<RecordedExchange>
        synchronized(lock) {
            snapshot = ArrayList(history)
        }

        val filtered = snapshot.indices.reversed().filter { idx ->
            val entry = snapshot[idx]
            (query.afterId == null || entry.id > query.afterId) &&
                (query.sourceFilter.isNullOrBlank() || query.sourceFilter.equals("all", ignoreCase = true) || entry.source.equals(query.sourceFilter, ignoreCase = true)) &&
                (query.urlFilter == null || entry.url.contains(query.urlFilter)) &&
                (query.methodFilter == null || entry.method.equals(query.methodFilter, ignoreCase = true)) &&
                (query.statusFilter == null || entry.status == query.statusFilter) &&
                (!query.hasNotes || !entry.notes.isNullOrBlank()) &&
                (query.colorFilter == null || entry.highlight.equals(query.colorFilter, ignoreCase = true))
        }

        val start = min(query.offset, filtered.size)
        val end = min(start + query.limit, filtered.size)
        val items = filtered.subList(start, end).map { idx ->
            val entry = snapshot[idx]
            LoggerHistoryItem(
                id = entry.id,
                index = idx,
                source = entry.source,
                method = entry.method,
                url = entry.url,
                status = entry.status,
                length = entry.length,
                hasResponse = entry.hasResponse,
                request = entry.request,
                response = entry.response,
                notes = entry.notes,
                highlight = entry.highlight,
                time = entry.time,
                contentType = entry.contentType,
            )
        }

        return LoggerHistoryPage(items, filtered.size, start)
    }

    fun detail(index: Int): LoggerDetail? {
        if (index < 0) return null
        val entry: RecordedExchange?
        synchronized(lock) {
            entry = history.getOrNull(index)
        }
        if (entry == null) return null
        return LoggerDetail(
            index = index,
            source = entry.source,
            request = entry.request,
            response = entry.response,
            notes = entry.notes,
            highlight = entry.highlight,
        )
    }
    fun clear() {
        synchronized(lock) {
            history.clear()
            inFlightRequests.clear()
        }
    }

    override fun close() {
        registration.deregister()
    }

    private fun resolveSource(toolType: ToolType): String =
        when (toolType) {
            ToolType.PROXY -> "proxy"
            ToolType.REPEATER -> "repeater"
            ToolType.SCANNER -> "scanner"
            ToolType.INTRUDER -> "intruder"
            ToolType.EXTENSIONS -> "extension"
            ToolType.SEQUENCER -> "sequencer"
            ToolType.DECODER -> "decoder"
            ToolType.COMPARER -> "comparer"
            ToolType.TARGET -> "target"
            ToolType.ORGANIZER -> "organizer"
            ToolType.LOGGER -> "logger"
            ToolType.RECORDED_LOGIN_REPLAYER -> "recorded_login"
            ToolType.SUITE -> "suite"
            else -> "unknown"
        }
}
