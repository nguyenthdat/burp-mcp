package io.github.nguyenthdat.burpmcp

import java.nio.charset.StandardCharsets
import java.security.MessageDigest

internal const val MAX_PAYLOAD_LISTS = 100
internal const val MAX_PAYLOAD_LIST_ENTRIES = 10_000
internal const val MAX_PAYLOAD_LIST_BYTES = 16 * 1024 * 1024
internal const val MAX_PAYLOAD_VALUE_BYTES = 64 * 1024

internal data class PayloadListDefinition(
    val id: String,
    val displayName: String,
    val payloads: List<String>,
) {
    val sizeBytes: Long = payloads.sumOf { it.toByteArray(StandardCharsets.UTF_8).size.toLong() }
    val fingerprint: String = payloadListFingerprint(payloads)
}

internal data class PayloadListPage(
    val list: PayloadListDefinition,
    val payloads: List<String>,
    val total: Int,
    val nextOffset: Int?,
)

internal class PayloadListFacade {
    private val lists = linkedMapOf<String, PayloadListDefinition>()

    @Synchronized
    fun create(id: String, displayName: String, payloads: List<String>): PayloadListDefinition {
        require(lists.size < MAX_PAYLOAD_LISTS) { "at most $MAX_PAYLOAD_LISTS payload lists may be retained" }
        require(!lists.containsKey(id)) { "payload list id already exists" }
        val definition = validatedPayloadList(id, displayName, payloads)
        lists[id] = definition
        return definition
    }

    @Synchronized
    fun import(id: String, displayName: String, content: String, format: String, keepEmpty: Boolean): PayloadListDefinition {
        require(content.toByteArray(StandardCharsets.UTF_8).size <= MAX_PAYLOAD_LIST_BYTES) {
            "payload list content exceeds $MAX_PAYLOAD_LIST_BYTES UTF-8 bytes"
        }
        val normalized = content.replace("\r\n", "\n").replace('\r', '\n')
        val payloads = when (format.trim().lowercase().ifBlank { "lines" }) {
            "lines", "text" -> normalized.split('\n').let { lines ->
                if (keepEmpty) lines else lines.filter(String::isNotEmpty)
            }
            "json" -> parseJsonStringList(normalized)
            else -> throw IllegalArgumentException("format must be lines, text, or json")
        }
        return create(id, displayName, payloads)
    }

    @Synchronized
    fun list(): List<PayloadListDefinition> = lists.values.toList()

    @Synchronized
    fun get(id: String): PayloadListDefinition = lists[id] ?: throw NoSuchElementException("payload list not found")

    @Synchronized
    fun page(id: String, offset: Int, limit: Int): PayloadListPage {
        require(offset >= 0) { "offset must be non-negative" }
        require(limit in 1..500) { "limit must be between 1 and 500" }
        val definition = get(id)
        val end = minOf(definition.payloads.size, offset + limit)
        val values = if (offset >= definition.payloads.size) emptyList() else definition.payloads.subList(offset, end)
        return PayloadListPage(definition, values, definition.payloads.size, end.takeIf { it < definition.payloads.size })
    }

    @Synchronized
    fun update(
        id: String,
        operation: String,
        payloads: List<String>,
        index: Int,
        indexes: List<Int>,
        displayName: String?,
    ): PayloadListDefinition {
        val current = get(id)
        val values = current.payloads.toMutableList()
        when (operation) {
            "append" -> values.addAll(payloads)
            "prepend" -> values.addAll(0, payloads)
            "insert" -> {
                require(index in 0..values.size) { "index must be between 0 and payload_count" }
                values.addAll(index, payloads)
            }
            "replace" -> {
                require(payloads.size == 1) { "replace requires exactly one payload" }
                require(index in values.indices) { "index is out of range" }
                values[index] = payloads.single()
            }
            "remove" -> {
                require(index in values.indices) { "index is out of range" }
                values.removeAt(index)
            }
            "remove_indexes" -> {
                require(indexes.isNotEmpty()) { "indexes must not be empty" }
                require(indexes.distinct().size == indexes.size) { "indexes must not contain duplicates" }
                require(indexes.all { it in values.indices }) { "one or more indexes are out of range" }
                indexes.sortedDescending().forEach(values::removeAt)
            }
            "clear" -> {
                require(payloads.isEmpty() && indexes.isEmpty()) { "clear does not accept payloads or indexes" }
                values.clear()
            }
            "rename" -> require(!displayName.isNullOrBlank()) { "display_name is required for rename" }
            else -> throw IllegalArgumentException("operation must be append, prepend, insert, replace, remove, remove_indexes, clear, or rename")
        }
        val updated = validatedPayloadList(id, displayName ?: current.displayName, values)
        lists[id] = updated
        return updated
    }

    @Synchronized
    fun delete(id: String): Boolean = lists.remove(id) != null

    @Synchronized
    fun boundedSlice(id: String, offset: Int, count: Int = MAX_INTRUDER_PAYLOADS): List<String> {
        require(offset >= 0) { "payload_offset must be non-negative" }
        val payloads = get(id).payloads
        require(offset < payloads.size) { "payload_offset must be less than payload_count" }
        return payloads.drop(offset).take(count)
    }
}

private fun validatedPayloadList(id: String, displayName: String, payloads: List<String>): PayloadListDefinition {
    require(id.isNotBlank()) { "id must not be blank" }
    require(displayName.isNotBlank()) { "display_name must not be blank" }
    require(id.length <= 120) { "id must contain at most 120 characters" }
    require(displayName.length <= 120) { "display_name must contain at most 120 characters" }
    require(payloads.size <= MAX_PAYLOAD_LIST_ENTRIES) { "payload list must contain at most $MAX_PAYLOAD_LIST_ENTRIES entries" }
    require(payloads.all { it.toByteArray(StandardCharsets.UTF_8).size <= MAX_PAYLOAD_VALUE_BYTES }) {
        "each payload must contain at most $MAX_PAYLOAD_VALUE_BYTES UTF-8 bytes"
    }
    val size = payloads.sumOf { it.toByteArray(StandardCharsets.UTF_8).size.toLong() }
    require(size <= MAX_PAYLOAD_LIST_BYTES) { "payload list exceeds $MAX_PAYLOAD_LIST_BYTES UTF-8 bytes" }
    return PayloadListDefinition(id, displayName, payloads.toList())
}

private fun parseJsonStringList(content: String): List<String> {
    val mapper = com.fasterxml.jackson.databind.ObjectMapper()
    val node = mapper.readTree(content)
    require(node.isArray) { "JSON payload list must be an array of strings" }
    return node.map { item ->
        require(item.isTextual) { "JSON payload list must contain only strings" }
        item.asText()
    }
}

private fun payloadListFingerprint(payloads: List<String>): String {
    val digest = MessageDigest.getInstance("SHA-256")
    payloads.forEach { payload ->
        val bytes = payload.toByteArray(StandardCharsets.UTF_8)
        digest.update((bytes.size ushr 24).toByte())
        digest.update((bytes.size ushr 16).toByte())
        digest.update((bytes.size ushr 8).toByte())
        digest.update(bytes.size.toByte())
        digest.update(bytes)
    }
    return digest.digest().joinToString("") { byte -> "%02x".format(byte.toInt() and 0xff) }
}
