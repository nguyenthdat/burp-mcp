package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.core.ByteArray as MontoyaByteArray
import burp.api.montoya.core.Registration
import burp.api.montoya.http.message.requests.HttpRequest
import burp.api.montoya.http.message.responses.HttpResponse
import burp.api.montoya.proxy.http.InterceptedRequest
import burp.api.montoya.proxy.http.InterceptedResponse
import burp.api.montoya.proxy.http.ProxyRequestHandler
import burp.api.montoya.proxy.http.ProxyRequestReceivedAction
import burp.api.montoya.proxy.http.ProxyRequestToBeSentAction
import burp.api.montoya.proxy.http.ProxyResponseHandler
import burp.api.montoya.proxy.http.ProxyResponseReceivedAction
import burp.api.montoya.proxy.http.ProxyResponseToBeSentAction

internal data class ProxyRule(
    val id: String,
    val urlContains: String,
    val phase: String,
    val action: String,
    val match: String,
    val replacement: String,
    val headerName: String,
    val headerValue: String,
    val enabled: Boolean,
)

internal class ProxyRuleFacade(
    private val api: MontoyaApi,
) : AutoCloseable {
    private var requestRegistration: Registration? = null
    private var responseRegistration: Registration? = null
    private val rules = linkedMapOf<String, ProxyRule>()

    init {
        requestRegistration = api.proxy().registerRequestHandler(requestHandler())
        responseRegistration = api.proxy().registerResponseHandler(responseHandler())
    }

    @Synchronized
    fun register(rule: ProxyRule) {
        validateProxyRule(rule)
        rules[rule.id] = rule
    }

    @Synchronized
    fun list(): List<ProxyRule> = rules.values.toList()

    @Synchronized
    fun clear(id: String? = null) {
        if (id.isNullOrEmpty()) rules.clear() else rules.remove(id)
    }

    override fun close() {
        requestRegistration?.deregister()
        responseRegistration?.deregister()
        requestRegistration = null
        responseRegistration = null
        synchronized(this) { rules.clear() }
    }

    private fun requestHandler(): ProxyRequestHandler =
        object : ProxyRequestHandler {
            override fun handleRequestReceived(request: InterceptedRequest): ProxyRequestReceivedAction {
                val rule = matchingRule("request", request.url()) ?: return ProxyRequestReceivedAction.continueWith(request)
                return when (rule.action) {
                    "intercept" -> ProxyRequestReceivedAction.intercept(request)
                    "forward", "edit" -> ProxyRequestReceivedAction.doNotIntercept(request)
                    "drop" -> ProxyRequestReceivedAction.drop()
                    else -> ProxyRequestReceivedAction.continueWith(request)
                }
            }

            override fun handleRequestToBeSent(request: InterceptedRequest): ProxyRequestToBeSentAction {
                val rule = matchingRule("request", request.url()) ?: return ProxyRequestToBeSentAction.continueWith(request)
                return ProxyRequestToBeSentAction.continueWith(editRequest(request, rule))
            }
        }

    private fun responseHandler(): ProxyResponseHandler =
        object : ProxyResponseHandler {
            override fun handleResponseReceived(response: InterceptedResponse): ProxyResponseReceivedAction {
                val requestUrl = response.initiatingRequest()?.url() ?: return ProxyResponseReceivedAction.continueWith(response)
                val rule = matchingRule("response", requestUrl) ?: return ProxyResponseReceivedAction.continueWith(response)
                val updated = editResponse(response, rule)
                return when (rule.action) {
                    "intercept" -> ProxyResponseReceivedAction.intercept(updated)
                    "forward", "edit" -> ProxyResponseReceivedAction.doNotIntercept(updated)
                    "drop" -> ProxyResponseReceivedAction.drop()
                    else -> ProxyResponseReceivedAction.continueWith(updated)
                }
            }

            override fun handleResponseToBeSent(response: InterceptedResponse): ProxyResponseToBeSentAction =
                ProxyResponseToBeSentAction.continueWith(response)
        }

    @Synchronized
    private fun matchingRule(phase: String, url: String): ProxyRule? =
        rules.values.firstOrNull { rule -> rule.enabled && rule.phase == phase && url.contains(rule.urlContains) }
}

