package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.http.HttpService
import burp.api.montoya.http.message.requests.HttpRequest

internal data class ExtensionInfo(
    val filename: String,
    val isBapp: Boolean,
    val commandLineArguments: List<String>,
)

internal class BurpCapabilityFacade(
    private val api: MontoyaApi,
) {
    fun sendToIntruder(
        request: ByteArray,
        host: String,
        port: Int,
        https: Boolean,
        tabName: String?,
    ) {
        require(host.isNotBlank()) { "host must not be blank" }
        require(port in 1..65535) { "port must be between 1 and 65535" }
        val service = HttpService.httpService(host, port, https)
        val message = HttpRequest.httpRequest(service, burp.api.montoya.core.ByteArray.byteArray(*request))
        if (tabName.isNullOrBlank()) {
            api.intruder().sendToIntruder(message)
        } else {
            api.intruder().sendToIntruder(message, tabName)
        }
    }

    fun extensionInfo(): ExtensionInfo =
        ExtensionInfo(
            filename = api.extension().filename(),
            isBapp = api.extension().isBapp,
            commandLineArguments = api.burpSuite().commandLineArguments(),
        )
}
