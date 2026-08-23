package io.github.nguyenthdat.burpmcp

import burp.api.montoya.core.ToolType
import burp.api.montoya.MontoyaApi
import burp.api.montoya.core.Registration
import burp.api.montoya.http.sessions.ActionResult
import burp.api.montoya.http.sessions.SessionHandlingAction
import burp.api.montoya.http.sessions.SessionHandlingActionData

internal data class SessionRule(
    val description: String,
    val actionType: String,
    val find: String,
    val replacement: String,
    val headerName: String,
    val parameterName: String,
    val macroDescription: String,
    val urlContains: String,
    val tools: Set<String>,
    val enabled: Boolean,
)

internal class SessionRuleFacade(
    private val api: MontoyaApi,
    private val macroRunner: (String) -> Unit,
) {
    private data class ActiveRule(
        val rule: SessionRule,
        val sessionRegistration: Registration,
        val proxyRegistration: Registration,
    )

    private var active: ActiveRule? = null

    @Synchronized
    fun create(rule: SessionRule) {
        validate(rule)
        remove()
        val sessionRegistration =
            api.http().registerSessionHandlingAction(
                object : SessionHandlingAction {
                    override fun name(): String = rule.description

                    override fun performAction(actionData: SessionHandlingActionData): ActionResult {
                        if (!rule.enabled) return ActionResult.actionResult(actionData.request(), actionData.annotations())
                        if (rule.actionType == "run_macro") macroRunner(rule.macroDescription)
                        return ActionResult.actionResult(applyRule(actionData.request(), rule), actionData.annotations())
                    }
                },
            )
        val proxyRegistration =
            api.proxy().registerRequestHandler(
                object : burp.api.montoya.proxy.http.ProxyRequestHandler {
                    override fun handleRequestReceived(
                        interceptedRequest: burp.api.montoya.proxy.http.InterceptedRequest,
                    ): burp.api.montoya.proxy.http.ProxyRequestReceivedAction {
                        val request =
                            if (matches(interceptedRequest, rule, "proxy")) {
                                if (rule.actionType == "run_macro") macroRunner(rule.macroDescription)
                                applyRule(interceptedRequest, rule)
                            } else {
                                interceptedRequest
                            }
                        return burp.api.montoya.proxy.http.ProxyRequestReceivedAction.continueWith(request)
                    }

                    override fun handleRequestToBeSent(
                        interceptedRequest: burp.api.montoya.proxy.http.InterceptedRequest,
                    ): burp.api.montoya.proxy.http.ProxyRequestToBeSentAction =
                        burp.api.montoya.proxy.http.ProxyRequestToBeSentAction.continueWith(interceptedRequest)
                },
            )
        active = ActiveRule(rule, sessionRegistration, proxyRegistration)
    }

    @Synchronized
    fun list(): List<SessionRule> = active?.let { listOf(it.rule) } ?: emptyList()

    @Synchronized
    fun remove() {
        active?.sessionRegistration?.deregister()
        active?.proxyRegistration?.deregister()
        active = null
    }

    private fun validate(rule: SessionRule) {
        require(rule.description.isNotBlank()) { "description must not be blank" }
        require(rule.actionType in SUPPORTED_ACTIONS) { "unsupported session action: ${rule.actionType}" }
        if (rule.actionType == "replace_text") require(rule.find.isNotEmpty()) { "find must not be empty" }
        if (rule.actionType == "set_header") require(rule.headerName.isNotBlank()) { "header_name must not be blank" }
        if (rule.actionType == "set_parameter") require(rule.parameterName.isNotBlank()) { "parameter_name must not be blank" }
        if (rule.actionType == "run_macro") require(rule.macroDescription.isNotBlank()) { "macro_description must not be blank" }
    }

    private companion object {
        val SUPPORTED_ACTIONS = setOf("replace_text", "set_header", "set_parameter", "run_macro")
    }
}

private fun matches(
    request: burp.api.montoya.http.message.requests.HttpRequest,
    rule: SessionRule,
    tool: String,
): Boolean =
    rule.enabled &&
        (rule.urlContains.isEmpty() || request.url().contains(rule.urlContains)) &&
        (rule.tools.isEmpty() || tool in rule.tools)

internal fun applyRule(
    request: burp.api.montoya.http.message.requests.HttpRequest,
    rule: SessionRule,
): burp.api.montoya.http.message.requests.HttpRequest =
    when (rule.actionType) {
        "replace_text" -> replaceInRequest(request, rule)
        "set_header" ->
            if (request.hasHeader(rule.headerName)) {
                request.withUpdatedHeader(rule.headerName, rule.replacement)
            } else {
                request.withAddedHeader(rule.headerName, rule.replacement)
            }
        "set_parameter" ->
            request.parameters()
                .firstOrNull { it.name() == rule.parameterName }
                ?.let { parameter ->
                    val updated =
                        when (parameter.type()) {
                            burp.api.montoya.http.message.params.HttpParameterType.URL ->
                                burp.api.montoya.http.message.params.HttpParameter.urlParameter(rule.parameterName, rule.replacement)
                            burp.api.montoya.http.message.params.HttpParameterType.BODY ->
                                burp.api.montoya.http.message.params.HttpParameter.bodyParameter(rule.parameterName, rule.replacement)
                            burp.api.montoya.http.message.params.HttpParameterType.COOKIE ->
                                burp.api.montoya.http.message.params.HttpParameter.cookieParameter(rule.parameterName, rule.replacement)
                            else -> null
                        }
                    updated?.let(request::withParameter) ?: request
                } ?: request
        "run_macro" -> request
        else -> error("unsupported session action: ${rule.actionType}")
    }

internal fun replaceInRequest(
    request: burp.api.montoya.http.message.requests.HttpRequest,
    rule: SessionRule,
): burp.api.montoya.http.message.requests.HttpRequest {
    var updated = request
    val path = updated.path()
    if (path.contains(rule.find)) updated = updated.withPath(path.replace(rule.find, rule.replacement))
    updated.headers().forEach { header ->
        if (header.value().contains(rule.find)) {
            updated = updated.withUpdatedHeader(header.name(), header.value().replace(rule.find, rule.replacement))
        }
    }
    val body = updated.bodyToString()
    if (body.contains(rule.find)) updated = updated.withBody(body.replace(rule.find, rule.replacement))
    return updated
}
