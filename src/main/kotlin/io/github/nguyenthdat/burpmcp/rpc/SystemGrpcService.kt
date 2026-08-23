package io.github.nguyenthdat.burpmcp.rpc

import burp.api.montoya.MontoyaApi
import io.github.nguyenthdat.burpmcp.grpc.v1.BurpServiceGrpc
import io.github.nguyenthdat.burpmcp.grpc.v1.EchoBytesRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.EchoBytesResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.PingRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.PingResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.ServerInfoRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ServerInfoResponse
import io.grpc.Context
import io.grpc.Status
import io.grpc.stub.StreamObserver
import java.time.Clock
import java.util.concurrent.TimeUnit
import kotlin.math.min

internal class SystemGrpcService(
    private val api: MontoyaApi,
    private val clock: Clock,
) : BurpServiceGrpc.BurpServiceImplBase() {
    override fun ping(request: PingRequest, responseObserver: StreamObserver<PingResponse>) {
        if (Context.current().isCancelled) {
            responseObserver.onError(Status.CANCELLED.asRuntimeException())
            return
        }
        responseObserver.onNext(
            PingResponse.newBuilder()
                .setServer("burp-mcp-kotlin")
                .setVersion(extensionVersion())
                .setUnixMillis(clock.millis())
                .build(),
        )
        responseObserver.onCompleted()
    }

    override fun echoBytes(request: EchoBytesRequest, responseObserver: StreamObserver<EchoBytesResponse>) {
        val delayMillis = request.delayMillis.toLong().coerceAtMost(5_000)
        if (delayMillis > 0) {
            val deadline = System.nanoTime() + TimeUnit.MILLISECONDS.toNanos(delayMillis)
            while (System.nanoTime() < deadline) {
                if (Context.current().isCancelled) {
                    responseObserver.onError(Status.CANCELLED.asRuntimeException())
                    return
                }
                Thread.sleep(min(10, delayMillis))
            }
        }
        if (Context.current().isCancelled) {
            responseObserver.onError(Status.CANCELLED.asRuntimeException())
            return
        }
        responseObserver.onNext(EchoBytesResponse.newBuilder().setPayload(request.payload).build())
        responseObserver.onCompleted()
    }

    override fun serverInfo(request: ServerInfoRequest, responseObserver: StreamObserver<ServerInfoResponse>) {
        val version = api.burpSuite().version()
        val builder = ServerInfoResponse.newBuilder()
            .setExtension("Burp MCP")
            .setVersion(extensionVersion())
            .addAllCapabilities(listOf("proxy.read", "sitemap.read", "scanner.read", "cookies.read", "transport.echo", "lifecycle.restart"))
            .setMaxMessageBytes(GRPC_MAX_MESSAGE_BYTES)
            .setMaxPageSize(GRPC_MAX_PAGE_SIZE)
            .setMaxConcurrentCallsPerConnection(GRPC_MAX_CONCURRENT_CALLS_PER_CONNECTION)
            .setMaxRpcTimeoutSeconds(GRPC_MAX_RPC_TIMEOUT_SECONDS.toInt())
            .setMaxResponseBytes(GRPC_MAX_RESPONSE_BYTES)
            .setBurpVersion(version?.toString().orEmpty())
            .setBurpEdition(version?.edition()?.name ?: "UNKNOWN")
            .setBurpBuildNumber(version?.buildNumber() ?: 0)
        version?.name()?.let(builder::setBurpName)
        responseObserver.onNext(builder.build())
        responseObserver.onCompleted()
    }

    private fun extensionVersion(): String =
        SystemGrpcService::class.java.`package`.implementationVersion ?: "development"
}
