package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.burpsuite.BurpSuite
import burp.api.montoya.proxy.Proxy
import com.fasterxml.jackson.databind.ObjectMapper
import java.lang.reflect.Proxy as ReflectionProxy
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue

class ProxyInterceptConfigFacadeTest {
    @Test
    fun `reads typed Proxy interception settings`() {
        val state = fakeState()
        val config = ProxyInterceptConfigFacade(state.api).read()

        assertTrue(config.masterInterceptEnabled)
        assertTrue(config.requestDoIntercept)
        assertFalse(config.responseDoIntercept)
        assertEquals("content_type_header", config.responseRules.single().matchType)
        assertTrue(config.websocketClientToServer)
        assertTrue(config.responseAutoContentLength)
    }

    @Test
    fun `patches selected settings and replaces response rules without losing request rules`() {
        val state = fakeState()
        val responseRule = ProxyInterceptRuleConfig(true, "and", "status_code", "does_not_match", "^304$")
        val updated =
            ProxyInterceptConfigFacade(state.api).update(
                ProxyInterceptConfigPatch(
                    masterInterceptEnabled = false,
                    responseDoIntercept = true,
                    websocketInScopeOnly = true,
                    responseRules = listOf(responseRule),
                    replaceResponseRules = true,
                    responseRemoveJavaScriptValidation = true,
                ),
            )

        assertFalse(updated.masterInterceptEnabled)
        assertTrue(updated.responseDoIntercept)
        assertTrue(updated.websocketInScopeOnly)
        assertEquals(listOf(responseRule), updated.responseRules)
        assertEquals("file_extension", updated.requestRules.single().matchType)
        assertTrue(updated.responseRemoveJavaScriptValidation)
        assertEquals(1, state.imports())
    }

    @Test
    fun `rejects accidental rule replacement`() {
        val state = fakeState()
        assertFailsWith<IllegalArgumentException> {
            ProxyInterceptConfigFacade(state.api).update(
                ProxyInterceptConfigPatch(
                    responseRules = listOf(ProxyInterceptRuleConfig(true, "and", "url", "is_in_target_scope", "")),
                ),
            )
        }
        assertEquals(0, state.imports())
    }

    @Test
    fun `rejects unsupported Proxy filter types and relationships`() {
        val state = fakeState()
        val facade = ProxyInterceptConfigFacade(state.api)

        assertFailsWith<IllegalArgumentException> {
            facade.update(
                ProxyInterceptConfigPatch(
                    responseRules = listOf(ProxyInterceptRuleConfig(true, "and", "invalid", "matches", "x")),
                    replaceResponseRules = true,
                ),
            )
        }
        assertFailsWith<IllegalArgumentException> {
            facade.update(
                ProxyInterceptConfigPatch(
                    responseRules = listOf(ProxyInterceptRuleConfig(true, "and", "status_code", "was_modified", "")),
                    replaceResponseRules = true,
                ),
            )
        }
        assertEquals(0, state.imports())
    }

    private data class FakeState(
        val api: MontoyaApi,
        val imports: () -> Int,
    )

    private fun fakeState(): FakeState {
        var config =
            """{"proxy":{"intercept_client_requests":{"do_intercept":true,"automatically_update_content_length_header_when_the_request_is_edited":true,"automatically_fix_missing_or_superfluous_new_lines_at_end_of_request":false,"rules":[{"enabled":true,"boolean_operator":"and","match_type":"file_extension","match_relationship":"does_not_match","match_condition":"js"}]},"intercept_server_responses":{"do_intercept":false,"automatically_update_content_length_header_when_the_response_is_edited":true,"rules":[{"enabled":true,"boolean_operator":"or","match_type":"content_type_header","match_relationship":"matches","match_condition":"text"}]},"intercept_web_sockets_messages":{"client_to_server_messages":true,"server_to_client_messages":false,"intercept_in_scope_only":false},"response_modification":{"unhide_hidden_form_fields":false,"enable_disabled_form_fields":false,"remove_input_field_length_limits":false,"remove_javascript_form_validation":false,"remove_all_javascript":false}}}"""
        var masterEnabled = true
        var imports = 0
        val proxy =
            fake<Proxy> { method, _ ->
                when (method.name) {
                    "isInterceptEnabled" -> masterEnabled
                    "enableIntercept" -> null.also { masterEnabled = true }
                    "disableIntercept" -> null.also { masterEnabled = false }
                    else -> null
                }
            }
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
                    "proxy" -> proxy
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
