package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.core.Registration
import burp.api.montoya.http.Http
import io.mockk.every
import io.mockk.mockk
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import java.time.Clock
import java.time.Instant
import java.time.ZoneOffset

class EventFacadeTest {
    private fun facade(capacity: Int): EventFacade {
        val api = mockk<MontoyaApi>()
        val http = mockk<Http>()
        val registration = mockk<Registration>(relaxed = true)
        every { api.http() } returns http
        every { http.registerHttpHandler(any()) } returns registration
        return EventFacade(api, Clock.fixed(Instant.ofEpochMilli(42), ZoneOffset.UTC), capacity)
    }

    @Test
    fun `sequence is monotonic and duplicate keys are coalesced`() {
        facade(8).use { events ->
            events.append("http_request", "GET https://example.test/a")
            events.append("http_response", "GET https://example.test/a")
            events.append("http_request", "GET https://example.test/b")

            val page = events.since(0, 8)

            assertEquals(listOf(2L, 3L), page.items.map { it.sequence })
            assertEquals(listOf("GET https://example.test/a", "GET https://example.test/b"), page.items.map { it.key })
            assertEquals(3L, page.latestSequence)
            assertFalse(page.gapDetected)
        }
    }

    @Test
    fun `overflow stays bounded and forces reconciliation`() {
        facade(3).use { events ->
            repeat(6) { index -> events.append("http_request", "GET https://example.test/$index") }

            val page = events.since(0, 10)

            assertTrue(page.gapDetected)
            assertTrue(page.items.size <= 3)
            assertTrue(page.items.any { it.reconcileRequired })
            assertTrue(page.items.zipWithNext().all { (left, right) -> left.sequence < right.sequence })
        }
    }

    @Test
    fun `pagination returns stable next sequence`() {
        facade(8).use { events ->
            repeat(4) { index -> events.append("http_request", "GET https://example.test/$index") }
            val first = events.since(0, 2)
            val second = events.since(first.nextSequence, 2)

            assertTrue(first.truncated)
            assertEquals(listOf(1L, 2L), first.items.map { it.sequence })
            assertEquals(listOf(3L, 4L), second.items.map { it.sequence })
            assertFalse(second.truncated)
        }
    }
}
