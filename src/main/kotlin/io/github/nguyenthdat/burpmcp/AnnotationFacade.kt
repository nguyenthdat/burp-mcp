package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.core.HighlightColor
import burp.api.montoya.proxy.ProxyHttpRequestResponse

internal class AnnotationFacade(
    private val api: MontoyaApi,
) {

    fun findEntry(id: Int? = null, index: Int? = null): ProxyHttpRequestResponse {
        if (id != null) {
            val matches = runCatching {
                api.proxy().history { entry -> runCatching { entry.id() == id }.getOrDefault(false) }
            }.getOrNull()
            matches?.firstOrNull()?.let { return it }

            val all = runCatching { api.proxy().history() }.getOrNull().orEmpty()
            return all.firstOrNull { runCatching { it.id() == id }.getOrDefault(false) }
                ?: error("proxy history entry with id $id not found")
        }
        if (index != null) {
            val history = api.proxy().history()
            return history.getOrNull(index) ?: error("proxy history index out of range: $index")
        }
        error("either id or index must be provided")
    }

    fun highlight(index: Int, color: String?): String = highlight(index = index, color = color, id = null)

    fun highlight(index: Int? = null, color: String?, id: Int? = null): String {
        val entry = findEntry(id, index)
        if (color.isNullOrBlank()) {
            entry.annotations().setHighlightColor(HighlightColor.NONE)
        } else {
            val parsed = HighlightColor.entries.firstOrNull { it.name.equals(color, ignoreCase = true) }
                ?: error("unknown highlight color: $color")
            entry.annotations().setHighlightColor(parsed)
        }
        return entry.annotations().highlightColor().name
    }

    fun annotate(index: Int, note: String) = annotate(index = index, note = note, id = null)

    fun annotate(index: Int? = null, note: String, id: Int? = null) {
        findEntry(id, index).annotations().setNotes(note)
    }

    fun annotateProxyEntries(entries: List<ProxyAnnotation>): Int {
        require(entries.size <= MAX_BATCH_SIZE) { "at most $MAX_BATCH_SIZE proxy entries may be annotated" }
        if (entries.isEmpty()) return 0
        val byId = entries.associateBy(ProxyAnnotation::id)
        val historyById = api.proxy().history { entry ->
            runCatching { entry.id() in byId }.getOrDefault(false)
        }.associateBy { entry -> entry.id() }
        var annotated = 0
        for ((id, annotation) in byId) {
            val entry = historyById[id] ?: continue
            applySiteGraphAnnotation(entry, annotation.highlight, annotation.marker)
            annotated++
        }
        return annotated
    }

    private fun applySiteGraphAnnotation(
        entry: ProxyHttpRequestResponse,
        highlight: String,
        marker: String,
    ) {
        val annotations = entry.annotations()
        if (annotations.highlightColor() == HighlightColor.NONE) {
            val parsed = HighlightColor.entries.firstOrNull { it.name.equals(highlight, ignoreCase = true) }
                ?: error("unknown highlight color: $highlight")
            annotations.setHighlightColor(parsed)
        }
        annotations.setNotes(mergeSiteGraphNote(annotations.notes().orEmpty(), marker))
    }

    data class ProxyAnnotation(
        val id: Int,
        val highlight: String,
        val marker: String,
    )


    internal fun mergeSiteGraphNote(existingNotes: String, newMarker: String): String {
        if (existingNotes.isBlank()) {
            return newMarker
        }
        val lines = existingNotes.lines()
        val siteGraphIndex = lines.indexOfFirst { it.startsWith("[SiteGraph]") }
        return if (siteGraphIndex >= 0) {
            val updatedLines = lines.toMutableList()
            updatedLines[siteGraphIndex] = newMarker
            var i = siteGraphIndex + 1
            while (i < updatedLines.size) {
                if (updatedLines[i].startsWith("[SiteGraph]")) {
                    updatedLines.removeAt(i)
                } else {
                    i++
                }
            }
            updatedLines.joinToString("\n")
        } else {
            if (existingNotes.endsWith("\n")) {
                "$existingNotes$newMarker"
            } else {
                "$existingNotes\n$newMarker"
            }
        }
    }
    private companion object {
        const val MAX_BATCH_SIZE = 50
    }
}
