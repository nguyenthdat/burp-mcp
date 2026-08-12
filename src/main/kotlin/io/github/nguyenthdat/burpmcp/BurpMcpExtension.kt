package io.github.nguyenthdat.burpmcp

import burp.api.montoya.BurpExtension
import burp.api.montoya.MontoyaApi
import burp.api.montoya.logging.Logging

class BurpMcpExtension : BurpExtension {
    private var server: McpHttpServer? = null

    override fun initialize(api: MontoyaApi) {
        val logging = api.logging()
        api.extension().setName("Burp MCP")

        val port = resolvePort(logging)
        try {
            server = McpHttpServer(api, port)
            server?.start()
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

        api.extension().registerUnloadingHandler {
            server?.let {
                it.stop()
                logging.logToOutput("[MCP] Server stopped")
            }
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
