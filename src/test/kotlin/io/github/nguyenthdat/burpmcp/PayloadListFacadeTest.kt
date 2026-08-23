package io.github.nguyenthdat.burpmcp

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class PayloadListFacadeTest {
    @Test
    fun `crud and paged reads preserve payload order`() {
        val lists = PayloadListFacade()
        lists.create("words", "Words", listOf("one", "two"))
        lists.update("words", "append", listOf("three"), 0, emptyList(), null)
        lists.update("words", "insert", listOf("zero"), 0, emptyList(), null)

        val page = lists.page("words", 1, 2)
        assertEquals(listOf("one", "two"), page.payloads)
        assertEquals(4, page.total)
        assertEquals(3, page.nextOffset)
        assertEquals(listOf("zero", "one", "two", "three"), lists.get("words").payloads)
    }

    @Test
    fun `import supports lines and json and rejects invalid updates`() {
        val lists = PayloadListFacade()
        assertEquals(listOf("a", "b"), lists.import("lines", "Lines", "a\r\nb\r\n", "lines", false).payloads)
        assertEquals(listOf("a", "", "b"), lists.import("json", "Json", "[\"a\",\"\",\"b\"]", "json", false).payloads)
        assertFailsWith<IllegalArgumentException> {
            lists.update("lines", "replace", listOf("x", "y"), 0, emptyList(), null)
        }
    }

    @Test
    fun `delete and bounded slices are safe`() {
        val lists = PayloadListFacade()
        lists.create("words", "Words", (1..600).map(Int::toString))
        assertEquals(500, lists.boundedSlice("words", 0).size)
        assertTrue(lists.delete("words"))
        assertFalse(lists.delete("words"))
    }
}
