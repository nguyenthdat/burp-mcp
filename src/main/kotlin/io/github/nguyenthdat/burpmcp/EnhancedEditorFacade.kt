package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.core.ByteArray as MontoyaByteArray
import burp.api.montoya.core.Registration
import burp.api.montoya.http.message.HttpRequestResponse
import burp.api.montoya.http.message.requests.HttpRequest
import burp.api.montoya.http.message.responses.HttpResponse
import burp.api.montoya.ui.Selection
import burp.api.montoya.ui.contextmenu.ContextMenuEvent
import burp.api.montoya.ui.contextmenu.ContextMenuItemsProvider
import burp.api.montoya.ui.contextmenu.WebSocketMessage
import burp.api.montoya.ui.editor.EditorOptions
import burp.api.montoya.ui.editor.RawEditor
import burp.api.montoya.ui.editor.extension.EditorCreationContext
import burp.api.montoya.ui.editor.extension.EditorMode
import burp.api.montoya.ui.editor.extension.ExtensionProvidedHttpRequestEditor
import burp.api.montoya.ui.editor.extension.ExtensionProvidedHttpResponseEditor
import burp.api.montoya.ui.editor.extension.ExtensionProvidedWebSocketMessageEditor
import burp.api.montoya.ui.editor.extension.HttpRequestEditorProvider
import burp.api.montoya.ui.editor.extension.HttpResponseEditorProvider
import burp.api.montoya.ui.editor.extension.WebSocketMessageEditorProvider
import java.awt.Component
import java.awt.KeyboardFocusManager
import java.lang.ref.WeakReference
import java.nio.charset.StandardCharsets
import java.security.MessageDigest
import java.time.Clock
import java.util.UUID
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.CopyOnWriteArrayList
import javax.swing.JMenuItem
import javax.swing.JTextArea
import javax.swing.SwingUtilities

internal enum class EditorKind {
    HTTP_REQUEST, HTTP_RESPONSE, WEBSOCKET, RAW_TEXT
}

internal data class UnifiedEditorSnapshot(
    val token: String,
    val kind: EditorKind,
    val toolSource: String,
    val tabName: String,
    val host: String,
    val port: Int,
    val https: Boolean,
    val text: String,
    val payload: ByteArray,
    val isJson: Boolean,
    val editable: Boolean,
    val sha256: String,
    val caretPosition: Int,
    val selectionStart: Int,
    val selectionEnd: Int,
    val selectedText: String,
    val expiresAtMillis: Long,
)

internal interface EditorTarget {
    fun isAlive(): Boolean
    fun isEditable(): Boolean
    fun currentText(): String
    fun currentPayload(): ByteArray
    fun currentSha256(): String
    fun kind(): EditorKind
    fun toolSource(): String
    fun tabName(): String
    fun host(): String
    fun port(): Int
    fun https(): Boolean
    fun caretPosition(): Int
    fun selectionStart(): Int
    fun selectionEnd(): Int
    fun selectedText(): String
    fun applyPatch(op: PatchOp)
}

