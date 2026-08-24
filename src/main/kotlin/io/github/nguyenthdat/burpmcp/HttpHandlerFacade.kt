package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.core.Registration
import burp.api.montoya.http.handler.HttpHandler
import burp.api.montoya.http.handler.HttpRequestToBeSent
import burp.api.montoya.http.handler.HttpResponseReceived
import burp.api.montoya.http.handler.RequestToBeSentAction
import burp.api.montoya.http.handler.ResponseReceivedAction

internal data class HttpHandlerRule(
    val headerName: String?,
    val headerValue: String?,
    val match: String?,
    val replace: String?,
)

internal fun mutateRequest(
    request: burp.api.montoya.http.message.requests.HttpRequest,
    rule: HttpHandlerRule,
): burp.api.montoya.http.message.requests.HttpRequest {
    var updated = request
    if (rule.headerName != null && rule.headerValue != null) {
        updated = if (updated.hasHeader(rule.headerName)) {
            updated.withUpdatedHeader(rule.headerName, rule.headerValue)
        } else {
            updated.withAddedHeader(rule.headerName, rule.headerValue)
        }
    }
    if (rule.match != null && rule.replace != null) {
        val path = updated.path()
        if (path.contains(rule.match)) {
            updated = updated.withPath(path.replace(rule.match, rule.replace))
        } else {
            val body = updated.bodyToString()
            if (body.contains(rule.match)) updated = updated.withBody(body.replace(rule.match, rule.replace))
        }
    }
    return updated
}

internal class HttpHandlerFacade(
    private val api: MontoyaApi,
) {
    private var registrations: List<Registration> = emptyList()

    @Synchronized
    fun register(rule: HttpHandlerRule) {
        require(
            (rule.headerName != null && rule.headerValue != null) ||
                (rule.match != null && rule.replace != null),
        ) { "header_name/header_value or match/replace is required" }
        registrations.forEach(Registration::deregister)
        val handler =
            object : HttpHandler {
                override fun handleHttpRequestToBeSent(requestToBeSent: HttpRequestToBeSent): RequestToBeSentAction =
                    RequestToBeSentAction.continueWith(mutateRequest(requestToBeSent, rule))

                override fun handleHttpResponseReceived(responseReceived: HttpResponseReceived): ResponseReceivedAction =
                    ResponseReceivedAction.continueWith(responseReceived)
            }
        val proxyHandler =
            object : burp.api.montoya.proxy.http.ProxyRequestHandler {
                override fun handleRequestReceived(
                    interceptedRequest: burp.api.montoya.proxy.http.InterceptedRequest,
                ): burp.api.montoya.proxy.http.ProxyRequestReceivedAction =
                    burp.api.montoya.proxy.http.ProxyRequestReceivedAction.continueWith(mutateRequest(interceptedRequest, rule))

                override fun handleRequestToBeSent(
                    interceptedRequest: burp.api.montoya.proxy.http.InterceptedRequest,
                ): burp.api.montoya.proxy.http.ProxyRequestToBeSentAction =
                    burp.api.montoya.proxy.http.ProxyRequestToBeSentAction.continueWith(mutateRequest(interceptedRequest, rule))
            }
        registrations = listOf(api.http().registerHttpHandler(handler), api.proxy().registerRequestHandler(proxyHandler))
    }

    @Synchronized
    fun clear() {
        registrations.forEach(Registration::deregister)
        registrations = emptyList()
    }
}
