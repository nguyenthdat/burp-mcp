package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.core.Annotations
import burp.api.montoya.core.HighlightColor
import burp.api.montoya.http.message.requests.HttpRequest
import burp.api.montoya.http.message.responses.HttpResponse
import burp.api.montoya.proxy.Proxy
import burp.api.montoya.proxy.ProxyHttpRequestResponse
import burp.api.montoya.core.Registration
import java.lang.reflect.Proxy as ReflectionProxy
import java.time.ZonedDateTime
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

class ProxyFacadeTest {
    @Test
    fun `history returns typed filtered page with stable source indices`() {
        val facade = ProxyFacade(api(listOf(entry("https://old.example/", "GET", 200), entry("https://new.example/", "POST", 201))))

        val page = facade.history(ProxyHistoryQuery(limit = 1, methodFilter = "post"))

        assertEquals(1, page.total)
        assertEquals(1, page.items.single().index)
        assertEquals("https://new.example/", page.items.single().url)
        assertEquals(201, page.items.single().status)
    }

    @Test
    fun `detail handles invalid indices without touching transport types`() {
        val facade = ProxyFacade(api(listOf(entry("https://example.test/", "GET", 200))))

        assertNull(facade.detail(-1))
        assertNull(facade.detail(1))
        assertEquals("https://example.test/", facade.detail(0)?.request?.decodeToString())
    }

    private fun api(history: List<ProxyHttpRequestResponse>): MontoyaApi {
        val proxy = fake<Proxy>(mapOf("history" to { history }))
        return fake(mapOf("proxy" to { proxy }))
    }

    private fun entry(url: String, method: String, status: Short): ProxyHttpRequestResponse {
        val requestBytes = fake<burp.api.montoya.core.ByteArray>(mapOf("getBytes" to { url.encodeToByteArray() }))
        val responseText = "HTTP/1.1 $status\r\n\r\nbody"
        val responseBytes = fake<burp.api.montoya.core.ByteArray>(mapOf("getBytes" to { responseText.encodeToByteArray() }))
        val request = fake<HttpRequest>(mapOf("url" to { url }, "method" to { method }, "toByteArray" to { requestBytes }, "toString" to { url }))
        val response =
            fake<HttpResponse>(
                mapOf(
                    "statusCode" to { status },
                    "body" to { null },
                    "toByteArray" to { responseBytes },
                    "toString" to { responseText },
                ),
            )
        val annotations =
            fake<Annotations>(
                mapOf(
                    "notes" to { null },
                    "highlightColor" to { HighlightColor.NONE },
                ),
            )
        return fake(
            mapOf(
                "finalRequest" to { request },
                "response" to { response },
                "annotations" to { annotations },
                "time" to { ZonedDateTime.now() },
            ),
        )
    }

