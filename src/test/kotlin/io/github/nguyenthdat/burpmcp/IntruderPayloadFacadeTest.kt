package io.github.nguyenthdat.burpmcp

import java.nio.charset.StandardCharsets
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

class IntruderPayloadFacadeTest {
    @Test
    fun `processor specs reject invalid declarative operations`() {
        assertFailsWith<IllegalArgumentException> {
            PayloadProcessorSpec("id", "name", PayloadProcessorOperation.PREFIX, "", "").validate()
        }
        assertFailsWith<IllegalArgumentException> {
            PayloadProcessorSpec("id", "name", PayloadProcessorOperation.REGEX_REPLACE, "(", "x").validate()
        }
        assertFailsWith<IllegalArgumentException> {
            PayloadProcessorSpec("id", "name", PayloadProcessorOperation.SHA256, "unexpected", "").validate()
        }
    }

    @Test
    fun `generator specs enforce nonempty bounded output`() {
        assertFailsWith<IllegalArgumentException> {
            PayloadGeneratorSpec("id", "name", emptyList(), 1).validate()
        }
        assertFailsWith<IllegalArgumentException> {
            PayloadGeneratorSpec("id", "name", listOf("one"), 2).validate()
        }
        PayloadGeneratorSpec("id", "name", listOf("one", "two"), 2).validate()
    }

    @Test
    fun `prefix and sha256 processors transform bytes deterministically`() {
        val current = "value".toByteArray(StandardCharsets.UTF_8)
        val prefix = transformPayload(
            PayloadProcessorSpec("prefix", "prefix", PayloadProcessorOperation.PREFIX, "pre-", ""),
            current,
            "value",
        )
        val digest = transformPayload(
            PayloadProcessorSpec("sha", "sha", PayloadProcessorOperation.SHA256, "", ""),
            current,
            "value",
        )

        assertEquals("pre-value", String(prefix, StandardCharsets.UTF_8))
        assertEquals(
            "cd42404d52ad55ccfa9aca4adc828aa5800ad9d385a0671fbcbf724118320619",
            String(digest, StandardCharsets.US_ASCII),
        )
    }

}
