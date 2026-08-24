package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.burpsuite.BurpSuite
import com.fasterxml.jackson.databind.ObjectMapper
import java.lang.reflect.Proxy as ReflectionProxy
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class ProxySettingsFacadeTest {
    @Test
    fun `creates updates and deletes listeners while preserving unrelated configuration`() {
        val state = fakeState()
        val facade = ProxySettingsFacade(state.api)
        val listener = ProxyListenerConfig(9090, true, "specific_address", "127.0.0.2", "per_host", false, true)

        assertEquals(listener, facade.upsertListener(listener))
        assertEquals(listOf(8080, 9090), facade.listeners().map { it.port })

        val stopped = listener.copy(running = false, listenMode = "loopback_only", listenSpecificAddress = "")
        assertEquals(stopped, facade.upsertListener(stopped))
        assertTrue(facade.deleteListener(9090))
        assertFalse(facade.deleteListener(9090))
        assertEquals(listOf(8080), facade.listeners().map { it.port })
        assertEquals(3, state.imports())
    }

    @Test
    fun `rejects invalid listener settings before import`() {
        val state = fakeState()
        val facade = ProxySettingsFacade(state.api)

        assertFailsWith<IllegalArgumentException> {
            facade.upsertListener(ProxyListenerConfig(0, true, "loopback_only", "", "per_host", true, false))
        }
        assertFailsWith<IllegalArgumentException> {
            facade.upsertListener(ProxyListenerConfig(9090, true, "specific_address", "", "per_host", true, false))
        }
        assertEquals(0, state.imports())
    }

    @Test
    fun `upserts and deletes script filters without changing listener settings`() {
        val state = fakeState()
        val facade = ProxySettingsFacade(state.api)
        val filter = ScriptFilterConfig("proxy_http_history", "script", "return requestResponse.request().method() == \"POST\";", "filter-1", "POST only")

        assertEquals(filter, facade.upsertScriptFilter(filter))
        assertEquals(listOf(8080), facade.listeners().map { it.port })

        val deleted = facade.deleteScriptFilter("proxy_http_history")
        assertEquals("settings", deleted.mode)
        assertEquals("return true;", deleted.script)
        assertEquals("", deleted.scriptId)
        assertEquals("", deleted.scriptName)
        assertEquals(listOf(8080), facade.listeners().map { it.port })
        assertEquals(2, state.imports())
    }

    @Test
    fun `rejects unsupported or blank script filters before import`() {
        val state = fakeState()
        val facade = ProxySettingsFacade(state.api)

        assertFailsWith<IllegalArgumentException> {
            facade.upsertScriptFilter(ScriptFilterConfig("unknown", "script", "return true;", "", ""))
        }
        assertFailsWith<IllegalArgumentException> {
            facade.upsertScriptFilter(ScriptFilterConfig("sitemap", "script", "", "", ""))
        }
        assertEquals(0, state.imports())
    }

    private data class FakeState(val api: MontoyaApi, val imports: () -> Int)

    private fun fakeState(): FakeState {
        var config =
            """{"proxy":{"request_listeners":[{"listener_port":8080,"running":true,"listen_mode":"loopback_only","certificate_mode":"per_host","enable_http2":true,"support_invisible_proxying":false,"use_custom_tls_protocols":false,"custom_tls_protocols":[]}],"http_history_display_filter":{"filter_mode":"SETTINGS"},"web_sockets_history_display_filter":{"filter_mode":"SETTINGS"}},"bambda":{"http_history_display_filter":{"bambda":"return true;","bambda_id":"","bambda_name":""},"web_sockets_history_display_filter":{"bambda":"return true;","bambda_id":"","bambda_name":""},"sitemap_display_filter":{"bambda":"return true;","bambda_id":"","bambda_name":""},"logger_capture_filter":{"bambda":"return true;","bambda_id":"","bambda_name":""},"logger_display_filter":{"bambda":"return true;","bambda_id":"","bambda_name":""}},"target":{"filter":{"filter_mode":"SETTINGS"}},"logger":{"capture_filter":{"filter_mode":"SETTINGS"},"display_filter":{"filter_mode":"SETTINGS"}}}"""
        var imports = 0
        val burp =
            fake<BurpSuite> { method, args ->
                when (method.name) {
                    "exportProjectOptionsAsJson" -> config
                    "importProjectOptionsFromJson" -> {
                        config = ObjectMapper().readTree(args!![0] as String).toString()
                        imports += 1
                        null
                    }
                    else -> null
                }
            }
        val api =
            fake<MontoyaApi> { method, _ ->
                when (method.name) {
                    "burpSuite" -> burp
                    else -> null
                }
            }
        return FakeState(api) { imports }
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
}