internal class EnhancedEditorFacade(
    private val api: MontoyaApi,
    private val clock: Clock = Clock.systemUTC(),
    private val defaultTtlMillis: Long = 120_000L,
    private val suiteRoot: () -> Component = { api.userInterface().swingUtils().suiteFrame() },
) : AutoCloseable {

    private data class ActiveLease(
        val target: EditorTarget,
        var expiresAtMillis: Long,
        var lastKnownSha256: String,
    )

    private val leases = ConcurrentHashMap<String, ActiveLease>()
    private val registeredHttpReqEditors = CopyOnWriteArrayList<WeakReference<McpHttpReqEditor>>()
    private val registeredHttpRespEditors = CopyOnWriteArrayList<WeakReference<McpHttpRespEditor>>()
    private val registeredWsEditors = CopyOnWriteArrayList<WeakReference<McpWebSocketEditor>>()

    private var lastActiveEditor: WeakReference<EditorTarget>? = null
    private var stagedBuffer: StagedBufferTarget? = null

    private val registrations = mutableListOf<Registration>()

    init {
        // 1. Register HTTP Request Editor Provider
        val reqReg = api.userInterface().registerHttpRequestEditorProvider(
            object : HttpRequestEditorProvider {
                override fun provideHttpRequestEditor(creationContext: EditorCreationContext): ExtensionProvidedHttpRequestEditor {
                    return McpHttpReqEditor(api, creationContext).also {
                        registeredHttpReqEditors.add(WeakReference(it))
                        lastActiveEditor = WeakReference(it)
                    }
                }
            }
        )
        registrations.add(reqReg)

        // 2. Register HTTP Response Editor Provider
        val respReg = api.userInterface().registerHttpResponseEditorProvider(
            object : HttpResponseEditorProvider {
                override fun provideHttpResponseEditor(creationContext: EditorCreationContext): ExtensionProvidedHttpResponseEditor {
                    return McpHttpRespEditor(api, creationContext).also {
                        registeredHttpRespEditors.add(WeakReference(it))
                        lastActiveEditor = WeakReference(it)
                    }
                }
            }
        )
        registrations.add(respReg)

        // 3. Register WebSocket Message Editor Provider
        val wsReg = api.userInterface().registerWebSocketMessageEditorProvider(
            object : WebSocketMessageEditorProvider {
                override fun provideMessageEditor(creationContext: EditorCreationContext): ExtensionProvidedWebSocketMessageEditor {
                    return McpWebSocketEditor(api, creationContext).also {
                        registeredWsEditors.add(WeakReference(it))
                        lastActiveEditor = WeakReference(it)
                    }
                }
            }
        )
        registrations.add(wsReg)

        // 4. Register Context Menu Provider
        val menuReg = api.userInterface().registerContextMenuItemsProvider(
            object : ContextMenuItemsProvider {
                override fun provideMenuItems(event: ContextMenuEvent): List<Component> {
                    val menuList = mutableListOf<Component>()
                    val editorOpt = runCatching { event.messageEditorRequestResponse() }.getOrNull()
                    val editorReqResp = if (editorOpt != null && editorOpt.isPresent) editorOpt.get().requestResponse() else null
                    val selectedList = runCatching { event.selectedRequestResponses() }.getOrDefault(emptyList())

                    val reqResp = editorReqResp ?: selectedList.firstOrNull()
                    if (reqResp != null) {
                        val item = JMenuItem("Send to MCP Active Buffer")
                        item.addActionListener {
                            onEdt {
                                val req = reqResp.request()
                                val reqBytes = runCatching { req.toByteArray().getBytes() }.getOrDefault(byteArrayOf())
                                val target = StagedBufferTarget(
                                    text = req?.toString().orEmpty(),
                                    payload = reqBytes,
                                    host = reqResp.httpService()?.host().orEmpty(),
                                    port = reqResp.httpService()?.port()?.toInt() ?: 80,
                                    https = reqResp.httpService()?.secure() ?: false,
                                    toolSource = runCatching { event.toolType().name.lowercase() }.getOrDefault("context_menu"),
                                    selectedText = "",
                                )
                                stagedBuffer = target
                                lastActiveEditor = WeakReference(target)
                            }
                        }
                        menuList.add(item)
                    }
                    return menuList
                }
            }
        )
        registrations.add(menuReg)
    }

    fun capture(targetHint: String? = null, ttlSeconds: Long? = null): UnifiedEditorSnapshot = onEdt {
        pruneExpired()
        val target = resolveTarget(targetHint)
            ?: throw NoSuchElementException("No active editor found. Please focus a Burp text editor or use context menu 'Send to MCP Active Buffer'.")

        val token = UUID.randomUUID().toString()
        val ttl = (ttlSeconds?.times(1000L)) ?: defaultTtlMillis
        val expiresAt = clock.millis() + ttl
        val snapshot = createSnapshot(target, token, expiresAt)

        leases[token] = ActiveLease(target, expiresAt, snapshot.sha256)
        lastActiveEditor = WeakReference(target)
        snapshot
    }

    fun patch(token: String, expectedSha256: String, op: PatchOp): UnifiedEditorSnapshot = onEdt {
        pruneExpired()
        val lease = leases[token]
            ?: throw NoSuchElementException("Editor lease token expired or not found")

        val target = lease.target
        check(target.isAlive()) { "Target editor is no longer attached or available" }
        check(target.isEditable()) { "Target editor is read only" }

        val currentSha = target.currentSha256()
        check(currentSha.equals(expectedSha256, ignoreCase = true)) {
            "Editor contents changed concurrently (expected: $expectedSha256, actual: $currentSha)"
        }

        // Apply surgical patch
        target.applyPatch(op)

        // Non-destructive update: update last known hash and return fresh snapshot
        val updatedSnapshot = createSnapshot(target, token, lease.expiresAtMillis)
        lease.lastKnownSha256 = updatedSnapshot.sha256
        updatedSnapshot
    }

    fun renew(token: String, extendSeconds: Long): Long = onEdt {
        pruneExpired()
        val lease = leases[token]
            ?: throw NoSuchElementException("Editor lease token expired or not found")
        lease.expiresAtMillis += (extendSeconds * 1000L)
        lease.expiresAtMillis
    }


    private fun resolveTarget(hint: String?): EditorTarget? {
        val normHint = hint?.trim()?.lowercase()

        // Tier 1: Focus Owner
        val focusOwner = KeyboardFocusManager.getCurrentKeyboardFocusManager().permanentFocusOwner
        if (focusOwner != null) {
            // Check extension-provided editors
            for (ref in registeredHttpReqEditors) {
                val ed = ref.get() ?: continue
                if (isDescendantOf(focusOwner, ed.uiComponent())) return ed
            }
            for (ref in registeredHttpRespEditors) {
                val ed = ref.get() ?: continue
                if (isDescendantOf(focusOwner, ed.uiComponent())) return ed
            }
            for (ref in registeredWsEditors) {
                val ed = ref.get() ?: continue
                if (isDescendantOf(focusOwner, ed.uiComponent())) return ed
            }
            // Check generic Swing JTextArea
            if (focusOwner is JTextArea && isInsideSuite(focusOwner)) {
                return SwingTextAreaTarget(focusOwner)
            }
        }

        // Tier 2: Hint Lookup
        if (!normHint.isNullOrBlank()) {
            if (normHint == "websocket" || normHint.contains("ws")) {
                registeredWsEditors.asSequence().mapNotNull { it.get() }.firstOrNull { it.isAlive() }?.let { return it }
            }
            if (normHint == "request" || normHint.contains("req")) {
                registeredHttpReqEditors.asSequence().mapNotNull { it.get() }.firstOrNull { it.isAlive() }?.let { return it }
            }
            if (normHint == "response" || normHint.contains("resp")) {
                registeredHttpRespEditors.asSequence().mapNotNull { it.get() }.firstOrNull { it.isAlive() }?.let { return it }
            }
        }

        // Tier 3: Last-Active Editor
        lastActiveEditor?.get()?.takeIf { it.isAlive() }?.let { return it }

        // Tier 4: Staged Buffer
        stagedBuffer?.takeIf { it.isAlive() }?.let { return it }

        return null
    }

    private fun createSnapshot(target: EditorTarget, token: String, expiresAtMillis: Long): UnifiedEditorSnapshot {
        val payload = target.currentPayload()
        val text = target.currentText()
        val isJson = text.trimStart().startsWith("{") || text.trimStart().startsWith("[")
        return UnifiedEditorSnapshot(
            token = token,
            kind = target.kind(),
            toolSource = target.toolSource(),
            tabName = target.tabName(),
            host = target.host(),
            port = target.port(),
            https = target.https(),
            text = text,
            payload = payload,
            isJson = isJson,
            editable = target.isEditable(),
            sha256 = target.currentSha256(),
            caretPosition = target.caretPosition(),
            selectionStart = target.selectionStart(),
            selectionEnd = target.selectionEnd(),
            selectedText = target.selectedText(),
            expiresAtMillis = expiresAtMillis,
        )
    }

    private fun isDescendantOf(child: Component, parent: Component): Boolean {
        var current: Component? = child
        while (current != null) {
            if (current === parent) return true
            current = current.parent
        }
        return false
    }

    private fun isInsideSuite(component: Component): Boolean {
        val root = suiteRoot()
        return isDescendantOf(component, root)
    }

    private fun pruneExpired() {
        val now = clock.millis()
        leases.entries.removeIf { (_, lease) -> lease.expiresAtMillis <= now }
    }

    private fun <T> onEdt(block: () -> T): T {
        if (SwingUtilities.isEventDispatchThread()) return block()
        var result: Result<T>? = null
        SwingUtilities.invokeAndWait { result = runCatching(block) }
        return checkNotNull(result).getOrThrow()
    }

    override fun close() {
        registrations.forEach { it.deregister() }
        leases.clear()
    }

    // --- Editor Target Implementations ---

    private inner class SwingTextAreaTarget(
        private val textArea: JTextArea,
    ) : EditorTarget {
        override fun isAlive(): Boolean = isInsideSuite(textArea)
        override fun isEditable(): Boolean = textArea.isEditable && textArea.isEnabled
        override fun currentText(): String = textArea.text.orEmpty()
        override fun currentPayload(): ByteArray = currentText().toByteArray(StandardCharsets.UTF_8)
        override fun currentSha256(): String = sha256Hex(currentPayload())
        override fun kind(): EditorKind = EditorKind.RAW_TEXT
        override fun toolSource(): String = "swing_active"
        override fun tabName(): String = ""
        override fun host(): String = ""
        override fun port(): Int = 0
        override fun https(): Boolean = false
        override fun caretPosition(): Int = textArea.caretPosition
        override fun selectionStart(): Int = textArea.selectionStart
        override fun selectionEnd(): Int = textArea.selectionEnd
        override fun selectedText(): String = textArea.selectedText.orEmpty()
        override fun applyPatch(op: PatchOp) {
            val patched = EditorPatchEngine.applyPatchToText(currentText(), op)
            textArea.text = patched
        }
    }

    private inner class StagedBufferTarget(
        private var text: String,
        private var payload: ByteArray,
        private val host: String,
        private val port: Int,
        private val https: Boolean,
        private val toolSource: String,
        private val selectedText: String,
    ) : EditorTarget {
        override fun isAlive(): Boolean = true
        override fun isEditable(): Boolean = true
        override fun currentText(): String = text
        override fun currentPayload(): ByteArray = payload
        override fun currentSha256(): String = sha256Hex(payload)
        override fun kind(): EditorKind = EditorKind.HTTP_REQUEST
        override fun toolSource(): String = toolSource
        override fun tabName(): String = "Staged Buffer"
        override fun host(): String = host
        override fun port(): Int = port
        override fun https(): Boolean = https
        override fun caretPosition(): Int = 0
        override fun selectionStart(): Int = 0
        override fun selectionEnd(): Int = 0
        override fun selectedText(): String = selectedText
        override fun applyPatch(op: PatchOp) {
            text = EditorPatchEngine.applyPatchToText(text, op)
            payload = text.toByteArray(StandardCharsets.UTF_8)
        }
    }

    private inner class McpHttpReqEditor(
        private val api: MontoyaApi,
        private val context: EditorCreationContext,
    ) : ExtensionProvidedHttpRequestEditor, EditorTarget {
        private val rawEditor: RawEditor = if (context.editorMode() == EditorMode.READ_ONLY) {
            api.userInterface().createRawEditor(EditorOptions.READ_ONLY)
        } else {
            api.userInterface().createRawEditor()
        }
        private var currentReqResp: HttpRequestResponse? = null
        private var changedByMcp = false

        override fun uiComponent(): Component = rawEditor.uiComponent()
        override fun caption(): String = "MCP"
        override fun isEnabledFor(requestResponse: HttpRequestResponse): Boolean = true
        override fun setRequestResponse(requestResponse: HttpRequestResponse) {
            currentReqResp = requestResponse
            val req = requestResponse.request()
            if (req != null) {
                rawEditor.setContents(req.toByteArray())
            }
            changedByMcp = false
        }
        override fun getRequest(): HttpRequest =
            HttpRequest.httpRequest(currentReqResp?.httpService(), rawEditor.getContents())

        override fun isModified(): Boolean = changedByMcp || rawEditor.isModified()
        override fun selectedData(): Selection? = rawEditor.selection().orElse(null)

        override fun isAlive(): Boolean = isInsideSuite(rawEditor.uiComponent())
        override fun isEditable(): Boolean = context.editorMode() != EditorMode.READ_ONLY
        override fun currentText(): String = rawEditor.getContents().toString()
        override fun currentPayload(): ByteArray = rawEditor.getContents().getBytes()
        override fun currentSha256(): String = sha256Hex(currentPayload())
        override fun kind(): EditorKind = EditorKind.HTTP_REQUEST
        override fun toolSource(): String = context.toolSource().toolType().name.lowercase()
        override fun tabName(): String = "MCP"
        override fun host(): String = currentReqResp?.httpService()?.host().orEmpty()
        override fun port(): Int = currentReqResp?.httpService()?.port()?.toInt() ?: 0
        override fun https(): Boolean = currentReqResp?.httpService()?.secure() ?: false
        override fun caretPosition(): Int = rawEditor.caretPosition()
        override fun selectionStart(): Int = rawEditor.selection().orElse(null)?.offsets()?.startIndexInclusive() ?: 0
        override fun selectionEnd(): Int = rawEditor.selection().orElse(null)?.offsets()?.endIndexExclusive() ?: 0
        override fun selectedText(): String = rawEditor.selection().orElse(null)?.offsets()?.let {
            currentText().substring(it.startIndexInclusive().coerceIn(0, currentText().length), it.endIndexExclusive().coerceIn(0, currentText().length))
        }.orEmpty()
        override fun applyPatch(op: PatchOp) {
            val patched = EditorPatchEngine.applyPatchToText(currentText(), op)
            rawEditor.setContents(MontoyaByteArray.byteArray(patched))
            changedByMcp = true
        }
    }

    private inner class McpHttpRespEditor(
        private val api: MontoyaApi,
        private val context: EditorCreationContext,
    ) : ExtensionProvidedHttpResponseEditor, EditorTarget {
        private val rawEditor: RawEditor = if (context.editorMode() == EditorMode.READ_ONLY) {
            api.userInterface().createRawEditor(EditorOptions.READ_ONLY)
        } else {
            api.userInterface().createRawEditor()
        }
        private var currentReqResp: HttpRequestResponse? = null
        private var changedByMcp = false

        override fun uiComponent(): Component = rawEditor.uiComponent()
        override fun caption(): String = "MCP"
        override fun isEnabledFor(requestResponse: HttpRequestResponse): Boolean = true
        override fun setRequestResponse(requestResponse: HttpRequestResponse) {
            currentReqResp = requestResponse
            val resp = requestResponse.response()
            if (resp != null) {
                rawEditor.setContents(resp.toByteArray())
            }
            changedByMcp = false
        }
        override fun getResponse(): HttpResponse =
            HttpResponse.httpResponse(rawEditor.getContents())

        override fun isModified(): Boolean = changedByMcp || rawEditor.isModified()
        override fun selectedData(): Selection? = rawEditor.selection().orElse(null)

        override fun isAlive(): Boolean = isInsideSuite(rawEditor.uiComponent())
        override fun isEditable(): Boolean = context.editorMode() != EditorMode.READ_ONLY
        override fun currentText(): String = rawEditor.getContents().toString()
        override fun currentPayload(): ByteArray = rawEditor.getContents().getBytes()
        override fun currentSha256(): String = sha256Hex(currentPayload())
        override fun kind(): EditorKind = EditorKind.HTTP_RESPONSE
        override fun toolSource(): String = context.toolSource().toolType().name.lowercase()
        override fun tabName(): String = "MCP"
        override fun host(): String = currentReqResp?.httpService()?.host().orEmpty()
        override fun port(): Int = currentReqResp?.httpService()?.port()?.toInt() ?: 0
        override fun https(): Boolean = currentReqResp?.httpService()?.secure() ?: false
        override fun caretPosition(): Int = rawEditor.caretPosition()
        override fun selectionStart(): Int = rawEditor.selection().orElse(null)?.offsets()?.startIndexInclusive() ?: 0
        override fun selectionEnd(): Int = rawEditor.selection().orElse(null)?.offsets()?.endIndexExclusive() ?: 0
        override fun selectedText(): String = rawEditor.selection().orElse(null)?.offsets()?.let {
            currentText().substring(it.startIndexInclusive().coerceIn(0, currentText().length), it.endIndexExclusive().coerceIn(0, currentText().length))
        }.orEmpty()
        override fun applyPatch(op: PatchOp) {
            val patched = EditorPatchEngine.applyPatchToText(currentText(), op)
            rawEditor.setContents(MontoyaByteArray.byteArray(patched))
            changedByMcp = true
        }
    }

    private inner class McpWebSocketEditor(
        private val api: MontoyaApi,
        private val context: EditorCreationContext,
    ) : ExtensionProvidedWebSocketMessageEditor, EditorTarget {
        private val rawEditor: RawEditor = if (context.editorMode() == EditorMode.READ_ONLY) {
            api.userInterface().createRawEditor(EditorOptions.READ_ONLY)
        } else {
            api.userInterface().createRawEditor()
        }
        private var changedByMcp = false

        override fun uiComponent(): Component = rawEditor.uiComponent()
        override fun caption(): String = "MCP"
        override fun isEnabledFor(message: WebSocketMessage): Boolean = true
        override fun setMessage(message: WebSocketMessage) {
            rawEditor.setContents(message.payload())
            changedByMcp = false
        }
        override fun getMessage(): MontoyaByteArray = rawEditor.getContents()
        override fun isModified(): Boolean = changedByMcp || rawEditor.isModified()
        override fun selectedData(): Selection? = rawEditor.selection().orElse(null)

        override fun isAlive(): Boolean = isInsideSuite(rawEditor.uiComponent())
        override fun isEditable(): Boolean = context.editorMode() != EditorMode.READ_ONLY
        override fun currentText(): String = rawEditor.getContents().toString()
        override fun currentPayload(): ByteArray = rawEditor.getContents().getBytes()
        override fun currentSha256(): String = sha256Hex(currentPayload())
        override fun kind(): EditorKind = EditorKind.WEBSOCKET
        override fun toolSource(): String = context.toolSource().toolType().name.lowercase()
        override fun tabName(): String = "MCP"
        override fun host(): String = ""
        override fun port(): Int = 0
        override fun https(): Boolean = false
        override fun caretPosition(): Int = rawEditor.caretPosition()
        override fun selectionStart(): Int = rawEditor.selection().orElse(null)?.offsets()?.startIndexInclusive() ?: 0
        override fun selectionEnd(): Int = rawEditor.selection().orElse(null)?.offsets()?.endIndexExclusive() ?: 0
        override fun selectedText(): String = rawEditor.selection().orElse(null)?.offsets()?.let {
            currentText().substring(it.startIndexInclusive().coerceIn(0, currentText().length), it.endIndexExclusive().coerceIn(0, currentText().length))
        }.orEmpty()
        override fun applyPatch(op: PatchOp) {
            val patched = EditorPatchEngine.applyPatchToPayload(currentPayload(), op)
            rawEditor.setContents(MontoyaByteArray.byteArray(*patched))
            changedByMcp = true
        }
    }

    private companion object {
        fun sha256Hex(bytes: ByteArray): String =
            MessageDigest.getInstance("SHA-256").digest(bytes).joinToString("") { "%02x".format(it) }
    }
}
