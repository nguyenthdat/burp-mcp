package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.core.Registration
import burp.api.montoya.http.sessions.ActionResult
import burp.api.montoya.http.sessions.SessionHandlingAction
import burp.api.montoya.http.sessions.SessionHandlingActionData

internal data class SessionRule(
    val find: String,
    val replacement: String,
)

internal class SessionRuleFacade(
    private val api: MontoyaApi,
) {
    private var active: Triple<SessionRule, Registration, Registration>? = null

    @Synchronized
    fun create(rule: SessionRule) {
        require(rule.find.isNotEmpty()) { "find must not be empty" }
        active?.let { (_, sessionRegistration, proxyRegistration) ->
            sessionRegistration.deregister()
            proxyRegistration.deregister()
        }
        val sessionRegistration =
            api.http().registerSessionHandlingAction(
                object : SessionHandlingAction {
                    override fun name(): String = "Burp MCP replace session token"

                    override fun performAction(actionData: SessionHandlingActionData): ActionResult =
                        ActionResult.actionResult(replaceInRequest(actionData.request(), rule), actionData.annotations())
                },
            )
        val proxyRegistration =
            api.proxy().registerRequestHandler(
                object : burp.api.montoya.proxy.http.ProxyRequestHandler {
                    override fun handleRequestReceived(
                        interceptedRequest: burp.api.montoya.proxy.http.InterceptedRequest,
                    ): burp.api.montoya.proxy.http.ProxyRequestReceivedAction =
                        burp.api.montoya.proxy.http.ProxyRequestReceivedAction.continueWith(replaceInRequest(interceptedRequest, rule))

                    override fun handleRequestToBeSent(
                        interceptedRequest: burp.api.montoya.proxy.http.InterceptedRequest,
                    ): burp.api.montoya.proxy.http.ProxyRequestToBeSentAction =
                        burp.api.montoya.proxy.http.ProxyRequestToBeSentAction.continueWith(replaceInRequest(interceptedRequest, rule))
                },
            )
        active = Triple(rule, sessionRegistration, proxyRegistration)
    }

    @Synchronized
    fun list(): List<SessionRule> = active?.let { listOf(it.first) } ?: emptyList()

    @Synchronized
    fun remove() {
        active?.let { (_, sessionRegistration, proxyRegistration) ->
            sessionRegistration.deregister()
            proxyRegistration.deregister()
        }
        active = null
    }
}

internal fun replaceInRequest(
    request: burp.api.montoya.http.message.requests.HttpRequest,
    rule: SessionRule,
): burp.api.montoya.http.message.requests.HttpRequest {
    val serialized = request.toString()
    if (!serialized.contains(rule.find)) return request
    return burp.api.montoya.http.message.requests.HttpRequest.httpRequest(
        request.httpService(),
        serialized.replace(rule.find, rule.replacement),
    )
}
