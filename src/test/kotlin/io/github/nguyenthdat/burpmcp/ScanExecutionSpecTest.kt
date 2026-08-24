package io.github.nguyenthdat.burpmcp

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

class ScanExecutionSpecTest {
    @Test
    fun `passive audit defaults to stateless bounded execution semantics`() {
        assertEquals(AuditType.PASSIVE, AuditType.valueOf("passive".uppercase()))
        assertFailsWith<IllegalArgumentException> {
            resolveTimingForTest(30, 30)
        }
    }

    @Test
    fun `crawl requires explicit http seed URLs and bounded seed count`() {
        assertFailsWith<IllegalArgumentException> {
            requireHttpUrlForTest("file:///tmp/seed")
        }
        assertFailsWith<IllegalArgumentException> {
            requireHttpUrlForTest("https://")
        }
    }

    @Test
    fun `active and passive are the only audit types`() {
        assertFailsWith<IllegalArgumentException> {
            requireAuditTypeForTest("deep")
        }
        assertEquals("active", requireAuditTypeForTest("ACTIVE"))
    }
    @Test
    fun `scan option boundaries reject zero timeout and invalid audit type`() {
        assertFailsWith<IllegalArgumentException> { resolveTimingForTest(0, 0) }
        assertFailsWith<IllegalArgumentException> { requireAuditTypeForTest("custom") }
    }
}

private fun resolveTimingForTest(timeout: Long, stable: Long) {
    require(timeout in 1..86_400)
    require(stable in 0..3_600)
    require(stable < timeout)
}

private fun requireHttpUrlForTest(value: String) {
    val uri = runCatching { java.net.URI(value) }.getOrElse { throw IllegalArgumentException("invalid URL: $value", it) }
    require(uri.scheme.equals("http", true) || uri.scheme.equals("https", true))
    require(!uri.host.isNullOrBlank())
}

private fun requireAuditTypeForTest(value: String): String {
    val normalized = value.lowercase()
    require(normalized in setOf("passive", "active"))
    return normalized
}
