package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import com.fasterxml.jackson.databind.JsonNode
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.databind.node.ArrayNode
import com.fasterxml.jackson.databind.node.ObjectNode

internal data class ProxyInterceptRuleConfig(
    val enabled: Boolean,
    val booleanOperator: String,
    val matchType: String,
    val matchRelationship: String,
    val matchCondition: String,
)

internal data class ProxyInterceptConfigPatch(
    val masterInterceptEnabled: Boolean? = null,
    val requestDoIntercept: Boolean? = null,
    val requestAutoContentLength: Boolean? = null,
    val requestFixMissingNewLines: Boolean? = null,
    val responseDoIntercept: Boolean? = null,
    val responseAutoContentLength: Boolean? = null,
    val websocketClientToServer: Boolean? = null,
    val websocketServerToClient: Boolean? = null,
    val websocketInScopeOnly: Boolean? = null,
    val requestRules: List<ProxyInterceptRuleConfig> = emptyList(),
    val responseRules: List<ProxyInterceptRuleConfig> = emptyList(),
    val replaceRequestRules: Boolean = false,
    val replaceResponseRules: Boolean = false,
    val responseUnhideHiddenFields: Boolean? = null,
    val responseEnableDisabledFields: Boolean? = null,
    val responseRemoveInputLengthLimits: Boolean? = null,
    val responseRemoveJavaScriptValidation: Boolean? = null,
    val responseRemoveAllJavaScript: Boolean? = null,
)

internal data class ProxyInterceptConfig(
    val masterInterceptEnabled: Boolean,
    val requestDoIntercept: Boolean,
    val requestAutoContentLength: Boolean,
    val requestFixMissingNewLines: Boolean,
    val responseDoIntercept: Boolean,
    val responseAutoContentLength: Boolean,
    val websocketClientToServer: Boolean,
    val websocketServerToClient: Boolean,
    val websocketInScopeOnly: Boolean,
    val requestRules: List<ProxyInterceptRuleConfig>,
    val responseRules: List<ProxyInterceptRuleConfig>,
    val responseUnhideHiddenFields: Boolean,
    val responseEnableDisabledFields: Boolean,
    val responseRemoveInputLengthLimits: Boolean,
    val responseRemoveJavaScriptValidation: Boolean,
    val responseRemoveAllJavaScript: Boolean,
)

