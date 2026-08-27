package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.scanner.Scanner
import burp.api.montoya.scanner.audit.issues.AuditIssue
import burp.api.montoya.sitemap.SiteMap
import java.nio.file.Files
import java.nio.file.Path
import java.util.concurrent.Executors
import java.util.concurrent.TimeUnit
import java.lang.reflect.Proxy
import kotlin.test.Test
import kotlin.test.assertFailsWith
import kotlin.test.assertEquals

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

    @Test
    fun `waits for scanner report to be written`() {
        val issue = fake<AuditIssue>(emptyMap())
        val siteMap = fake<SiteMap>(mapOf("issues" to { listOf(issue) }))
        val writer = Executors.newSingleThreadExecutor()
        val scanner =
            java.lang.reflect.Proxy.newProxyInstance(Scanner::class.java.classLoader, arrayOf(Scanner::class.java)) { proxy, method, args ->
                when (method.name) {
                    "generateReport" -> {
                        val path = args!![2] as Path
                        writer.submit {
                            Thread.sleep(75)
                            Files.writeString(path, "report")
                        }
                        null
                    }
                    "toString" -> "Scanner"
                    "hashCode" -> System.identityHashCode(proxy)
                    "equals" -> proxy === args?.firstOrNull()
                    else -> throw UnsupportedOperationException("unexpected method: ${method.name}")
                }
            } as Scanner
        val facade = ScannerFacade(fake(mapOf("siteMap" to { siteMap }, "scanner" to { scanner })))
        val directory = Files.createTempDirectory("burp-mcp-report")
        val path = directory.resolve("report.html")

        try {
            val result = facade.generateReport("html", path.toString(), emptyList())

            assertEquals("report", Files.readString(path))
            assertEquals(6L, result.sizeBytes)
        } finally {
            writer.shutdownNow()
            writer.awaitTermination(1, TimeUnit.SECONDS)
            Files.deleteIfExists(path)
            Files.deleteIfExists(directory)
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
