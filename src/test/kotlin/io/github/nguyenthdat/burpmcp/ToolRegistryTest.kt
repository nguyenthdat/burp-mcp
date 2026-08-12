package io.github.nguyenthdat.burpmcp

import com.google.gson.JsonObject
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNull

class ToolRegistryTest {
    @Test
    fun `lists advertised tools while keeping legacy tools callable`() {
        // Given
        val registry =
            ToolRegistry(
                listOf(
                    RegisteredTool("visible", advertised = true) { params -> params },
                    RegisteredTool("legacy", advertised = false) { params -> params },
                ),
            )
        val params = JsonObject().apply { addProperty("value", "kept") }

        // When
        val advertisedNames = registry.advertisedNames()
        val legacyResult = registry.invoke("legacy", params)

        // Then
        assertEquals(listOf("visible"), advertisedNames)
        assertEquals("kept", legacyResult?.get("value")?.asString)
        assertNull(registry.invoke("missing", params))
    }

    @Test
    fun `rejects duplicate tool names`() {
        // Given
        val tools =
            listOf(
                RegisteredTool("duplicate") { JsonObject() },
                RegisteredTool("duplicate") { JsonObject() },
            )

        // When / Then
        assertFailsWith<IllegalArgumentException> { ToolRegistry(tools) }
    }
}
