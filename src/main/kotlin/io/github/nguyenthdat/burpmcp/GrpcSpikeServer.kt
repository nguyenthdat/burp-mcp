package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.proxy.ProxyHttpRequestResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.BurpServiceGrpc
import io.github.nguyenthdat.burpmcp.grpc.v1.EchoBytesRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.EchoBytesResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.PageInfo
import io.github.nguyenthdat.burpmcp.grpc.v1.PingRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.PingResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.ProxyHistoryEntry
import io.github.nguyenthdat.burpmcp.grpc.v1.ProxyHistoryRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ProxyHistoryResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.ServerInfoRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ServerInfoResponse
import io.grpc.Context
import io.grpc.Server
import io.grpc.Status
import io.grpc.netty.shaded.io.grpc.netty.NettyServerBuilder
import io.grpc.stub.StreamObserver
import java.net.InetAddress
import java.net.InetSocketAddress
import java.time.Clock
import java.util.concurrent.ArrayBlockingQueue
import java.util.concurrent.ExecutorService
import java.util.concurrent.ThreadPoolExecutor
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicBoolean
import kotlin.math.min

internal const val GRPC_MAX_MESSAGE_BYTES: Int = 16 * 1024 * 1024
internal const val GRPC_MAX_PAGE_SIZE: Int = 500
private const val GRPC_DEFAULT_PAGE_SIZE: Int = 100
private const val GRPC_SHUTDOWN_SECONDS: Long = 5

internal class GrpcSpikeServer(
    private val api: MontoyaApi,
    private val port: Int,
    private val clock: Clock = Clock.systemUTC(),
    private val serverFactory: (InetSocketAddress, BurpServiceGrpc.BurpServiceImplBase, ExecutorService) -> Server =
        { address, service, executor ->
            NettyServerBuilder
                .forAddress(address)
                .executor(executor)
                .addService(service)
                .maxInboundMessageSize(GRPC_MAX_MESSAGE_BYTES)
                .build()
        },
) : AutoCloseable {
    private val running = AtomicBoolean(false)
    private var server: Server? = null
    private var executor: ExecutorService? = null

    fun start() {
        check(running.compareAndSet(false, true)) { "gRPC server is already running" }
        val address = loopbackAddress(port)
        val workerPool =
            ThreadPoolExecutor(
                8,
                8,
                0,
                TimeUnit.MILLISECONDS,
                ArrayBlockingQueue(64),
                { runnable -> Thread(runnable, "burp-mcp-grpc").apply { isDaemon = true } },
                ThreadPoolExecutor.AbortPolicy(),
            )
        executor = workerPool
        try {
            server = serverFactory(address, GrpcBurpService(api, clock), workerPool).start()
        } catch (exception: Exception) {
            workerPool.shutdownNow()
            executor = null
            running.set(false)
            throw exception
        }
    }

    fun isRunning(): Boolean = running.get() && server?.isShutdown == false

    override fun close() {
        val current = server
        server = null
        running.set(false)
        if (current != null) {
            current.shutdown()
            if (!current.awaitTermination(GRPC_SHUTDOWN_SECONDS, TimeUnit.SECONDS)) {
                current.shutdownNow()
                current.awaitTermination(GRPC_SHUTDOWN_SECONDS, TimeUnit.SECONDS)
            }
        }
        executor?.let { workerPool ->
            workerPool.shutdown()
            if (!workerPool.awaitTermination(GRPC_SHUTDOWN_SECONDS, TimeUnit.SECONDS)) {
                workerPool.shutdownNow()
                workerPool.awaitTermination(GRPC_SHUTDOWN_SECONDS, TimeUnit.SECONDS)
            }
        }
        executor = null
    }

    internal companion object {
        fun loopbackAddress(port: Int): InetSocketAddress {
            require(port in 1..65535) { "gRPC port must be between 1 and 65535" }
            val address = InetAddress.getByName("127.0.0.1")
            require(address.isLoopbackAddress) { "gRPC server address must be loopback" }
            return InetSocketAddress(address, port)
        }
    }
}

