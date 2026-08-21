package io.github.nguyenthdat.burpmcp

import burp.api.montoya.BurpExtension
import burp.api.montoya.MontoyaApi
import burp.api.montoya.logging.Logging

class BurpMcpExtension : BurpExtension {
    private var httpServer: McpHttpServer? = null
    private var grpcServer: GrpcSpikeServer? = null

    override fun initialize(api: MontoyaApi) {
        val logging = api.logging()
        api.extension().setName("Burp MCP")

        val config = ExtensionConfigResolver.resolve()
        config.messages.forEach { message -> logging.logToError("[MCP] $message") }
        logging.logToOutput(
            "[MCP] Transport mode=${config.transportMode.name.lowercase()}, " +
                "HTTP=127.0.0.1:${config.httpPort}, gRPC=127.0.0.1:${config.grpcPort}",
        )

        if (config.transportMode.startsHttp) {
            startHttp(api, config.httpPort, logging)
        }
        if (config.transportMode.startsGrpc) {
            startGrpc(api, config.grpcPort, logging)
        }

        api.extension().registerUnloadingHandler {
            stopServers(logging)
        }
    }

    private fun startHttp(
        api: MontoyaApi,
        port: Int,
        logging: Logging,
    ) {
        try {
            httpServer = McpHttpServer(api, port).also { it.start() }
            logging.logToOutput("[MCP] HTTP compatibility server ready on http://127.0.0.1:$port")
            logging.logToOutput("[MCP] HTTP auth token: ~/.burp-mcp-token")
        } catch (exception: Exception) {
            httpServer = null
            logging.logToError("[MCP] HTTP compatibility server failed on port $port", exception)
        }
    }

    private fun startGrpc(
        api: MontoyaApi,
        port: Int,
        logging: Logging,
    ) {
        try {
            grpcServer = GrpcSpikeServer(api, port).also { it.start() }
            logging.logToOutput(
                "[MCP] gRPC server ready on 127.0.0.1:$port " +
                    "(HTTP/2, max message ${GRPC_MAX_MESSAGE_BYTES / (1024 * 1024)} MiB, " +
                    "max RPC ${GRPC_MAX_RPC_TIMEOUT_SECONDS}s)",
            )
            logging.logToOutput("[MCP] gRPC has no application token; any local process can connect")
        } catch (exception: Exception) {
            grpcServer?.close()
            grpcServer = null
            logging.logToError("[MCP] gRPC server failed on port $port", exception)
        }
    }

    private fun stopServers(logging: Logging) {
        grpcServer?.close()
        grpcServer = null
        httpServer?.stop()
        httpServer = null
        logging.logToOutput("[MCP] HTTP and gRPC servers stopped")
    }
}
