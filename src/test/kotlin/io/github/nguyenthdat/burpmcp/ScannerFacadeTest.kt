package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.scanner.audit.issues.AuditIssue
import burp.api.montoya.sitemap.SiteMap
import java.lang.reflect.Proxy
import kotlin.test.Test
import kotlin.test.assertFailsWith

class ScannerFacadeTest {
    @Test
    fun `rejects duplicate issue provenance for normalized name and url`() {
        val existing = fake<AuditIssue>(
            mapOf(
                "name" to { "Duplicate finding" },
                "baseUrl" to { "https://example.test/path/" },
            ),
        )
        val siteMap = fake<SiteMap>(mapOf("issues" to { listOf(existing) }))
        val facade = ScannerFacade(fake(mapOf("siteMap" to { siteMap })))

        assertFailsWith<IllegalArgumentException> {
            facade.addIssue(
                "Duplicate finding",
                "https://example.test/path",
                "detail",
                "remediation",
                "low",
                "firm",
            )
        }
    }

    @Suppress("UNCHECKED_CAST")
    private inline fun <reified T> fake(methods: Map<String, () -> Any?>): T =
        Proxy.newProxyInstance(T::class.java.classLoader, arrayOf(T::class.java)) { proxy, method, args ->
            when (method.name) {
                "toString" -> T::class.simpleName
                "hashCode" -> System.identityHashCode(proxy)
                "equals" -> proxy === args?.firstOrNull()
                else -> methods[method.name]?.invoke()
                    ?: throw UnsupportedOperationException("unexpected method: ${method.name}")
            }
        } as T
}
