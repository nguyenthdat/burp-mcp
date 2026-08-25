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
    private var currentSettings: GrpcSettings? = null
    private var closed = false

    @Synchronized
    fun start(settings: GrpcSettings) {
        check(!closed) { "transport lifecycle is closed" }
        settings.validate()
        val tlsBundle = if (settings.securityMode == GrpcSecurityMode.REMOTE_MTLS) TlsBundleManager().ensure(settings) else null
        try {
            rpcServer = BurpRpcServer(api, settings, tlsBundle).also { it.start() }
            currentSettings = settings
            logging.logToOutput(
                "[MCP] gRPC server ready on ${settings.bindAddress}:${settings.port} " +
                    "(${settings.securityMode}, HTTP/2, max message ${GRPC_MAX_MESSAGE_BYTES / (1024 * 1024)} MiB, " +
                    "max RPC ${GRPC_MAX_RPC_TIMEOUT_SECONDS}s)",
            )
            if (tlsBundle == null) {
                logging.logToOutput("[MCP] local plaintext is restricted to IPv4 loopback")
            } else {
                logging.logToOutput("[MCP] remote gRPC requires a client certificate trusted by ${tlsBundle.caCertificate}")
            }
        } catch (exception: Exception) {
            rpcServer?.close()
            rpcServer = null
            currentSettings = null
            logging.logToError("[MCP] gRPC server failed on ${settings.bindAddress}:${settings.port}", exception)
            throw exception
        }
    }

    @Synchronized
    fun restart(settings: GrpcSettings) {
        check(!closed) { "transport lifecycle is closed" }
        val previousSettings = currentSettings
        rpcServer?.close()
        rpcServer = null
        currentSettings = null
        try {
            start(settings)
        } catch (exception: Exception) {
            if (previousSettings != null) {
                runCatching { start(previousSettings) }
                    .onFailure { rollback -> logging.logToError("[MCP] failed to restore previous gRPC server", rollback) }
            }
            throw exception
        }
    }

    @Synchronized
    fun settings(): GrpcSettings? = currentSettings

    @Synchronized
    override fun close() {
        if (closed) return
        closed = true
        rpcServer?.close()
        rpcServer = null
        currentSettings = null
        logging.logToOutput("[MCP] gRPC server stopped")
    }
}
