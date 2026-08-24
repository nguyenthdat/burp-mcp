package io.github.nguyenthdat.burpmcp

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class ExtensionConfigTest {
    @Test
    fun `defaults to sole gRPC transport`() {
        val config = ExtensionConfigResolver.resolve(property = { null }, environment = { null })

        assertEquals(DEFAULT_GRPC_PORT, config.grpcPort)
        assertTrue(config.messages.isEmpty())
    }

    @Test
    fun `reads gRPC port from JVM property before environment`() {
        val config =
            ExtensionConfigResolver.resolve(
                property = { name -> if (name == "burp.mcp.grpc.port") "10002" else null },
                environment = { name -> if (name == "BURP_MCP_GRPC_PORT") "10003" else null },
            )

        assertEquals(10002, config.grpcPort)
    }

    @Test
    fun `rejects invalid port and reports removed HTTP configuration`() {
        val config =
            ExtensionConfigResolver.resolve(
                property = { name ->
                    when (name) {
                        "burp.mcp.grpc.port" -> "70000"
                        "burp.mcp.transport" -> "dual"
                        "burp.mcp.port" -> "9876"
                        else -> null
                    }
                },
                environment = { null },
            )

        assertEquals(DEFAULT_GRPC_PORT, config.grpcPort)
        assertEquals(3, config.messages.size)
        assertTrue(config.messages.any { it.contains("only transport") })
        assertTrue(config.messages.any { it.contains("removed in v3") })
        assertTrue(config.messages.any { it.contains("Invalid gRPC port") })
    }
}
