package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import com.fasterxml.jackson.databind.JsonNode
import com.fasterxml.jackson.databind.ObjectMapper

internal data class ConfigInspection(
    val config: String,
    val paths: List<String>,
    val sizeBytes: Int,
)

internal class ConfigFacade(
    private val api: MontoyaApi,
    private val mapper: ObjectMapper = ObjectMapper(),
) {
    fun export(paths: List<String>): String =
        api.burpSuite().exportProjectOptionsAsJson(*validatedPaths(paths).toTypedArray())

    fun inspect(paths: List<String>): ConfigInspection {
        val config = export(paths)
        val root = mapper.readTree(config) ?: error("Burp returned invalid project configuration")
        val leafPaths = mutableListOf<String>()
        collectLeafPaths(root, "", leafPaths)
        return ConfigInspection(config, leafPaths.sorted(), config.toByteArray(Charsets.UTF_8).size)
    }

    fun import(config: String) {
        require(config.isNotBlank()) { "config must not be blank" }
        val size = config.toByteArray(Charsets.UTF_8).size
        require(size <= MAX_IMPORT_BYTES) { "config exceeds ${MAX_IMPORT_BYTES / (1024 * 1024)} MiB" }
        val root = mapper.readTree(config)
        require(root?.isObject == true) { "config must be a JSON object" }
        api.burpSuite().importProjectOptionsFromJson(config)
    }

    private fun validatedPaths(paths: List<String>): List<String> {
        require(paths.size <= MAX_PATHS) { "paths must contain at most $MAX_PATHS entries" }
        return paths.map { path ->
            val normalized = path.trim()
            require(normalized.isNotEmpty()) { "config path must not be blank" }
            require(PATH.matches(normalized)) { "invalid config path: $normalized" }
            normalized
        }.distinct()
    }

    private fun collectLeafPaths(
        node: JsonNode,
        prefix: String,
        result: MutableList<String>,
    ) {
        when {
            node.isObject -> node.fields().forEach { (name, child) -> collectLeafPaths(child, append(prefix, name), result) }
            node.isArray -> node.forEachIndexed { index, child -> collectLeafPaths(child, append(prefix, index.toString()), result) }
            prefix.isNotEmpty() -> result += prefix
        }
    }

    private fun append(prefix: String, segment: String): String = if (prefix.isEmpty()) segment else "$prefix.$segment"

    private companion object {
        const val MAX_PATHS = 64
        const val MAX_IMPORT_BYTES = 4 * 1024 * 1024
        val PATH = Regex("[A-Za-z0-9_-]+(?:\\.[A-Za-z0-9_-]+)*")
    }
}
