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

        val port = resolvePort(logging)
        try {
            httpServer = McpHttpServer(api, port)
            httpServer?.start()
            logging.logToOutput("[MCP] Server started on http://127.0.0.1:$port")
            logging.logToOutput("[MCP] Configure port via -Dburp.mcp.port=<n> or BURP_MCP_PORT env var (default 9876)")
            logging.logToOutput("[MCP] Auth token: ~/.burp-mcp-token (override with -Dburp.mcp.token or BURP_MCP_TOKEN)")
            logging.logToOutput(
                "[MCP] Tools: proxy_history, send_request, intruder_attack, repeater, scanner, sitemap, intercept, encode/decode",
            )
        } catch (exception: Exception) {
            logging.logToError("[MCP] Failed to start server on port $port: ${exception.message}")
            logging.logToError("[MCP] If port is in use, set -Dburp.mcp.port=<other> and restart Burp.")
        }

        val grpcPort = resolveGrpcPort(port, logging)
        if (grpcPort != null) {
            try {
                grpcServer = GrpcSpikeServer(api, grpcPort)
                grpcServer?.start()
                logging.logToOutput("[MCP] Phase 0 gRPC spike started on 127.0.0.1:$grpcPort")
            } catch (exception: Exception) {
                grpcServer?.close()
                grpcServer = null
                logging.logToError("[MCP] Failed to start Phase 0 gRPC spike on port $grpcPort: ${exception.message}")
            }
        }

        api.extension().registerUnloadingHandler {
            grpcServer?.close()
            grpcServer = null
            httpServer?.stop()
            httpServer = null
            logging.logToOutput("[MCP] Servers stopped")
        }
    }

    private fun resolveGrpcPort(
        httpPort: Int,
        logging: Logging,
    ): Int? {
        val raw = System.getProperty("burp.mcp.grpc.port") ?: System.getenv("BURP_MCP_GRPC_PORT")
        if (raw.isNullOrBlank()) return null
        return try {
            val grpcPort = raw.trim().toInt()
            require(grpcPort in 1..65535) { "out of range" }
            require(grpcPort != httpPort) { "must differ from HTTP port" }
            grpcPort
        } catch (exception: Exception) {
            logging.logToError("[MCP] Invalid gRPC spike port '$raw'; gRPC disabled. ${exception.message}")
            null
        }
    }

    private fun resolvePort(logging: Logging): Int {
        val property = System.getProperty("burp.mcp.port")
        val environment = System.getenv("BURP_MCP_PORT")
        val raw = property ?: environment
        if (raw.isNullOrBlank()) return 9876

        return try {
            val port = raw.trim().toInt()
            require(port in 1..65535) { "out of range" }
            if (property != null) {
                logging.logToOutput("[MCP] Port from -Dburp.mcp.port: $port")
            } else {
                logging.logToOutput("[MCP] Port from BURP_MCP_PORT: $port")
            }
            port
        } catch (exception: Exception) {
            logging.logToError("[MCP] Invalid port '$raw', falling back to 9876. ${exception.message}")
            9876
        }
    }
}
