package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.core.Registration
import burp.api.montoya.http.Http
import burp.api.montoya.proxy.Proxy
import java.lang.reflect.Proxy as ReflectionProxy
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotEquals
import kotlin.test.assertTrue

class SessionRuleFacadeTest {
    @Test
    fun `creates gets updates lists and deletes rules by stable id`() {
        val registrations = mutableListOf<FakeRegistration>()
        val facade = SessionRuleFacade(fakeApi(registrations), {})
        val created = facade.create(rule(description = "first"))

        assertTrue(created.id.isNotBlank())
        assertEquals(created, facade.get(created.id))
        assertEquals(listOf(created), facade.list())

        val updated = facade.update(created.copy(description = "updated", enabled = false))
        assertEquals(created.id, updated.id)
        assertEquals("updated", facade.get(created.id).description)
        assertFalse(facade.get(created.id).enabled)
        assertEquals(2, registrations.count { it.deregistered })

        assertTrue(facade.remove(created.id))
        assertTrue(facade.list().isEmpty())
        assertFalse(facade.remove(created.id))
    }

    @Test
    fun `preserves caller supplied ids and rejects duplicate ids`() {
        val facade = SessionRuleFacade(fakeApi(mutableListOf()), {})
        val supplied = facade.create(rule(id = "rule-1"))
        assertEquals("rule-1", supplied.id)

        kotlin.test.assertFailsWith<IllegalArgumentException> {
            facade.create(rule(id = "rule-1"))
        }
    }

    private fun rule(id: String = "", description: String = "test rule") =
        SessionRule(
            id = id,
            description = description,
            actionType = "replace_text",
            find = "one",
            replacement = "two",
            headerName = "",
            parameterName = "",
            macroDescription = "",
            urlContains = "example.test",
            tools = setOf("proxy"),
            enabled = true,
        )

    private fun fakeApi(registrations: MutableList<FakeRegistration>): MontoyaApi {
        val http = fake<Http> {
            when (it.name) {
                "registerSessionHandlingAction" -> FakeRegistration().also(registrations::add)
                else -> null
            }
        }
        val proxy = fake<Proxy> {
            when (it.name) {
                "registerRequestHandler" -> FakeRegistration().also(registrations::add)
                else -> null
            }
        }
        return fake {
            when (it.name) {
                "http" -> http
                "proxy" -> proxy
                else -> null
            }
        }
    }

    private class FakeRegistration : Registration {
        var deregistered = false
        override fun isRegistered(): Boolean = !deregistered
        override fun deregister() { deregistered = true }
    }

    @Suppress("UNCHECKED_CAST")
    private inline fun <reified T> fake(crossinline handler: (java.lang.reflect.Method) -> Any?): T =
        ReflectionProxy.newProxyInstance(T::class.java.classLoader, arrayOf(T::class.java)) { proxy, method, _ ->
            when (method.name) {
                "toString" -> "Fake${T::class.simpleName}"
                "hashCode" -> System.identityHashCode(proxy)
                "equals" -> false
                else -> handler(method)
            }
        } as T
}
