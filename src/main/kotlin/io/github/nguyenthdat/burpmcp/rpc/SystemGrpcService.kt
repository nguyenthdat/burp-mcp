package io.github.nguyenthdat.burpmcp.rpc

import burp.api.montoya.MontoyaApi
import io.github.nguyenthdat.burpmcp.ProjectFacade
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
    override fun ping(request: PingRequest, responseObserver: StreamObserver<PingResponse>) =
        responseObserver.respond { pingValue(request) }

    internal fun pingValue(@Suppress("UNUSED_PARAMETER") request: PingRequest): PingResponse {
        if (Context.current().isCancelled) throw Status.CANCELLED.asException()
        return PingResponse.newBuilder()
            .setServer("burp-mcp")
            .setVersion(extensionVersion())
            .setUnixMillis(clock.millis())
            .build()
    }

    override fun echoBytes(request: EchoBytesRequest, responseObserver: StreamObserver<EchoBytesResponse>) =
        responseObserver.respond { echoBytesValue(request) }

    internal fun echoBytesValue(request: EchoBytesRequest): EchoBytesResponse {
        val delayMillis = request.delayMillis.toLong().coerceAtMost(5_000)
        if (delayMillis > 0) {
            val deadline = System.nanoTime() + TimeUnit.MILLISECONDS.toNanos(delayMillis)
            while (System.nanoTime() < deadline) {
                if (Context.current().isCancelled) throw Status.CANCELLED.asException()
                Thread.sleep(min(10, delayMillis))
            }
        }
        if (Context.current().isCancelled) throw Status.CANCELLED.asException()
        return EchoBytesResponse.newBuilder().setPayload(request.payload).build()
    }

    override fun serverInfo(request: ServerInfoRequest, responseObserver: StreamObserver<ServerInfoResponse>) =
        responseObserver.respond { serverInfoValue(request) }

    internal fun serverInfoValue(@Suppress("UNUSED_PARAMETER") request: ServerInfoRequest): ServerInfoResponse {
        val version = api.burpSuite().version()
        val project = ProjectFacade(api).identity()
        val builder = ServerInfoResponse.newBuilder()
            .setExtension("burp-mcp")
            .setVersion(extensionVersion())
            .addAllCapabilities(
                listOf(
                    "proxy.read",
                    "sitemap.read",
                    "scanner.read",
                    "cookies.read",
                    "transport.echo",
                    "lifecycle.restart",
                    "editor.active.read",
                    "editor.active.write_guarded",
                    "editor.websocket.read",
                    "editor.websocket.write_guarded",
                ),
            )
            .setMaxMessageBytes(GRPC_MAX_MESSAGE_BYTES)
            .setMaxPageSize(GRPC_MAX_PAGE_SIZE)
            .setMaxConcurrentCallsPerConnection(GRPC_MAX_CONCURRENT_CALLS_PER_CONNECTION)
            .setMaxRpcTimeoutSeconds(GRPC_MAX_RPC_TIMEOUT_SECONDS.toInt())
            .setMaxResponseBytes(GRPC_MAX_RESPONSE_BYTES)
            .setProjectId(project.projectId)
            .setProjectName(project.projectName)
            .setGraphId(project.graphId)
            .setProjectTemporary(project.temporary)
        version?.name()?.let(builder::setBurpName)
        version?.edition()?.name?.let(builder::setBurpEdition)
        version?.buildNumber()?.let(builder::setBurpBuildNumber)
        return builder.build()
    }

    private fun extensionVersion(): String =
        SystemGrpcService::class.java.`package`.implementationVersion ?: "development"
}
