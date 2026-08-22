package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi

internal class ConfigFacade(
    private val api: MontoyaApi,
) {
    fun export(paths: List<String>): String =
        api.burpSuite().exportProjectOptionsAsJson(*paths.toTypedArray())

    fun import(config: String) {
        require(config.isNotBlank()) { "config must not be blank" }
        api.burpSuite().importProjectOptionsFromJson(config)
    }
}