    @Test
    fun `websocket history isolates unavailable optional fields`() {
        val payload = fake<burp.api.montoya.core.ByteArray>(mapOf("getBytes" to { "hello".toByteArray() }))
        val message =
            fake<burp.api.montoya.proxy.ProxyWebSocketMessage>(
                mapOf(
                    "id" to { 7 },
                    "webSocketId" to { 3 },
                    "direction" to { burp.api.montoya.websocket.Direction.CLIENT_TO_SERVER },
                    "payload" to { payload },
                    "editedPayload" to { null },
                    "time" to { throw IllegalStateException("stale time") },
                    "listenerPort" to { throw IllegalStateException("listener removed") },
                    "upgradeRequest" to { throw IllegalStateException("upgrade request unavailable") },
                ),
            )
        val proxy = fake<Proxy>(mapOf("webSocketHistory" to { listOf(message) }))
        val facade = ProxyFacade(fake(mapOf("proxy" to { proxy })))

        val item = facade.webSocketHistory(5, 0).items.single()

        assertEquals(7, item.id)
        assertEquals("hello", item.payload.decodeToString())
        assertEquals(0, item.editedPayload.size)
        assertEquals("", item.time)
        assertEquals(0, item.listenerPort)
        assertEquals("", item.upgradeUrl)
    }
    @Test
    fun `proxy rule lifecycle preserves handlers until close`() {
        var requestHandler: burp.api.montoya.proxy.http.ProxyRequestHandler? = null
        var responseHandler: burp.api.montoya.proxy.http.ProxyResponseHandler? = null
        var requestDeregistered = false
        var responseDeregistered = false
        val requestRegistration = fake<Registration>(mapOf("deregister" to { requestDeregistered = true }))
        val responseRegistration = fake<Registration>(mapOf("deregister" to { responseDeregistered = true }))
        val proxy =
            ReflectionProxy.newProxyInstance(Proxy::class.java.classLoader, arrayOf(Proxy::class.java)) { proxyObject, method, args ->
                when (method.name) {
                    "registerRequestHandler" -> {
                        requestHandler = args!![0] as burp.api.montoya.proxy.http.ProxyRequestHandler
                        requestRegistration
                    }
                    "registerResponseHandler" -> {
                        responseHandler = args!![0] as burp.api.montoya.proxy.http.ProxyResponseHandler
                        responseRegistration
                    }
                    "toString" -> "FakeProxy"
                    "hashCode" -> System.identityHashCode(proxyObject)
                    "equals" -> proxyObject === args?.firstOrNull()
                    else -> defaultValue(method.returnType)
                }
            } as Proxy
        val facade = ProxyRuleFacade(fake(mapOf("proxy" to { proxy })))

        facade.register(ProxyRule("first", "example.test", "request", "forward", "", "", "", "", true))
        facade.register(ProxyRule("second", "example.test", "response", "drop", "", "", "", "", true))
        assertEquals(listOf("first", "second"), facade.list().map(ProxyRule::id))
        assertTrue(requestHandler != null)
        assertTrue(responseHandler != null)

        facade.clear("first")
        assertEquals(listOf("second"), facade.list().map(ProxyRule::id))
        assertEquals(false, requestDeregistered)
        assertEquals(false, responseDeregistered)

        facade.close()
        assertTrue(facade.list().isEmpty())
        assertTrue(requestDeregistered)
        assertTrue(responseDeregistered)
    }

    @Test
    fun `proxy rule close is idempotent`() {
        var deregisterCalls = 0
        val registration = fake<Registration>(mapOf("deregister" to { deregisterCalls += 1 }))
        val proxy = fake<Proxy>(mapOf("registerRequestHandler" to { registration }, "registerResponseHandler" to { registration }))
        val facade = ProxyRuleFacade(fake(mapOf("proxy" to { proxy })))

        facade.close()
        facade.close()

        assertEquals(2, deregisterCalls)
    }


    @Suppress("UNCHECKED_CAST")
    private inline fun <reified T> fake(methods: Map<String, () -> Any?>): T =
        ReflectionProxy.newProxyInstance(T::class.java.classLoader, arrayOf(T::class.java)) { proxy, method, args ->
            when (method.name) {
                "toString" -> methods[method.name]?.invoke() ?: "Fake${T::class.simpleName}"
                "hashCode" -> System.identityHashCode(proxy)
                "equals" -> proxy === args?.firstOrNull()
                else -> methods[method.name]?.invoke() ?: defaultValue(method.returnType)
            }
        } as T

    private fun defaultValue(type: Class<*>): Any? =
        when (type) {
            java.lang.Boolean.TYPE -> false
            java.lang.Byte.TYPE -> 0.toByte()
            java.lang.Short.TYPE -> 0.toShort()
            java.lang.Integer.TYPE -> 0
            java.lang.Long.TYPE -> 0L
            java.lang.Float.TYPE -> 0F
            java.lang.Double.TYPE -> 0.0
            java.lang.Character.TYPE -> '\u0000'
            else -> null
        }
}
