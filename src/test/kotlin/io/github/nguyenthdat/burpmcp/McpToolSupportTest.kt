package io.github.nguyenthdat.burpmcp

import kotlin.test.Test
import kotlin.test.assertEquals

class McpToolSupportTest {
    @Test
    fun `counts UTF-8 bytes for content length`() {
        // Given / When / Then
        assertEquals(3, utf8Length("✓"))
    }

    @Test
    fun `replaces a request body and recalculates content length`() {
        // Given
        val request = "POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: 3\r\n\r\nold"

        // When
        val modified = replaceRequestBody(request, "✓new")

        // Then
        assertEquals(
            "POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: 6\r\n\r\n✓new",
            modified,
        )
    }

    @Test
    fun `quotes apostrophes for POSIX shell`() {
        // Given / When / Then
        assertEquals("'it'\\''s'", shellQuote("it's"))
    }
}
