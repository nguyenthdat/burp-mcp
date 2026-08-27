package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.core.ByteArray as MontoyaByteArray
import burp.api.montoya.core.Registration
import burp.api.montoya.http.message.requests.HttpRequest
import burp.api.montoya.ui.contextmenu.WebSocketMessage
import burp.api.montoya.ui.editor.RawEditor
import burp.api.montoya.ui.editor.extension.EditorCreationContext
import burp.api.montoya.ui.editor.extension.EditorMode
import burp.api.montoya.ui.editor.extension.ExtensionProvidedWebSocketMessageEditor
import burp.api.montoya.ui.editor.extension.WebSocketMessageEditorProvider
import burp.api.montoya.websocket.Direction
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import java.time.Clock
import java.time.Instant
import java.time.ZoneOffset
import java.util.Optional
import javax.swing.JPanel
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNotEquals
import kotlin.test.assertTrue

class WebSocketEditorFacadeTest {
    @Test
    fun `extension tab stages binary replacement through typed editor contract`() {
        val fixture = fixture(EditorMode.DEFAULT, byteArrayOf(0x00, 0xff.toByte(), 0x41))

        val captured = fixture.facade.capture()
        val updated = fixture.facade.replace(captured.token, captured.sha256, byteArrayOf(0x42, 0x00))

        assertContentEquals(byteArrayOf(0x42, 0x00), updated.payload)
        assertNotEquals(captured.sha256, updated.sha256)
        assertTrue(fixture.extensionEditor.isModified)
        assertTrue(updated.applyRequired)
        assertEquals("extension_tab", updated.source)
        assertEquals("CLIENT_TO_SERVER", updated.direction)
        assertEquals("wss://example.test/socket", updated.upgradeUrl)
        assertContentEquals(byteArrayOf(0x42, 0x00), fixture.editor.getContents().getBytes())
    }

    @Test
    fun `read only WebSocket editor rejects replacement`() {
        val fixture = fixture(EditorMode.READ_ONLY, "message".encodeToByteArray())
        val captured = fixture.facade.capture()

        val error = assertFailsWith<IllegalStateException> {
            fixture.facade.replace(captured.token, captured.sha256, "changed".encodeToByteArray())
        }

        assertEquals("WebSocket editor is read only", error.message)
    }

    @Test
    fun `WebSocket replacement rejects stale contents and consumes token`() {
        val fixture = fixture(EditorMode.DEFAULT, "first".encodeToByteArray())
        val captured = fixture.facade.capture()
        fixture.editor.setContents(byteArray("changed".encodeToByteArray()))

        assertFailsWith<IllegalStateException> {
            fixture.facade.replace(captured.token, captured.sha256, "replacement".encodeToByteArray())
        }
        assertFailsWith<NoSuchElementException> {
            fixture.facade.replace(captured.token, captured.sha256, "replacement".encodeToByteArray())
        }
    }

    @Test
    fun `close deregisters WebSocket editor provider`() {
        val fixture = fixture(EditorMode.DEFAULT, byteArrayOf())

        fixture.facade.close()

        verify(exactly = 1) { fixture.registration.deregister() }
    }

    private fun fixture(mode: EditorMode, initialPayload: ByteArray): Fixture {
        val editor = mockk<RawEditor>()
        val panel = JPanel()
        var contents = byteArray(initialPayload)
        var modified = false
        every { editor.uiComponent() } returns panel
        every { editor.getContents() } answers { contents }
        every { editor.setContents(any()) } answers {
            contents = byteArray(firstArg<MontoyaByteArray>().getBytes())
            modified = false
        }
        every { editor.isModified() } answers { modified }
        every { editor.caretPosition() } answers { contents.length() }
        every { editor.selection() } returns Optional.empty()

        val api = mockk<MontoyaApi>(relaxed = true)
        val registration = mockk<Registration>(relaxed = true)
        var provider: WebSocketMessageEditorProvider? = null
        val context = mockk<EditorCreationContext>()
        every { context.editorMode() } returns mode
        val facade = WebSocketEditorFacade(
            api = api,
            clock = Clock.fixed(Instant.ofEpochMilli(1_000), ZoneOffset.UTC),
            tokenFactory = { "ws-token" },
            focusOwner = { panel },
            byteArrayFactory = ::byteArray,
            createRawEditor = { editor },
            registerProvider = {
                provider = it
                registration
            },
        )
        val extensionEditor = requireNotNull(provider).provideMessageEditor(context)
        val upgrade = mockk<HttpRequest>()
        every { upgrade.url() } returns "wss://example.test/socket"
        val message = mockk<WebSocketMessage>()
        every { message.payload() } returns byteArray(initialPayload)
        every { message.direction() } returns Direction.CLIENT_TO_SERVER
        every { message.upgradeRequest() } returns upgrade
        extensionEditor.setMessage(message)
        return Fixture(facade, editor, extensionEditor, registration)
    }

    private data class Fixture(
        val facade: WebSocketEditorFacade,
        val editor: RawEditor,
        val extensionEditor: ExtensionProvidedWebSocketMessageEditor,
        val registration: Registration,
    )

    private fun byteArray(value: ByteArray): MontoyaByteArray {
        val result = mockk<MontoyaByteArray>()
        every { result.getBytes() } returns value.copyOf()
        every { result.length() } returns value.size
        return result
    }
}
