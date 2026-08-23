package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import io.github.nguyenthdat.burpmcp.rpc.BurpRpcServer
import io.github.nguyenthdat.burpmcp.rpc.GRPC_MAX_CONCURRENT_CALLS_PER_CONNECTION
import io.github.nguyenthdat.burpmcp.rpc.GRPC_MAX_MESSAGE_BYTES
import io.github.nguyenthdat.burpmcp.rpc.GRPC_MAX_PAGE_SIZE
import io.github.nguyenthdat.burpmcp.rpc.GRPC_MAX_RESPONSE_BYTES
import io.github.nguyenthdat.burpmcp.rpc.GRPC_MAX_RPC_TIMEOUT_SECONDS
import io.github.nguyenthdat.burpmcp.grpc.v1.EchoBytesRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.PingRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ServerInfoRequest
import io.grpc.ManagedChannel
import io.grpc.netty.shaded.io.grpc.netty.NettyChannelBuilder
import io.github.nguyenthdat.burpmcp.grpc.v1.BurpServiceGrpc
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.Test
import java.net.ServerSocket
import java.time.Clock
import java.time.Instant
import java.time.ZoneOffset
import java.util.concurrent.TimeUnit
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue
import kotlin.test.assertFailsWith

class BurpRpcServerTest {
    private var server: BurpRpcServer? = null
    private var channel: ManagedChannel? = null

    @AfterEach
    fun closeResources() {
        channel?.shutdownNow()?.awaitTermination(5, TimeUnit.SECONDS)
        server?.close()
    }

    @Test
    fun `binds to IPv4 loopback and serves typed calls`() {
        val port = availablePort()
        server = BurpRpcServer(fake(MontoyaApi::class.java), port, Clock.fixed(Instant.ofEpochMilli(42), ZoneOffset.UTC))
        server?.start()

        channel = NettyChannelBuilder.forAddress("127.0.0.1", port).usePlaintext().build()
        val client = BurpServiceGrpc.newBlockingStub(channel)
        val ping = client.withDeadlineAfter(2, TimeUnit.SECONDS).ping(PingRequest.newBuilder().setClient("test").build())
        val info = client.withDeadlineAfter(2, TimeUnit.SECONDS).serverInfo(ServerInfoRequest.getDefaultInstance())

        assertEquals("burp-mcp-kotlin", ping.server)
        assertEquals(42, ping.unixMillis)
        assertEquals(listOf("proxy.read", "sitemap.read", "scanner.read", "cookies.read", "transport.echo", "lifecycle.restart"), info.capabilitiesList)
        assertEquals(GRPC_MAX_MESSAGE_BYTES, info.maxMessageBytes)
        assertEquals(GRPC_MAX_PAGE_SIZE, info.maxPageSize)
        assertEquals(GRPC_MAX_CONCURRENT_CALLS_PER_CONNECTION, info.maxConcurrentCallsPerConnection)
        assertEquals(GRPC_MAX_RPC_TIMEOUT_SECONDS.toInt(), info.maxRpcTimeoutSeconds)
        assertEquals(GRPC_MAX_RESPONSE_BYTES, info.maxResponseBytes)
        assertTrue(server?.isRunning() == true)
    }

    @Test
    fun `echoes zero one and ten MiB payloads byte exactly`() {
        val port = availablePort()
        server = BurpRpcServer(fake(MontoyaApi::class.java), port)
        server?.start()
        channel = NettyChannelBuilder
            .forAddress("127.0.0.1", port)
            .usePlaintext()
            .maxInboundMessageSize(GRPC_MAX_MESSAGE_BYTES)
            .build()
        val client = BurpServiceGrpc.newBlockingStub(channel).withDeadlineAfter(10, TimeUnit.SECONDS)

        for (payload in listOf(ByteArray(0), byteArrayOf(0xA5.toByte()), ByteArray(10 * 1024 * 1024) { (it % 251).toByte() })) {
            val response = client.echoBytes(EchoBytesRequest.newBuilder().setPayload(com.google.protobuf.ByteString.copyFrom(payload)).build())
            assertContentEquals(payload, response.payload.toByteArray())
        }
    }

    @Test
    fun `rejects calls without the mandatory client deadline`() {
        val port = availablePort()
        server = BurpRpcServer(fake(MontoyaApi::class.java), port)
        server?.start()
        channel = NettyChannelBuilder.forAddress("127.0.0.1", port).usePlaintext().build()
        val client = BurpServiceGrpc.newBlockingStub(channel)

        val exception =
            assertFailsWith<io.grpc.StatusRuntimeException> {
                client.ping(PingRequest.newBuilder().setClient("no-client-deadline").build())
            }

        assertEquals(io.grpc.Status.Code.INVALID_ARGUMENT, exception.status.code)
    }

    @Test
    fun `rejects a client deadline above the server maximum`() {
        val port = availablePort()
        server = BurpRpcServer(fake(MontoyaApi::class.java), port)
        server?.start()
        channel = NettyChannelBuilder.forAddress("127.0.0.1", port).usePlaintext().build()
        val client =
            BurpServiceGrpc
                .newBlockingStub(channel)
                .withDeadlineAfter(GRPC_MAX_RPC_TIMEOUT_SECONDS + 30, TimeUnit.SECONDS)

        val exception =
            assertFailsWith<io.grpc.StatusRuntimeException> {
                client.ping(PingRequest.newBuilder().setClient("long-deadline").build())
            }

        assertEquals(io.grpc.Status.Code.INVALID_ARGUMENT, exception.status.code)
    }

