package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.burpsuite.BurpSuite
import java.lang.reflect.Proxy
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class MacroFacadeTest {
    @Test
    fun `create list and remove persist Burp session macros`() {
        var config = """{"project_options":{"sessions":{"macros":{"macros":[]}}}}"""
        val burp = fake<BurpSuite> { method, args ->
            when (method.name) {
                "exportProjectOptionsAsJson" -> config
                "importProjectOptionsFromJson" -> {
                    config = args!![0] as String
                    null
                }
                else -> null
            }
        }
        val api = fake<MontoyaApi> { method, _ -> if (method.name == "burpSuite") burp else null }
        val facade = MacroFacade(api)
        val macro = MacroDefinition(
            description = "Login",
            serialNumber = 42,
            items = listOf(
                MacroItemDefinition(
                    request = "GET /login HTTP/1.1\r\nHost: example.test\r\n\r\n",
                    method = "GET",
                    url = "https://example.test/login",
                    response = "HTTP/1.1 200 OK\r\n\r\n",
                    statusCode = 200,
                    cookiesReceived = "session",
                    requestParameters = emptyList(),
                    customParameters = emptyList(),
                ),
            ),
        )

        facade.create(macro)
        assertEquals(macro, facade.list().single())
        assertTrue(config.contains("\"description\":\"Login\""))
        assertTrue(facade.remove("Login"))
        assertTrue(facade.list().isEmpty())
        assertFalse(facade.remove("missing"))
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
