package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.core.ByteArray
import burp.api.montoya.http.HttpService
import burp.api.montoya.websocket.extension.ExtensionWebSocket
import burp.api.montoya.core.Registration
import burp.api.montoya.websocket.BinaryMessage
import burp.api.montoya.websocket.TextMessage
import burp.api.montoya.websocket.extension.ExtensionWebSocketMessageHandler
import java.util.concurrent.ConcurrentLinkedQueue
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.atomic.AtomicLong

internal data class WebSocketCreation(
    val id: String?,
    val status: String,
)
internal data class ManagedWebSocketMessage(
    val index: Long,
    val webSocketId: String,
    val direction: String,
    val type: String,
    val payload: kotlin.ByteArray,
)

internal data class ManagedWebSocketMessagePage(
    val items: List<ManagedWebSocketMessage>,
    val total: Int,
    val offset: Int,
)

internal class WebSocketFacade(
    private val api: MontoyaApi,
) : AutoCloseable {
    private data class Connection(
        val socket: ExtensionWebSocket,
        val registration: Registration,
    )

    private val ids = AtomicLong()
    private val messageIds = AtomicLong()
    private val connections = ConcurrentHashMap<String, Connection>()
    private val messages = ConcurrentLinkedQueue<ManagedWebSocketMessage>()
    fun create(host: String, port: Int, https: Boolean, path: String): WebSocketCreation {
        require(host.isNotBlank()) { "host must not be blank" }
        val creation = api.websockets().createWebSocket(HttpService.httpService(host, port, https), path)
        val socket = creation.webSocket().orElse(null)
        val id = socket?.let { webSocket ->
            "ws-${ids.incrementAndGet()}".also { connectionId ->
                val registration =
                    webSocket.registerMessageHandler(
                        object : ExtensionWebSocketMessageHandler {
                            override fun textMessageReceived(textMessage: TextMessage) {
                                record(connectionId, textMessage.direction().name, "text", textMessage.payload().toByteArray())
                            }

                            override fun binaryMessageReceived(binaryMessage: BinaryMessage) {
                                record(connectionId, binaryMessage.direction().name, "binary", binaryMessage.payload().bytes)
                            }

                            override fun onClose() {
                                connections.remove(connectionId)?.registration?.deregister()
                            }
                        },
                    )
                connections[connectionId] = Connection(webSocket, registration)
            }
        }
        return WebSocketCreation(id, creation.status().name)
    }

    fun sendText(id: String, message: String) {
        connection(id).sendTextMessage(message)
        record(id, "CLIENT_TO_SERVER", "text", message.toByteArray())
    }

    fun sendBinary(id: String, message: kotlin.ByteArray) {
        connection(id).sendBinaryMessage(ByteArray.byteArray(*message))
        record(id, "CLIENT_TO_SERVER", "binary", message.copyOf())
    }

    fun list(): List<String> = connections.keys().toList().sorted()

    fun history(id: String?, offset: Int, limit: Int): ManagedWebSocketMessagePage {
        require(offset >= 0 && limit >= 0) { "offset and limit must be non-negative" }
        val matching = messages.filter { id.isNullOrEmpty() || it.webSocketId == id }
        val end = minOf(offset + limit, matching.size)
        val items = if (offset >= matching.size) emptyList() else matching.subList(offset, end)
        return ManagedWebSocketMessagePage(items, matching.size, offset)
    }

    fun close(id: String) {
        val connection = connections.remove(id) ?: throw NoSuchElementException("WebSocket not found")
        connection.registration.deregister()
        connection.socket.close()
    }

    private fun connection(id: String): ExtensionWebSocket =
        connections[id]?.socket ?: throw NoSuchElementException("WebSocket not found")

    private fun record(id: String, direction: String, type: String, payload: kotlin.ByteArray) {
        messages.add(ManagedWebSocketMessage(messageIds.getAndIncrement(), id, direction, type, payload))
        while (messages.size > MAX_RETAINED_MESSAGES) messages.poll()
    }

    override fun close() {
        connections.keys.toList().forEach { id -> runCatching { close(id) } }
    }

    private companion object {
        const val MAX_RETAINED_MESSAGES = 10_000
    }
}
