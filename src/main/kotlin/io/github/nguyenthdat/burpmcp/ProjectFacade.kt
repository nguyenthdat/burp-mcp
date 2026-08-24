package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import java.security.SecureRandom

internal data class ProjectIdentity(
    val projectId: String,
    val projectName: String,
    val graphId: String,
    val temporary: Boolean,
)

internal class ProjectFacade(
    private val api: MontoyaApi,
) {
    fun identity(): ProjectIdentity {
        val project = api.project()
        val projectId = runCatching { project.id() }.getOrNull().orEmpty()
        val projectName = runCatching { project.name() }.getOrNull().orEmpty()
        val extensionData = api.persistence().extensionData()
        val existingGraphId = runCatching { extensionData.getString(GRAPH_ID_KEY) }.getOrNull()
        val graphId = existingGraphId?.takeIf(String::isNotBlank) ?: randomGraphId().also {
            extensionData.setString(GRAPH_ID_KEY, it)
        }
        return ProjectIdentity(
            projectId = projectId,
            projectName = projectName,
            graphId = graphId,
            temporary = projectId.isBlank(),
        )
    }

    private fun randomGraphId(): String {
        val bytes = ByteArray(16)
        random.nextBytes(bytes)
        return bytes.joinToString(separator = "") { byte -> "%02x".format(byte.toInt() and 0xff) }
    }

    private companion object {
        const val GRAPH_ID_KEY = "burp_mcp.sitegraph_db_id"
        val random = SecureRandom()
    }
}