    @Test
    fun `deadline cancels a delayed unary call`() {
        val port = availablePort()
        server = BurpRpcServer(fake(MontoyaApi::class.java), port)
        server?.start()
        channel = NettyChannelBuilder.forAddress("127.0.0.1", port).usePlaintext().build()
        val client = BurpServiceGrpc.newBlockingStub(channel).withDeadlineAfter(25, TimeUnit.MILLISECONDS)

        assertFailsWith<io.grpc.StatusRuntimeException> {
            client.echoBytes(
                EchoBytesRequest
                    .newBuilder()
                    .setDelayMillis(500)
                    .build(),
            )
        }.also { exception -> assertEquals(io.grpc.Status.Code.DEADLINE_EXCEEDED, exception.status.code) }
    }

    @Test
    fun `rejects requests above the configured message limit`() {
        val port = availablePort()
        server = BurpRpcServer(fake(MontoyaApi::class.java), port)
        server?.start()
        channel =
            NettyChannelBuilder
                .forAddress("127.0.0.1", port)
                .usePlaintext()
                .maxInboundMessageSize(GRPC_MAX_MESSAGE_BYTES + 1024)
                .build()
        val client = BurpServiceGrpc.newBlockingStub(channel).withDeadlineAfter(5, TimeUnit.SECONDS)

        val exception =
            assertFailsWith<io.grpc.StatusRuntimeException> {
                client.echoBytes(
                    EchoBytesRequest
                        .newBuilder()
                        .setPayload(com.google.protobuf.ByteString.copyFrom(ByteArray(GRPC_MAX_MESSAGE_BYTES + 1)))
                        .build(),
                )
            }
        assertEquals(io.grpc.Status.Code.RESOURCE_EXHAUSTED, exception.status.code)
    }
    @Test
    fun `validation failures are returned as invalid argument`() {
        val port = availablePort()
        server = BurpRpcServer(fake(MontoyaApi::class.java), port)
        server?.start()
        channel = NettyChannelBuilder.forAddress("127.0.0.1", port).usePlaintext().build()
        val client = BurpServiceGrpc.newBlockingStub(channel).withDeadlineAfter(5, TimeUnit.SECONDS)

        val malformedCursor =
            assertFailsWith<io.grpc.StatusRuntimeException> {
                client.scanIssues(
                    io.github.nguyenthdat.burpmcp.grpc.v1.ScanIssuesRequest.newBuilder()
                        .setPage(io.github.nguyenthdat.burpmcp.grpc.v1.PageRequest.newBuilder().setCursor("invalid"))
                        .build(),
                )
            }
        assertEquals(io.grpc.Status.Code.INVALID_ARGUMENT, malformedCursor.status.code)
        assertTrue(malformedCursor.status.description.orEmpty().contains("scanner cursor"))

        val invalidIssue =
            assertFailsWith<io.grpc.StatusRuntimeException> {
                client.addIssue(
                    io.github.nguyenthdat.burpmcp.grpc.v1.AddIssueRequest.newBuilder()
                        .setName("probe")
                        .setUrl("http://example.test/")
                        .setSeverity("invalid")
                        .setConfidence("certain")
                        .build(),
                )
            }
        assertEquals(io.grpc.Status.Code.INVALID_ARGUMENT, invalidIssue.status.code)
        assertTrue(invalidIssue.status.description.orEmpty().contains("severity"))
    }

    @Test
    fun `close releases listener and is idempotent`() {
        val port = availablePort()
        server = BurpRpcServer(fake(MontoyaApi::class.java), port)
        server?.start()
        server?.close()
        server?.close()

        assertFalse(server?.isRunning() == true)
        ServerSocket(port, 1, java.net.InetAddress.getByName("127.0.0.1")).use { rebound ->
            assertEquals(port, rebound.localPort)
        }
    }

    private fun availablePort(): Int =
        ServerSocket(0, 1, java.net.InetAddress.getByName("127.0.0.1")).use { it.localPort }

    @Suppress("UNCHECKED_CAST")
    private fun <T> fake(type: Class<T>): T =
        java.lang.reflect.Proxy.newProxyInstance(type.classLoader, arrayOf(type)) { _, method, _ ->
            when (method.name) {
                "toString" -> "fake-${type.simpleName}"
                "hashCode" -> 0
                "equals" -> false
                else -> defaultValue(method.returnType)
            }
        } as T

    private fun defaultValue(type: Class<*>): Any? =
        when {
            !type.isPrimitive -> if (type.isInterface) fake(type) else null
            type == Boolean::class.javaPrimitiveType -> false
            type == Char::class.javaPrimitiveType -> '\u0000'
            type == Byte::class.javaPrimitiveType -> 0.toByte()
            type == Short::class.javaPrimitiveType -> 0.toShort()
            type == Int::class.javaPrimitiveType -> 0
            type == Long::class.javaPrimitiveType -> 0L
            type == Float::class.javaPrimitiveType -> 0f
            type == Double::class.javaPrimitiveType -> 0.0
            else -> null
        }
}
