package io.github.nguyenthdat.burpmcp

internal const val DEFAULT_GRPC_PORT: Int = 9877

internal data class ExtensionConfig(
    val grpcPort: Int,
    val messages: List<String>,
)

internal object ExtensionConfigResolver {
    fun resolve(
        property: (String) -> String? = System::getProperty,
        environment: (String) -> String? = System::getenv,
    ): ExtensionConfig {
        val messages = mutableListOf<String>()
        val deprecatedTransport = property("burp.mcp.transport") ?: environment("BURP_MCP_TRANSPORT")
        if (!deprecatedTransport.isNullOrBlank()) {
            messages += "burp.mcp.transport/BURP_MCP_TRANSPORT is ignored in v3; gRPC is the only transport"
        }
        val deprecatedHttpPort = property("burp.mcp.port") ?: environment("BURP_MCP_PORT")
        if (!deprecatedHttpPort.isNullOrBlank()) {
            messages += "burp.mcp.port/BURP_MCP_PORT is removed in v3; use burp.mcp.grpc.port/BURP_MCP_GRPC_PORT"
        }
        val grpcPort =
            parsePort(
                property("burp.mcp.grpc.port") ?: environment("BURP_MCP_GRPC_PORT"),
                messages,
            )
        return ExtensionConfig(grpcPort, messages)
    }

    private fun parsePort(
        raw: String?,
        messages: MutableList<String>,
    ): Int {
        if (raw.isNullOrBlank()) return DEFAULT_GRPC_PORT
        val parsed = raw.trim().toIntOrNull()
        if (parsed == null || parsed !in 1..65535) {
            messages += "Invalid gRPC port '$raw'; using $DEFAULT_GRPC_PORT"
            return DEFAULT_GRPC_PORT
        }
        return parsed
    }
}
