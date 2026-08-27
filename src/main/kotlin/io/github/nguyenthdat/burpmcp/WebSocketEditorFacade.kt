package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.core.ByteArray as MontoyaByteArray
import burp.api.montoya.core.Registration
import burp.api.montoya.ui.Selection
import burp.api.montoya.ui.contextmenu.WebSocketMessage
import burp.api.montoya.ui.editor.EditorOptions
import burp.api.montoya.ui.editor.RawEditor
import burp.api.montoya.ui.editor.extension.EditorCreationContext
import burp.api.montoya.ui.editor.extension.EditorMode
import burp.api.montoya.ui.editor.extension.ExtensionProvidedWebSocketMessageEditor
import burp.api.montoya.ui.editor.extension.WebSocketMessageEditorProvider
import java.awt.Component
import java.lang.ref.WeakReference
import java.security.MessageDigest
import java.time.Clock
import java.util.UUID
import javax.swing.SwingUtilities

internal data class WebSocketEditorSnapshot(
    val token: String,
    val payload: ByteArray,
    val editable: Boolean,
    val sha256: String,
    val caretPosition: Int,
    val selectionStart: Int,
    val selectionEnd: Int,
    val direction: String,
    val upgradeUrl: String,
    val source: String,
    val applyRequired: Boolean,
)

