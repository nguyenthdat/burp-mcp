package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi

internal data class CookieQuery(
    val domain: String? = null,
    val limit: Int = 100,
)

internal data class CookieItem(
    val name: String,
    val value: String,
    val domain: String?,
    val path: String?,
    val expiration: String?,
)

internal class CookieFacade(
    private val api: MontoyaApi,
) {
    fun cookies(query: CookieQuery): List<CookieItem> {
        require(query.limit in 0..500) { "limit must be between 0 and 500" }
        return api
            .http()
            .cookieJar()
            .cookies()
            .asSequence()
            .filter { cookie ->
                query.domain == null || cookie.domain()?.contains(query.domain, ignoreCase = true) == true
            }.take(query.limit)
            .map { cookie ->
                CookieItem(
                    name = cookie.name(),
                    value = cookie.value(),
                    domain = cookie.domain(),
                    path = cookie.path(),
                    expiration = cookie.expiration().map { it.toString() }.orElse(null),
                )
            }.toList()
    }

    fun setCookie(
        name: String,
        value: String,
        domain: String,
        path: String,
        expiration: String?,
    ) {
        require(name.isNotBlank()) { "cookie name must not be blank" }
        require(domain.isNotBlank()) { "cookie domain must not be blank" }
        require(path.startsWith('/')) { "cookie path must start with '/'" }
        val parsedExpiration = expiration?.takeIf(String::isNotBlank)?.let(java.time.ZonedDateTime::parse)
        api.http().cookieJar().setCookie(name, value, path, domain, parsedExpiration)
    }
}
