package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import io.github.nguyenthdat.burpmcp.rpc.BurpRpcServer
import java.lang.reflect.Proxy
import java.nio.file.Files
import java.nio.file.Path
import kotlin.io.path.deleteIfExists
import kotlin.io.path.exists
import kotlin.io.path.writeText

/** Test-only process used by the Rust/Kotlin interoperability suite. */
object GrpcInteropServerMain {
    @JvmStatic
    fun main(args: Array<String>) {
        require(args.size in 1..2) { "usage: GrpcInteropServerMain <port> [control-directory]" }
        val port = args[0].toInt()
        val controlDirectory =
            if (args.size == 2) {
                Path.of(args[1]).toAbsolutePath()
            } else {
                Files.createTempDirectory("burp-mcp-grpc-control")
            }
        Files.createDirectories(controlDirectory)
        val ready = controlDirectory.resolve("ready")
        val stop = controlDirectory.resolve("stop")
        val stopped = controlDirectory.resolve("stopped")
        val start = controlDirectory.resolve("start")
        val exit = controlDirectory.resolve("exit")
        listOf(ready, stop, stopped, start, exit).forEach(Path::deleteIfExists)

        val api = fakeMontoyaApi()
        var server: BurpRpcServer? = startServer(api, port)
        ready.writeText("127.0.0.1:$port")
        try {
            while (!exit.exists()) {
                if (stop.exists()) {
                    stop.deleteIfExists()
                    ready.deleteIfExists()
                    server?.close()
                    server = null
                    stopped.writeText("stopped")
                }
                if (start.exists() && server == null) {
                    start.deleteIfExists()
                    stopped.deleteIfExists()
                    server = startServer(api, port)
                    ready.writeText("127.0.0.1:$port")
                }
                Thread.sleep(20)
            }
        } finally {
            server?.close()
            ready.deleteIfExists()
            exit.deleteIfExists()
        }
    }

    private fun startServer(
        api: MontoyaApi,
        port: Int,
    ): BurpRpcServer = BurpRpcServer(api, port).also(BurpRpcServer::start)

    @Suppress("UNCHECKED_CAST")
    private fun fakeMontoyaApi(): MontoyaApi =
        Proxy.newProxyInstance(MontoyaApi::class.java.classLoader, arrayOf(MontoyaApi::class.java)) { _, method, _ ->
            when (method.name) {
                "toString" -> "fake-MontoyaApi"
                "hashCode" -> 0
                "equals" -> false
                else -> fakeReturn(method.returnType)
            }
        } as MontoyaApi

    @Suppress("UNCHECKED_CAST")
    private fun fakeReturn(type: Class<*>): Any? =
        when {
            type == List::class.java -> emptyList<Any>()
            type.isInterface ->
                Proxy.newProxyInstance(type.classLoader, arrayOf(type)) { _, method, _ -> fakeReturn(method.returnType) }
            !type.isPrimitive -> null
            type == Boolean::class.javaPrimitiveType -> false
            type == Int::class.javaPrimitiveType -> 0
            type == Long::class.javaPrimitiveType -> 0L
            type == Short::class.javaPrimitiveType -> 0.toShort()
            type == Byte::class.javaPrimitiveType -> 0.toByte()
            type == Float::class.javaPrimitiveType -> 0f
            type == Double::class.javaPrimitiveType -> 0.0
            type == Char::class.javaPrimitiveType -> '\u0000'
            else -> null
        }
}
