package io.github.nguyenthdat.burpmcp

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class ExtensionConfigTest {
    @Test
    fun `defaults to dual transport with distinct loopback ports`() {
        val config = ExtensionConfigResolver.resolve(property = { null }, environment = { null })

        assertEquals(TransportMode.DUAL, config.transportMode)
        assertEquals(DEFAULT_HTTP_PORT, config.httpPort)
        assertEquals(DEFAULT_GRPC_PORT, config.grpcPort)
        assertTrue(config.messages.isEmpty())
    }

    @Test
    fun `system properties override environment values`() {
        val properties =
            mapOf(
                "burp.mcp.transport" to "grpc",
                "burp.mcp.port" to "10001",
                "burp.mcp.grpc.port" to "10002",
            )
        val environment =
            mapOf(
                "BURP_MCP_TRANSPORT" to "http",
                "BURP_MCP_PORT" to "20001",
                "BURP_MCP_GRPC_PORT" to "20002",
            )

        val config =
            ExtensionConfigResolver.resolve(
                property = properties::get,
                environment = environment::get,
            )

        assertEquals(TransportMode.GRPC, config.transportMode)
        assertFalse(config.transportMode.startsHttp)
        assertTrue(config.transportMode.startsGrpc)
        assertEquals(10001, config.httpPort)
        assertEquals(10002, config.grpcPort)
    }

    @Test
    fun `invalid settings fall back safely and keep dual ports distinct`() {
        val properties =
            mapOf(
                "burp.mcp.transport" to "external",
                "burp.mcp.port" to "9877",
                "burp.mcp.grpc.port" to "9877",
            )

        val config = ExtensionConfigResolver.resolve(property = properties::get, environment = { null })

        assertEquals(TransportMode.DUAL, config.transportMode)
        assertEquals(9877, config.httpPort)
        assertEquals(9878, config.grpcPort)
        assertEquals(2, config.messages.size)
    }

    @Test
    fun `transport modes expose only their selected server`() {
        assertTrue(TransportMode.HTTP.startsHttp)
        assertFalse(TransportMode.HTTP.startsGrpc)
        assertFalse(TransportMode.GRPC.startsHttp)
        assertTrue(TransportMode.GRPC.startsGrpc)
    }
}
