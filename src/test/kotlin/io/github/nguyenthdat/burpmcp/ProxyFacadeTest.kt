package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.core.Annotations
import burp.api.montoya.core.HighlightColor
import burp.api.montoya.http.message.requests.HttpRequest
import burp.api.montoya.http.message.responses.HttpResponse
import burp.api.montoya.proxy.Proxy
import burp.api.montoya.proxy.ProxyHttpRequestResponse
import java.lang.reflect.Proxy as ReflectionProxy
import java.time.ZonedDateTime
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

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
        assertEquals("https://example.test/", facade.detail(0)?.request)
    }

    private fun api(history: List<ProxyHttpRequestResponse>): MontoyaApi {
        val proxy = fake<Proxy>(mapOf("history" to { history }))
        return fake(mapOf("proxy" to { proxy }))
    }

    private fun entry(url: String, method: String, status: Short): ProxyHttpRequestResponse {
        val request = fake<HttpRequest>(mapOf("url" to { url }, "method" to { method }, "toString" to { url }))
        val response =
            fake<HttpResponse>(
                mapOf(
                    "statusCode" to { status },
                    "body" to { null },
                    "toString" to { "HTTP/1.1 $status\r\n\r\nbody" },
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

    @Suppress("UNCHECKED_CAST")
    private inline fun <reified T> fake(methods: Map<String, () -> Any?>): T =
        ReflectionProxy.newProxyInstance(T::class.java.classLoader, arrayOf(T::class.java)) { proxy, method, _ ->
            when (method.name) {
                "toString" -> methods[method.name]?.invoke() ?: "Fake${T::class.simpleName}"
                "hashCode" -> System.identityHashCode(proxy)
                "equals" -> proxy === null
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
