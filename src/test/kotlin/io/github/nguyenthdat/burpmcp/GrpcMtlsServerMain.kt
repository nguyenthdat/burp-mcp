package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import io.github.nguyenthdat.burpmcp.rpc.BurpRpcServer
import io.mockk.every
import io.mockk.mockk
import java.nio.file.Path
import kotlin.io.path.Path

object GrpcMtlsServerMain {
    @JvmStatic
    fun main(args: Array<String>) {
        require(args.size == 2) { "usage: GrpcMtlsServerMain <port> <tls-directory>" }
        val port = args[0].toInt()
        val directory: Path = Path(args[1]).toAbsolutePath().normalize()
        val settings = GrpcSettings(
            bindAddress = "127.0.0.1",
            port = port,
            securityMode = GrpcSecurityMode.REMOTE_MTLS,
            serverNames = listOf("localhost", "127.0.0.1"),
            tlsDirectory = directory,
        )
        val api = mockk<MontoyaApi>(relaxed = true)
        every { api.extension().filename() } returns "burp-mcp.jar"
        val bundle = TlsBundleManager().generate(directory, settings.serverNames)
        BurpRpcServer(api, settings, bundle).use { server ->
            server.start()
            println("READY")
            System.out.flush()
            while (System.`in`.read() != -1) Unit
        }
    }
}
