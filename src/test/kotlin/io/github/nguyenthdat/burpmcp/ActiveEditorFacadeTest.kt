package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import io.mockk.mockk
import java.time.Clock
import java.time.Instant
import java.time.ZoneOffset
import javax.swing.JPanel
import javax.swing.JTextArea
import javax.swing.SwingUtilities
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNotEquals
import kotlin.test.assertTrue

class ActiveEditorFacadeTest {
    @Test
    fun `capture and guarded replacement target the same editor on the EDT`() {
        val fixture = fixture("GET /old HTTP/1.1\r\n\r\n")
        fixture.editor.select(4, 8)

        val captured = fixture.facade.capture()
        val updated = fixture.facade.replace(captured.token, captured.sha256, "GET /new HTTP/1.1\r\n\r\n")

        assertEquals("GET /new HTTP/1.1\r\n\r\n", fixture.editor.text)
        assertEquals(fixture.editor.text, updated.text)
        assertNotEquals(captured.sha256, updated.sha256)
        assertEquals(4, captured.selectionStart)
        assertEquals(8, captured.selectionEnd)
        assertTrue(fixture.editor.lastMutationWasOnEdt)
    }

    @Test
    fun `replacement rejects stale content and consumes the token`() {
        val fixture = fixture("first")
        val captured = fixture.facade.capture()
        SwingUtilities.invokeAndWait { fixture.editor.text = "changed by user" }

        val stale = assertFailsWith<IllegalStateException> {
            fixture.facade.replace(captured.token, captured.sha256, "replacement")
        }
        assertEquals("active editor contents changed after capture", stale.message)
        assertFailsWith<NoSuchElementException> {
            fixture.facade.replace(captured.token, captured.sha256, "replacement")
        }
    }

    @Test
    fun `capture rejects focus outside Burp`() {
        val fixture = fixture("request", attachEditor = false)

        val error = assertFailsWith<NoSuchElementException> { fixture.facade.capture() }

        assertEquals("no active Burp text editor", error.message)
    }

    private fun fixture(text: String, attachEditor: Boolean = true): Fixture {
        val root = JPanel()
        val editor = TrackingTextArea(text)
        if (attachEditor) root.add(editor)
        val api = mockk<MontoyaApi>(relaxed = true)
        val facade = ActiveEditorFacade(
            api = api,
            clock = Clock.fixed(Instant.ofEpochMilli(1_000), ZoneOffset.UTC),
            tokenFactory = { "editor-token" },
            focusOwner = { editor },
            suiteRoot = { root },
        )
        return Fixture(root, editor, facade)
    }

    private data class Fixture(
        val root: JPanel,
        val editor: TrackingTextArea,
        val facade: ActiveEditorFacade,
    )

    private class TrackingTextArea(text: String) : JTextArea(text) {
        var lastMutationWasOnEdt = false

        override fun setText(text: String?) {
            lastMutationWasOnEdt = SwingUtilities.isEventDispatchThread()
            super.setText(text)
        }
    }
}
