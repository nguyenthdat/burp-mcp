package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import java.awt.Component
import java.awt.KeyboardFocusManager
import java.lang.ref.WeakReference
import java.security.MessageDigest
import java.time.Clock
import java.util.UUID
import javax.swing.SwingUtilities
import javax.swing.JTextArea

internal data class ActiveEditorSnapshot(
    val token: String,
    val text: String,
    val editable: Boolean,
    val sha256: String,
    val caretPosition: Int,
    val selectionStart: Int,
    val selectionEnd: Int,
)

internal class ActiveEditorFacade(
    private val api: MontoyaApi,
    private val clock: Clock = Clock.systemUTC(),
    private val tokenFactory: () -> String = { UUID.randomUUID().toString() },
    private val focusOwner: () -> Component? = {
        KeyboardFocusManager.getCurrentKeyboardFocusManager().permanentFocusOwner
    },
    private val suiteRoot: () -> Component = {
        api.userInterface().swingUtils().suiteFrame()
    },
) : AutoCloseable {
    private data class Lease(
        val editor: WeakReference<JTextArea>,
        val expiresAtMillis: Long,
    )

    private val leases = LinkedHashMap<String, Lease>()

    fun capture(): ActiveEditorSnapshot = onEdt {
        pruneExpired()
        val editor = activeEditor() ?: throw NoSuchElementException("no active Burp text editor")
        val token = tokenFactory()
        leases[token] = Lease(WeakReference(editor), clock.millis() + TOKEN_TTL_MILLIS)
        editor.snapshot(token)
    }

    fun replace(token: String, expectedSha256: String, text: String): ActiveEditorSnapshot = onEdt {
        require(token.isNotBlank()) { "token is required" }
        require(expectedSha256.matches(SHA256_REGEX)) { "expected_sha256 must be a lowercase SHA-256 digest" }
        require(text.toByteArray(Charsets.UTF_8).size <= MAX_EDITOR_TEXT_BYTES) {
            "text exceeds $MAX_EDITOR_TEXT_BYTES UTF-8 bytes"
        }
        pruneExpired()
        val lease = leases.remove(token) ?: throw NoSuchElementException("active editor token was not found or expired")
        val editor = lease.editor.get() ?: throw NoSuchElementException("active editor is no longer available")
        check(isInsideSuite(editor)) { "active editor is no longer attached to Burp" }
        check(editor.isEditable && editor.isEnabled) { "active editor is not editable" }
        check(sha256(editor.text) == expectedSha256) { "active editor contents changed after capture" }
        editor.text = text
        editor.snapshot(token)
    }

    override fun close() = onEdt { leases.clear() }

    private fun activeEditor(): JTextArea? {
        val editor = focusOwner() as? JTextArea ?: return null
        return editor.takeIf(::isInsideSuite)
    }

    private fun isInsideSuite(component: Component): Boolean {
        val root = suiteRoot()
        return generateSequence(component) { it.parent }.any { it === root }
    }

    private fun pruneExpired() {
        val now = clock.millis()
        leases.entries.removeIf { (_, lease) -> lease.expiresAtMillis <= now || lease.editor.get() == null }
        while (leases.size >= MAX_ACTIVE_LEASES) {
            leases.entries.iterator().run {
                next()
                remove()
            }
        }
    }
    private fun JTextArea.snapshot(token: String): ActiveEditorSnapshot =
        ActiveEditorSnapshot(
            token = token,
            text = text,
            editable = isEditable && isEnabled,
            sha256 = sha256(text),
            caretPosition = caretPosition,
            selectionStart = selectionStart,
            selectionEnd = selectionEnd,
        )

    private fun <T> onEdt(block: () -> T): T {
        if (SwingUtilities.isEventDispatchThread()) return block()
        var result: Result<T>? = null
        SwingUtilities.invokeAndWait { result = runCatching(block) }
        return checkNotNull(result).getOrThrow()
    }

    private companion object {
        const val TOKEN_TTL_MILLIS = 30_000L
        const val MAX_ACTIVE_LEASES = 32
        const val MAX_EDITOR_TEXT_BYTES = 16 * 1024 * 1024
        val SHA256_REGEX = Regex("[0-9a-f]{64}")

        fun sha256(value: String): String =
            MessageDigest.getInstance("SHA-256")
                .digest(value.toByteArray(Charsets.UTF_8))
                .joinToString("") { byte -> "%02x".format(byte.toInt() and 0xff) }
    }
}
