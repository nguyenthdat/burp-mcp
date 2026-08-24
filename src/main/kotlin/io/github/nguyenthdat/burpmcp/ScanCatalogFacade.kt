package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.http.execution.ResourcePool
import com.fasterxml.jackson.core.type.TypeReference
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.module.kotlin.registerKotlinModule
import java.time.Duration
import java.util.UUID

internal data class ScanConfigurationDefinition(
    val id: String,
    val name: String,
    val scanType: String,
    val auditType: String,
    val includeOutOfScope: Boolean,
    val timeoutSeconds: Long,
    val stableSeconds: Long,
    val resourcePoolId: String,
    val source: String = "extension",
)

internal data class ScanResourcePoolDefinition(
    val id: String,
    val name: String,
    val kind: String,
    val existingPoolName: String,
    val concurrentRequestLimit: Int,
    val throttleMillis: Long,
    val maxRetries: Int,
    val source: String = "extension",
)

internal class ScanCatalogFacade(
    api: MontoyaApi,
    private val mapper: ObjectMapper = ObjectMapper().registerKotlinModule(),
) {
    private val persisted = api.persistence().extensionData()

    @Synchronized
    fun configurations(): List<ScanConfigurationDefinition> = builtInConfigurations + readConfigurations().sortedBy(ScanConfigurationDefinition::name)

    @Synchronized
    fun configuration(id: String): ScanConfigurationDefinition =
        configurations().firstOrNull { it.id == id } ?: error("scan configuration not found")

    @Synchronized
    fun createConfiguration(value: ScanConfigurationDefinition): ScanConfigurationDefinition {
        val normalized = validateConfiguration(value.copy(id = value.id.ifBlank(::newId)), creating = true)
        val items = readConfigurations().toMutableList()
        require(items.none { it.id == normalized.id }) { "scan configuration already exists" }
        require(items.none { it.name.equals(normalized.name, ignoreCase = true) }) { "scan configuration name already exists" }
        items += normalized
        write(CONFIGURATIONS_KEY, items)
        return normalized
    }

    @Synchronized
    fun updateConfiguration(value: ScanConfigurationDefinition): ScanConfigurationDefinition {
        require(value.id.isNotBlank()) { "scan configuration id must not be blank" }
        val normalized = validateConfiguration(value, creating = false)
        val items = readConfigurations().toMutableList()
        val index = items.indexOfFirst { it.id == normalized.id }
        require(index >= 0) { "scan configuration not found or immutable" }
        require(items.none { it.id != normalized.id && it.name.equals(normalized.name, ignoreCase = true) }) { "scan configuration name already exists" }
        items[index] = normalized
        write(CONFIGURATIONS_KEY, items)
        return normalized
    }

    @Synchronized
    fun deleteConfiguration(id: String): Boolean {
        val items = readConfigurations().toMutableList()
        val removed = items.removeIf { it.id == id }
        if (removed) write(CONFIGURATIONS_KEY, items)
        return removed
    }

    @Synchronized
    fun pools(): List<ScanResourcePoolDefinition> = builtInPools + readPools().sortedBy(ScanResourcePoolDefinition::name)

    @Synchronized
    fun pool(id: String): ScanResourcePoolDefinition = pools().firstOrNull { it.id == id } ?: error("scan resource pool not found")

    @Synchronized
    fun createPool(value: ScanResourcePoolDefinition): ScanResourcePoolDefinition {
        val normalized = validatePool(value.copy(id = value.id.ifBlank(::newId)))
        val items = readPools().toMutableList()
        require(items.none { it.id == normalized.id }) { "scan resource pool already exists" }
        require(items.none { it.name.equals(normalized.name, ignoreCase = true) }) { "scan resource pool name already exists" }
        items += normalized
        write(POOLS_KEY, items)
        return normalized
    }

    @Synchronized
    fun updatePool(value: ScanResourcePoolDefinition): ScanResourcePoolDefinition {
        require(value.id.isNotBlank()) { "scan resource pool id must not be blank" }
        val normalized = validatePool(value)
        val items = readPools().toMutableList()
        val index = items.indexOfFirst { it.id == normalized.id }
        require(index >= 0) { "scan resource pool not found or immutable" }
        require(items.none { it.id != normalized.id && it.name.equals(normalized.name, ignoreCase = true) }) { "scan resource pool name already exists" }
        items[index] = normalized
        write(POOLS_KEY, items)
        return normalized
    }

    @Synchronized
    fun deletePool(id: String): Boolean {
        require(readConfigurations().none { it.resourcePoolId == id }) { "scan resource pool is referenced by a scan configuration" }
        val items = readPools().toMutableList()
        val removed = items.removeIf { it.id == id }
        if (removed) write(POOLS_KEY, items)
        return removed
    }

    fun runtimePool(id: String): ResourcePool {
        val definition = pool(id.ifBlank { DEFAULT_POOL_ID })
        return when (definition.kind) {
            "default" -> ResourcePool.defaultResourcePool()
            "existing" -> ResourcePool.existingResourcePool(definition.existingPoolName)
            "private" -> ResourcePool.resourcePool()
                .withConcurrentRequestLimit(definition.concurrentRequestLimit)
                .withThrottle(Duration.ofMillis(definition.throttleMillis))
                .withMaxRetries(definition.maxRetries)
            else -> error("unsupported scan resource pool kind: ${definition.kind}")
        }
    }

    private fun validateConfiguration(value: ScanConfigurationDefinition, creating: Boolean): ScanConfigurationDefinition {
        val name = value.name.trim()
        val scanType = value.scanType.trim().lowercase()
        val auditType = value.auditType.trim().lowercase().ifEmpty { "passive" }
        require(name.isNotEmpty()) { "scan configuration name must not be blank" }
        require(scanType in SCAN_TYPES) { "scan_type must be crawl, audit, or crawl_and_audit" }
        require(auditType in AUDIT_TYPES) { "audit_type must be passive or active" }
        require(value.timeoutSeconds in 1..MAX_TIMEOUT_SECONDS) { "timeout_seconds must be between 1 and $MAX_TIMEOUT_SECONDS" }
        require(value.stableSeconds in 0..MAX_STABLE_SECONDS) { "stable_seconds must be between 0 and $MAX_STABLE_SECONDS" }
        require(value.stableSeconds < value.timeoutSeconds) { "stable_seconds must be less than timeout_seconds" }
        val resourcePoolId = value.resourcePoolId.ifBlank { DEFAULT_POOL_ID }
        require(pools().any { it.id == resourcePoolId }) { "scan resource pool not found" }
        require(creating || value.source != "built_in") { "built-in scan configurations are immutable" }
        return value.copy(name = name, scanType = scanType, auditType = auditType, resourcePoolId = resourcePoolId, source = "extension")
    }

    private fun validatePool(value: ScanResourcePoolDefinition): ScanResourcePoolDefinition {
        val name = value.name.trim()
        val kind = value.kind.trim().lowercase()
        val existingPoolName = value.existingPoolName.trim()
        require(name.isNotEmpty()) { "scan resource pool name must not be blank" }
        require(kind in POOL_KINDS) { "resource pool kind must be private or existing" }
        require(kind != "existing" || existingPoolName.isNotEmpty()) { "existing_pool_name is required for existing resource pools" }
        require(value.concurrentRequestLimit in 1..999) { "concurrent_request_limit must be between 1 and 999" }
        require(value.throttleMillis in 0..MAX_THROTTLE_MILLIS) { "throttle_millis must be between 0 and $MAX_THROTTLE_MILLIS" }
        require(value.maxRetries in 0..MAX_RETRIES) { "max_retries must be between 0 and $MAX_RETRIES" }
        return value.copy(name = name, kind = kind, existingPoolName = existingPoolName, source = "extension")
    }

    private fun readConfigurations(): List<ScanConfigurationDefinition> = read(CONFIGURATIONS_KEY, object : TypeReference<List<ScanConfigurationDefinition>>() {})
    private fun readPools(): List<ScanResourcePoolDefinition> = read(POOLS_KEY, object : TypeReference<List<ScanResourcePoolDefinition>>() {})

    private fun <T> read(key: String, type: TypeReference<List<T>>): List<T> {
        val json = runCatching { persisted.getString(key) }.getOrNull()?.takeIf(String::isNotBlank) ?: return emptyList()
        return mapper.readValue(json, type)
    }

    private fun write(key: String, value: Any) = persisted.setString(key, mapper.writeValueAsString(value))
    private fun newId(): String = UUID.randomUUID().toString()

    private companion object {
        const val CONFIGURATIONS_KEY = "burp_mcp.scan_configurations.v1"
        const val POOLS_KEY = "burp_mcp.scan_resource_pools.v1"
        const val DEFAULT_POOL_ID = "built-in-default"
        const val MAX_TIMEOUT_SECONDS = 86_400L
        const val MAX_STABLE_SECONDS = 3_600L
        const val MAX_THROTTLE_MILLIS = 3_600_000L
        const val MAX_RETRIES = 100
        val SCAN_TYPES = setOf("crawl", "audit", "crawl_and_audit")
        val AUDIT_TYPES = setOf("passive", "active")
        val POOL_KINDS = setOf("private", "existing")
        val builtInPools = listOf(ScanResourcePoolDefinition(DEFAULT_POOL_ID, "Burp default", "default", "", 10, 0, 0, "built_in"))
        val builtInConfigurations = listOf(
            ScanConfigurationDefinition("built-in-lightweight", "Lightweight", "crawl_and_audit", "passive", false, 900, 2, DEFAULT_POOL_ID, "built_in"),
            ScanConfigurationDefinition("built-in-fast", "Fast", "crawl_and_audit", "active", false, 3_600, 3, DEFAULT_POOL_ID, "built_in"),
            ScanConfigurationDefinition("built-in-balanced", "Balanced", "crawl_and_audit", "active", false, 14_400, 5, DEFAULT_POOL_ID, "built_in"),
            ScanConfigurationDefinition("built-in-deep", "Deep", "crawl_and_audit", "active", false, 86_400, 10, DEFAULT_POOL_ID, "built_in"),
            ScanConfigurationDefinition("built-in-passive", "Passive audit snapshot", "audit", "passive", false, 30, 0, DEFAULT_POOL_ID, "built_in"),
        )
    }
}
