package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.collaborator.CollaboratorClient

internal data class CollaboratorInteraction(
    val id: String,
    val type: String,
    val clientIp: String,
    val clientPort: Int,
    val timestamp: String,
)

internal class CollaboratorFacade(
    private val api: MontoyaApi,
) {
    private var client: CollaboratorClient? = null

    @Synchronized
    fun generate(count: Int): List<String> {
        require(count in 1..100) { "count must be between 1 and 100" }
        val current = client ?: api.collaborator().createClient().also { client = it }
        return List(count) { current.generatePayload().toString() }
    }

    @Synchronized
    fun interactions(): List<CollaboratorInteraction> {
        val current = checkNotNull(client) { "generate at least one Collaborator payload first" }
        return current.allInteractions.map { interaction ->
            CollaboratorInteraction(
                interaction.id().toString(),
                interaction.type().name,
                interaction.clientIp().hostAddress,
                interaction.clientPort(),
                interaction.timeStamp().toString(),
            )
        }
    }
}
