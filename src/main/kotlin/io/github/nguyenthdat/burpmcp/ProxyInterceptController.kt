package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.core.ByteArray as MontoyaByteArray
import burp.api.montoya.core.Registration
import burp.api.montoya.http.message.requests.HttpRequest
import burp.api.montoya.http.message.responses.HttpResponse
import burp.api.montoya.proxy.http.InterceptedRequest
import burp.api.montoya.proxy.http.InterceptedResponse
import burp.api.montoya.proxy.http.ProxyRequestHandler
import burp.api.montoya.proxy.http.ProxyRequestReceivedAction
import burp.api.montoya.proxy.http.ProxyRequestToBeSentAction
import burp.api.montoya.proxy.http.ProxyResponseHandler
import burp.api.montoya.proxy.http.ProxyResponseReceivedAction
import burp.api.montoya.proxy.http.ProxyResponseToBeSentAction
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicLong

internal enum class InterceptDirection { REQUEST, RESPONSE }
internal enum class InterceptPhase { RECEIVED, TO_BE_SENT }
internal enum class InterceptDecision { FORWARD, DROP, INTERCEPT }

internal data class PendingIntercept(
    val id: Long,
    val messageId: Int,
    val direction: InterceptDirection,
    val phase: InterceptPhase,
    val url: String,
    val method: String,
    val status: Int,
    val isInScope: Boolean,
    val request: ByteArray,
    val response: ByteArray,
)

internal data class InterceptControllerState(
    val enabled: Boolean,
    val timeoutSeconds: Int,
    val pending: Int,
    val urlFilter: String = "",
    val inScopeOnly: Boolean = false,
)

internal class ProxyInterceptController(private val api: MontoyaApi) : AutoCloseable {
    private data class Resolution(val decision: InterceptDecision, val message: ByteArray?)
    private data class InterceptFilter(val urlContains: String, val inScopeOnly: Boolean)
    private class Pending(val snapshot: PendingIntercept) {
        val latch = CountDownLatch(1)
        @Volatile var resolution: Resolution? = null
    }

    private val nextId = AtomicLong(1)
    private val pending = ConcurrentHashMap<Long, Pending>()
    private val requestRegistration: Registration
    private val responseRegistration: Registration
    @Volatile private var enabled = false
    @Volatile private var timeoutSeconds = DEFAULT_TIMEOUT_SECONDS
    @Volatile private var filter = InterceptFilter("", false)

    init {
        requestRegistration = api.proxy().registerRequestHandler(object : ProxyRequestHandler {
            override fun handleRequestReceived(request: InterceptedRequest): ProxyRequestReceivedAction {
                if (!enabled) return ProxyRequestReceivedAction.continueWith(request)
                val snapshot = requestSnapshot(request, InterceptPhase.RECEIVED)
                if (!shouldPause(snapshot)) return ProxyRequestReceivedAction.continueWith(request)
                val resolution = await(snapshot)
                return when (resolution.decision) {
                    InterceptDecision.FORWARD -> ProxyRequestReceivedAction.doNotIntercept(requestMessage(request, resolution.message))
                    InterceptDecision.DROP -> ProxyRequestReceivedAction.drop()
                    InterceptDecision.INTERCEPT -> ProxyRequestReceivedAction.intercept(requestMessage(request, resolution.message))
                }
            }

            override fun handleRequestToBeSent(request: InterceptedRequest): ProxyRequestToBeSentAction {
                if (!enabled) return ProxyRequestToBeSentAction.continueWith(request)
                val snapshot = requestSnapshot(request, InterceptPhase.TO_BE_SENT)
                if (!shouldPause(snapshot)) return ProxyRequestToBeSentAction.continueWith(request)
                val resolution = await(snapshot)
                return when (resolution.decision) {
                    InterceptDecision.DROP -> ProxyRequestToBeSentAction.drop()
                    InterceptDecision.FORWARD, InterceptDecision.INTERCEPT ->
                        ProxyRequestToBeSentAction.continueWith(requestMessage(request, resolution.message))
                }
            }
        })
        responseRegistration = api.proxy().registerResponseHandler(object : ProxyResponseHandler {
            override fun handleResponseReceived(response: InterceptedResponse): ProxyResponseReceivedAction {
                if (!enabled) return ProxyResponseReceivedAction.continueWith(response)
                val snapshot = responseSnapshot(response, InterceptPhase.RECEIVED)
                if (!shouldPause(snapshot)) return ProxyResponseReceivedAction.continueWith(response)
                val resolution = await(snapshot)
                return when (resolution.decision) {
                    InterceptDecision.FORWARD -> ProxyResponseReceivedAction.doNotIntercept(responseMessage(response, resolution.message))
                    InterceptDecision.DROP -> ProxyResponseReceivedAction.drop()
                    InterceptDecision.INTERCEPT -> ProxyResponseReceivedAction.intercept(responseMessage(response, resolution.message))
                }
            }

            override fun handleResponseToBeSent(response: InterceptedResponse): ProxyResponseToBeSentAction {
                if (!enabled) return ProxyResponseToBeSentAction.continueWith(response)
                val snapshot = responseSnapshot(response, InterceptPhase.TO_BE_SENT)
                if (!shouldPause(snapshot)) return ProxyResponseToBeSentAction.continueWith(response)
                val resolution = await(snapshot)
                return when (resolution.decision) {
                    InterceptDecision.DROP -> ProxyResponseToBeSentAction.drop()
                    InterceptDecision.FORWARD, InterceptDecision.INTERCEPT ->
                        ProxyResponseToBeSentAction.continueWith(responseMessage(response, resolution.message))
                }
            }
        })
    }

