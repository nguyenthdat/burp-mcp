package io.github.nguyenthdat.burpmcp

import com.google.gson.JsonObject

internal typealias ToolHandler = (JsonObject) -> JsonObject

internal data class RegisteredTool(
    val name: String,
    val advertised: Boolean = true,
    val handler: ToolHandler,
)

internal class ToolRegistry(
    tools: List<RegisteredTool>,
) {
    private val toolsByName: Map<String, RegisteredTool>
    private val publicNames: List<String>

    init {
        val duplicateNames: Set<String> =
            tools
                .groupingBy(RegisteredTool::name)
                .eachCount()
                .filterValues { count -> count > 1 }
                .keys
        require(duplicateNames.isEmpty()) {
            "Duplicate tool names: ${duplicateNames.sorted().joinToString()}"
        }
        toolsByName = tools.associateBy(RegisteredTool::name)
        publicNames = tools.filter(RegisteredTool::advertised).map(RegisteredTool::name)
    }

    fun advertisedNames(): List<String> = publicNames

    fun invoke(
        name: String,
        params: JsonObject,
    ): JsonObject? = toolsByName[name]?.handler?.invoke(params)
}
