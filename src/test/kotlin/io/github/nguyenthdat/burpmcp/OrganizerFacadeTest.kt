package io.github.nguyenthdat.burpmcp

import burp.api.montoya.core.Annotations
import burp.api.montoya.http.message.requests.HttpRequest
import burp.api.montoya.organizer.Organizer
import burp.api.montoya.organizer.OrganizerItem
import burp.api.montoya.organizer.OrganizerItemStatus
import java.lang.reflect.Proxy as ReflectionProxy
import kotlin.test.Test
import kotlin.test.assertEquals

class OrganizerFacadeTest {
    @Test
    fun `list uses request metadata when optional response access fails`() {
        val annotations = fake<Annotations>(mapOf("notes" to { "operator note" }, "highlightColor" to { throw IllegalStateException("unavailable") }))
        val request = fake<HttpRequest>(mapOf("url" to { "https://example.test/path" }, "method" to { "GET" }))
        val item =
            fake<OrganizerItem>(
                mapOf(
                    "id" to { 7 },
                    "status" to { OrganizerItemStatus.NEW },
                    "request" to { request },
                    "response" to { throw IllegalStateException("response unavailable") },
                    "annotations" to { annotations },
                ),
            )
        val organizer = fake<Organizer>(mapOf("items" to { listOf(item) }))
        val page = OrganizerFacade(fake(mapOf("organizer" to { organizer }))).list(OrganizerQuery())

        assertEquals(1, page.total)
        assertEquals("https://example.test/path", page.items.single().url)
        assertEquals("GET", page.items.single().method)
        assertEquals(0, page.items.single().statusCode)
        assertEquals("operator note", page.items.single().notes)
        assertEquals("", page.items.single().highlight)
        assertEquals(false, page.items.single().hasResponse)
        assertEquals("", page.items.single().contentType)
    }

    @Test
    fun `list skips malformed items before filtering and pagination`() {
        val malformed = fake<OrganizerItem>(mapOf("request" to { throw IllegalStateException("malformed") }))
        val request = fake<HttpRequest>(mapOf("url" to { "https://example.test/ok" }, "method" to { "POST" }))
        val valid =
            fake<OrganizerItem>(
                mapOf(
                    "id" to { 8 },
                    "status" to { OrganizerItemStatus.DONE },
                    "request" to { request },
                    "response" to { null },
                    "annotations" to { null },
                ),
            )
        val organizer = fake<Organizer>(mapOf("items" to { listOf(malformed, valid) }))
        val page =
            OrganizerFacade(fake(mapOf("organizer" to { organizer }))).list(
                OrganizerQuery(limit = 1, statusFilter = "done", urlFilter = "/ok"),
            )

        assertEquals(1, page.total)
        assertEquals(1, page.items.single().index)
        assertEquals("https://example.test/ok", page.items.single().url)
    }

    @Suppress("UNCHECKED_CAST")
    private inline fun <reified T> fake(methods: Map<String, () -> Any?>): T =
        ReflectionProxy.newProxyInstance(T::class.java.classLoader, arrayOf(T::class.java)) { proxy, method, args ->
            when (method.name) {
                "toString" -> "Fake${T::class.simpleName}"
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
