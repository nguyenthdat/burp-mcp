package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.core.Annotations
import burp.api.montoya.core.HighlightColor
import burp.api.montoya.proxy.Proxy
import burp.api.montoya.proxy.ProxyHttpRequestResponse
import java.lang.reflect.Proxy as ReflectionProxy
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

class AnnotationFacadeTest {
    @Test
    fun `sitegraph batch uses stable ids and preserves operator annotations`() {
        val first = entry(10, null, HighlightColor.NONE)
        val manual = entry(42, "manual finding", HighlightColor.MAGENTA)
        val facade = AnnotationFacade(api(listOf(first.item, manual.item)))

        val count = facade.annotateProxyEntries(
            listOf(
                AnnotationFacade.ProxyAnnotation(
                    id = 42,
                    highlight = "RED",
                    marker = "[SiteGraph] severity=high rules=jwt direction=response",
                ),
            ),
        )

        assertEquals(1, count)
        assertEquals(
            "manual finding\n[SiteGraph] severity=high rules=jwt direction=response",
            manual.notes(),
        )
        assertEquals(HighlightColor.MAGENTA, manual.highlight())
        assertEquals(null, first.notes())
        assertEquals(HighlightColor.NONE, first.highlight())
    }

    @Test
    fun `sitegraph marker is replaced idempotently and empty highlight is filled`() {
        val target = entry(
            7,
            "operator note\n[SiteGraph] severity=medium rules=internal_ipv4 direction=response",
            HighlightColor.NONE,
        )
        val facade = AnnotationFacade(api(listOf(target.item)))
        val update = AnnotationFacade.ProxyAnnotation(
            id = 7,
            highlight = "RED",
            marker = "[SiteGraph] severity=high rules=jwt direction=both",
        )

        assertEquals(1, facade.annotateProxyEntries(listOf(update)))
        assertEquals(1, facade.annotateProxyEntries(listOf(update)))

        assertEquals(
            "operator note\n[SiteGraph] severity=high rules=jwt direction=both",
            target.notes(),
        )
        assertEquals(HighlightColor.RED, target.highlight())
    }

    @Test
    fun `sitegraph batch is bounded`() {
        val facade = AnnotationFacade(api(emptyList()))
        val entries = (1..51).map { id ->
            AnnotationFacade.ProxyAnnotation(id, "ORANGE", "[SiteGraph] severity=medium rules=rule$id direction=request")
        }

        assertFailsWith<IllegalArgumentException> { facade.annotateProxyEntries(entries) }
    }

    private data class Entry(
        val item: ProxyHttpRequestResponse,
        val notes: () -> String?,
        val highlight: () -> HighlightColor,
    )

    private fun entry(id: Int, initialNotes: String?, initialHighlight: HighlightColor): Entry {
        var notes = initialNotes
        var highlight = initialHighlight
        val annotations = fake<Annotations> { method, args ->
            when (method.name) {
                "notes" -> notes
                "highlightColor" -> highlight
                "setNotes" -> {
                    notes = args?.firstOrNull() as String
                    null
                }
                "setHighlightColor" -> {
                    highlight = args?.firstOrNull() as HighlightColor
                    null
                }
                else -> defaultValue(method.returnType)
            }
        }
        val item = fake<ProxyHttpRequestResponse> { method, _ ->
            when (method.name) {
                "id" -> id
                "annotations" -> annotations
                else -> defaultValue(method.returnType)
            }
        }
        return Entry(item, { notes }, { highlight })
    }

    private fun api(history: List<ProxyHttpRequestResponse>): MontoyaApi {
        val proxy = fake<Proxy> { method, args ->
            when (method.name) {
                "history" -> {
                    if (args.isNullOrEmpty()) {
                        history
                    } else {
                        val filter = args[0] as burp.api.montoya.proxy.ProxyHistoryFilter
                        history.filter { filter.matches(it) }
                    }
                }
                else -> defaultValue(method.returnType)
            }
        }
        return fake { method, _ ->
            when (method.name) {
                "proxy" -> proxy
                else -> defaultValue(method.returnType)
            }
        }
    }

    @Suppress("UNCHECKED_CAST")
    private inline fun <reified T> fake(crossinline handler: (java.lang.reflect.Method, Array<out Any?>?) -> Any?): T =
        ReflectionProxy.newProxyInstance(T::class.java.classLoader, arrayOf(T::class.java)) { proxy, method, args ->
            when (method.name) {
                "toString" -> "Fake${T::class.simpleName}"
                "hashCode" -> System.identityHashCode(proxy)
                "equals" -> proxy === args?.firstOrNull()
                else -> handler(method, args)
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
