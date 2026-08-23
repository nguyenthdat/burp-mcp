package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.core.Registration
import burp.api.montoya.http.handler.HttpHandler
import burp.api.montoya.http.handler.HttpRequestToBeSent
import burp.api.montoya.http.handler.HttpResponseReceived
import burp.api.montoya.http.handler.RequestToBeSentAction
import burp.api.montoya.http.handler.ResponseReceivedAction
import java.time.Clock
import java.util.LinkedHashMap
import java.util.concurrent.atomic.AtomicLong

internal data class BurpEventRecord(
    val sequence: Long,
    val kind: String,
    val key: String,
    val reconcileRequired: Boolean,
    val observedUnixMillis: Long,
)

internal data class BurpEventPage(
    val items: List<BurpEventRecord>,
    val latestSequence: Long,
    val gapDetected: Boolean,
    val truncated: Boolean,
    val nextSequence: Long,
)

internal class EventFacade(
    api: MontoyaApi,
    private val clock: Clock = Clock.systemUTC(),
    private val capacity: Int = 4096,
) : AutoCloseable {
    private val sequence = AtomicLong()
    private val lock = Any()
    private val events = ArrayDeque<BurpEventRecord>(capacity)
    private val latestByKey = LinkedHashMap<String, BurpEventRecord>()
    private var oldestEvictedSequence = 0L
    private val registration: Registration = api.http().registerHttpHandler(
        object : HttpHandler {
            override fun handleHttpRequestToBeSent(requestToBeSent: HttpRequestToBeSent): RequestToBeSentAction {
                append("http_request", requestToBeSent.method() + " " + requestToBeSent.url())
                return RequestToBeSentAction.continueWith(requestToBeSent)
            }

            override fun handleHttpResponseReceived(responseReceived: HttpResponseReceived): ResponseReceivedAction {
                append("http_response", responseReceived.initiatingRequest().method() + " " + responseReceived.initiatingRequest().url())
                return ResponseReceivedAction.continueWith(responseReceived)
            }
        },
    )

    init {
        require(capacity > 1) { "event queue capacity must exceed one" }
    }

    fun append(kind: String, key: String) {
        val next = BurpEventRecord(sequence.incrementAndGet(), kind, key, false, clock.millis())
        synchronized(lock) {
            latestByKey[key]?.let { previous -> events.remove(previous) }
            latestByKey[key] = next
            events.addLast(next)
            if (events.size > capacity) {
                val removed = events.removeFirst()
                latestByKey.remove(removed.key, removed)
                oldestEvictedSequence = removed.sequence
                val marker = BurpEventRecord(sequence.incrementAndGet(), "reconcile", "overflow", true, clock.millis())
                events.addLast(marker)
                while (events.size > capacity) events.removeFirst()
            }
        }
    }

    fun since(afterSequence: Long, limit: Int): BurpEventPage {
        require(afterSequence >= 0) { "after_sequence must be non-negative" }
        require(limit in 1..500) { "limit must be between 1 and 500" }
        synchronized(lock) {
            val gap = afterSequence < oldestEvictedSequence
            val matching = events.asSequence().filter { it.sequence > afterSequence }.toList()
            val items = matching.take(limit)
            val latest = sequence.get()
            return BurpEventPage(
                items = items,
                latestSequence = latest,
                gapDetected = gap,
                truncated = matching.size > items.size,
                nextSequence = items.lastOrNull()?.sequence ?: afterSequence,
            )
        }
    }

    override fun close() {
        registration.deregister()
    }
}
