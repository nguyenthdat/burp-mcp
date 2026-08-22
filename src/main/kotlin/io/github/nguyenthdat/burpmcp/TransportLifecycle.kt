package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.logging.Logging
import io.github.nguyenthdat.burpmcp.rpc.BurpRpcServer
import io.github.nguyenthdat.burpmcp.rpc.GRPC_MAX_MESSAGE_BYTES
import io.github.nguyenthdat.burpmcp.rpc.GRPC_MAX_RPC_TIMEOUT_SECONDS

/** Owns the sole production transport and its shutdown ordering. */
internal class TransportLifecycle(
    private val api: MontoyaApi,
    private val logging: Logging,
) : AutoCloseable {
    private var rpcServer: BurpRpcServer? = null
    private var closed = false

    fun start(config: ExtensionConfig) {
        check(!closed) { "transport lifecycle is closed" }
        try {
            rpcServer = BurpRpcServer(api, config.grpcPort).also { it.start() }
            logging.logToOutput(
                "[MCP] gRPC server ready on 127.0.0.1:${config.grpcPort} " +
                    "(HTTP/2, max message ${GRPC_MAX_MESSAGE_BYTES / (1024 * 1024)} MiB, " +
                    "max RPC ${GRPC_MAX_RPC_TIMEOUT_SECONDS}s)",
            )
            logging.logToOutput("[MCP] gRPC has no application token; any local process can connect")
        } catch (exception: Exception) {
            rpcServer?.close()
            rpcServer = null
            logging.logToError("[MCP] gRPC server failed on port ${config.grpcPort}", exception)
        }
    }

    override fun close() {
        if (closed) return
        closed = true
        rpcServer?.close()
        rpcServer = null
        logging.logToOutput("[MCP] gRPC server stopped")
    }
}
