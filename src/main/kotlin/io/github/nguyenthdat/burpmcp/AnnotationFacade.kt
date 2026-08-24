package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.core.HighlightColor

internal class AnnotationFacade(
    private val api: MontoyaApi,
) {
    fun highlight(index: Int, color: String?): String {
        val entry = historyEntry(index)
        if (color.isNullOrBlank()) {
            entry.annotations().setHighlightColor(HighlightColor.NONE)
        } else {
            val parsed = HighlightColor.entries.firstOrNull { it.name.equals(color, ignoreCase = true) }
                ?: error("unknown highlight color: $color")
            entry.annotations().setHighlightColor(parsed)
        }
        return entry.annotations().highlightColor().name
    }

    fun annotate(index: Int, note: String) {
        historyEntry(index).annotations().setNotes(note)
    }

    private fun historyEntry(index: Int) =
        api.proxy().history().getOrNull(index) ?: error("proxy history index out of range: $index")
}
