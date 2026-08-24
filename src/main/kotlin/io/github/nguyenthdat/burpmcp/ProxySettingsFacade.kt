package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import com.fasterxml.jackson.databind.JsonNode
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.databind.node.ArrayNode
import com.fasterxml.jackson.databind.node.ObjectNode

internal data class ProxyListenerConfig(
    val port: Int,
    val running: Boolean,
    val listenMode: String,
    val listenSpecificAddress: String,
    val certificateMode: String,
    val enableHttp2: Boolean,
    val supportInvisibleProxying: Boolean,
)

internal data class ScriptFilterConfig(
    val target: String,
    val mode: String,
    val script: String,
    val scriptId: String,
    val scriptName: String,
)

internal class ProxySettingsFacade(
    private val api: MontoyaApi,
    private val mapper: ObjectMapper = ObjectMapper(),
) {
    @Synchronized
    fun listeners(): List<ProxyListenerConfig> = listenersFromJson(exportedConfig())

    @Synchronized
    fun upsertListener(listener: ProxyListenerConfig): ProxyListenerConfig {
        validateListener(listener)
        val root = exportedConfig()
        val proxy = root.withObject("proxy")
        val listeners = proxy.withArray("request_listeners")
        val index = listeners.indexOfFirst { it.path("listener_port").asInt() == listener.port }
        val updated = listenerToJson(listener)
        if (index >= 0) listeners.set(index, updated) else listeners.add(updated)
        import(root)
        return listeners().first { it.port == listener.port }
    }

    @Synchronized
    fun deleteListener(port: Int): Boolean {
        require(port in 1..65535) { "port must be between 1 and 65535" }
        val root = exportedConfig()
        val listeners = root.withObject("proxy").withArray("request_listeners")
        val index = listeners.indexOfFirst { it.path("listener_port").asInt() == port }
        if (index < 0) return false
        listeners.remove(index)
        import(root)
        return true
    }

    @Synchronized
    fun scriptFilters(): List<ScriptFilterConfig> {
        val root = exportedConfig()
        return SCRIPT_FILTERS.keys.map { target -> scriptFilterFromJson(root, target) }
    }

    @Synchronized
    fun scriptFilter(target: String): ScriptFilterConfig = scriptFilterFromJson(exportedConfig(), normalizedTarget(target))

    @Synchronized
    fun upsertScriptFilter(value: ScriptFilterConfig): ScriptFilterConfig {
        val target = normalizedTarget(value.target)
        val mode = value.mode.trim().uppercase()
        require(mode in FILTER_MODES) { "mode must be settings or script" }
        require(value.script.toByteArray(Charsets.UTF_8).size <= MAX_SCRIPT_BYTES) {
            "script exceeds ${MAX_SCRIPT_BYTES / 1024} KiB"
        }
        if (mode == "SCRIPT") require(value.script.isNotBlank()) { "script must not be blank in script mode" }

        val root = exportedConfig()
        val mapping = SCRIPT_FILTERS.getValue(target)
        root.withObject("bambda").withObject(mapping.bambdaKey).apply {
            put("bambda", value.script.ifBlank { "return true;" })
            put("bambda_id", value.scriptId.trim())
            put("bambda_name", value.scriptName.trim())
        }
        root.withObject(mapping.owner).withObject(mapping.filterKey).put("filter_mode", mode.toBurpFilterMode())
        import(root)
        return scriptFilter(target)
    }

    @Synchronized
    fun deleteScriptFilter(target: String): ScriptFilterConfig {
        val normalized = normalizedTarget(target)
        val root = exportedConfig()
        val mapping = SCRIPT_FILTERS.getValue(normalized)
        root.withObject("bambda").withObject(mapping.bambdaKey).apply {
            put("bambda", "return true;")
            put("bambda_id", "")
            put("bambda_name", "")
        }
        root.withObject(mapping.owner).withObject(mapping.filterKey).put("filter_mode", "SETTINGS")
        import(root)
        return scriptFilter(normalized)
    }

    private fun exportedConfig(): ObjectNode {
        val json = api.burpSuite().exportProjectOptionsAsJson("proxy", "bambda", "logger", "target.filter")
        return mapper.readTree(json) as? ObjectNode ?: error("Burp returned invalid Proxy configuration")
    }

    private fun import(root: ObjectNode) {
        api.burpSuite().importProjectOptionsFromJson(mapper.writeValueAsString(root))
    }

    private fun listenersFromJson(root: ObjectNode): List<ProxyListenerConfig> =
        root.path("proxy").path("request_listeners").takeIf(JsonNode::isArray)?.map { item ->
            ProxyListenerConfig(
                port = item.path("listener_port").asInt(),
                running = item.path("running").asBoolean(false),
                listenMode = item.path("listen_mode").asText("loopback_only"),
                listenSpecificAddress = item.path("listen_specific_address").asText(""),
                certificateMode = item.path("certificate_mode").asText("per_host"),
                enableHttp2 = item.path("enable_http2").asBoolean(true),
                supportInvisibleProxying = item.path("support_invisible_proxying").asBoolean(false),
            )
        }.orEmpty().sortedBy(ProxyListenerConfig::port)

    private fun listenerToJson(listener: ProxyListenerConfig): ObjectNode = mapper.createObjectNode().apply {
        put("listener_port", listener.port)
        put("running", listener.running)
        put("listen_mode", listener.listenMode)
        if (listener.listenMode == "specific_address") put("listen_specific_address", listener.listenSpecificAddress)
        put("certificate_mode", listener.certificateMode)
        put("enable_http2", listener.enableHttp2)
        put("support_invisible_proxying", listener.supportInvisibleProxying)
        put("use_custom_tls_protocols", false)
        set<ArrayNode>("custom_tls_protocols", mapper.createArrayNode())
    }

    private fun validateListener(listener: ProxyListenerConfig) {
        require(listener.port in 1..65535) { "port must be between 1 and 65535" }
        require(listener.listenMode in LISTEN_MODES) { "listen_mode must be loopback_only, all_interfaces, or specific_address" }
        require(listener.certificateMode in CERTIFICATE_MODES) {
            "certificate_mode must be per_host, use_custom_certificate, or invisible"
        }
        if (listener.listenMode == "specific_address") {
            require(listener.listenSpecificAddress.isNotBlank()) { "listen_specific_address is required for specific_address mode" }
        }
    }

    private fun scriptFilterFromJson(root: ObjectNode, target: String): ScriptFilterConfig {
        val mapping = SCRIPT_FILTERS.getValue(target)
        val bambda = root.path("bambda").path(mapping.bambdaKey)
        val mode = root.path(mapping.owner).path(mapping.filterKey).path("filter_mode").asText("SETTINGS")
        return ScriptFilterConfig(
            target = target,
            mode = if (mode.equals("BAMBDA", true)) "script" else "settings",
            script = bambda.path("bambda").asText("return true;"),
            scriptId = bambda.path("bambda_id").asText(""),
            scriptName = bambda.path("bambda_name").asText(""),
        )
    }

    private fun normalizedTarget(value: String): String {
        val target = value.trim().lowercase()
        require(target in SCRIPT_FILTERS) { "target must be proxy_http_history, proxy_websocket_history, sitemap, logger_capture, or logger_display" }
        return target
    }

    private data class ScriptFilterMapping(val owner: String, val filterKey: String, val bambdaKey: String)

    private companion object {
        const val MAX_SCRIPT_BYTES = 256 * 1024
        val LISTEN_MODES = setOf("loopback_only", "all_interfaces", "specific_address")
        val CERTIFICATE_MODES = setOf("per_host", "use_custom_certificate", "invisible")
        val FILTER_MODES = setOf("SETTINGS", "SCRIPT")
        val SCRIPT_FILTERS = linkedMapOf(
            "proxy_http_history" to ScriptFilterMapping("proxy", "http_history_display_filter", "http_history_display_filter"),
            "proxy_websocket_history" to ScriptFilterMapping("proxy", "web_sockets_history_display_filter", "web_sockets_history_display_filter"),
            "sitemap" to ScriptFilterMapping("target", "filter", "sitemap_display_filter"),
            "logger_capture" to ScriptFilterMapping("logger", "capture_filter", "logger_capture_filter"),
            "logger_display" to ScriptFilterMapping("logger", "display_filter", "logger_display_filter"),
        )
    }
}

private fun String.toBurpFilterMode(): String = if (this == "SCRIPT") "BAMBDA" else "SETTINGS"

private fun ArrayNode.indexOfFirst(predicate: (JsonNode) -> Boolean): Int {
    for (index in 0 until size()) if (predicate(get(index))) return index
    return -1
}
