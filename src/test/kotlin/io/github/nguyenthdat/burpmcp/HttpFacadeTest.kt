package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.http.Http
import burp.api.montoya.http.message.HttpRequestResponse
import burp.api.montoya.http.message.requests.HttpRequest
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import io.mockk.verify
import kotlin.test.Test

class HttpFacadeTest {
    @Test
    fun `sending adds absent headers and updates existing headers`() {
        val api = mockk<MontoyaApi>()
        val http = mockk<Http>()
        val initial = mockk<HttpRequest>()
        val withMethod = mockk<HttpRequest>()
        val withUserAgent = mockk<HttpRequest>()
        val withCustomHeader = mockk<HttpRequest>(relaxed = true)
        val exchange = mockk<HttpRequestResponse>(relaxed = true)
        mockkStatic(HttpRequest::class)
        try {
            every { HttpRequest.httpRequestFromUrl("http://127.0.0.1/") } returns initial
            every { initial.withMethod("GET") } returns withMethod
            every { withMethod.hasHeader("User-Agent") } returns true
            every { withMethod.withUpdatedHeader("User-Agent", "test-agent") } returns withUserAgent
            every { withMethod.withAddedHeader("User-Agent", "test-agent") } returns withUserAgent
            every { withUserAgent.hasHeader("X-Burp-MCP-Test") } returns false
            every { withUserAgent.withUpdatedHeader("X-Burp-MCP-Test", "present") } returns withCustomHeader
            every { withUserAgent.withAddedHeader("X-Burp-MCP-Test", "present") } returns withCustomHeader
            every { api.http() } returns http
            every { http.sendRequest(withCustomHeader) } returns exchange
            every { exchange.request() } returns withCustomHeader

            HttpFacade(api).send(
                HttpRequestSpec(
                    url = "http://127.0.0.1/",
                    headers = linkedMapOf(
                        "User-Agent" to "test-agent",
                        "X-Burp-MCP-Test" to "present",
                    ),
                ),
            )

            verify(exactly = 1) { withMethod.withUpdatedHeader("User-Agent", "test-agent") }
            verify(exactly = 1) { withUserAgent.withAddedHeader("X-Burp-MCP-Test", "present") }
            verify(exactly = 0) { withUserAgent.withUpdatedHeader("X-Burp-MCP-Test", any()) }
        } finally {
            unmockkStatic(HttpRequest::class)
        }
    }
}
