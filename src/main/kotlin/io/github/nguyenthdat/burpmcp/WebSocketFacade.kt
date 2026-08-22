package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.core.ByteArray
import burp.api.montoya.http.HttpService
import burp.api.montoya.websocket.extension.ExtensionWebSocket
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.atomic.AtomicLong

internal data class WebSocketCreation(
    val id: String?,
    val status: String,
)

internal class WebSocketFacade(
    private val api: MontoyaApi,
) : AutoCloseable {
    private val ids = AtomicLong()
    private val connections = ConcurrentHashMap<String, ExtensionWebSocket>()

    fun create(host: String, port: Int, https: Boolean, path: String): WebSocketCreation {
        require(host.isNotBlank()) { "host must not be blank" }
        val creation = api.websockets().createWebSocket(HttpService.httpService(host, port, https), path)
        val socket = creation.webSocket().orElse(null)
        val id = socket?.let {
            "ws-${ids.incrementAndGet()}".also { connectionId -> connections[connectionId] = it }
        }
        return WebSocketCreation(id, creation.status().name)
    }

    fun sendText(id: String, message: String) {
        connection(id).sendTextMessage(message)
    }

    fun sendBinary(id: String, message: kotlin.ByteArray) {
        connection(id).sendBinaryMessage(ByteArray.byteArray(*message))
    }

    fun list(): List<String> = connections.keys().toList().sorted()

    fun close(id: String) {
        val socket = connections.remove(id) ?: throw NoSuchElementException("WebSocket not found")
        socket.close()
    }

    private fun connection(id: String): ExtensionWebSocket =
        connections[id] ?: throw NoSuchElementException("WebSocket not found")

    override fun close() {
        connections.values.forEach(ExtensionWebSocket::close)
        connections.clear()
    }
}
