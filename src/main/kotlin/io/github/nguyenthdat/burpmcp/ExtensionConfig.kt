package io.github.nguyenthdat.burpmcp

internal const val DEFAULT_HTTP_PORT: Int = 9876
internal const val DEFAULT_GRPC_PORT: Int = 9877

internal enum class TransportMode {
    HTTP,
    GRPC,
    DUAL,
    ;

    val startsHttp: Boolean
        get() = this == HTTP || this == DUAL

    val startsGrpc: Boolean
        get() = this == GRPC || this == DUAL
}

internal data class ExtensionConfig(
    val transportMode: TransportMode,
    val httpPort: Int,
    val grpcPort: Int,
    val messages: List<String>,
)

internal object ExtensionConfigResolver {
    fun resolve(
        property: (String) -> String? = System::getProperty,
        environment: (String) -> String? = System::getenv,
    ): ExtensionConfig {
        val messages = mutableListOf<String>()
        val transport =
            parseTransport(
                property("burp.mcp.transport") ?: environment("BURP_MCP_TRANSPORT"),
                messages,
            )
        val httpPort =
            parsePort(
                property("burp.mcp.port") ?: environment("BURP_MCP_PORT"),
                DEFAULT_HTTP_PORT,
                "HTTP",
                messages,
            )
        val grpcPort =
            parsePort(
                property("burp.mcp.grpc.port") ?: environment("BURP_MCP_GRPC_PORT"),
                DEFAULT_GRPC_PORT,
                "gRPC",
                messages,
            )
        if (transport == TransportMode.DUAL && httpPort == grpcPort) {
            val fallback = fallbackGrpcPort(httpPort)
            messages += "HTTP and gRPC ports must differ in dual mode; gRPC reset to $fallback"
            return ExtensionConfig(transport, httpPort, fallback, messages)
        }
        return ExtensionConfig(transport, httpPort, grpcPort, messages)
    }

    private fun parseTransport(
        raw: String?,
        messages: MutableList<String>,
    ): TransportMode {
        if (raw.isNullOrBlank()) return TransportMode.DUAL
        return when (raw.trim().lowercase()) {
            "http" -> TransportMode.HTTP
            "grpc" -> TransportMode.GRPC
            "dual" -> TransportMode.DUAL
            else -> {
                messages += "Invalid transport '$raw'; using dual mode"
                TransportMode.DUAL
            }
        }
    }

    private fun parsePort(
        raw: String?,
        fallback: Int,
        label: String,
        messages: MutableList<String>,
    ): Int {
        if (raw.isNullOrBlank()) return fallback
        val parsed = raw.trim().toIntOrNull()
        if (parsed == null || parsed !in 1..65535) {
            messages += "Invalid $label port '$raw'; using $fallback"
            return fallback
        }
        return parsed
    }

    private fun fallbackGrpcPort(httpPort: Int): Int =
        if (httpPort != DEFAULT_GRPC_PORT) DEFAULT_GRPC_PORT else DEFAULT_GRPC_PORT + 1
}
