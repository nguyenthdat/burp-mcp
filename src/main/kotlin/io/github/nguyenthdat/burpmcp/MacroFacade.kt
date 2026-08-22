package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.http.HttpService
import burp.api.montoya.http.message.requests.HttpRequest
import com.fasterxml.jackson.databind.JsonNode
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.databind.node.ArrayNode
import com.fasterxml.jackson.databind.node.ObjectNode

internal data class MacroParameterDefinition(
    val name: String,
    val originalValue: String,
    val parameterHandling: String,
    val presetValue: String,
    val type: String,
)

internal data class MacroItemDefinition(
    val request: String,
    val method: String,
    val url: String,
    val response: String,
    val statusCode: Int,
    val cookiesReceived: String,
    val requestParameters: List<MacroParameterDefinition>,
    val customParameters: List<String>,
)

internal data class MacroDefinition(
    val description: String,
    val serialNumber: Long,
    val items: List<MacroItemDefinition>,
)

internal class MacroFacade(
    private val api: MontoyaApi,
    private val mapper: ObjectMapper = ObjectMapper(),
) {
    @Synchronized
    fun create(definition: MacroDefinition): MacroDefinition {
        validate(definition)
        val serialNumber = definition.serialNumber.takeIf { it > 0 } ?: positiveSerialNumber()
        val stored = definition.copy(serialNumber = serialNumber)
        val root = exportedConfig()
        val macros = macros(root)
        val retained = macros.filterNot { it.path("description").asText() == stored.description }
        macros.removeAll()
        retained.forEach(macros::add)
        macros.add(toJson(stored))
        import(root)
        return stored
    }

    @Synchronized
    fun list(): List<MacroDefinition> = macros(exportedConfig()).map(::fromJson)

    fun run(description: String): List<HttpExchange> {
        val definition = synchronized(this) {
            list().firstOrNull { it.description == description }
        } ?: error("macro not found")
        return definition.items.map { item -> send(item) }
    }

    @Synchronized
    fun remove(description: String): Boolean {
        val root = exportedConfig()
        val macros = macros(root)
        val retained = macros.filterNot { it.path("description").asText() == description }
        if (retained.size == macros.size()) return false
        macros.removeAll()
        retained.forEach(macros::add)
        import(root)
        return true
    }

    private fun send(item: MacroItemDefinition): HttpExchange {
        val service = java.net.URI(item.url).let { uri ->
            HttpService.httpService(uri.host, explicitPort(uri), uri.scheme.equals("https", ignoreCase = true))
        }
        val request = HttpRequest.httpRequest(service, item.request)
        val exchange = api.http().sendRequest(request)
        return HttpExchange(
            request = exchange.request().toString(),
            response = exchange.response()?.toString(),
            status = exchange.response()?.statusCode()?.toInt(),
        )
    }

    private fun exportedConfig(): ObjectNode {
        val json = api.burpSuite().exportProjectOptionsAsJson(MACRO_CONFIG_PATH)
        val exported = mapper.readTree(json) as? ObjectNode ?: error("Burp returned invalid project configuration")
        return if (exported.has("project_options")) exported else mapper.createObjectNode().set<ObjectNode>("project_options", exported)
    }

    private fun macros(root: ObjectNode): ArrayNode {
        val project = root.withObject("project_options")
        val sessions = project.withObject("sessions")
        val section = sessions.withObject("macros")
        return section.withArray("macros")
    }

    private fun import(root: ObjectNode) {
        api.burpSuite().importProjectOptionsFromJson(mapper.writeValueAsString(root))
    }

    private fun validate(definition: MacroDefinition) {
        require(definition.description.isNotBlank()) { "macro description must not be blank" }
        require(definition.items.isNotEmpty()) { "macro must contain at least one item" }
        require(definition.items.size <= MAX_MACRO_ITEMS) { "macro must contain at most $MAX_MACRO_ITEMS items" }
        definition.items.forEach { item ->
            require(item.request.isNotBlank()) { "macro item request must not be blank" }
            require(item.url.isNotBlank()) { "macro item URL must not be blank" }
            require(item.request.length <= MAX_ITEM_CHARS) { "macro item request must be at most $MAX_ITEM_CHARS characters" }
            require(item.response.length <= MAX_ITEM_CHARS) { "macro item response must be at most $MAX_ITEM_CHARS characters" }
        }
    }

    private fun toJson(definition: MacroDefinition): ObjectNode = mapper.createObjectNode().apply {
        put("description", definition.description)
        put("serial_number", definition.serialNumber)
        set<ArrayNode>("items", mapper.createArrayNode().apply {
            definition.items.forEach { item ->
                add(mapper.createObjectNode().apply {
                    put("request", item.request)
                    put("method", item.method)
                    put("url", item.url)
                    put("response", item.response)
                    put("status_code", item.statusCode)
                    if (item.cookiesReceived.isNotEmpty()) put("cookies_received", item.cookiesReceived)
                    set<ArrayNode>("request_parameters", mapper.createArrayNode().apply {
                        item.requestParameters.forEach { parameter ->
                            add(mapper.createObjectNode().apply {
                                put("name", parameter.name)
                                put("original_value", parameter.originalValue)
                                put("parameter_handling", parameter.parameterHandling)
                                put("preset_value", parameter.presetValue)
                                put("type", parameter.type)
                            })
                        }
                    })
                    set<ArrayNode>("custom_parameters", mapper.createArrayNode().apply {
                        item.customParameters.forEach(::add)
                    })
                })
            }
        })
    }

    private fun fromJson(node: JsonNode): MacroDefinition = MacroDefinition(
        description = node.path("description").asText(),
        serialNumber = node.path("serial_number").asLong(),
        items = node.path("items").map { item ->
            MacroItemDefinition(
                request = item.path("request").asText(),
                method = item.path("method").asText(),
                url = item.path("url").asText(),
                response = item.path("response").asText(),
                statusCode = item.path("status_code").asInt(),
                cookiesReceived = item.path("cookies_received").asText(),
                requestParameters = item.path("request_parameters").map { parameter ->
                    MacroParameterDefinition(
                        name = parameter.path("name").asText(),
                        originalValue = parameter.path("original_value").asText(),
                        parameterHandling = parameter.path("parameter_handling").asText(),
                        presetValue = parameter.path("preset_value").asText(),
                        type = parameter.path("type").asText(),
                    )
                },
                customParameters = item.path("custom_parameters").map(JsonNode::asText),
            )
        },
    )

    private fun explicitPort(uri: java.net.URI): Int =
        uri.port.takeIf { it >= 0 } ?: if (uri.scheme.equals("https", ignoreCase = true)) 443 else 80

    private fun positiveSerialNumber(): Long = System.nanoTime().and(Long.MAX_VALUE)

    private companion object {
        const val MACRO_CONFIG_PATH = "project_options.sessions.macros"
        const val MAX_MACRO_ITEMS = 50
        const val MAX_ITEM_CHARS = 2 * 1024 * 1024
    }
}
