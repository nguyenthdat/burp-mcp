package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.core.ByteArray as MontoyaByteArray
import burp.api.montoya.core.Registration
import burp.api.montoya.proxy.websocket.BinaryMessageReceivedAction
import burp.api.montoya.proxy.websocket.BinaryMessageToBeSentAction
import burp.api.montoya.proxy.websocket.InterceptedBinaryMessage
import burp.api.montoya.proxy.websocket.InterceptedTextMessage
import burp.api.montoya.proxy.websocket.ProxyMessageHandler
import burp.api.montoya.proxy.websocket.ProxyWebSocketCreation
import burp.api.montoya.proxy.websocket.ProxyWebSocketCreationHandler
import burp.api.montoya.proxy.websocket.TextMessageReceivedAction
import burp.api.montoya.proxy.websocket.TextMessageToBeSentAction
import java.nio.charset.StandardCharsets
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicLong

internal enum class WebSocketMessageType { TEXT, BINARY }

internal data class PendingWebSocketIntercept(
    val id: Long,
    val webSocketId: Int,
    val upgradeUrl: String,
    val direction: String,
    val messageType: WebSocketMessageType,
    val phase: InterceptPhase,
    val payload: ByteArray,
)

internal class ProxyWebSocketInterceptController(private val api: MontoyaApi) : AutoCloseable {
    private data class Resolution(val decision: InterceptDecision, val payload: ByteArray?)
    private data class InterceptFilter(val urlContains: String, val inScopeOnly: Boolean)
    private class Pending(val snapshot: PendingWebSocketIntercept) {
        val latch = CountDownLatch(1)
        @Volatile var resolution: Resolution? = null
    }

    private val nextId = AtomicLong(1)
    private val pending = ConcurrentHashMap<Long, Pending>()
    private val sockets = ConcurrentHashMap<Int, Registration>()
    private val registration: Registration
    @Volatile private var enabled = false
    @Volatile private var timeoutSeconds = DEFAULT_TIMEOUT_SECONDS
    @Volatile private var filter = InterceptFilter("", false)
    init {
        registration = api.proxy().registerWebSocketCreationHandler(object : ProxyWebSocketCreationHandler {
            override fun handleWebSocketCreation(creation: ProxyWebSocketCreation) {
                val socket = creation.proxyWebSocket()
                val webSocketId = System.identityHashCode(socket)
                val upgradeRequest = runCatching { creation.upgradeRequest() }.getOrNull()
                val upgradeUrl = runCatching { upgradeRequest?.url() }.getOrNull().orEmpty()
                val isInScope = upgradeRequest?.let { runCatching { it.isInScope }.getOrDefault(false) } ?: false
                sockets[webSocketId] = socket.registerProxyMessageHandler(messageHandler(webSocketId, upgradeUrl, isInScope))
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

    fun list(offset: Int, limit: Int): Pair<List<PendingWebSocketIntercept>, Int> {
        require(offset >= 0) { "offset must be non-negative" }
        require(limit in 0..500) { "limit must be between 0 and 500" }
        val values = pending.values.map(Pending::snapshot).sortedBy { it.id }
        val start = offset.coerceAtMost(values.size)
        return values.drop(start).take(limit) to values.size
    }

    fun resolve(id: Long, decision: InterceptDecision, payload: ByteArray?): PendingWebSocketIntercept {
        val item = pending[id] ?: throw NoSuchElementException("intercepted WebSocket message $id was not found or already resolved")
        synchronized(item) {
            check(item.resolution == null) { "intercepted WebSocket message $id is already resolved" }
            item.resolution = Resolution(decision, payload)
            item.latch.countDown()
        }
        return item.snapshot
    }

    private fun messageHandler(webSocketId: Int, upgradeUrl: String, isInScope: Boolean) = object : ProxyMessageHandler {
        override fun handleTextMessageReceived(message: InterceptedTextMessage): TextMessageReceivedAction {
            if (!shouldPause(upgradeUrl, isInScope)) return TextMessageReceivedAction.continueWith(message)
            val resolution = await(snapshot(webSocketId, upgradeUrl, message.direction().name, WebSocketMessageType.TEXT, InterceptPhase.RECEIVED, message.payload().toByteArray(StandardCharsets.UTF_8)))
            val payload = resolution.payload?.toString(StandardCharsets.UTF_8) ?: message.payload()
            return when (resolution.decision) {
                InterceptDecision.FORWARD -> TextMessageReceivedAction.doNotIntercept(payload)
                InterceptDecision.DROP -> TextMessageReceivedAction.drop()
                InterceptDecision.INTERCEPT -> TextMessageReceivedAction.intercept(payload)
            }
        }

        override fun handleTextMessageToBeSent(message: InterceptedTextMessage): TextMessageToBeSentAction =
            TextMessageToBeSentAction.continueWith(message)

        override fun handleBinaryMessageReceived(message: InterceptedBinaryMessage): BinaryMessageReceivedAction {
            if (!shouldPause(upgradeUrl, isInScope)) return BinaryMessageReceivedAction.continueWith(message)
            val resolution = await(snapshot(webSocketId, upgradeUrl, message.direction().name, WebSocketMessageType.BINARY, InterceptPhase.RECEIVED, message.payload().getBytes()))
            val payload = resolution.payload ?: message.payload().getBytes()
            return when (resolution.decision) {
                InterceptDecision.FORWARD -> BinaryMessageReceivedAction.doNotIntercept(MontoyaByteArray.byteArray(*payload))
                InterceptDecision.DROP -> BinaryMessageReceivedAction.drop()
                InterceptDecision.INTERCEPT -> BinaryMessageReceivedAction.intercept(MontoyaByteArray.byteArray(*payload))
            }
        }

        override fun handleBinaryMessageToBeSent(message: InterceptedBinaryMessage): BinaryMessageToBeSentAction =
            BinaryMessageToBeSentAction.continueWith(message)

        override fun onClose() {
            sockets.remove(webSocketId)?.deregister()
        }
    }

    internal fun shouldPause(upgradeUrl: String, isInScope: Boolean): Boolean {
        if (!enabled) return false
        val currentFilter = filter
        return (!currentFilter.inScopeOnly || isInScope) &&
            (currentFilter.urlContains.isEmpty() || upgradeUrl.contains(currentFilter.urlContains, ignoreCase = true))
    }

    private fun snapshot(webSocketId: Int, upgradeUrl: String, direction: String, type: WebSocketMessageType, phase: InterceptPhase, payload: ByteArray) =
        PendingWebSocketIntercept(nextId.getAndIncrement(), webSocketId, upgradeUrl, direction, type, phase, payload)

    private fun await(snapshot: PendingWebSocketIntercept): Resolution {
        val item = Pending(snapshot)
        pending[snapshot.id] = item
        return try {
            item.latch.await(timeoutSeconds.toLong(), TimeUnit.SECONDS)
            item.resolution ?: Resolution(InterceptDecisionPolicy.fallbackOnTimeout(), null)
        } finally {
            pending.remove(snapshot.id, item)
        }
    }

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
        registration.deregister()
        sockets.values.forEach(Registration::deregister)
        sockets.clear()
        pending.clear()
    }

    private companion object {
        const val DEFAULT_TIMEOUT_SECONDS = 30
        const val MAX_TIMEOUT_SECONDS = 300
    }
}
