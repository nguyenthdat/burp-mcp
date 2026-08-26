package io.github.nguyenthdat.burpmcp.rpc

import burp.api.montoya.MontoyaApi
import io.github.nguyenthdat.burpmcp.BurpCapabilityFacade
import io.github.nguyenthdat.burpmcp.ConfigFacade
import io.github.nguyenthdat.burpmcp.CookieFacade
import io.github.nguyenthdat.burpmcp.HttpFacade
import io.github.nguyenthdat.burpmcp.HttpHandlerFacade
import io.github.nguyenthdat.burpmcp.IntruderPayloadFacade
import io.github.nguyenthdat.burpmcp.JobFacade
import io.github.nguyenthdat.burpmcp.EventFacade
import io.github.nguyenthdat.burpmcp.LongOperationFacade
import io.github.nguyenthdat.burpmcp.MacroFacade
import io.github.nguyenthdat.burpmcp.PayloadListFacade
import io.github.nguyenthdat.burpmcp.ProxyFacade
import io.github.nguyenthdat.burpmcp.ProxyInterceptConfigFacade
import io.github.nguyenthdat.burpmcp.ProxyRuleFacade
import io.github.nguyenthdat.burpmcp.ProxyInterceptController
import io.github.nguyenthdat.burpmcp.ProxySettingsFacade
import io.github.nguyenthdat.burpmcp.ScannerFacade
import io.github.nguyenthdat.burpmcp.ScanCatalogFacade
import io.github.nguyenthdat.burpmcp.ScriptImportFacade
import io.github.nguyenthdat.burpmcp.SessionRuleFacade
import io.github.nguyenthdat.burpmcp.SitemapFacade
import io.github.nguyenthdat.burpmcp.TargetFacade
import io.github.nguyenthdat.burpmcp.AnnotationFacade
import io.github.nguyenthdat.burpmcp.CollaboratorFacade
import io.github.nguyenthdat.burpmcp.WebSocketFacade

internal class BurpServiceResources(api: MontoyaApi) : AutoCloseable {
    val proxy = ProxyFacade(api)
    val sitemap = SitemapFacade(api)
    val target = TargetFacade(api)
    val scanner = ScannerFacade(api)
    val scanCatalog = ScanCatalogFacade(api)
    val cookies = CookieFacade(api)
    val http = HttpFacade(api)
    val annotations = AnnotationFacade(api)
    val collaborator = CollaboratorFacade(api)
    val scripts = ScriptImportFacade(api)
    val webSockets = WebSocketFacade(api)
    val config = ConfigFacade(api)
    val httpHandlers = HttpHandlerFacade(api)
    val proxyRules = ProxyRuleFacade(api)
    val proxySettings = ProxySettingsFacade(api)
    val proxyIntercept = ProxyInterceptConfigFacade(api)
    val interceptController = ProxyInterceptController(api)
    val macros = MacroFacade(api)
    val sessionRules = SessionRuleFacade(api) { description -> macros.run(description) }
    val payloadLists = PayloadListFacade()
    val jobs = JobFacade()
    val longOperations = LongOperationFacade(api, jobs)
    val capabilities = BurpCapabilityFacade(api)
    val intruderPayloads = IntruderPayloadFacade(api)
    val events = EventFacade(api)

    override fun close() {
        events.close()
        jobs.close()
        httpHandlers.clear()
        interceptController.close()
        proxyRules.close()
        sessionRules.removeAll()
        webSockets.close()
        intruderPayloads.close()
    }
}
