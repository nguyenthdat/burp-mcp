package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.collaborator.CollaboratorClient
import java.time.Instant
import java.util.concurrent.ConcurrentHashMap

internal data class CollaboratorInteraction(
    val id: String,
    val type: String,
    val clientIp: String,
    val clientPort: Int,
    val timestamp: String,
    val targetUrl: String? = null,
    val injectionPoint: String? = null,
    val payload: String? = null,
)

internal data class CollaboratorCorrelation(
    val payload: String,
    val targetUrl: String,
    val injectionPoint: String,
    val createdAt: String,
)

internal class CollaboratorFacade(
    private val api: MontoyaApi,
) {
    private var client: CollaboratorClient? = null
    private val correlationMap = ConcurrentHashMap<String, CollaboratorCorrelation>()

    @Synchronized
    fun generate(count: Int, targetUrl: String? = null, injectionPoint: String? = null): List<String> {
        require(count in 1..100) { "count must be between 1 and 100" }
        val current = client ?: api.collaborator().createClient().also { client = it }
        val payloads = List(count) { current.generatePayload().toString() }
        
        if (!targetUrl.isNullOrBlank() || !injectionPoint.isNullOrBlank()) {
            val now = Instant.now().toString()
            val url = targetUrl.orEmpty()
            val point = injectionPoint.orEmpty()
            for (p in payloads) {
                val subdomain = p.split('.').firstOrNull()?.lowercase() ?: p.lowercase()
                val metadata = CollaboratorCorrelation(p, url, point, now)
                correlationMap[subdomain] = metadata
                correlationMap[p.lowercase()] = metadata
            }
        }
        return payloads
    }

    @Synchronized
    fun interactions(): List<CollaboratorInteraction> {
        val current = client ?: return emptyList()
        return current.allInteractions.map { interaction ->
            val interactionId = interaction.id().toString().lowercase()
            val correlation = correlationMap[interactionId]
                ?: correlationMap.entries.firstOrNull { (k, _) -> interactionId.contains(k) }?.value

            CollaboratorInteraction(
                id = interaction.id().toString(),
                type = interaction.type().name,
                clientIp = interaction.clientIp().hostAddress,
                clientPort = interaction.clientPort(),
                timestamp = interaction.timeStamp().toString(),
                targetUrl = correlation?.targetUrl,
                injectionPoint = correlation?.injectionPoint,
                payload = correlation?.payload,
            )
        }
    }
}
