package io.github.nguyenthdat.burpmcp

import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.databind.node.ObjectNode
import java.nio.charset.StandardCharsets

internal sealed class PatchOp {
    data class ReplaceAllText(val newText: String) : PatchOp()
    data class ReplaceAllPayload(val newPayload: ByteArray) : PatchOp()
    data class ReplaceSelection(val replacement: String, val selectionStart: Int, val selectionEnd: Int) : PatchOp()
    data class RegexPatch(val pattern: String, val replacement: String, val replaceAll: Boolean, val caseInsensitive: Boolean) : PatchOp()
    data class HeaderPatch(val name: String, val value: String, val remove: Boolean) : PatchOp()
    data class JsonPatch(val jsonPath: String, val valueJson: String) : PatchOp()
    data class ParamPatch(val name: String, val value: String, val remove: Boolean, val paramType: String?) : PatchOp()
}

internal object EditorPatchEngine {
    private val mapper = ObjectMapper()

    fun applyPatchToText(originalText: String, op: PatchOp): String {
        return when (op) {
            is PatchOp.ReplaceAllText -> op.newText
            is PatchOp.ReplaceAllPayload -> String(op.newPayload, StandardCharsets.UTF_8)
            is PatchOp.ReplaceSelection -> {
                val start = op.selectionStart.coerceIn(0, originalText.length)
                val end = op.selectionEnd.coerceIn(start, originalText.length)
                originalText.substring(0, start) + op.replacement + originalText.substring(end)
            }
            is PatchOp.RegexPatch -> {
                val options = if (op.caseInsensitive) setOf(RegexOption.IGNORE_CASE) else emptySet()
                val regex = Regex(op.pattern, options)
                if (op.replaceAll) {
                    regex.replace(originalText, op.replacement)
                } else {
                    regex.replaceFirst(originalText, op.replacement)
                }
            }
            is PatchOp.HeaderPatch -> {
                applyHeaderChange(originalText, op.name, op.value, op.remove)
            }
            is PatchOp.JsonPatch -> {
                applyJsonPatch(originalText, op.jsonPath, op.valueJson)
            }
            is PatchOp.ParamPatch -> {
                applyParamPatch(originalText, op.name, op.value, op.remove, op.paramType)
            }
        }
    }

    fun applyPatchToPayload(originalPayload: ByteArray, op: PatchOp): ByteArray {
        return when (op) {
            is PatchOp.ReplaceAllPayload -> op.newPayload
            is PatchOp.ReplaceAllText -> op.newText.toByteArray(StandardCharsets.UTF_8)
            else -> {
                val text = String(originalPayload, StandardCharsets.UTF_8)
                val patched = applyPatchToText(text, op)
                patched.toByteArray(StandardCharsets.UTF_8)
            }
        }
    }

    private fun applyHeaderChange(rawHttp: String, headerName: String, headerValue: String, remove: Boolean): String {
        val newline = if (rawHttp.contains("\r\n")) "\r\n" else "\n"
        val delimiter = "$newline$newline"
        val parts = rawHttp.split(delimiter, limit = 2)
        val headerPart = parts[0]
        val bodyPart = if (parts.size > 1) parts[1] else ""

        val lines = headerPart.split(newline).toMutableList()
        if (lines.isEmpty()) return rawHttp

        val requestLine = lines[0]
        val headerLines = lines.drop(1).toMutableList()

        val prefix = "${headerName.trim().lowercase()}:"
        val existingIdx = headerLines.indexOfFirst { it.trim().lowercase().startsWith(prefix) }

        if (remove) {
            if (existingIdx >= 0) {
                headerLines.removeAt(existingIdx)
            }
        } else {
            val formatted = "${headerName.trim()}: ${headerValue.trim()}"
            if (existingIdx >= 0) {
                headerLines[existingIdx] = formatted
            } else {
                headerLines.add(formatted)
            }
        }

        // Auto update Content-Length if body exists
        if (bodyPart.isNotEmpty()) {
            val bodyBytes = bodyPart.toByteArray(StandardCharsets.UTF_8)
            val clIdx = headerLines.indexOfFirst { it.trim().lowercase().startsWith("content-length:") }
            if (clIdx >= 0) {
                headerLines[clIdx] = "Content-Length: ${bodyBytes.size}"
            }
        }

        val newHead = (listOf(requestLine) + headerLines).joinToString("\r\n")
        return if (parts.size > 1) "$newHead\r\n\r\n$bodyPart" else "$newHead\r\n\r\n"
    }