internal class GrpcBurpService(
    private val api: MontoyaApi,
    private val clock: Clock,
) : BurpServiceGrpc.BurpServiceImplBase() {
    override fun ping(
        request: PingRequest,
        responseObserver: StreamObserver<PingResponse>,
    ) {
        if (Context.current().isCancelled) {
            responseObserver.onError(Status.CANCELLED.asRuntimeException())
            return
        }
        responseObserver.onNext(
            PingResponse
                .newBuilder()
                .setServer("burp-mcp-kotlin")
                .setVersion(extensionVersion())
                .setUnixMillis(clock.millis())
                .build(),
        )
        responseObserver.onCompleted()
    }

    override fun echoBytes(
        request: EchoBytesRequest,
        responseObserver: StreamObserver<EchoBytesResponse>,
    ) {
        val delayMillis = request.delayMillis.toLong().coerceAtMost(5_000)
        if (delayMillis > 0) {
            val deadline = System.nanoTime() + TimeUnit.MILLISECONDS.toNanos(delayMillis)
            while (System.nanoTime() < deadline) {
                if (Context.current().isCancelled) {
                    responseObserver.onError(Status.CANCELLED.asRuntimeException())
                    return
                }
                Thread.sleep(minOf(10, delayMillis))
            }
        }
        if (Context.current().isCancelled) {
            responseObserver.onError(Status.CANCELLED.asRuntimeException())
            return
        }
        responseObserver.onNext(EchoBytesResponse.newBuilder().setPayload(request.payload).build())
        responseObserver.onCompleted()
    }

    override fun proxyHistory(
        request: ProxyHistoryRequest,
        responseObserver: StreamObserver<ProxyHistoryResponse>,
    ) {
        try {
            val history: List<ProxyHttpRequestResponse> = api.proxy().history()
            val requestedLimit = request.page?.limit ?: 0
            val limit = if (requestedLimit <= 0) GRPC_DEFAULT_PAGE_SIZE else requestedLimit.coerceAtMost(GRPC_MAX_PAGE_SIZE)
            val cursor = request.page?.cursor?.takeIf(String::isNotBlank)
            val parsedOffset = cursor?.toIntOrNull()
            val offset = parsedOffset ?: 0
            if ((cursor != null && parsedOffset == null) || offset < 0 || offset > history.size) {
                responseObserver.onError(Status.INVALID_ARGUMENT.withDescription("cursor must be a valid history offset").asRuntimeException())
                return
            }
            val reversedIndices = history.indices.reversed().toList()
            val end = min(offset + limit, reversedIndices.size)
            val builder = ProxyHistoryResponse.newBuilder()
            for (position in offset until end) {
                if (Context.current().isCancelled) {
                    responseObserver.onError(Status.CANCELLED.asRuntimeException())
                    return
                }
                val index = reversedIndices[position]
                val entry = history[index]
                val response = entry.response()
                builder.addItems(
                    ProxyHistoryEntry
                        .newBuilder()
                        .setIndex(index)
                        .setMethod(entry.finalRequest().method())
                        .setUrl(entry.finalRequest().url())
                        .setStatus(response?.statusCode()?.toInt() ?: 0)
                        .setLength(response?.body()?.length()?.toLong() ?: 0)
                        .build(),
                )
            }
            builder.page =
                PageInfo
                    .newBuilder()
                    .setTotal(history.size)
                    .setTruncated(end < history.size)
                    .setNextCursor(if (end < history.size) end.toString() else "")
                    .build()
            responseObserver.onNext(builder.build())
            responseObserver.onCompleted()
        } catch (exception: Exception) {
            responseObserver.onError(Status.INTERNAL.withDescription("unable to read proxy history").withCause(exception).asRuntimeException())
        }
    }

    override fun serverInfo(
        request: ServerInfoRequest,
        responseObserver: StreamObserver<ServerInfoResponse>,
    ) {
        responseObserver.onNext(
            ServerInfoResponse
                .newBuilder()
                .setExtension("Burp MCP")
                .setVersion(extensionVersion())
                .addAllCapabilities(listOf("proxy.read", "transport.echo"))
                .build(),
        )
        responseObserver.onCompleted()
    }

    private fun extensionVersion(): String =
        GrpcBurpService::class.java.`package`.implementationVersion ?: "development"
}
