package io.github.nguyenthdat.burpmcp

import burp.api.montoya.BurpExtension
import burp.api.montoya.MontoyaApi

class BurpMcpExtension : BurpExtension {
    private var transports: TransportLifecycle? = null

    override fun initialize(api: MontoyaApi) {
        val logging = api.logging()
        api.extension().setName("Burp MCP")

        val config = ExtensionConfigResolver.resolve()
        config.messages.forEach { message -> logging.logToError("[MCP] $message") }
        logging.logToOutput("[MCP] gRPC transport=127.0.0.1:${config.grpcPort}")

        transports = TransportLifecycle(api, logging).also { it.start(config) }
        api.extension().registerUnloadingHandler {
            transports?.close()
            transports = null
        }
    }
}
