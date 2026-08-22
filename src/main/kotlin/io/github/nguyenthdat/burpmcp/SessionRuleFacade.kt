package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.core.Registration
import burp.api.montoya.http.sessions.ActionResult
import burp.api.montoya.http.sessions.SessionHandlingAction
import burp.api.montoya.http.sessions.SessionHandlingActionData

internal data class SessionRule(
    val find: String,
    val replacement: String,
)

internal class SessionRuleFacade(
    private val api: MontoyaApi,
) {
    private var active: Pair<SessionRule, Registration>? = null

    @Synchronized
    fun create(rule: SessionRule) {
        require(rule.find.isNotEmpty()) { "find must not be empty" }
        active?.second?.deregister()
        val registration =
            api.http().registerSessionHandlingAction(
                object : SessionHandlingAction {
                    override fun name(): String = "Burp MCP replace session token"

                    override fun performAction(actionData: SessionHandlingActionData): ActionResult {
                        val request = actionData.request()
                        val updated = request.withBody(request.bodyToString().replace(rule.find, rule.replacement))
                        return ActionResult.actionResult(updated, actionData.annotations())
                    }
                },
            )
        active = rule to registration
    }

    @Synchronized
    fun list(): List<SessionRule> = active?.let { listOf(it.first) } ?: emptyList()

    @Synchronized
    fun remove() {
        active?.second?.deregister()
        active = null
    }
}
