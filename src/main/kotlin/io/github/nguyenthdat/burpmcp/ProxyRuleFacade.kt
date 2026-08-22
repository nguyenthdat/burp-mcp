package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.core.Registration
import burp.api.montoya.proxy.http.InterceptedRequest
import burp.api.montoya.proxy.http.ProxyRequestHandler
import burp.api.montoya.proxy.http.ProxyRequestReceivedAction
import burp.api.montoya.proxy.http.ProxyRequestToBeSentAction

internal class ProxyRuleFacade(
    private val api: MontoyaApi,
) {
    private var registration: Registration? = null

    @Synchronized
    fun register(urlContains: String, intercept: Boolean) {
        require(urlContains.isNotEmpty()) { "url_contains must not be empty" }
        registration?.deregister()
        registration =
            api.proxy().registerRequestHandler(
                object : ProxyRequestHandler {
                    override fun handleRequestReceived(request: InterceptedRequest): ProxyRequestReceivedAction =
                        if (request.url().contains(urlContains)) {
                            if (intercept) ProxyRequestReceivedAction.intercept(request) else ProxyRequestReceivedAction.doNotIntercept(request)
                        } else {
                            ProxyRequestReceivedAction.continueWith(request)
                        }

                    override fun handleRequestToBeSent(request: InterceptedRequest): ProxyRequestToBeSentAction =
                        ProxyRequestToBeSentAction.continueWith(request)
                },
            )
    }

    @Synchronized
    fun clear() {
        registration?.deregister()
        registration = null
    }
}
