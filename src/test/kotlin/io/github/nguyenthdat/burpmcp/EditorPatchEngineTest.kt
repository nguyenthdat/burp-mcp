package io.github.nguyenthdat.burpmcp

import org.junit.jupiter.api.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class EditorPatchEngineTest {

    @Test
    fun `replace selection works correctly`() {
        val original = "GET /user?id=12345 HTTP/1.1\r\nHost: example.com\r\n\r\n"
        val start = original.indexOf("12345")
        val end = start + "12345".length

        val patched = EditorPatchEngine.applyPatchToText(
            original,
            PatchOp.ReplaceSelection("admin' OR '1'='1", start, end)
        )

        assertTrue(patched.contains("id=admin' OR '1'='1"))
        assertTrue(patched.contains("Host: example.com"))
    }

    @Test
    fun `regex replace works with case sensitivity option`() {
        val original = "POST /api HTTP/1.1\r\nHost: example.com\r\n\r\n{\"role\": \"user\", \"name\": \"USER\"}"

        val patched = EditorPatchEngine.applyPatchToText(
            original,
            PatchOp.RegexPatch("\"role\":\\s*\"user\"", "\"role\": \"admin\"", replaceAll = false, caseInsensitive = true)
        )

        assertTrue(patched.contains("\"role\": \"admin\""))
        assertTrue(patched.contains("\"name\": \"USER\""))
    }

    @Test
    fun `header patch updates existing header and recalculates Content-Length`() {
        val original = "POST /api HTTP/1.1\r\nHost: example.com\r\nAuthorization: Bearer old\r\nContent-Length: 13\r\n\r\n{\"hello\": 123}"

        val patched = EditorPatchEngine.applyPatchToText(
            original,
            PatchOp.HeaderPatch("Authorization", "Bearer new_secret_token", remove = false)
        )

        assertTrue(patched.contains("Authorization: Bearer new_secret_token"))
        assertTrue(patched.contains("Content-Length: 14"))
        assertTrue(patched.contains("{\"hello\": 123}"))
    }

    @Test
    fun `header patch removes header`() {
        val original = "GET /api HTTP/1.1\r\nHost: example.com\r\nCookie: session=123\r\n\r\n"

        val patched = EditorPatchEngine.applyPatchToText(
            original,
            PatchOp.HeaderPatch("Cookie", "", remove = true)
        )

        assertTrue(!patched.contains("Cookie:"))
        assertTrue(patched.contains("Host: example.com"))
    }

    @Test
    fun `json patch updates nested field and auto-updates Content-Length`() {
        val original = "POST /api/user HTTP/1.1\r\nHost: example.com\r\nContent-Type: application/json\r\nContent-Length: 35\r\n\r\n{\"user\": {\"role\": \"guest\", \"id\": 5}}"

        val patched = EditorPatchEngine.applyPatchToText(
            original,
            PatchOp.JsonPatch("user.role", "\"administrator\"")
        )

        assertTrue(patched.contains("\"role\":\"administrator\"") || patched.contains("\"role\": \"administrator\""))
        assertTrue(patched.contains("Content-Length:"))
    }

    @Test
    fun `param patch modifies query parameter in GET request`() {
        val original = "GET /search?q=test&page=1 HTTP/1.1\r\nHost: example.com\r\n\r\n"

        val patched = EditorPatchEngine.applyPatchToText(
            original,
            PatchOp.ParamPatch("q", "security vulnerability", remove = false, paramType = "query")
        )

        assertTrue(patched.contains("q=security+vulnerability"))
        assertTrue(patched.contains("page=1"))
    }
}
