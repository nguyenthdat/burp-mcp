package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import java.lang.reflect.Proxy
import kotlin.test.Test
import kotlin.test.assertEquals

class CollaboratorFacadeTest {
    @Test
    fun `poll before payload generation is empty`() {
        val api = fake<MontoyaApi> { _, _ -> error("Collaborator API must not be opened while no client exists") }

        assertEquals(emptyList(), CollaboratorFacade(api).interactions())
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
