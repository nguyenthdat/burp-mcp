package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.core.ByteArray
import burp.api.montoya.http.message.HttpRequestResponse
import burp.api.montoya.http.message.requests.HttpRequest
import burp.api.montoya.http.message.responses.HttpResponse
import burp.api.montoya.sitemap.SiteMap
import java.lang.reflect.Proxy as ReflectionProxy
import kotlin.test.Test
import kotlin.test.assertEquals

class SitemapFacadeTest {
    @Test
    fun `preserves accessible request metadata when response access fails`() {
        val request = fake<HttpRequest>(mapOf("url" to { "https://example.test/socket.io/" }, "method" to { "GET" }))
        val broken =
            fake<HttpRequestResponse>(
                mapOf(
                    "request" to { request },
                    "response" to { throw IllegalStateException("response unavailable") },
                ),
            )
        val siteMap = fake<SiteMap>(mapOf("requestResponses" to { listOf(broken) }))
        val facade = SitemapFacade(fake(mapOf("siteMap" to { siteMap })))

        val item = facade.snapshot(SitemapQuery("", 10, 0)).items.single()

        assertEquals("https://example.test/socket.io/", item.url)
        assertEquals("GET", item.method)
        assertEquals(0, item.status)
        assertEquals("", item.contentType)
        assertEquals(0, item.responseBody.size)
    }

    @Test
    fun `copies complete response body bytes without calling Montoya subArray`() {
        val expected = ByteArray(2 * 1024 * 1024) { index -> (index and 0xff).toByte() }
        val body = fake<burp.api.montoya.core.ByteArray>(mapOf("getBytes" to { expected }, "length" to { expected.size }))
        val response = fake<HttpResponse>(mapOf("body" to { body }))
        val entry = fake<HttpRequestResponse>(mapOf("response" to { response }))
        val siteMap = fake<SiteMap>(mapOf("requestResponses" to { listOf(entry) }))

        val item = SitemapFacade(fake(mapOf("siteMap" to { siteMap })))
            .snapshot(SitemapQuery("", 10, 0))
            .items
            .single()

        assertEquals(expected.size, item.responseBody.size)
        assertEquals(expected.toList(), item.responseBody.toList())
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