    private fun applyJsonPatch(rawHttp: String, jsonPath: String, valueJson: String): String {
        val newline = if (rawHttp.contains("\r\n")) "\r\n" else "\n"
        val delimiter = "$newline$newline"
        val parts = rawHttp.split(delimiter, limit = 2)

        if (parts.size < 2) {
            // Raw text is standalone JSON
            return updateJsonString(rawHttp, jsonPath, valueJson)
        }

        val headers = parts[0]
        val body = parts[1]
        val newBody = updateJsonString(body, jsonPath, valueJson)
        return applyHeaderChange("$headers\r\n\r\n$newBody", "Content-Length", newBody.toByteArray(StandardCharsets.UTF_8).size.toString(), false)
    }

    private fun updateJsonString(jsonText: String, path: String, valueJson: String): String {
        val rootNode = runCatching { mapper.readTree(jsonText) }.getOrNull() ?: return jsonText
        val parsedValue = runCatching { mapper.readTree(valueJson) }.getOrElse { mapper.valueToTree(valueJson) }

        val cleanPath = path.trim().removePrefix("$").removePrefix(".")
        val segments = cleanPath.split(".").filter { it.isNotBlank() }

        if (segments.isEmpty()) {
            return mapper.writeValueAsString(parsedValue)
        }

        if (rootNode is ObjectNode) {
            var current: ObjectNode = rootNode
            for (i in 0 until segments.size - 1) {
                val seg = segments[i]
                current = current.withObject(seg)
            }
            current.set<ObjectNode>(segments.last(), parsedValue)
            return mapper.writeValueAsString(rootNode)
        }

        return jsonText
    }

    private fun applyParamPatch(rawHttp: String, name: String, value: String, remove: Boolean, paramType: String?): String {
        val newline = if (rawHttp.contains("\r\n")) "\r\n" else "\n"
        val delimiter = "$newline$newline"
        val parts = rawHttp.split(delimiter, limit = 2)
        val headerPart = parts[0]
        val bodyPart = if (parts.size > 1) parts[1] else ""

        val lines = headerPart.split(newline).toMutableList()
        if (lines.isEmpty()) return rawHttp

        val reqLine = lines[0]
        val reqTokens = reqLine.split(" ", limit = 3)
        if (reqTokens.size < 2) return rawHttp

        val method = reqTokens[0]
        val uri = reqTokens[1]
        val version = if (reqTokens.size > 2) reqTokens[2] else "HTTP/1.1"

        if (paramType == "body" || (paramType == null && method.equals("POST", ignoreCase = true) && bodyPart.isNotEmpty())) {
            val newBody = mutateQueryString(bodyPart, name, value, remove)
            val updated = applyHeaderChange("$headerPart\r\n\r\n$newBody", "Content-Length", newBody.toByteArray(StandardCharsets.UTF_8).size.toString(), false)
            return updated
        } else {
            val (path, query) = if (uri.contains("?")) uri.split("?", limit = 2) else listOf(uri, "")
            val newQuery = mutateQueryString(query, name, value, remove)
            val newUri = if (newQuery.isNotEmpty()) "$path?$newQuery" else path
            val newReqLine = "$method $newUri $version"
            lines[0] = newReqLine
            val newHead = lines.joinToString("\r\n")
            return if (parts.size > 1) "$newHead\r\n\r\n$bodyPart" else "$newHead\r\n\r\n"
        }
    }

    private fun mutateQueryString(query: String, name: String, value: String, remove: Boolean): String {
        val pairs = if (query.isBlank()) mutableListOf() else query.split("&").filter { it.isNotBlank() }.toMutableList()
        val encName = java.net.URLEncoder.encode(name, "UTF-8")
        val encVal = java.net.URLEncoder.encode(value, "UTF-8")

        val prefix = "$encName="
        val idx = pairs.indexOfFirst { it == encName || it.startsWith(prefix) }

        if (remove) {
            if (idx >= 0) pairs.removeAt(idx)
        } else {
            val formatted = "$encName=$encVal"
            if (idx >= 0) {
                pairs[idx] = formatted
            } else {
                pairs.add(formatted)
            }
        }
        return pairs.joinToString("&")
    }
}