    fun configure(
        enabled: Boolean?,
        timeoutSeconds: Int?,
        urlFilter: String?,
        inScopeOnly: Boolean?,
    ): InterceptControllerState {
        val currentFilter = filter
        val nextFilter =
            InterceptFilter(
                urlContains = urlFilter?.trim() ?: currentFilter.urlContains,
                inScopeOnly = inScopeOnly ?: currentFilter.inScopeOnly,
            )
        val nextEnabled = enabled ?: this.enabled
        require(!nextEnabled || nextFilter.urlContains.isNotEmpty() || nextFilter.inScopeOnly) {
            "refusing unscoped interception; set url_filter or in_scope_only=true before enabling"
        }
        timeoutSeconds?.let { require(it in 1..MAX_TIMEOUT_SECONDS) { "timeout_seconds must be between 1 and $MAX_TIMEOUT_SECONDS" } }
        if (timeoutSeconds != null) this.timeoutSeconds = timeoutSeconds
        filter = nextFilter
        this.enabled = nextEnabled
        if (!nextEnabled) releaseAll()
        return state()
    }

    fun state(): InterceptControllerState {
        val currentFilter = filter
        return InterceptControllerState(
            enabled,
            timeoutSeconds,
            pending.size,
            currentFilter.urlContains,
            currentFilter.inScopeOnly,
        )
    }

    fun list(offset: Int, limit: Int): Pair<List<PendingIntercept>, Int> {
        require(offset >= 0) { "offset must be non-negative" }
        require(limit in 0..500) { "limit must be between 0 and 500" }
        val values = pending.values.map(Pending::snapshot).sortedBy { it.id }
        val start = offset.coerceAtMost(values.size)
        return values.drop(start).take(limit) to values.size
    }

    fun resolve(id: Long, decision: InterceptDecision, message: ByteArray?): PendingIntercept {
        val item = pending[id] ?: throw NoSuchElementException("intercepted message $id was not found or already resolved")
        synchronized(item) {
            check(item.resolution == null) { "intercepted message $id is already resolved" }
            item.resolution = Resolution(decision, message)
            item.latch.countDown()
        }
        return item.snapshot
    }

    internal fun shouldPause(snapshot: PendingIntercept): Boolean {
        if (!enabled) return false
        val currentFilter = filter
        return (!currentFilter.inScopeOnly || snapshot.isInScope) &&
            (currentFilter.urlContains.isEmpty() || snapshot.url.contains(currentFilter.urlContains, ignoreCase = true))
    }

    private fun await(snapshot: PendingIntercept): Resolution {
        val item = Pending(snapshot)
        pending[snapshot.id] = item
        return try {
            item.latch.await(timeoutSeconds.toLong(), TimeUnit.SECONDS)
            item.resolution ?: Resolution(InterceptDecisionPolicy.fallbackOnTimeout(), null)
        } finally {
            pending.remove(snapshot.id, item)
        }
    }

    private fun requestSnapshot(request: InterceptedRequest, phase: InterceptPhase): PendingIntercept = PendingIntercept(
        id = nextId.getAndIncrement(),
        messageId = request.messageId(),
        direction = InterceptDirection.REQUEST,
        phase = phase,
        url = runCatching { request.url() }.getOrDefault(""),
        method = runCatching { request.method() }.getOrDefault(""),
        status = 0,
        isInScope = runCatching { request.isInScope }.getOrDefault(false),
        request = runCatching { request.toByteArray().getBytes() }.getOrDefault(byteArrayOf()),
        response = byteArrayOf(),
    )

    private fun responseSnapshot(response: InterceptedResponse, phase: InterceptPhase): PendingIntercept {
        val request = runCatching { response.initiatingRequest() }.getOrNull()
        return PendingIntercept(
            id = nextId.getAndIncrement(),
            messageId = response.messageId(),
            direction = InterceptDirection.RESPONSE,
            phase = phase,
            url = runCatching { request?.url() }.getOrNull().orEmpty(),
            method = runCatching { request?.method() }.getOrNull().orEmpty(),
            status = runCatching { response.statusCode().toInt() }.getOrDefault(0),
            isInScope = request?.let { runCatching { it.isInScope }.getOrDefault(false) } ?: false,
            request = runCatching { request?.toByteArray()?.getBytes() }.getOrNull() ?: byteArrayOf(),
            response = runCatching { response.toByteArray().getBytes() }.getOrDefault(byteArrayOf()),
        )
    }

    private fun requestMessage(original: InterceptedRequest, replacement: ByteArray?): HttpRequest =
        replacement?.let { HttpRequest.httpRequest(original.httpService(), MontoyaByteArray.byteArray(*it)) } ?: original

    private fun responseMessage(original: InterceptedResponse, replacement: ByteArray?): HttpResponse =
        replacement?.let { HttpResponse.httpResponse(MontoyaByteArray.byteArray(*it)) } ?: original

    private fun releaseAll() {
        pending.values.forEach { item ->
            synchronized(item) {
                if (item.resolution == null) item.resolution = Resolution(InterceptDecision.FORWARD, null)
                item.latch.countDown()
            }
        }
    }

    override fun close() {
        enabled = false
        releaseAll()
        requestRegistration.deregister()
        responseRegistration.deregister()
        pending.clear()
    }

    private companion object {
        const val DEFAULT_TIMEOUT_SECONDS = 30
        const val MAX_TIMEOUT_SECONDS = 300
    }
}


internal object InterceptDecisionPolicy {
    fun fallbackOnTimeout(): InterceptDecision = InterceptDecision.FORWARD
}