internal class WebSocketEditorFacade(
    private val api: MontoyaApi,
    private val clock: Clock = Clock.systemUTC(),
    private val tokenFactory: () -> String = { UUID.randomUUID().toString() },
    private val focusOwner: () -> Component? = {
        java.awt.KeyboardFocusManager.getCurrentKeyboardFocusManager().permanentFocusOwner
    },
    private val byteArrayFactory: (ByteArray) -> MontoyaByteArray = { MontoyaByteArray.byteArray(*it) },
    private val createRawEditor: (EditorCreationContext) -> RawEditor = { creationContext ->
        if (creationContext.editorMode() == EditorMode.READ_ONLY) {
            api.userInterface().createRawEditor(EditorOptions.READ_ONLY)
        } else {
            api.userInterface().createRawEditor()
        }
    },
    private val registerProvider: (WebSocketMessageEditorProvider) -> Registration = {
        api.userInterface().registerWebSocketMessageEditorProvider(it)
    },
) : AutoCloseable {
    private data class Lease(
        val editor: WeakReference<McpWebSocketEditor>,
        val expiresAtMillis: Long,
    )

    private val leases = LinkedHashMap<String, Lease>()
    private val editors = mutableListOf<WeakReference<McpWebSocketEditor>>()
    private val registration = registerProvider(
        object : WebSocketMessageEditorProvider {
            override fun provideMessageEditor(creationContext: EditorCreationContext): ExtensionProvidedWebSocketMessageEditor =
                McpWebSocketEditor(creationContext).also { editors.add(WeakReference(it)) }
        },
    )

    fun capture(): WebSocketEditorSnapshot = onEdt {
        pruneExpired()
        val editor = activeEditor() ?: throw NoSuchElementException("no active Burp WebSocket editor")
        val token = tokenFactory()
        leases[token] = Lease(WeakReference(editor), clock.millis() + TOKEN_TTL_MILLIS)
        editor.snapshot(token)
    }

    fun replace(token: String, expectedSha256: String, payload: ByteArray): WebSocketEditorSnapshot = onEdt {
        require(token.isNotBlank()) { "token is required" }
        require(expectedSha256.matches(SHA256_REGEX)) { "expected_sha256 must be a lowercase SHA-256 digest" }
        require(payload.size <= MAX_WEBSOCKET_EDITOR_BYTES) { "payload exceeds $MAX_WEBSOCKET_EDITOR_BYTES bytes" }
        pruneExpired()
        val lease = leases.remove(token) ?: throw NoSuchElementException("WebSocket editor token was not found or expired")
        val editor = lease.editor.get() ?: throw NoSuchElementException("WebSocket editor is no longer available")
        check(editor.editable) { "WebSocket editor is read only" }
        check(sha256(editor.messageBytes()) == expectedSha256) { "WebSocket editor contents changed after capture" }
        editor.replace(payload)
        editor.snapshot(token)
    }

    override fun close() {
        onEdt {
            leases.clear()
            editors.clear()
        }
        registration.deregister()
    }

    private fun activeEditor(): McpWebSocketEditor? {
        val focused = focusOwner() ?: return null
        return editors.asSequence()
            .mapNotNull(WeakReference<McpWebSocketEditor>::get)
            .firstOrNull { editor -> generateSequence(focused) { it.parent }.any { it === editor.component } }
    }

    private fun pruneExpired() {
        val now = clock.millis()
        leases.entries.removeIf { (_, lease) -> lease.expiresAtMillis <= now || lease.editor.get() == null }
        editors.removeIf { it.get() == null }
        while (leases.size >= MAX_ACTIVE_LEASES) {
            leases.entries.iterator().run {
                next()
                remove()
            }
        }
    }

    private inner class McpWebSocketEditor(
        private val creationContext: EditorCreationContext,
    ) : ExtensionProvidedWebSocketMessageEditor {
        private val rawEditor: RawEditor = createRawEditor(creationContext)
        val component: Component = rawEditor.uiComponent()
        private var currentMessage: WebSocketMessage? = null
        private var changedByMcp = false

        val editable: Boolean
            get() = creationContext.editorMode() != EditorMode.READ_ONLY

        override fun getMessage(): MontoyaByteArray = rawEditor.getContents()

        override fun setMessage(message: WebSocketMessage) {
            currentMessage = message
            rawEditor.setContents(message.payload())
            changedByMcp = false
        }

        override fun isEnabledFor(message: WebSocketMessage): Boolean = true

        override fun caption(): String = "MCP"

        override fun uiComponent(): Component = component

        override fun selectedData(): Selection? = rawEditor.selection().orElse(null)

        override fun isModified(): Boolean = changedByMcp || rawEditor.isModified()

        fun messageBytes(): ByteArray = rawEditor.getContents().getBytes()

        fun replace(payload: ByteArray) {
            rawEditor.setContents(byteArrayFactory(payload))
            changedByMcp = true
        }

        fun snapshot(token: String): WebSocketEditorSnapshot {
            val selection = rawEditor.selection().orElse(null)?.offsets()
            val message = currentMessage
            val payload = messageBytes()
            val caret = rawEditor.caretPosition()
            return WebSocketEditorSnapshot(
                token = token,
                payload = payload,
                editable = editable,
                sha256 = sha256(payload),
                caretPosition = caret,
                selectionStart = selection?.startIndexInclusive() ?: caret,
                selectionEnd = selection?.endIndexExclusive() ?: caret,
                direction = runCatching { message?.direction()?.name }.getOrNull().orEmpty(),
                upgradeUrl = runCatching { message?.upgradeRequest()?.url() }.getOrNull().orEmpty(),
                source = "extension_tab",
                applyRequired = changedByMcp,
            )
        }
    }

    private fun <T> onEdt(block: () -> T): T {
        if (SwingUtilities.isEventDispatchThread()) return block()
        var result: Result<T>? = null
        SwingUtilities.invokeAndWait { result = runCatching(block) }
        return checkNotNull(result).getOrThrow()
    }

    private companion object {
        const val TOKEN_TTL_MILLIS = 30_000L
        const val MAX_ACTIVE_LEASES = 32
        const val MAX_WEBSOCKET_EDITOR_BYTES = 16 * 1024 * 1024
        val SHA256_REGEX = Regex("[0-9a-f]{64}")

        fun sha256(value: ByteArray): String =
            MessageDigest.getInstance("SHA-256")
                .digest(value)
                .joinToString("") { byte -> "%02x".format(byte.toInt() and 0xff) }
    }
}
