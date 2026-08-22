package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.sitemap.SiteMapFilter
import java.util.Locale

internal data class ScopeResult(val url: String, val inScope: Boolean)
internal data class TargetResult(val hosts: Set<String>, val technologies: Set<String>, val requestsSampled: Int)

internal class TargetFacade(
    private val api: MontoyaApi,
) {
    fun scope(url: String): ScopeResult = ScopeResult(url, api.scope().isInScope(url))

    fun include(url: String) {
        api.scope().includeInScope(url)
    }

    fun exclude(url: String) {
        api.scope().excludeFromScope(url)
    }

    fun info(urlPrefix: String, limit: Int = 500): TargetResult {
        require(limit >= 0) { "limit must be non-negative" }
        val hosts = linkedSetOf<String>()
        val technologies = linkedSetOf<String>()
        var sampled = 0
        for (entry in api.siteMap().requestResponses(SiteMapFilter.prefixFilter(urlPrefix))) {
            if (sampled >= limit) break
            hosts += entry.request().httpService().host()
            entry.response()?.headers()?.forEach { header ->
                val name = header.name().lowercase(Locale.ROOT)
                if (name == "server" || name == "x-powered-by" || name == "x-aspnet-version") {
                    technologies += "${header.name()}: ${header.value()}"
                }
            }
            sampled++
        }
        return TargetResult(hosts, technologies, sampled)
    }
}