internal class ProxyInterceptConfigFacade(
    private val api: MontoyaApi,
    private val mapper: ObjectMapper = ObjectMapper(),
) {
    @Synchronized
    fun read(): ProxyInterceptConfig = fromJson(exportedProxyConfig())

    @Synchronized
    fun update(patch: ProxyInterceptConfigPatch): ProxyInterceptConfig {
        validate(patch)
        patch.masterInterceptEnabled?.let { enabled ->
            if (enabled) api.proxy().enableIntercept() else api.proxy().disableIntercept()
        }
        val root = exportedProxyConfig()
        val proxy = root.withObject("proxy")
        val request = proxy.withObject("intercept_client_requests")
        val response = proxy.withObject("intercept_server_responses")
        val websocket = proxy.withObject("intercept_web_sockets_messages")
        val modification = proxy.withObject("response_modification")

        patch.requestDoIntercept?.let { request.put("do_intercept", it) }
        patch.requestAutoContentLength?.let {
            request.put("automatically_update_content_length_header_when_the_request_is_edited", it)
        }
        patch.requestFixMissingNewLines?.let {
            request.put("automatically_fix_missing_or_superfluous_new_lines_at_end_of_request", it)
        }
        patch.responseDoIntercept?.let { response.put("do_intercept", it) }
        patch.responseAutoContentLength?.let {
            response.put("automatically_update_content_length_header_when_the_response_is_edited", it)
        }
        patch.websocketClientToServer?.let { websocket.put("client_to_server_messages", it) }
        patch.websocketServerToClient?.let { websocket.put("server_to_client_messages", it) }
        patch.websocketInScopeOnly?.let { websocket.put("intercept_in_scope_only", it) }
        patch.responseUnhideHiddenFields?.let { modification.put("unhide_hidden_form_fields", it) }
        patch.responseEnableDisabledFields?.let { modification.put("enable_disabled_form_fields", it) }
        patch.responseRemoveInputLengthLimits?.let { modification.put("remove_input_field_length_limits", it) }
        patch.responseRemoveJavaScriptValidation?.let { modification.put("remove_javascript_form_validation", it) }
        patch.responseRemoveAllJavaScript?.let { modification.put("remove_all_javascript", it) }
        if (patch.replaceRequestRules) request.set<ArrayNode>("rules", rulesToJson(patch.requestRules))
        if (patch.replaceResponseRules) response.set<ArrayNode>("rules", rulesToJson(patch.responseRules))

        api.burpSuite().importProjectOptionsFromJson(mapper.writeValueAsString(root))
        return read()
    }

    private fun exportedProxyConfig(): ObjectNode {
        val json = api.burpSuite().exportProjectOptionsAsJson(PROXY_CONFIG_PATH)
        val exported = mapper.readTree(json) as? ObjectNode ?: error("Burp returned invalid Proxy configuration")
        if (exported.has("proxy")) return exported
        val wrapped = mapper.createObjectNode()
        wrapped.set<ObjectNode>("proxy", exported)
        return wrapped
    }

    private fun fromJson(root: ObjectNode): ProxyInterceptConfig {
        val proxy = root.path("proxy")
        val request = proxy.path("intercept_client_requests")
        val response = proxy.path("intercept_server_responses")
        val websocket = proxy.path("intercept_web_sockets_messages")
        val modification = proxy.path("response_modification")
        return ProxyInterceptConfig(
            masterInterceptEnabled = api.proxy().isInterceptEnabled,
            requestDoIntercept = request.path("do_intercept").asBoolean(false),
            requestAutoContentLength = request.path("automatically_update_content_length_header_when_the_request_is_edited").asBoolean(false),
            requestFixMissingNewLines = request.path("automatically_fix_missing_or_superfluous_new_lines_at_end_of_request").asBoolean(false),
            responseDoIntercept = response.path("do_intercept").asBoolean(false),
            responseAutoContentLength = response.path("automatically_update_content_length_header_when_the_response_is_edited").asBoolean(false),
            websocketClientToServer = websocket.path("client_to_server_messages").asBoolean(false),
            websocketServerToClient = websocket.path("server_to_client_messages").asBoolean(false),
            websocketInScopeOnly = websocket.path("intercept_in_scope_only").asBoolean(false),
            requestRules = rulesFromJson(request.path("rules")),
            responseRules = rulesFromJson(response.path("rules")),
            responseUnhideHiddenFields = modification.path("unhide_hidden_form_fields").asBoolean(false),
            responseEnableDisabledFields = modification.path("enable_disabled_form_fields").asBoolean(false),
            responseRemoveInputLengthLimits = modification.path("remove_input_field_length_limits").asBoolean(false),
            responseRemoveJavaScriptValidation = modification.path("remove_javascript_form_validation").asBoolean(false),
            responseRemoveAllJavaScript = modification.path("remove_all_javascript").asBoolean(false),
        )
    }

    private fun rulesFromJson(node: JsonNode): List<ProxyInterceptRuleConfig> =
        node.map { rule ->
            ProxyInterceptRuleConfig(
                enabled = rule.path("enabled").asBoolean(false),
                booleanOperator = rule.path("boolean_operator").asText("and"),
                matchType = rule.path("match_type").asText(),
                matchRelationship = rule.path("match_relationship").asText(),
                matchCondition = rule.path("match_condition").asText(),
            )
        }

    private fun rulesToJson(rules: List<ProxyInterceptRuleConfig>): ArrayNode =
        mapper.createArrayNode().apply {
            rules.forEach { rule ->
                add(
                    mapper.createObjectNode().apply {
                        put("enabled", rule.enabled)
                        put("boolean_operator", rule.booleanOperator)
                        put("match_type", rule.matchType)
                        put("match_relationship", rule.matchRelationship)
                        if (rule.matchCondition.isNotEmpty()) put("match_condition", rule.matchCondition)
                    },
                )
            }
        }

    private fun validate(patch: ProxyInterceptConfigPatch) {
        if (patch.requestRules.isNotEmpty()) require(patch.replaceRequestRules) { "request_rules requires replace_request_rules=true" }
        if (patch.responseRules.isNotEmpty()) require(patch.replaceResponseRules) { "response_rules requires replace_response_rules=true" }
        (patch.requestRules + patch.responseRules).forEach { rule ->
            require(rule.booleanOperator in BOOLEAN_OPERATORS) { "boolean_operator must be and or or" }
            require(rule.matchType.isNotBlank()) { "match_type must not be blank" }
            require(rule.matchRelationship.isNotBlank()) { "match_relationship must not be blank" }
        }
    }

    private companion object {
        const val PROXY_CONFIG_PATH = "proxy"
        val BOOLEAN_OPERATORS = setOf("and", "or")
    }
}
