package io.github.nguyenthdat.burpmcp

import burp.api.montoya.persistence.Preferences
import java.net.InetAddress
import java.nio.file.Path
import kotlin.io.path.Path

internal const val DEFAULT_GRPC_PORT: Int = 9877
internal const val DEFAULT_GRPC_BIND_ADDRESS: String = "127.0.0.1"
internal const val DEFAULT_TLS_SERVER_NAMES: String = "localhost,127.0.0.1"

internal enum class GrpcSecurityMode {
    LOCAL_PLAINTEXT,
    REMOTE_MTLS,
}

internal data class GrpcSettings(
    val bindAddress: String = DEFAULT_GRPC_BIND_ADDRESS,
    val port: Int = DEFAULT_GRPC_PORT,
    val securityMode: GrpcSecurityMode = GrpcSecurityMode.LOCAL_PLAINTEXT,
    val serverNames: List<String> = listOf("localhost", DEFAULT_GRPC_BIND_ADDRESS),
    val tlsDirectory: Path = defaultTlsDirectory(),
) {
    fun validate() {
        require(port in 1..65535) { "gRPC port must be between 1 and 65535" }
        val address = InetAddress.getByName(bindAddress)
        if (securityMode == GrpcSecurityMode.LOCAL_PLAINTEXT) {
            require(address.isLoopbackAddress) { "Plaintext gRPC must bind to a loopback address" }
            require(address.hostAddress == DEFAULT_GRPC_BIND_ADDRESS) { "Plaintext gRPC currently supports IPv4 loopback only" }
        } else {
            require(serverNames.isNotEmpty()) { "Remote mTLS requires at least one certificate DNS name or IP address" }
            require(serverNames.none { it == "0.0.0.0" || it == "::" }) { "Wildcard bind addresses cannot be certificate identities" }
        }
        require(tlsDirectory.isAbsolute) { "TLS directory must be an absolute path" }
    }

    val endpointScheme: String
        get() = if (securityMode == GrpcSecurityMode.REMOTE_MTLS) "https" else "http"
}

internal class GrpcSettingsStore(private val preferences: Preferences) {
    fun load(): GrpcSettings {
        val mode = runCatching {
            GrpcSecurityMode.valueOf(preferences.getString(KEY_SECURITY_MODE) ?: GrpcSecurityMode.LOCAL_PLAINTEXT.name)
        }.getOrDefault(GrpcSecurityMode.LOCAL_PLAINTEXT)
        val settings = GrpcSettings(
            bindAddress = preferences.getString(KEY_BIND_ADDRESS)?.trim().orEmpty().ifBlank { DEFAULT_GRPC_BIND_ADDRESS },
            port = preferences.getInteger(KEY_PORT)?.takeIf { it in 1..65535 } ?: DEFAULT_GRPC_PORT,
            securityMode = mode,
            serverNames = parseServerNames(preferences.getString(KEY_SERVER_NAMES) ?: DEFAULT_TLS_SERVER_NAMES),
            tlsDirectory = Path(preferences.getString(KEY_TLS_DIRECTORY)?.trim().orEmpty().ifBlank { defaultTlsDirectory().toString() }).toAbsolutePath().normalize(),
        )
        return runCatching { settings.also(GrpcSettings::validate) }.getOrDefault(GrpcSettings())
    }

    fun save(settings: GrpcSettings) {
        settings.validate()
        preferences.setString(KEY_BIND_ADDRESS, settings.bindAddress)
        preferences.setInteger(KEY_PORT, settings.port)
        preferences.setString(KEY_SECURITY_MODE, settings.securityMode.name)
        preferences.setString(KEY_SERVER_NAMES, settings.serverNames.joinToString(","))
        preferences.setString(KEY_TLS_DIRECTORY, settings.tlsDirectory.toString())
    }

    companion object {
        private const val KEY_BIND_ADDRESS = "grpc.bindAddress"
        private const val KEY_PORT = "grpc.port"
        private const val KEY_SECURITY_MODE = "grpc.securityMode"
        private const val KEY_SERVER_NAMES = "grpc.serverNames"
        private const val KEY_TLS_DIRECTORY = "grpc.tlsDirectory"

        fun parseServerNames(raw: String): List<String> =
            raw.split(',')
                .map(String::trim)
                .filter(String::isNotEmpty)
                .distinct()
    }
}

internal fun defaultTlsDirectory(): Path =
    Path(System.getenv("XDG_CONFIG_HOME") ?: Path(System.getProperty("user.home"), ".config").toString(), "burp-mcp", "tls")
        .toAbsolutePath()
        .normalize()
