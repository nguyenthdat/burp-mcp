package io.github.nguyenthdat.burpmcp

import burp.api.montoya.BurpExtension
import burp.api.montoya.MontoyaApi

class BurpMcpExtension : BurpExtension {
    private var transports: TransportLifecycle? = null
    private var settingsPanel: GrpcSettingsPanel? = null

    override fun initialize(api: MontoyaApi) {
        val logging = api.logging()
        api.extension().setName("Burp MCP")

        val store = GrpcSettingsStore(api.persistence().preferences())
        val initialSettings = store.load()
        transports = TransportLifecycle(api, logging).also { it.start(initialSettings) }
        settingsPanel = GrpcSettingsPanel(api, store, requireNotNull(transports))
        api.extension().registerUnloadingHandler {
            settingsPanel?.close()
            settingsPanel = null
            transports?.close()
            transports = null
        }
    }
}
