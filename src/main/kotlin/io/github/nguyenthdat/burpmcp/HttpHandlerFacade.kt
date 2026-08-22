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

internal class HttpHandlerFacade(
    private val api: MontoyaApi,
) {
    private var registration: Registration? = null

    @Synchronized
    fun register(rule: HttpHandlerRule) {
        require(
            (rule.headerName != null && rule.headerValue != null) ||
                (rule.match != null && rule.replace != null),
        ) { "header_name/header_value or match/replace is required" }
        registration?.deregister()
        registration =
            api.http().registerHttpHandler(
                object : HttpHandler {
                    override fun handleHttpRequestToBeSent(requestToBeSent: HttpRequestToBeSent): RequestToBeSentAction {
                        var request: burp.api.montoya.http.message.requests.HttpRequest = requestToBeSent
                        if (rule.headerName != null && rule.headerValue != null) {
                            request = request.withUpdatedHeader(rule.headerName, rule.headerValue)
                        }
                        if (rule.match != null && rule.replace != null) {
                            request = request.withBody(request.bodyToString().replace(rule.match, rule.replace))
                        }
                        return RequestToBeSentAction.continueWith(request)
                    }

                    override fun handleHttpResponseReceived(responseReceived: HttpResponseReceived): ResponseReceivedAction =
                        ResponseReceivedAction.continueWith(responseReceived)
                },
            )
    }

    @Synchronized
    fun clear() {
        registration?.deregister()
        registration = null
    }
}
