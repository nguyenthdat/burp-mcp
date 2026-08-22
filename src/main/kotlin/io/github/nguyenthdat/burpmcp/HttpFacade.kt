package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.http.message.requests.HttpRequest

internal data class HttpRequestSpec(
    val method: String = "GET",
    val url: String,
    val body: String = "",
    val headers: Map<String, String> = emptyMap(),
)

internal data class HttpExchange(
    val request: String,
    val response: String?,
    val status: Int?,
)

internal class HttpFacade(
    private val api: MontoyaApi,
) {
    fun send(spec: HttpRequestSpec): HttpExchange {
        var request = HttpRequest.httpRequestFromUrl(spec.url).withMethod(spec.method.uppercase())
        spec.headers.forEach { (name, value) -> request = request.withUpdatedHeader(name, value) }
        if (spec.body.isNotEmpty()) request = request.withBody(spec.body)
        val exchange = api.http().sendRequest(request)
        return HttpExchange(
            request = exchange.request().toString(),
            response = exchange.response()?.toString(),
            status = exchange.response()?.statusCode()?.toInt(),
        )
    }

    fun sendParallel(specs: List<HttpRequestSpec>): List<HttpExchange> {
        require(specs.size <= 32) { "at most 32 requests may be sent in one batch" }
        val requests = specs.map { spec ->
            var request = HttpRequest.httpRequestFromUrl(spec.url).withMethod(spec.method.uppercase())
            spec.headers.forEach { (name, value) -> request = request.withUpdatedHeader(name, value) }
            if (spec.body.isNotEmpty()) request = request.withBody(spec.body)
            request
        }
        return api.http().sendRequests(requests).map { exchange ->
            HttpExchange(
                request = exchange.request().toString(),
                response = exchange.response()?.toString(),
                status = exchange.response()?.statusCode()?.toInt(),
            )
        }
    }

    fun sendToRepeater(request: String, host: String, port: Int, https: Boolean, tabName: String?) {
        val service = burp.api.montoya.http.HttpService.httpService(host, port, https)
        val message = HttpRequest.httpRequest(service, request)
        api.repeater().sendToRepeater(message, tabName)
    }
}