internal fun validateProxyRule(rule: ProxyRule) {
    require(rule.id.isNotBlank()) { "id must not be blank" }
    require(rule.urlContains.isNotEmpty()) { "url_contains must not be empty" }
    require(rule.phase in setOf("request", "response")) { "phase must be request or response" }
    require(rule.action in setOf("forward", "intercept", "drop", "edit")) { "action must be forward, intercept, drop, or edit" }
    if (rule.action == "edit") {
        require(
            (rule.match.isNotEmpty() && rule.replacement.isNotEmpty()) ||
                (rule.headerName.isNotBlank() && rule.headerValue.isNotEmpty()),
        ) { "edit requires match/replacement or header_name/header_value" }
    }
}

internal fun editRequest(request: HttpRequest, rule: ProxyRule): HttpRequest {
    if (rule.action != "edit") return request
    var updated = request
    if (rule.headerName.isNotBlank()) {
        updated = if (updated.hasHeader(rule.headerName)) {
            updated.withUpdatedHeader(rule.headerName, rule.headerValue)
        } else {
            updated.withAddedHeader(rule.headerName, rule.headerValue)
        }
    }
    if (rule.match.isNotEmpty()) {
        val path = updated.path()
        if (path.contains(rule.match)) updated = updated.withPath(path.replace(rule.match, rule.replacement))
        updated.headers().forEach { header ->
            if (header.value().contains(rule.match)) {
                updated = updated.withUpdatedHeader(header.name(), header.value().replace(rule.match, rule.replacement))
            }
        }
        val body = replaceBytes(updated.body().getBytes(), rule.match.toByteArray(), rule.replacement.toByteArray())
        if (body != null) updated = updated.withBody(MontoyaByteArray.byteArray(*body))
    }
    return updated
}

internal fun editResponse(response: HttpResponse, rule: ProxyRule): HttpResponse {
    if (rule.action != "edit") return response
    var updated = response
    if (rule.headerName.isNotBlank()) {
        updated = if (updated.hasHeader(rule.headerName)) {
            updated.withUpdatedHeader(rule.headerName, rule.headerValue)
        } else {
            updated.withAddedHeader(rule.headerName, rule.headerValue)
        }
    }
    if (rule.match.isNotEmpty()) {
        updated.headers().forEach { header ->
            if (header.value().contains(rule.match)) {
                updated = updated.withUpdatedHeader(header.name(), header.value().replace(rule.match, rule.replacement))
            }
        }
        val body = replaceBytes(updated.body().getBytes(), rule.match.toByteArray(), rule.replacement.toByteArray())
        if (body != null) updated = updated.withBody(MontoyaByteArray.byteArray(*body))
    }
    return updated
}

internal fun replaceBytes(
    input: kotlin.ByteArray,
    match: kotlin.ByteArray,
    replacement: kotlin.ByteArray,
): kotlin.ByteArray? {
    if (match.isEmpty()) return null
    var index = input.indexOfSlice(match)
    if (index < 0) return null

    val output = java.io.ByteArrayOutputStream(input.size)
    var offset = 0
    while (index >= 0) {
        output.write(input, offset, index - offset)
        output.write(replacement)
        offset = index + match.size
        index = input.indexOfSlice(match, offset)
    }
    output.write(input, offset, input.size - offset)
    return output.toByteArray()
}

private fun kotlin.ByteArray.indexOfSlice(needle: kotlin.ByteArray, start: Int = 0): Int {
    if (needle.isEmpty()) return start.coerceAtMost(size)
    if (start < 0 || start > size || needle.size > size - start) return -1
    for (index in start..(size - needle.size)) {
        var matches = true
        for (needleIndex in needle.indices) {
            if (this[index + needleIndex] != needle[needleIndex]) {
                matches = false
                break
            }
        }
        if (matches) return index
    }
    return -1
}
