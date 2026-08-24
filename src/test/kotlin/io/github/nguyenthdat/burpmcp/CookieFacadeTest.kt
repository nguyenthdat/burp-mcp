package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.http.Http
import burp.api.montoya.http.sessions.CookieJar
import java.lang.reflect.Proxy
import java.time.ZonedDateTime
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

class CookieFacadeTest {
    @Test
    fun `set cookie forwards path before domain`() {
        var captured: List<Any?> = emptyList()
        val jar = fake<CookieJar> { method, args ->
            if (method.name == "setCookie") captured = args?.toList().orEmpty()
            null
        }
        val http = fake<Http> { method, _ -> if (method.name == "cookieJar") jar else null }
        val api = fake<MontoyaApi> { method, _ -> if (method.name == "http") http else null }

        CookieFacade(api).setCookie("name", "value", "example.test", "/account", "2026-08-23T00:00:00Z")

        assertEquals("/account", captured[2])
        assertEquals("example.test", captured[3])
        assertEquals(ZonedDateTime.parse("2026-08-23T00:00:00Z"), captured[4])
    }

    @Test
    fun `set cookie rejects non-path values`() {
        val api = fake<MontoyaApi> { _, _ -> null }
        assertFailsWith<IllegalArgumentException> {
            CookieFacade(api).setCookie("name", "value", "example.test", "example.test", null)
        }
    }

    @Suppress("UNCHECKED_CAST")
    private inline fun <reified T> fake(crossinline handler: (java.lang.reflect.Method, Array<out Any?>?) -> Any?): T =
        Proxy.newProxyInstance(T::class.java.classLoader, arrayOf(T::class.java)) { proxy, method, args ->
            when (method.name) {
                "toString" -> "Fake${T::class.simpleName}"
                "hashCode" -> System.identityHashCode(proxy)
                "equals" -> proxy === args?.firstOrNull()
                else -> handler(method, args)
            }
        } as T
}
