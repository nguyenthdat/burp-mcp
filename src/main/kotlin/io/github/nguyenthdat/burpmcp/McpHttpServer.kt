package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.collaborator.CollaboratorClient
import burp.api.montoya.core.ByteArray
import burp.api.montoya.core.Registration
import burp.api.montoya.http.HttpService
import burp.api.montoya.http.handler.HttpHandler
import burp.api.montoya.http.handler.HttpRequestToBeSent
import burp.api.montoya.http.handler.HttpResponseReceived
import burp.api.montoya.http.handler.RequestToBeSentAction
import burp.api.montoya.http.handler.ResponseReceivedAction
import burp.api.montoya.http.message.HttpRequestResponse
import burp.api.montoya.http.message.requests.HttpRequest
import burp.api.montoya.http.message.responses.HttpResponse
import burp.api.montoya.http.sessions.ActionResult
import burp.api.montoya.http.sessions.SessionHandlingAction
import burp.api.montoya.http.sessions.SessionHandlingActionData
import burp.api.montoya.proxy.ProxyHttpRequestResponse
import burp.api.montoya.proxy.http.InterceptedRequest
import burp.api.montoya.proxy.http.ProxyRequestHandler
import burp.api.montoya.proxy.http.ProxyRequestReceivedAction
import burp.api.montoya.proxy.http.ProxyRequestToBeSentAction
import burp.api.montoya.scanner.AuditConfiguration
import burp.api.montoya.scanner.BuiltInAuditConfiguration
import burp.api.montoya.scanner.Crawl
import burp.api.montoya.scanner.CrawlConfiguration
import burp.api.montoya.scanner.audit.Audit
import burp.api.montoya.scanner.audit.issues.AuditIssue
import burp.api.montoya.scanner.audit.issues.AuditIssueConfidence
import burp.api.montoya.scanner.audit.issues.AuditIssueSeverity
import burp.api.montoya.sitemap.SiteMapFilter
import burp.api.montoya.websocket.extension.ExtensionWebSocket
import com.google.gson.Gson
import com.google.gson.GsonBuilder
import com.google.gson.JsonArray
import com.google.gson.JsonElement
import com.google.gson.JsonObject
import com.google.gson.JsonParser
import fi.iki.elonen.NanoHTTPD
import java.io.IOException
import java.net.URI
import java.nio.charset.Charset
import java.nio.charset.StandardCharsets
import java.nio.file.AtomicMoveNotSupportedException
import java.nio.file.FileSystems
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.Paths
import java.nio.file.StandardCopyOption
import java.nio.file.StandardOpenOption
import java.nio.file.attribute.PosixFilePermission
import java.nio.file.attribute.PosixFilePermissions
import java.security.SecureRandom
import java.time.ZonedDateTime
import java.util.Base64
import java.util.Collections
import java.util.EnumSet
import java.util.HexFormat
import java.util.Locale
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.ConcurrentLinkedQueue
import java.util.concurrent.ExecutorService
import java.util.concurrent.Executors
import java.util.concurrent.Future
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicInteger
import java.util.regex.Pattern
import kotlin.math.ln
import kotlin.math.max
import kotlin.math.min

class McpHttpServer(
    private val api: MontoyaApi,
    port: Int,
) : NanoHTTPD("127.0.0.1", port) {
    private val gson: Gson = GsonBuilder().setPrettyPrinting().create()
    private val authToken: String = resolveAuthToken()
    private var collaborator: CollaboratorClient? = null

    @Volatile
    private var activeAudit: Audit? = null

    @Volatile
    private var activeCrawl: Crawl? = null

    private val toolRegistry: ToolRegistry by lazy {
        ToolRegistry(
            listOf(
                RegisteredTool("proxy_history", handler = ::proxyHistory),
                RegisteredTool("proxy_detail", handler = ::proxyDetail),
                RegisteredTool("proxy_websocket", handler = ::proxyWebSocket),
                RegisteredTool("proxy_clear", handler = ::proxyClear),
                RegisteredTool("proxy_history_filtered", handler = ::proxyHistoryFiltered),
                RegisteredTool("send_request", handler = ::sendRequest),
                RegisteredTool("send_to_repeater", handler = ::sendToRepeater),
                RegisteredTool("repeater_send", handler = ::repeaterSend),
                RegisteredTool("repeater_modify_send", handler = ::repeaterModifySend),
                RegisteredTool("send_to_intruder", handler = ::sendToIntruder),
                RegisteredTool("intruder_attack", handler = ::intruderAttack),
                RegisteredTool("intruder_attack_async", handler = ::intruderAttackAsync),
                RegisteredTool("intruder_attack_wordlist", handler = ::intruderAttackWordlist),
                RegisteredTool("intruder_pitchfork", handler = ::intruderPitchfork),
                RegisteredTool("intruder_cluster_bomb", handler = ::intruderClusterBomb),
                RegisteredTool("intruder_battering_ram", handler = ::intruderBatteringRam),
                RegisteredTool("intruder_with_options", handler = ::intruderWithOptions),
                RegisteredTool("sitemap", handler = ::getSitemap),
                RegisteredTool("target_info", handler = ::targetInfo),
                RegisteredTool("intercept_toggle", handler = ::interceptToggle),
                RegisteredTool("encode", handler = ::encode),
                RegisteredTool("decode", handler = ::decode),
                RegisteredTool("convert_request", handler = ::convertRequest),
                RegisteredTool("export_request", handler = ::exportRequest),
                RegisteredTool("generate_csrf_poc", handler = ::generateCsrfPoc),
                RegisteredTool("extract_from_response", handler = ::extractFromResponse),
                RegisteredTool("payload_process", handler = ::payloadProcess),
                RegisteredTool("scan", handler = ::scan),
                RegisteredTool("scan_active", handler = ::scanActive),
                RegisteredTool("bambda_import", handler = ::bambdaImport),
                RegisteredTool("bcheck_import", handler = ::bcheckImport),
                RegisteredTool("scan_results", handler = ::scanResults),
                RegisteredTool("scan_issue_detail", handler = ::scanIssueDetail),
                RegisteredTool("crawl", handler = ::crawl),
                RegisteredTool("get_scope", handler = ::getScope),
                RegisteredTool("add_to_scope", handler = ::addToScope),
                RegisteredTool("remove_from_scope", handler = ::removeFromScope),
                RegisteredTool("collaborator_generate", handler = ::collaboratorGenerate),
                RegisteredTool("collaborator_poll", handler = ::collaboratorPoll),
                RegisteredTool("search_history", handler = ::searchHistory),
                RegisteredTool("highlight", handler = ::highlightItem),
                RegisteredTool("annotate", handler = ::annotate),
                RegisteredTool("compare", handler = ::compare),
                RegisteredTool("export_config", handler = ::exportConfig),
                RegisteredTool("import_config", handler = ::importConfig),
                RegisteredTool("set_upstream_proxy", handler = ::setUpstreamProxy),
                RegisteredTool("set_dns_override", handler = ::setDnsOverride),
                RegisteredTool("set_http2", handler = ::setHttp2),
                RegisteredTool("cookie_jar", handler = ::cookieJar),
                RegisteredTool("token_analysis", handler = ::tokenAnalysis),
                RegisteredTool("sequencer", handler = ::sequencer),
                RegisteredTool("save_project", handler = ::saveProject),
                RegisteredTool("burp_version", handler = ::burpVersion),
                RegisteredTool("add_issue", handler = ::addIssue),
                RegisteredTool("register_http_handler", handler = ::registerHttpHandler),
                RegisteredTool("remove_http_handler", handler = ::removeHttpHandler),
                RegisteredTool("register_proxy_rule", handler = ::registerProxyRule),
                RegisteredTool("remove_proxy_rule", handler = ::removeProxyRule),
                RegisteredTool("extensions_list", handler = ::extensionsList),
                RegisteredTool("log", handler = ::logMessage),
                RegisteredTool("cookie_jar_set", handler = ::cookieJarSet),
                RegisteredTool("send_request_parallel", handler = ::sendRequestParallel),
                RegisteredTool("websocket_create", handler = ::websocketCreate),
                RegisteredTool("websocket_send_text", handler = ::websocketSendText),
                RegisteredTool("websocket_send_binary", handler = ::websocketSendBinary),
                RegisteredTool("websocket_close", handler = ::websocketClose),
                RegisteredTool("websocket_list", handler = ::websocketList),
                RegisteredTool("passive_intel", handler = ::passiveIntel),
                RegisteredTool("session_create_rule", handler = ::sessionCreateRule),
                RegisteredTool("session_list_rules", handler = ::sessionListRules),
                RegisteredTool("session_remove_rule", handler = ::sessionRemoveRule),
                RegisteredTool("jwt_decode", handler = ::jwtDecode),
                RegisteredTool("jwt_attack", handler = ::jwtAttack),
                RegisteredTool("injection_probe", handler = ::injectionProbe),
                RegisteredTool("access_control_sweep", handler = ::accessControlSweep),
                RegisteredTool("race_condition", handler = ::raceCondition),
                RegisteredTool("inline_fuzzer", handler = ::inlineFuzzer),
                RegisteredTool("scope_gate", handler = ::scopeGate),
                RegisteredTool("privacy_mode", handler = ::privacyMode),
                RegisteredTool("audit_log", handler = ::auditLog),
                RegisteredTool("proxy_listeners", advertised = false, handler = ::proxyListeners),
                RegisteredTool("proxy_match_replace", advertised = false, handler = ::proxyMatchReplace),
                RegisteredTool("intercept_modify", advertised = false, handler = ::interceptModify),
                RegisteredTool("export_cert", advertised = false, handler = ::exportCert),
                RegisteredTool("websocket_send", advertised = false, handler = ::websocketSend),
            ),
        )
    }

    private fun resolveAuthToken(): String {
        var token: String? = System.getProperty("burp.mcp.token")
        if (token == null || token.isBlank()) token = System.getenv("BURP_MCP_TOKEN")
        if (token == null || token.isBlank()) {
            val bytes = ByteArray(32)
            SecureRandom().nextBytes(bytes)
            val builder = StringBuilder()
            for (byte in bytes) builder.append(String.format("%02x", byte))
            token = builder.toString()
        }
        val resolvedToken: String = token.trim()
        try {
            val tokenFile: Path = Paths.get(System.getProperty("user.home"), ".burp-mcp-token")
            writeTokenFile(tokenFile, resolvedToken)
        } catch (_: IOException) {
        }
        return resolvedToken
    }

    @Throws(IOException::class)
    private fun writeTokenFile(
        tokenFile: Path,
        token: String,
    ) {
        val ownerOnly: Set<PosixFilePermission> =
            EnumSet.of(
                PosixFilePermission.OWNER_READ,
                PosixFilePermission.OWNER_WRITE,
            )
        val supportsPosix: Boolean = FileSystems.getDefault().supportedFileAttributeViews().contains("posix")
        val parent: Path = tokenFile.toAbsolutePath().parent
        val temporaryFile: Path =
            if (supportsPosix) {
                Files.createTempFile(parent, ".burp-mcp-token-", ".tmp", PosixFilePermissions.asFileAttribute(ownerOnly))
            } else {
                Files.createTempFile(parent, ".burp-mcp-token-", ".tmp")
            }
        try {
            Files.writeString(
                temporaryFile,
                token,
                StandardCharsets.UTF_8,
                StandardOpenOption.TRUNCATE_EXISTING,
                StandardOpenOption.WRITE,
            )
            if (supportsPosix) Files.setPosixFilePermissions(temporaryFile, ownerOnly)
            try {
                Files.move(temporaryFile, tokenFile, StandardCopyOption.ATOMIC_MOVE, StandardCopyOption.REPLACE_EXISTING)
            } catch (_: AtomicMoveNotSupportedException) {
                Files.move(temporaryFile, tokenFile, StandardCopyOption.REPLACE_EXISTING)
            }
            if (supportsPosix) Files.setPosixFilePermissions(tokenFile, ownerOnly)
        } finally {
            Files.deleteIfExists(temporaryFile)
        }
    }

    private fun isAuthorized(session: IHTTPSession): Boolean {
        val authorization: String? = session.headers["authorization"]
        return authorization != null && authorization == "Bearer $authToken"
    }

    override fun serve(session: IHTTPSession): Response {
        if (Method.OPTIONS == session.method) {
            val response: Response = newFixedLengthResponse(Response.Status.OK, "text/plain", "")
            addCorsHeaders(response)
            return response
        }
        if (!isAuthorized(session)) {
            val response: Response =
                newFixedLengthResponse(
                    Response.Status.FORBIDDEN,
                    "application/json",
                    "{\"error\":\"unauthorized: missing or invalid Authorization: Bearer <token> header. Token is in ~/.burp-mcp-token (or set -Dburp.mcp.token / BURP_MCP_TOKEN).\"}",
                )
            addCorsHeaders(response)
            return response
        }
        if (Method.GET == session.method && "/health" == session.uri) {
            val health =
                JsonObject().apply {
                    addProperty("status", "ok")
                    addProperty("version", "2.0.0")
                    add("tools", getToolList())
                }
            val response: Response =
                newFixedLengthResponse(
                    Response.Status.OK,
                    "application/json",
                    health.toString(),
                )
            addCorsHeaders(response)
            return response
        }
        if (Method.GET == session.method && "/tools" == session.uri) {
            val response: Response = newFixedLengthResponse(Response.Status.OK, "application/json", getToolList().toString())
            addCorsHeaders(response)
            return response
        }
        if (Method.POST != session.method) {
            val response: Response = newFixedLengthResponse(Response.Status.METHOD_NOT_ALLOWED, "text/plain", "POST only")
            addCorsHeaders(response)
            return response
        }
        return try {
            val bodyMap: MutableMap<String, String> = HashMap()
            session.parseBody(bodyMap)
            val body: String = bodyMap["postData"] ?: ""
            val request: JsonObject = JsonParser.parseString(body).asJsonObject
            val tool: String = if (request.has("tool")) request.get("tool").asString else ""
            val params: JsonObject = if (request.has("params")) request.getAsJsonObject("params") else JsonObject()
            val result: JsonObject = dispatch(tool, params)
            val response: Response = newFixedLengthResponse(Response.Status.OK, "application/json", gson.toJson(result))
            addCorsHeaders(response)
            response
        } catch (exception: Exception) {
            val error = JsonObject()
            error.addProperty("error", exception.message)
            val response: Response = newFixedLengthResponse(Response.Status.INTERNAL_ERROR, "application/json", gson.toJson(error))
            addCorsHeaders(response)
            response
        }
    }

    private fun dispatch(
        tool: String,
        params: JsonObject,
    ): JsonObject =
        toolRegistry.invoke(tool, params)
            ?: JsonObject().apply {
                addProperty("error", "Unknown tool: $tool")
                add("available_tools", getToolList())
            }

    private fun proxyHistory(params: JsonObject): JsonObject {
        val result = JsonObject()
        val history: List<ProxyHttpRequestResponse> = api.proxy().history()
        val limit: Int = if (params.has("limit")) params.get("limit").asInt else 100
        val offset: Int = if (params.has("offset")) params.get("offset").asInt else 0
        val filterUrl: String? = if (params.has("url_filter")) params.get("url_filter").asString else null
        val filterMethod: String? = if (params.has("method_filter")) params.get("method_filter").asString else null
        val filterStatus: Int = if (params.has("status_filter")) params.get("status_filter").asInt else 0
        var filtered: MutableList<ProxyHttpRequestResponse> = ArrayList(history)
        if (filterUrl != null) filtered = filtered.filter { it.finalRequest().url().contains(filterUrl) }.toMutableList()
        if (filterMethod !=
            null
        ) {
            filtered = filtered.filter { it.finalRequest().method().equals(filterMethod, ignoreCase = true) }.toMutableList()
        }
        if (filterStatus > 0) filtered = filtered.filter { it.response()?.statusCode()?.toInt() == filterStatus }.toMutableList()
        Collections.reverse(filtered)
        val end: Int = min(offset + limit, filtered.size)
        val items = JsonArray()
        for (index in offset until end) {
            val entry: ProxyHttpRequestResponse = filtered[index]
            val response: HttpResponse? = entry.response()
            items.add(
                JsonObject().apply {
                    addProperty("index", index)
                    addProperty("method", entry.finalRequest().method())
                    addProperty("url", entry.finalRequest().url())
                    addProperty("status", response?.statusCode() ?: 0)
                    addProperty("length", response?.body()?.length() ?: 0)
                },
            )
        }
        result.addProperty("total", filtered.size)
        result.add("items", items)
        return result
    }

    private fun sendRequest(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val method: String = if (params.has("method")) params.get("method").asString else "GET"
            val url: String = params.get("url").asString
            val body: String = if (params.has("body")) params.get("body").asString else ""
            val headers: JsonObject = if (params.has("headers")) params.getAsJsonObject("headers") else JsonObject()
            val uri = URI(url)
            val host: String = uri.host
            val port: Int =
                if (uri.port == -1) {
                    if (uri.scheme == "https") {
                        443
                    } else {
                        80
                    }
                } else {
                    uri.port
                }
            val path: String = uri.rawPath + if (uri.rawQuery != null) "?" + uri.rawQuery else ""
            val isHttps: Boolean = uri.scheme == "https"
            val rawRequest =
                StringBuilder()
                    .append(method)
                    .append(" ")
                    .append(path)
                    .append(" HTTP/1.1\r\n")
                    .append("Host: ")
                    .append(host)
                    .append("\r\n")
            for ((name: String, value: JsonElement) in headers.entrySet()) {
                rawRequest
                    .append(
                        name,
                    ).append(": ")
                    .append(value.asString)
                    .append("\r\n")
            }
            if (body.isNotEmpty()) {
                rawRequest
                    .append(
                        "Content-Length: ",
                    ).append(body.length)
                    .append("\r\n\r\n")
                    .append(body)
            } else {
                rawRequest.append("\r\n")
            }
            val service: HttpService = HttpService.httpService(host, port, isHttps)
            val response: HttpResponse = api.http().sendRequest(HttpRequest.httpRequest(service, rawRequest.toString())).response()
            result.addProperty("status", response.statusCode())
            result.addProperty("length", response.body().length())
            var responseBody: String = response.bodyToString()
            if (responseBody.length > 10000) responseBody = responseBody.substring(0, 10000) + "...[truncated]"
            result.addProperty("body", responseBody)
            val responseHeaders = JsonObject()
            response.headers().forEach { responseHeaders.addProperty(it.name(), it.value()) }
            result.add("headers", responseHeaders)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun sendToRepeater(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val request: HttpRequest = buildRequestWithService(params.get("request").asString)
            val tabName: String = if (params.has("tab_name")) params.get("tab_name").asString else "MCP"
            api.repeater().sendToRepeater(request, tabName)
            result.addProperty("success", true)
            result.addProperty("request_sent", false)
            result.addProperty("tab_caption", tabName)
            result.addProperty("message", "Request displayed in Repeater but not sent. tab_name is a caption, not a tag.")
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun sendToIntruder(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            api.intruder().sendToIntruder(buildRequestWithService(params.get("request").asString))
            result.addProperty("success", true)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun buildRequestWithService(rawRequest: String): HttpRequest {
        val matcher = Pattern.compile("(?im)^Host:\\s*([^:\r\n]+)(?::(\\d+))?\\s*$").matcher(rawRequest)
        if (!matcher.find()) return HttpRequest.httpRequest(rawRequest)
        val host: String = matcher.group(1).trim()
        val isHttps: Boolean = rawRequest.contains("https://") || rawRequest.contains(":443")
        val port: Int = matcher.group(2)?.toInt() ?: if (isHttps) 443 else 80
        return HttpRequest.httpRequest(HttpService.httpService(host, port, isHttps), rawRequest)
    }

    private fun intruderAttack(params: JsonObject): JsonObject = runSequentialAttack(params, false)

    private fun runSequentialAttack(
        params: JsonObject,
        wordlistMode: Boolean,
    ): JsonObject {
        val result = JsonObject()
        try {
            val urlTemplate: String = params.get("url_template").asString
            val placeholder: String = if (params.has("placeholder")) params.get("placeholder").asString else "@@"
            val method: String = if (params.has("method")) params.get("method").asString else "GET"
            val bodyTemplate: String = if (params.has("body_template")) params.get("body_template").asString else ""
            val headers: JsonObject = if (params.has("headers")) params.getAsJsonObject("headers") else JsonObject()
            val successLengthNot: Int = if (params.has("success_length_not")) params.get("success_length_not").asInt else -1
            val successContains: String? =
                if (!wordlistMode &&
                    params.has("success_contains")
                ) {
                    params.get("success_contains").asString
                } else {
                    null
                }
            val payloads: Sequence<String> =
                if (wordlistMode) {
                    params.getAsJsonArray("wordlist").asSequence().map { it.asString }
                } else {
                    val from: Int = if (params.has("from")) params.get("from").asInt else 0
                    val to: Int = if (params.has("to")) params.get("to").asInt else 100
                    val padDigits: Int = if (params.has("pad_digits")) params.get("pad_digits").asInt else 0
                    (from..to).asSequence().map {
                        if (padDigits > 0) String.format("%0" + padDigits + "d", it) else it.toString()
                    }
                }
            val hits = JsonArray()
            var count = 0
            var errors = 0
            for (payload in payloads) {
                val url: String = urlTemplate.replace(placeholder, payload)
                val body: String = bodyTemplate.replace(placeholder, payload)
                try {
                    val uri = URI(url)
                    val host: String = uri.host
                    val port: Int =
                        if (uri.port == -1) {
                            if (uri.scheme == "https") {
                                443
                            } else {
                                80
                            }
                        } else {
                            uri.port
                        }
                    val path: String = uri.rawPath + if (uri.rawQuery != null) "?" + uri.rawQuery else ""
                    val rawRequest =
                        StringBuilder()
                            .append(
                                method,
                            ).append(" ")
                            .append(path)
                            .append(" HTTP/1.1\r\nHost: ")
                            .append(host)
                            .append("\r\n")
                    for ((name: String, value: JsonElement) in headers.entrySet()) {
                        rawRequest
                            .append(
                                name,
                            ).append(": ")
                            .append(value.asString)
                            .append("\r\n")
                    }
                    if (body.isNotEmpty()) {
                        rawRequest
                            .append(
                                "Content-Length: ",
                            ).append(body.length)
                            .append("\r\n\r\n")
                            .append(body)
                    } else {
                        rawRequest.append("\r\n")
                    }
                    val response: HttpResponse =
                        api
                            .http()
                            .sendRequest(
                                HttpRequest.httpRequest(
                                    HttpService.httpService(
                                        host,
                                        port,
                                        uri.scheme == "https",
                                    ),
                                    rawRequest.toString(),
                                ),
                            ).response()
                    count++
                    val isHit: Boolean =
                        (successLengthNot > 0 && response.body().length() != successLengthNot) ||
                            (successContains != null && response.bodyToString().contains(successContains))
                    if (isHit) {
                        hits.add(
                            JsonObject().apply {
                                addProperty("payload", payload)
                                addProperty("status", response.statusCode())
                                addProperty("length", response.body().length())
                                if (!wordlistMode) {
                                    addProperty(
                                        "body_preview",
                                        response.bodyToString().substring(0, min(300, response.bodyToString().length)),
                                    )
                                }
                            },
                        )
                    }
                } catch (_: Exception) {
                    errors++
                }
            }
            result.addProperty("total_requests", count)
            result.addProperty("errors", errors)
            result.addProperty("hits", hits.size())
            result.add("hit_details", hits)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun intruderAttackWordlist(params: JsonObject): JsonObject = runSequentialAttack(params, true)

    private fun intruderAttackAsync(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val urlTemplate: String = params.get("url_template").asString
            val placeholder: String = if (params.has("placeholder")) params.get("placeholder").asString else "@@"
            val from: Int = if (params.has("from")) params.get("from").asInt else 0
            val to: Int = if (params.has("to")) params.get("to").asInt else 100
            val padDigits: Int = if (params.has("pad_digits")) params.get("pad_digits").asInt else 0
            val method: String = if (params.has("method")) params.get("method").asString else "GET"
            val bodyTemplate: String = if (params.has("body_template")) params.get("body_template").asString else ""
            val headers: JsonObject = if (params.has("headers")) params.getAsJsonObject("headers") else JsonObject()
            val successLengthNot: Int = if (params.has("success_length_not")) params.get("success_length_not").asInt else -1
            val threads: Int = if (params.has("threads")) params.get("threads").asInt else 50
            val executor: ExecutorService = Executors.newFixedThreadPool(threads)
            val hits = ConcurrentLinkedQueue<JsonObject>()
            val count = AtomicInteger(0)
            val errors = AtomicInteger(0)
            val found = AtomicBoolean(false)
            val futures: MutableList<Future<*>> = ArrayList()
            for (index in from..to) {
                futures.add(
                    executor.submit {
                        if (!found.get()) {
                            val payload: String = if (padDigits > 0) String.format("%0" + padDigits + "d", index) else index.toString()
                            val url: String = urlTemplate.replace(placeholder, payload)
                            val body: String = bodyTemplate.replace(placeholder, payload)
                            try {
                                val uri = URI(url)
                                val host: String = uri.host
                                val port: Int =
                                    if (uri.port == -1) {
                                        if (uri.scheme == "https") {
                                            443
                                        } else {
                                            80
                                        }
                                    } else {
                                        uri.port
                                    }
                                val path: String = uri.rawPath + if (uri.rawQuery != null) "?" + uri.rawQuery else ""
                                val rawRequest =
                                    StringBuilder()
                                        .append(
                                            method,
                                        ).append(" ")
                                        .append(path)
                                        .append(" HTTP/1.1\r\nHost: ")
                                        .append(host)
                                        .append("\r\n")
                                for ((name: String, value: JsonElement) in headers.entrySet()) {
                                    rawRequest
                                        .append(
                                            name,
                                        ).append(": ")
                                        .append(value.asString)
                                        .append("\r\n")
                                }
                                if (body.isNotEmpty()) {
                                    rawRequest
                                        .append(
                                            "Content-Length: ",
                                        ).append(body.length)
                                        .append("\r\n\r\n")
                                        .append(body)
                                } else {
                                    rawRequest.append("\r\n")
                                }
                                val response: HttpResponse =
                                    api
                                        .http()
                                        .sendRequest(
                                            HttpRequest.httpRequest(
                                                HttpService.httpService(
                                                    host,
                                                    port,
                                                    uri.scheme == "https",
                                                ),
                                                rawRequest.toString(),
                                            ),
                                        ).response()
                                count.incrementAndGet()
                                if (successLengthNot > 0 && response.body().length() != successLengthNot) {
                                    hits.add(
                                        JsonObject().apply {
                                            addProperty("payload", payload)
                                            addProperty("length", response.body().length())
                                            addProperty(
                                                "body_preview",
                                                response.bodyToString().substring(0, min(300, response.bodyToString().length)),
                                            )
                                        },
                                    )
                                    found.set(true)
                                }
                            } catch (_: Exception) {
                                errors.incrementAndGet()
                            }
                        }
                    },
                )
            }
            for (future: Future<*> in futures) {
                try {
                    future.get(120, TimeUnit.SECONDS)
                } catch (_: Exception) {
                }
            }
            executor.shutdownNow()
            result.addProperty("total_requests", count.get())
            result.addProperty("errors", errors.get())
            result.addProperty("hits", hits.size)
            val hitArray = JsonArray()
            hits.forEach { hitArray.add(it) }
            result.add("hit_details", hitArray)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun getSitemap(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val urlPrefix: String = if (params.has("url_prefix")) params.get("url_prefix").asString else ""
            val limit: Int = if (params.has("limit")) params.get("limit").asInt else 50
            val items = JsonArray()
            var count = 0
            for (entry in api.siteMap().requestResponses(SiteMapFilter.prefixFilter(urlPrefix))) {
                if (count >= limit) break
                items.add(
                    JsonObject().apply {
                        addProperty("url", entry.request().url())
                        addProperty("method", entry.request().method())
                        addProperty("status", entry.response()?.statusCode() ?: 0)
                    },
                )
                count++
            }
            result.addProperty("total", count)
            result.add("items", items)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun interceptToggle(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val enable: Boolean = if (params.has("enable")) params.get("enable").asBoolean else true
            if (enable) api.proxy().enableIntercept() else api.proxy().disableIntercept()
            result.addProperty("success", true)
            result.addProperty("intercept_enabled", enable)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun interceptModify(params: JsonObject): JsonObject =
        JsonObject().apply {
            addProperty("deprecated", true)
            addProperty(
                "info",
                "Removed from tools/list. Use intercept_toggle to enable intercept, then modify requests via proxy_history + send_request workflow.",
            )
        }

    private fun encode(params: JsonObject): JsonObject {
        val result = JsonObject()
        val input: String = params.get("input").asString
        when (if (params.has("type")) params.get("type").asString else "base64") {
            "base64" -> {
                result.addProperty("output", Base64.getEncoder().encodeToString(input.toByteArray(Charset.defaultCharset())))
            }

            "url" -> {
                try {
                    result.addProperty("output", java.net.URLEncoder.encode(input, "UTF-8"))
                } catch (exception: Exception) {
                    result.addProperty("error", exception.message)
                }
            }

            "hex" -> {
                val hex = StringBuilder()
                for (byte in input.toByteArray(Charset.defaultCharset())) hex.append(String.format("%02x", byte))
                result.addProperty("output", hex.toString())
            }

            else -> {
                result.addProperty("error", "Types: base64, url, hex")
            }
        }
        return result
    }

    private fun decode(params: JsonObject): JsonObject {
        val result = JsonObject()
        val input: String = params.get("input").asString
        when (if (params.has("type")) params.get("type").asString else "base64") {
            "base64" -> {
                result.addProperty("output", String(Base64.getDecoder().decode(input), Charset.defaultCharset()))
            }

            "url" -> {
                try {
                    result.addProperty("output", java.net.URLDecoder.decode(input, "UTF-8"))
                } catch (
                    exception: Exception,
                ) {
                    result.addProperty("error", exception.message)
                }
            }

            else -> {
                result.addProperty("error", "Types: base64, url")
            }
        }
        return result
    }

    private fun scan(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val url: String = if (params.has("url")) params.get("url").asString else ""
            val mode: String = if (params.has("mode")) params.get("mode").asString.lowercase(Locale.getDefault()) else "active"
            val active: Boolean = mode != "passive"
            if (url.isEmpty()) {
                result.addProperty("error", "url is required")
                return result
            }
            api.scope().includeInScope(url)
            val configuration: AuditConfiguration =
                AuditConfiguration.auditConfiguration(
                    if (active) BuiltInAuditConfiguration.LEGACY_ACTIVE_AUDIT_CHECKS else BuiltInAuditConfiguration.LEGACY_PASSIVE_AUDIT_CHECKS,
                )
            val audit: Audit = api.scanner().startAudit(configuration)
            activeAudit = audit
            val target = java.net.URL(url)
            val host: String = target.host
            val isHttps: Boolean = "https".equals(target.protocol, ignoreCase = true)
            val port: Int =
                if (target.port > 0) {
                    target.port
                } else if (isHttps) {
                    443
                } else {
                    80
                }
            val path: String = if (target.path == null || target.path.isEmpty()) "/" else target.path
            val pathQuery: String = if (target.query != null) "$path?${target.query}" else path
            val seedRequest: HttpRequest =
                HttpRequest.httpRequest(
                    HttpService.httpService(host, port, isHttps),
                    "GET $pathQuery HTTP/1.1\r\nHost: $host\r\nConnection: close\r\n\r\n",
                )
            audit.addRequest(seedRequest)
            result.addProperty("success", true)
            result.addProperty("mode", if (active) "active" else "passive")
            result.addProperty("url", url)
            result.addProperty(
                "message",
                "Audit started and seeded with GET request. Use scan_results to retrieve issues. Active scan requires Burp Professional.",
            )
            result.addProperty("note", "Audit runs asynchronously in Burp; issues surface via scan_results / Site map once complete.")
        } catch (exception: Exception) {
            val message: String = exception.message ?: exception.javaClass.simpleName
            if (message.lowercase(Locale.getDefault()).contains("professional") ||
                message.lowercase(Locale.getDefault()).contains("community") ||
                message.lowercase(Locale.getDefault()).contains("license")
            ) {
                result.addProperty("error", "Active/passive audit requires Burp Professional. Detected: $message")
            } else {
                result.addProperty("error", message)
            }
        }
        return result
    }

    private fun getScope(params: JsonObject): JsonObject {
        val result = JsonObject()
        val url: String = if (params.has("url")) params.get("url").asString else "https://example.com"
        result.addProperty("url", url)
        result.addProperty("in_scope", api.scope().isInScope(url))
        return result
    }

    private fun addToScope(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            api.scope().includeInScope(params.get("url").asString)
            result.addProperty("success", true)
        } catch (
            exception: Exception,
        ) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun removeFromScope(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            api.scope().excludeFromScope(params.get("url").asString)
            result.addProperty("success", true)
        } catch (
            exception: Exception,
        ) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun collaboratorGenerate(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            var client: CollaboratorClient? = collaborator
            if (client == null) {
                client = api.collaborator().createClient()
                collaborator = client
            }
            val count: Int = if (params.has("count")) params.get("count").asInt else 1
            val payloads = JsonArray()
            for (index in 0 until count) payloads.add(client.generatePayload().toString())
            result.add("payloads", payloads)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun collaboratorPoll(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val client: CollaboratorClient? = collaborator
            if (client == null) {
                result.addProperty("error", "No collaborator client. Call collaborator_generate first.")
                return result
            }
            val items = JsonArray()
            for (interaction in client.allInteractions) {
                items.add(
                    JsonObject().apply {
                        addProperty("type", interaction.type().name)
                        addProperty("client_ip", interaction.clientIp().toString())
                        addProperty("timestamp", interaction.timeStamp().toString())
                    },
                )
            }
            result.addProperty("count", items.size())
            result.add("interactions", items)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun searchHistory(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val pattern: Pattern =
                Pattern.compile(
                    if (params.has("regex")) params.get("regex").asString else ".*",
                    Pattern.CASE_INSENSITIVE,
                )
            val searchIn: String = if (params.has("search_in")) params.get("search_in").asString else "url"
            val limit: Int = if (params.has("limit")) params.get("limit").asInt else 20
            val history: List<ProxyHttpRequestResponse> = api.proxy().history()
            val items = JsonArray()
            var count = 0
            var index = history.size - 1
            while (index >= 0 && count < limit) {
                val entry: ProxyHttpRequestResponse = history[index]
                val response: HttpResponse? = entry.response()
                val matches: Boolean =
                    when (searchIn) {
                        "url" -> pattern.matcher(entry.finalRequest().url()).find()
                        "request" -> pattern.matcher(entry.finalRequest().toString()).find()
                        "response" -> response != null && pattern.matcher(response.toString()).find()
                        else -> false
                    }
                if (matches) {
                    items.add(
                        JsonObject().apply {
                            addProperty("index", index)
                            addProperty("method", entry.finalRequest().method())
                            addProperty("url", entry.finalRequest().url())
                            addProperty("status", response?.statusCode() ?: 0)
                        },
                    )
                    count++
                }
                index--
            }
            result.addProperty("matches", count)
            result.add("items", items)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun proxyDetail(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val index: Int = params.get("index").asInt
            val history: List<ProxyHttpRequestResponse> = api.proxy().history()
            if (index < 0 || index >= history.size) {
                result.addProperty("error", "Index out of range")
                return result
            }
            val entry: ProxyHttpRequestResponse = history[index]
            result.addProperty("method", entry.finalRequest().method())
            result.addProperty("url", entry.finalRequest().url())
            result.addProperty("request", entry.finalRequest().toString())
            val response: HttpResponse? = entry.response()
            if (response != null) {
                var responseText: String = response.toString()
                if (responseText.length > 50000) responseText = responseText.substring(0, 50000) + "...[truncated]"
                result.addProperty("response", responseText)
                result.addProperty("status", response.statusCode())
                result.addProperty("response_length", response.body().length())
            }
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun proxyWebSocket(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val limit: Int = if (params.has("limit")) params.get("limit").asInt else 50
            val items = JsonArray()
            var count = 0
            for (message in api.proxy().webSocketHistory()) {
                if (count >= limit) break
                val payload: String = message.payload().toString()
                items.add(
                    JsonObject().apply {
                        addProperty("direction", message.direction().name)
                        addProperty("payload", payload.substring(0, min(500, payload.length)))
                    },
                )
                count++
            }
            result.addProperty("total", count)
            result.add("messages", items)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun scanResults(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val limit: Int = if (params.has("limit")) params.get("limit").asInt else 50
            val items = JsonArray()
            var count = 0
            for (issue in api.siteMap().issues()) {
                if (count >= limit) break
                val detail: String? = issue.detail()
                items.add(
                    JsonObject().apply {
                        addProperty("name", issue.name())
                        addProperty("severity", issue.severity().name)
                        addProperty("confidence", issue.confidence().name)
                        addProperty("url", issue.baseUrl())
                        addProperty("detail", detail?.substring(0, min(200, detail.length)) ?: "")
                    },
                )
                count++
            }
            result.addProperty("total", count)
            result.add("issues", items)
            val audit: Audit? = activeAudit
            if (audit != null) {
                val status = JsonObject()
                try {
                    status.addProperty("request_count", audit.requestCount())
                    status.addProperty("error_count", audit.errorCount())
                    status.addProperty("insertion_point_count", audit.insertionPointCount())
                    status.addProperty("status_message", audit.statusMessage())
                    status.addProperty("audit_issue_count", audit.issues().size)
                } catch (_: Exception) {
                }
                result.add("active_audit", status)
            }
            val crawl: Crawl? = activeCrawl
            if (crawl != null) {
                val status = JsonObject()
                try {
                    status.addProperty("request_count", crawl.requestCount())
                    status.addProperty("error_count", crawl.errorCount())
                } catch (_: Exception) {
                }
                result.add("active_crawl", status)
            }
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun highlightItem(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val index: Int = params.get("index").asInt
            val color: String = if (params.has("color")) params.get("color").asString else "red"
            val history: List<ProxyHttpRequestResponse> = api.proxy().history()
            if (index >= 0 && index < history.size) {
                history[index].annotations().setHighlightColor(
                    burp.api.montoya.core.HighlightColor
                        .highlightColor(color),
                )
                result.addProperty("success", true)
            } else {
                result.addProperty("error", "Index out of range")
            }
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun annotate(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val index: Int = params.get("index").asInt
            val history: List<ProxyHttpRequestResponse> = api.proxy().history()
            if (index >= 0 && index < history.size) {
                history[index].annotations().setNotes(params.get("note").asString)
                result.addProperty("success", true)
            } else {
                result.addProperty("error", "Index out of range")
            }
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun compare(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val history: List<ProxyHttpRequestResponse> = api.proxy().history()
            val first: String = history[params.get("index1").asInt].response()?.bodyToString() ?: ""
            val second: String = history[params.get("index2").asInt].response()?.bodyToString() ?: ""
            result.addProperty("length1", first.length)
            result.addProperty("length2", second.length)
            result.addProperty("same", first == second)
            if (first != second) {
                var difference = 0
                for (index in 0 until min(first.length, second.length)) {
                    if (first[index] != second[index]) {
                        difference = index
                        break
                    }
                }
                result.addProperty("first_diff_at", difference)
                result.addProperty("context1", first.substring(max(0, difference - 20), min(first.length, difference + 50)))
                result.addProperty("context2", second.substring(max(0, difference - 20), min(second.length, difference + 50)))
            }
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun cookieJar(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val limit: Int = if (params.has("limit")) params.get("limit").asInt else 100
            val domainFilter: String? = if (params.has("domain")) params.get("domain").asString else null
            val items = JsonArray()
            var count = 0
            for (cookie in api.http().cookieJar().cookies()) {
                if (count >= limit) break
                if (domainFilter != null && !cookie.domain().contains(domainFilter)) continue
                items.add(
                    JsonObject().apply {
                        addProperty("name", cookie.name())
                        addProperty("value", cookie.value())
                        addProperty("domain", cookie.domain())
                        addProperty("path", cookie.path())
                    },
                )
                count++
            }
            result.addProperty("total", count)
            result.add("cookies", items)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun tokenAnalysis(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val analysis = JsonArray()
            for (element: JsonElement in params.getAsJsonArray("tokens")) {
                val token: String = element.asString
                val frequencies: MutableMap<Char, Int> = HashMap()
                for (character in token.toCharArray()) frequencies.merge(character, 1, Int::plus)
                var entropy = 0.0
                for (count: Int in frequencies.values) {
                    val probability: Double = count.toDouble() / token.length
                    entropy -= probability * (ln(probability) / ln(2.0))
                }
                analysis.add(
                    JsonObject().apply {
                        addProperty("token", token)
                        addProperty("length", token.length)
                        addProperty("entropy", java.lang.Math.round(entropy * 100.0) / 100.0)
                        addProperty("unique_chars", frequencies.size)
                        addProperty(
                            "quality",
                            if (entropy > 3.5) {
                                "good"
                            } else if (entropy > 2.0) {
                                "medium"
                            } else {
                                "weak"
                            },
                        )
                    },
                )
            }
            result.add("analysis", analysis)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun extensionsList(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            result.addProperty("current_extension", api.extension().filename())
            result.addProperty("info", "Use Burp UI Extensions tab to view all loaded extensions.")
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun logMessage(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val message: String = params.get("message").asString
            val level: String = if (params.has("level")) params.get("level").asString else "info"
            if (level == "error") api.logging().logToError(message) else api.logging().logToOutput(message)
            result.addProperty("success", true)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun repeaterSend(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val host: String = params.get("host").asString
            val port: Int = if (params.has("port")) params.get("port").asInt else 443
            val isHttps: Boolean = if (params.has("https")) params.get("https").asBoolean else true
            val response: HttpResponse =
                api
                    .http()
                    .sendRequest(
                        HttpRequest.httpRequest(HttpService.httpService(host, port, isHttps), params.get("request").asString),
                    ).response()
            result.addProperty("status", response.statusCode())
            result.addProperty("length", response.body().length())
            var body: String = response.bodyToString()
            if (body.length > 10000) body = body.substring(0, 10000) + "...[truncated]"
            result.addProperty("body", body)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun intruderPitchfork(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val urlTemplate: String = params.get("url_template").asString
            val method: String = if (params.has("method")) params.get("method").asString else "GET"
            val bodyTemplate: String = if (params.has("body_template")) params.get("body_template").asString else ""
            val headers: JsonObject = if (params.has("headers")) params.getAsJsonObject("headers") else JsonObject()
            val placeholders: JsonObject = params.getAsJsonObject("placeholders")
            val successLengthNot: Int = if (params.has("success_length_not")) params.get("success_length_not").asInt else -1
            val keys: List<String> = ArrayList(placeholders.keySet())
            val valueLists: MutableList<JsonArray> = ArrayList()
            for (key: String in keys) valueLists.add(placeholders.getAsJsonArray(key))
            val iterations: Int = valueLists[0].size()
            val hits = JsonArray()
            var count = 0
            var errors = 0
            for (index in 0 until iterations) {
                var url: String = urlTemplate
                var body: String = bodyTemplate
                for (keyIndex in keys.indices) {
                    val value: String = valueLists[keyIndex][index].asString
                    url = url.replace(keys[keyIndex], value)
                    body = body.replace(keys[keyIndex], value)
                }
                try {
                    val uri = URI(url)
                    val host: String = uri.host
                    val port: Int =
                        if (uri.port == -1) {
                            if (uri.scheme == "https") {
                                443
                            } else {
                                80
                            }
                        } else {
                            uri.port
                        }
                    val path: String = uri.rawPath + if (uri.rawQuery != null) "?" + uri.rawQuery else ""
                    val rawRequest =
                        StringBuilder()
                            .append(
                                method,
                            ).append(" ")
                            .append(path)
                            .append(" HTTP/1.1\r\nHost: ")
                            .append(host)
                            .append("\r\n")
                    for ((name: String, value: JsonElement) in headers.entrySet()) {
                        rawRequest
                            .append(
                                name,
                            ).append(": ")
                            .append(value.asString)
                            .append("\r\n")
                    }
                    if (body.isNotEmpty()) {
                        rawRequest
                            .append(
                                "Content-Length: ",
                            ).append(body.length)
                            .append("\r\n\r\n")
                            .append(body)
                    } else {
                        rawRequest.append("\r\n")
                    }
                    val response: HttpResponse =
                        api
                            .http()
                            .sendRequest(
                                HttpRequest.httpRequest(
                                    HttpService.httpService(
                                        host,
                                        port,
                                        uri.scheme == "https",
                                    ),
                                    rawRequest.toString(),
                                ),
                            ).response()
                    count++
                    if (successLengthNot > 0 && response.body().length() != successLengthNot) {
                        hits.add(
                            JsonObject().apply {
                                addProperty("iteration", index)
                                addProperty("length", response.body().length())
                            },
                        )
                    }
                } catch (_: Exception) {
                    errors++
                }
            }
            result.addProperty("total_requests", count)
            result.addProperty("errors", errors)
            result.addProperty("hits", hits.size())
            result.add("hit_details", hits)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun proxyListeners(params: JsonObject): JsonObject =
        JsonObject().apply {
            addProperty("deprecated", true)
            addProperty("info", "Removed from tools/list. Manage via Burp UI or export_config/import_config. Default: 127.0.0.1:8080")
        }

    private fun proxyMatchReplace(params: JsonObject): JsonObject =
        JsonObject().apply {
            addProperty("deprecated", true)
            addProperty("info", "Removed from tools/list. Use export_config -> modify proxy.match_replace_rules -> import_config")
        }

    private fun targetInfo(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val urlPrefix: String = if (params.has("url")) params.get("url").asString else ""
            val hosts: MutableSet<String> = HashSet()
            val technologies: MutableSet<String> = HashSet()
            var total = 0
            for (entry in api.siteMap().requestResponses(SiteMapFilter.prefixFilter(urlPrefix))) {
                hosts.add(entry.request().httpService().host())
                total++
                entry.response()?.headers()?.forEach { header ->
                    if (header.name().lowercase(Locale.getDefault()).matches(Regex("server|x-powered-by|x-aspnet-version"))) {
                        technologies.add(header.name() + ": " + header.value())
                    }
                }
                if (total > 500) break
            }
            val hostArray = JsonArray()
            hosts.forEach { hostArray.add(it) }
            val technologyArray = JsonArray()
            technologies.forEach { technologyArray.add(it) }
            result.add("hosts", hostArray)
            result.add("technologies", technologyArray)
            result.addProperty("requests_sampled", total)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun convertRequest(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val targetMethod: String = if (params.has("convert_to")) params.get("convert_to").asString else "POST"
            result.addProperty(
                "converted",
                params.get("request").asString.replaceFirst(Regex("^(GET|POST|PUT|DELETE|PATCH)"), targetMethod),
            )
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun exportRequest(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val request: String = params.get("request").asString
            val host: String = if (params.has("host")) params.get("host").asString else "example.com"
            val https: Boolean = if (params.has("https")) params.get("https").asBoolean else true
            val lines: kotlin.Array<String> = request.split("\r\n").toTypedArray()
            val parts: kotlin.Array<String> = lines[0].split(" ").toTypedArray()
            val url: String = (if (https) "https://" else "http://") + host + if (parts.size > 1) parts[1] else "/"
            result.addProperty("curl", "curl -X ${parts[0]} '$url'")
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun generateCsrfPoc(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val request: String = params.get("request").asString
            val host: String = if (params.has("host")) params.get("host").asString else "example.com"
            val https: Boolean = if (params.has("https")) params.get("https").asBoolean else true
            val lines: kotlin.Array<String> = request.split("\r\n").toTypedArray()
            val parts: kotlin.Array<String> = lines[0].split(" ").toTypedArray()
            val url: String = (if (https) "https://" else "http://") + host + if (parts.size > 1) parts[1] else "/"
            result.addProperty(
                "poc_html",
                "<html><body><form id='f' method='${parts[0]}' action='$url'></form><script>document.getElementById('f').submit()</script></body></html>",
            )
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun scanActive(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val host: String = params.get("host").asString
            val port: Int = if (params.has("port")) params.get("port").asInt else 443
            val isHttps: Boolean = if (params.has("https")) params.get("https").asBoolean else true
            val request: HttpRequest = HttpRequest.httpRequest(HttpService.httpService(host, port, isHttps), params.get("request").asString)
            api.scope().includeInScope(request.url())
            val audit: Audit =
                api.scanner().startAudit(
                    AuditConfiguration.auditConfiguration(BuiltInAuditConfiguration.LEGACY_ACTIVE_AUDIT_CHECKS),
                )
            activeAudit = audit
            audit.addRequest(request)
            result.addProperty("success", true)
            result.addProperty("url", request.url())
            result.addProperty(
                "message",
                "Standard active audit started with a seeded request. This cannot target or prove execution of a particular BCheck. Poll scan_results for issues. Requires Burp Professional.",
            )
        } catch (exception: Exception) {
            val message: String = exception.message ?: exception.javaClass.simpleName
            if (message.lowercase(Locale.getDefault()).contains("professional") ||
                message.lowercase(Locale.getDefault()).contains("community") ||
                message.lowercase(Locale.getDefault()).contains("license")
            ) {
                result.addProperty("error", "Active scan requires Burp Professional. Detected: $message")
            } else {
                result.addProperty("error", message)
            }
        }
        return result
    }

    private fun bambdaImport(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val importResult = api.bambda().importBambda(params.get("script").asString)
            result.addProperty("status", importResult.status().name)
            val importErrors = JsonArray()
            importResult.importErrors().forEach(importErrors::add)
            result.add("import_errors", importErrors)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun bcheckImport(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val enabled: Boolean = params.get("enabled").asBoolean
            val importResult = api.scanner().bChecks().importBCheck(params.get("script").asString, enabled)
            result.addProperty("status", importResult.status().name)
            val importErrors = JsonArray()
            importResult.importErrors().forEach(importErrors::add)
            result.add("import_errors", importErrors)
            result.addProperty("enabled", enabled)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun scanIssueDetail(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val targetIndex: Int = params.get("index").asInt
            var index = 0
            for (issue in api.siteMap().issues()) {
                if (index == targetIndex) {
                    result.addProperty("name", issue.name())
                    result.addProperty("severity", issue.severity().name)
                    result.addProperty("url", issue.baseUrl())
                    result.addProperty("detail", issue.detail() ?: "")
                    break
                }
                index++
            }
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun setUpstreamProxy(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            if (!params.has("proxy_host") || !params.has("proxy_port")) {
                result.addProperty("error", "proxy_host and proxy_port are required")
                return result
            }
            val proxyHost: String = params.get("proxy_host").asString
            val proxyPort: Int = params.get("proxy_port").asInt
            val type: String = if (params.has("type")) params.get("type").asString.lowercase(Locale.getDefault()) else "http"
            val configuration = "{\"project_options\":{\"connections\":{\"upstream_proxy\":{\"servers\":[{\"destination_host\":\"*\",\"proxy_host\":\"$proxyHost\",\"proxy_port\":$proxyPort,\"enabled\":true}]}}}}"
            api.burpSuite().importProjectOptionsFromJson(configuration)
            result.addProperty("success", true)
            result.addProperty("proxy_host", proxyHost)
            result.addProperty("proxy_port", proxyPort)
            result.addProperty(
                "note",
                "Upstream proxy set. Pass the same host/port to set_upstream_proxy again to change; restart Burp or clear via project options to disable.",
            )
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun sequencer(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val tokens: JsonArray = params.getAsJsonArray("tokens")
            val unique: MutableSet<String> = HashSet()
            val frequencies: MutableMap<Char, Int> = HashMap()
            var totalCharacters = 0
            for (element: JsonElement in tokens) {
                val token: String = element.asString
                unique.add(token)
                for (character in token.toCharArray()) {
                    frequencies.merge(character, 1, Int::plus)
                    totalCharacters++
                }
            }
            var entropy = 0.0
            for (count: Int in frequencies.values) {
                val probability: Double = count.toDouble() / totalCharacters
                entropy -= probability * (ln(probability) / ln(2.0))
            }
            result.addProperty("total", tokens.size())
            result.addProperty("unique", unique.size)
            result.addProperty("entropy_bits", java.lang.Math.round(entropy * 100.0) / 100.0)
            result.addProperty(
                "quality",
                if (entropy > 4.0) {
                    "excellent"
                } else if (entropy > 3.0) {
                    "good"
                } else {
                    "fair"
                },
            )
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun proxyClear(params: JsonObject): JsonObject =
        JsonObject().apply {
            addProperty(
                "info",
                "Proxy history cannot be cleared via API. Use Burp UI: Proxy -> HTTP history -> right-click -> Clear history",
            )
        }

    private fun repeaterModifySend(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            var rawRequest: String = params.get("request").asString
            val host: String = params.get("host").asString
            val port: Int = if (params.has("port")) params.get("port").asInt else 443
            val isHttps: Boolean = if (params.has("https")) params.get("https").asBoolean else true
            if (params.has("replace_header")) {
                for ((name: String, value: JsonElement) in params.getAsJsonObject("replace_header").entrySet()) {
                    rawRequest = rawRequest.replace(Regex("(?i)$name: [^\r\n]+"), "$name: ${value.asString}")
                }
            }
            if (params.has("add_header")) {
                val insertPosition: Int = rawRequest.indexOf("\r\n") + 2
                val headers = StringBuilder()
                for ((name: String, value: JsonElement) in params
                    .getAsJsonObject(
                        "add_header",
                    ).entrySet()) {
                    headers
                        .append(name)
                        .append(": ")
                        .append(value.asString)
                        .append("\r\n")
                }
                rawRequest = rawRequest.substring(0, insertPosition) + headers + rawRequest.substring(insertPosition)
            }
            if (params.has("replace_body")) {
                val bodyStart: Int = rawRequest.indexOf("\r\n\r\n")
                if (bodyStart > 0) rawRequest = rawRequest.substring(0, bodyStart + 4) + params.get("replace_body").asString
            }
            val response: HttpResponse =
                api
                    .http()
                    .sendRequest(
                        HttpRequest.httpRequest(HttpService.httpService(host, port, isHttps), rawRequest),
                    ).response()
            result.addProperty("status", response.statusCode())
            result.addProperty("length", response.body().length())
            result.addProperty("body", response.bodyToString().substring(0, min(10000, response.bodyToString().length)))
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun intruderClusterBomb(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val urlTemplate: String = params.get("url_template").asString
            val method: String = if (params.has("method")) params.get("method").asString else "GET"
            val bodyTemplate: String = if (params.has("body_template")) params.get("body_template").asString else ""
            val headers: JsonObject = if (params.has("headers")) params.getAsJsonObject("headers") else JsonObject()
            val placeholders: JsonObject = params.getAsJsonObject("placeholders")
            val successLengthNot: Int = if (params.has("success_length_not")) params.get("success_length_not").asInt else -1
            val maxRequests: Int = if (params.has("max_requests")) params.get("max_requests").asInt else 10000
            val keys: List<String> = ArrayList(placeholders.keySet())
            val valueLists: MutableList<JsonArray> = ArrayList()
            for (key: String in keys) valueLists.add(placeholders.getAsJsonArray(key))
            val hits = JsonArray()
            var count = 0
            var errors = 0
            val indices = IntArray(keys.size)
            var done = false
            while (!done && count < maxRequests) {
                var url: String = urlTemplate
                var body: String = bodyTemplate
                val payloadDescription = StringBuilder()
                for (index in keys.indices) {
                    val value: String = valueLists[index][indices[index]].asString
                    url = url.replace(keys[index], value)
                    body = body.replace(keys[index], value)
                    payloadDescription.append(value).append("|")
                }
                try {
                    val uri = URI(url)
                    val host: String = uri.host
                    val port: Int =
                        if (uri.port == -1) {
                            if (uri.scheme == "https") {
                                443
                            } else {
                                80
                            }
                        } else {
                            uri.port
                        }
                    val path: String = uri.rawPath + if (uri.rawQuery != null) "?" + uri.rawQuery else ""
                    val rawRequest =
                        StringBuilder()
                            .append(
                                method,
                            ).append(" ")
                            .append(path)
                            .append(" HTTP/1.1\r\nHost: ")
                            .append(host)
                            .append("\r\n")
                    for ((name: String, value: JsonElement) in headers.entrySet()) {
                        rawRequest
                            .append(
                                name,
                            ).append(": ")
                            .append(value.asString)
                            .append("\r\n")
                    }
                    if (body.isNotEmpty()) {
                        rawRequest
                            .append(
                                "Content-Length: ",
                            ).append(body.length)
                            .append("\r\n\r\n")
                            .append(body)
                    } else {
                        rawRequest.append("\r\n")
                    }
                    val response: HttpResponse =
                        api
                            .http()
                            .sendRequest(
                                HttpRequest.httpRequest(
                                    HttpService.httpService(
                                        host,
                                        port,
                                        uri.scheme == "https",
                                    ),
                                    rawRequest.toString(),
                                ),
                            ).response()
                    count++
                    if (successLengthNot > 0 && response.body().length() != successLengthNot) {
                        hits.add(
                            JsonObject().apply {
                                addProperty("payload", payloadDescription.toString())
                                addProperty("status", response.statusCode())
                                addProperty("length", response.body().length())
                            },
                        )
                    }
                } catch (_: Exception) {
                    errors++
                    count++
                }
                var carry: Int = keys.size - 1
                while (carry >= 0) {
                    indices[carry]++
                    if (indices[carry] < valueLists[carry].size()) break
                    indices[carry] = 0
                    carry--
                }
                if (carry < 0) done = true
            }
            result.addProperty("total_requests", count)
            result.addProperty("errors", errors)
            result.addProperty("hits", hits.size())
            result.add("hit_details", hits)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun intruderBatteringRam(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val urlTemplate: String = params.get("url_template").asString
            val placeholder: String = if (params.has("placeholder")) params.get("placeholder").asString else "@@"
            val method: String = if (params.has("method")) params.get("method").asString else "GET"
            val bodyTemplate: String = if (params.has("body_template")) params.get("body_template").asString else ""
            val headers: JsonObject = if (params.has("headers")) params.getAsJsonObject("headers") else JsonObject()
            val wordlist: JsonArray = params.getAsJsonArray("wordlist")
            val successLengthNot: Int = if (params.has("success_length_not")) params.get("success_length_not").asInt else -1
            val hits = JsonArray()
            var count = 0
            var errors = 0
            for (element: JsonElement in wordlist) {
                val payload: String = element.asString
                val url: String = urlTemplate.replace(placeholder, payload)
                val body: String = bodyTemplate.replace(placeholder, payload)
                try {
                    val uri = URI(url)
                    val host: String = uri.host
                    val port: Int =
                        if (uri.port == -1) {
                            if (uri.scheme == "https") {
                                443
                            } else {
                                80
                            }
                        } else {
                            uri.port
                        }
                    val path: String = uri.rawPath + if (uri.rawQuery != null) "?" + uri.rawQuery else ""
                    val rawRequest =
                        StringBuilder()
                            .append(
                                method,
                            ).append(" ")
                            .append(path)
                            .append(" HTTP/1.1\r\nHost: ")
                            .append(host)
                            .append("\r\n")
                    for ((name: String, value: JsonElement) in headers.entrySet()) {
                        rawRequest
                            .append(
                                name,
                            ).append(": ")
                            .append(value.asString.replace(placeholder, payload))
                            .append("\r\n")
                    }
                    if (body.isNotEmpty()) {
                        rawRequest
                            .append(
                                "Content-Length: ",
                            ).append(body.length)
                            .append("\r\n\r\n")
                            .append(body)
                    } else {
                        rawRequest.append("\r\n")
                    }
                    val response: HttpResponse =
                        api
                            .http()
                            .sendRequest(
                                HttpRequest.httpRequest(
                                    HttpService.httpService(
                                        host,
                                        port,
                                        uri.scheme == "https",
                                    ),
                                    rawRequest.toString(),
                                ),
                            ).response()
                    count++
                    if (successLengthNot > 0 && response.body().length() != successLengthNot) {
                        hits.add(
                            JsonObject().apply {
                                addProperty("payload", payload)
                                addProperty("status", response.statusCode())
                                addProperty("length", response.body().length())
                            },
                        )
                    }
                } catch (_: Exception) {
                    errors++
                }
            }
            result.addProperty("total_requests", count)
            result.addProperty("errors", errors)
            result.addProperty("hits", hits.size())
            result.add("hit_details", hits)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun intruderWithOptions(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val urlTemplate: String = params.get("url_template").asString
            val placeholder: String = if (params.has("placeholder")) params.get("placeholder").asString else "@@"
            val from: Int = if (params.has("from")) params.get("from").asInt else 0
            val to: Int = if (params.has("to")) params.get("to").asInt else 100
            val padDigits: Int = if (params.has("pad_digits")) params.get("pad_digits").asInt else 0
            val method: String = if (params.has("method")) params.get("method").asString else "GET"
            val headers: JsonObject = if (params.has("headers")) params.getAsJsonObject("headers") else JsonObject()
            val successLengthNot: Int = if (params.has("success_length_not")) params.get("success_length_not").asInt else -1
            val throttleMs: Int = if (params.has("throttle_ms")) params.get("throttle_ms").asInt else 0
            val payloadPrefix: String = if (params.has("payload_prefix")) params.get("payload_prefix").asString else ""
            val payloadSuffix: String = if (params.has("payload_suffix")) params.get("payload_suffix").asString else ""
            val payloadEncoding: String = if (params.has("payload_encoding")) params.get("payload_encoding").asString else "none"
            val grepExtract: String? = if (params.has("grep_extract")) params.get("grep_extract").asString else null
            val recordTime: Boolean = if (params.has("record_time")) params.get("record_time").asBoolean else false
            val hits = JsonArray()
            var count = 0
            var errors = 0
            for (index in from..to) {
                val rawPayload: String = if (padDigits > 0) String.format("%0" + padDigits + "d", index) else index.toString()
                var payload: String = payloadPrefix + rawPayload + payloadSuffix
                if (payloadEncoding == "base64") {
                    payload = Base64.getEncoder().encodeToString(payload.toByteArray(Charset.defaultCharset()))
                } else if (payloadEncoding == "url") {
                    try {
                        payload = java.net.URLEncoder.encode(payload, "UTF-8")
                    } catch (_: Exception) {
                    }
                } else if (payloadEncoding ==
                    "md5"
                ) {
                    try {
                        payload =
                            HexFormat.of().formatHex(
                                java.security.MessageDigest
                                    .getInstance("MD5")
                                    .digest(payload.toByteArray(Charset.defaultCharset())),
                            )
                    } catch (
                        _: Exception,
                    ) {
                    }
                }
                val url: String = urlTemplate.replace(placeholder, payload)
                try {
                    val uri = URI(url)
                    val host: String = uri.host
                    val port: Int =
                        if (uri.port == -1) {
                            if (uri.scheme == "https") {
                                443
                            } else {
                                80
                            }
                        } else {
                            uri.port
                        }
                    val path: String = uri.rawPath + if (uri.rawQuery != null) "?" + uri.rawQuery else ""
                    val rawRequest =
                        StringBuilder()
                            .append(
                                method,
                            ).append(" ")
                            .append(path)
                            .append(" HTTP/1.1\r\nHost: ")
                            .append(host)
                            .append("\r\n")
                    for ((name: String, value: JsonElement) in headers.entrySet()) {
                        rawRequest
                            .append(
                                name,
                            ).append(": ")
                            .append(value.asString)
                            .append("\r\n")
                    }
                    rawRequest.append("\r\n")
                    val startMilliseconds: Long = System.currentTimeMillis()
                    val response: HttpResponse =
                        api
                            .http()
                            .sendRequest(
                                HttpRequest.httpRequest(
                                    HttpService.httpService(
                                        host,
                                        port,
                                        uri.scheme == "https",
                                    ),
                                    rawRequest.toString(),
                                ),
                            ).response()
                    val elapsed: Long = System.currentTimeMillis() - startMilliseconds
                    count++
                    if (successLengthNot > 0 && response.body().length() != successLengthNot) {
                        hits.add(
                            JsonObject().apply {
                                addProperty("payload", rawPayload)
                                addProperty("status", response.statusCode())
                                addProperty("length", response.body().length())
                                if (recordTime) addProperty("time_ms", elapsed)
                                if (grepExtract != null) {
                                    val matcher = Pattern.compile(grepExtract).matcher(response.bodyToString())
                                    if (matcher.find()) addProperty("extracted", matcher.group())
                                }
                            },
                        )
                    }
                    if (throttleMs > 0) Thread.sleep(throttleMs.toLong())
                } catch (_: Exception) {
                    errors++
                }
            }
            result.addProperty("total_requests", count)
            result.addProperty("errors", errors)
            result.addProperty("hits", hits.size())
            result.add("hit_details", hits)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun extractFromResponse(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val index: Int = params.get("index").asInt
            val history: List<ProxyHttpRequestResponse> = api.proxy().history()
            if (index < 0 || index >= history.size) {
                result.addProperty("error", "Index out of range")
                return result
            }
            val body: String = history[index].response()?.bodyToString() ?: ""
            val matcher = Pattern.compile(params.get("regex").asString).matcher(body)
            val matches = JsonArray()
            while (matcher.find()) matches.add(matcher.group())
            result.addProperty("total_matches", matches.size())
            result.add("matches", matches)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun crawl(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val url: String = params.get("url").asString
            api.scope().includeInScope(url)
            activeCrawl = api.scanner().startCrawl(CrawlConfiguration.crawlConfiguration(url))
            result.addProperty("success", true)
            result.addProperty("url", url)
            result.addProperty(
                "message",
                "Crawl started. Requires Burp Professional. New URLs surface in sitemap; check Crawl progress via request count.",
            )
            result.addProperty(
                "note",
                "Crawl.statusMessage() is unimplemented in the current API; completion has no reliable polling signal.",
            )
        } catch (exception: Exception) {
            val message: String = exception.message ?: exception.javaClass.simpleName
            if (message.lowercase(Locale.getDefault()).contains("professional") ||
                message.lowercase(Locale.getDefault()).contains("community") ||
                message.lowercase(Locale.getDefault()).contains("license")
            ) {
                result.addProperty("error", "Crawl requires Burp Professional. Detected: $message")
            } else {
                result.addProperty("error", message)
            }
        }
        return result
    }

    private fun setDnsOverride(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val hostname: String = params.get("hostname").asString
            val ip: String = params.get("ip").asString
            api.burpSuite().importProjectOptionsFromJson(
                "{\"project_options\":{\"connections\":{\"hostname_resolution\":[{\"enabled\":true,\"hostname\":\"$hostname\",\"ip_address\":\"$ip\"}]}}}",
            )
            result.addProperty("success", true)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun setHttp2(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val enable: Boolean = if (params.has("enable")) params.get("enable").asBoolean else false
            api.burpSuite().importProjectOptionsFromJson("{\"project_options\":{\"http\":{\"http2\":{\"enabled\":$enable}}}}")
            result.addProperty("success", true)
            result.addProperty("http2_enabled", enable)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun exportCert(params: JsonObject): JsonObject =
        JsonObject().apply {
            addProperty("deprecated", true)
            addProperty(
                "info",
                "Removed from tools/list. Export Burp CA cert: Proxy -> Options -> Import/Export CA certificate -> Export Certificate in DER format",
            )
            addProperty("path_hint", "Or visit http://burp/cert in browser with Burp proxy enabled")
        }

    private fun websocketSend(params: JsonObject): JsonObject =
        JsonObject().apply {
            addProperty("deprecated", true)
            addProperty(
                "info",
                "Removed from tools/list. Use websocket_send_text / websocket_send_binary on an active websocket_create connection.",
            )
        }

    private fun payloadProcess(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val input: String = params.get("input").asString
            val operation: String = params.get("operation").asString
            val output: String =
                when (operation) {
                    "base64_encode" -> {
                        Base64.getEncoder().encodeToString(input.toByteArray(Charset.defaultCharset()))
                    }

                    "base64_decode" -> {
                        String(Base64.getDecoder().decode(input), Charset.defaultCharset())
                    }

                    "url_encode" -> {
                        java.net.URLEncoder.encode(input, "UTF-8")
                    }

                    "url_decode" -> {
                        java.net.URLDecoder.decode(input, "UTF-8")
                    }

                    "md5" -> {
                        HexFormat.of().formatHex(
                            java.security.MessageDigest
                                .getInstance("MD5")
                                .digest(input.toByteArray(Charset.defaultCharset())),
                        )
                    }

                    "sha1" -> {
                        HexFormat.of().formatHex(
                            java.security.MessageDigest
                                .getInstance("SHA-1")
                                .digest(input.toByteArray(Charset.defaultCharset())),
                        )
                    }

                    "sha256" -> {
                        HexFormat.of().formatHex(
                            java.security.MessageDigest
                                .getInstance("SHA-256")
                                .digest(input.toByteArray(Charset.defaultCharset())),
                        )
                    }

                    "hex_encode" -> {
                        StringBuilder()
                            .apply {
                                for (byte in input.toByteArray(
                                    Charset.defaultCharset(),
                                )) {
                                    append(String.format("%02x", byte))
                                }
                            }.toString()
                    }

                    "lowercase" -> {
                        input.lowercase(Locale.getDefault())
                    }

                    "uppercase" -> {
                        input.uppercase(Locale.getDefault())
                    }

                    "reverse" -> {
                        StringBuilder(input).reverse().toString()
                    }

                    "length" -> {
                        input.length.toString()
                    }

                    else -> {
                        result.addProperty(
                            "error",
                            "Operations: base64_encode/decode, url_encode/decode, md5, sha1, sha256, hex_encode, lowercase, uppercase, reverse, length",
                        )
                        return result
                    }
                }
            result.addProperty("input", input)
            result.addProperty("output", output)
            result.addProperty("operation", operation)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun saveProject(params: JsonObject): JsonObject =
        JsonObject().apply {
            addProperty("info", "Project auto-saves. Use Burp menu: Burp -> Save project to save explicitly.")
        }

    private fun exportConfig(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val configuration: String = api.burpSuite().exportProjectOptionsAsJson()
            result.addProperty("config", configuration.substring(0, min(5000, configuration.length)))
            result.addProperty("truncated", configuration.length > 5000)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun importConfig(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            api.burpSuite().importProjectOptionsFromJson(params.get("config").asString)
            result.addProperty("success", true)
        } catch (
            exception: Exception,
        ) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun burpVersion(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val version = api.burpSuite().version()
            result.addProperty("name", version.name())
            result.addProperty("build_number", version.buildNumber())
            result.addProperty("edition", version.edition().toString())
            result.addProperty("version", version.toString())
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun addIssue(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val name: String = params.get("name").asString
            val url: String = params.get("url").asString
            val detail: String = if (params.has("detail")) params.get("detail").asString else ""
            val remediation: String = if (params.has("remediation")) params.get("remediation").asString else ""
            val severityText: String =
                if (params.has(
                        "severity",
                    )
                ) {
                    params.get("severity").asString.uppercase(Locale.getDefault())
                } else {
                    "INFORMATION"
                }
            val confidenceText: String =
                if (params.has(
                        "confidence",
                    )
                ) {
                    params.get("confidence").asString.uppercase(Locale.getDefault())
                } else {
                    "TENTATIVE"
                }
            val severity: AuditIssueSeverity =
                try {
                    AuditIssueSeverity.valueOf(severityText)
                } catch (
                    _: IllegalArgumentException,
                ) {
                    AuditIssueSeverity.INFORMATION
                }
            val confidence: AuditIssueConfidence =
                try {
                    AuditIssueConfidence.valueOf(confidenceText)
                } catch (
                    _: IllegalArgumentException,
                ) {
                    AuditIssueConfidence.TENTATIVE
                }
            val issue: AuditIssue = AuditIssue.auditIssue(name, detail, remediation, url, severity, confidence, "", "", severity)
            api.siteMap().add(issue)
            result.addProperty("success", true)
            result.addProperty("message", "Issue added to sitemap: $name at $url")
            result.addProperty("severity", severity.name)
            result.addProperty("confidence", confidence.name)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun proxyHistoryFiltered(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val hasNotes: String? = if (params.has("has_notes")) params.get("has_notes").asString else null
            val limit: Int = if (params.has("limit")) params.get("limit").asInt else 50
            val history: List<ProxyHttpRequestResponse> = api.proxy().history()
            val items = JsonArray()
            var count = 0
            var index = history.size - 1
            while (index >= 0 && count < limit) {
                val entry: ProxyHttpRequestResponse = history[index]
                var matches = true
                if (hasNotes != null && hasNotes == "true") {
                    val notes: String? = entry.annotations().notes()
                    matches = notes != null && notes.isNotEmpty()
                }
                if (matches) {
                    items.add(
                        JsonObject().apply {
                            addProperty("index", index)
                            addProperty("url", entry.finalRequest().url())
                            addProperty("notes", entry.annotations().notes() ?: "")
                        },
                    )
                    count++
                }
                index--
            }
            result.addProperty("matches", count)
            result.add("items", items)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    @Volatile
    private var httpHandlerHeader: String = ""

    @Volatile
    private var httpHandlerHeaderValue: String = ""

    @Volatile
    private var httpHandlerMatch: String = ""

    @Volatile
    private var httpHandlerReplace: String = ""

    @Volatile
    private var httpHandlerActive: Boolean = false

    @Volatile
    private var httpHandlerRegistration: Registration? = null

    private fun registerHttpHandler(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            if (params.has("header_name")) {
                httpHandlerHeader = params.get("header_name").asString
                httpHandlerHeaderValue = params.get("header_value").asString
            }
            if (params.has("match")) {
                httpHandlerMatch = params.get("match").asString
                httpHandlerReplace = params.get("replace").asString
            }
            if (!httpHandlerActive) {
                httpHandlerRegistration =
                    api.http().registerHttpHandler(
                        object : HttpHandler {
                            override fun handleHttpRequestToBeSent(requestToBeSent: HttpRequestToBeSent): RequestToBeSentAction {
                                var modified: HttpRequest = requestToBeSent
                                if (httpHandlerHeader.isNotEmpty()) {
                                    modified =
                                        modified.withAddedHeader(httpHandlerHeader, httpHandlerHeaderValue)
                                }
                                if (httpHandlerMatch.isNotEmpty()) {
                                    modified =
                                        HttpRequest.httpRequest(
                                            modified.httpService(),
                                            modified.toString().replace(httpHandlerMatch, httpHandlerReplace),
                                        )
                                }
                                return RequestToBeSentAction.continueWith(modified)
                            }

                            override fun handleHttpResponseReceived(responseReceived: HttpResponseReceived): ResponseReceivedAction =
                                ResponseReceivedAction.continueWith(responseReceived)
                        },
                    )
                httpHandlerActive = true
            }
            result.addProperty("success", true)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun removeHttpHandler(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val registration: Registration? = httpHandlerRegistration
            if (registration != null && registration.isRegistered) registration.deregister()
            httpHandlerRegistration = null
            httpHandlerActive = false
            httpHandlerHeader = ""
            httpHandlerHeaderValue = ""
            httpHandlerMatch = ""
            httpHandlerReplace = ""
            result.addProperty("success", true)
            result.addProperty("message", "HTTP handler deregistered")
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    @Volatile
    private var proxyRuleUrl: String = ""

    @Volatile
    private var proxyRuleActive: Boolean = false

    @Volatile
    private var proxyRuleRegistration: Registration? = null

    private fun registerProxyRule(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val urlContains: String = params.get("url_contains").asString
            val intercept: Boolean = if (params.has("intercept")) params.get("intercept").asBoolean else true
            if (proxyRuleActive) {
                result.addProperty("error", "A proxy rule is already active. Call remove_proxy_rule first.")
                return result
            }
            proxyRuleUrl = urlContains
            proxyRuleRegistration =
                api.proxy().registerRequestHandler(
                    object : ProxyRequestHandler {
                        override fun handleRequestReceived(request: InterceptedRequest): ProxyRequestReceivedAction {
                            if (proxyRuleUrl.isNotEmpty() && request.url().contains(proxyRuleUrl)) {
                                return if (intercept) {
                                    ProxyRequestReceivedAction.intercept(
                                        request,
                                    )
                                } else {
                                    ProxyRequestReceivedAction.doNotIntercept(request)
                                }
                            }
                            return ProxyRequestReceivedAction.continueWith(request)
                        }

                        override fun handleRequestToBeSent(request: InterceptedRequest): ProxyRequestToBeSentAction =
                            ProxyRequestToBeSentAction.continueWith(request)
                    },
                )
            proxyRuleActive = true
            result.addProperty("success", true)
            result.addProperty(
                "message",
                "Proxy rule active: ${if (intercept) "intercept" else "do-not-intercept"} URLs containing $urlContains",
            )
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun removeProxyRule(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val registration: Registration? = proxyRuleRegistration
            if (registration != null && registration.isRegistered) registration.deregister()
            proxyRuleRegistration = null
            proxyRuleActive = false
            val previous: String = proxyRuleUrl
            proxyRuleUrl = ""
            result.addProperty("success", true)
            result.addProperty("message", "Proxy rule removed" + if (previous.isEmpty()) "" else " (was: $previous)")
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private val wsConnections: MutableMap<String, ExtensionWebSocket> = ConcurrentHashMap()
    private var wsCounter: Int = 0

    @Volatile
    private var sessionActionRegistration: Registration? = null

    private var scopeGateEnabled: Boolean = false
    private var privacyStrict: Boolean = false
    private val auditLogEntries: java.util.ArrayList<String> = java.util.ArrayList()
    private val threadPool: ExecutorService = Executors.newFixedThreadPool(20)

    private fun addAuditLog(entry: String) {
        auditLogEntries.add(
            java.time.Instant
                .now()
                .toString() + " " + entry,
        )
        if (auditLogEntries.size > 1000) auditLogEntries.removeAt(0)
    }

    private fun sendOne(
        method: String,
        url: String,
        body: String?,
    ): HttpRequestResponse? =
        try {
            val target = java.net.URL(url)
            val https: Boolean = "https" == target.protocol
            val service: HttpService =
                HttpService.httpService(
                    target.host,
                    if (target.port >
                        0
                    ) {
                        target.port
                    } else if (https) {
                        443
                    } else {
                        80
                    },
                    https,
                )
            val path: String = if (target.path == null || target.path.isEmpty()) "/" else target.path
            val pathQuery: String = if (target.query != null) "$path?${target.query}" else path
            var request = "$method $pathQuery HTTP/1.1\r\nHost: ${target.host}\r\nConnection: close\r\n\r\n"
            if (body != null &&
                body.isNotEmpty()
            ) {
                request =
                    "$method $pathQuery HTTP/1.1\r\nHost: ${target.host}\r\nContent-Length: ${body.length}\r\nConnection: close\r\n\r\n$body"
            }
            api.http().sendRequest(HttpRequest.httpRequest(service, request))
        } catch (_: Exception) {
            null
        }

    private fun cookieJarSet(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            api.http().cookieJar().setCookie(
                params.get("url").asString,
                params.get("name").asString,
                params.get("value").asString,
                "/",
                ZonedDateTime.now().plusDays(30),
            )
            result.addProperty("success", true)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun sendRequestParallel(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val requests: JsonArray = params.getAsJsonArray("requests")
            val futures: MutableList<Future<HttpRequestResponse?>> = ArrayList()
            for (index in 0 until requests.size()) {
                val request: JsonObject = requests[index].asJsonObject
                futures.add(
                    threadPool.submit<HttpRequestResponse?> {
                        sendOne(
                            request.get("method").asString,
                            request.get("url").asString,
                            if (request.has("body")) request.get("body").asString else null,
                        )
                    },
                )
            }
            val items = JsonArray()
            for (future: Future<HttpRequestResponse?> in futures) {
                try {
                    val requestResponse: HttpRequestResponse = future.get() ?: continue
                    val response: HttpResponse? = requestResponse.response()
                    items.add(
                        JsonObject().apply {
                            addProperty("status", response?.statusCode() ?: 0)
                            addProperty("length", response?.bodyToString()?.length ?: 0)
                        },
                    )
                } catch (_: Exception) {
                }
            }
            result.add("results", items)
            result.addProperty("total", items.size())
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun websocketCreate(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val https: Boolean = if (params.has("https")) params.get("https").asBoolean else true
            val port: Int =
                if (params.has("port")) {
                    params.get("port").asInt
                } else if (https) {
                    443
                } else {
                    80
                }
            val creationResult =
                api.websockets().createWebSocket(
                    HttpService.httpService(params.get("host").asString, port, https),
                    if (params.has("path")) params.get("path").asString else "/",
                )
            result.addProperty("status", creationResult.status().toString())
            creationResult.webSocket().ifPresent { webSocket: ExtensionWebSocket ->
                val id = "ws-${++wsCounter}"
                wsConnections[id] = webSocket
                result.addProperty("id", id)
            }
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun websocketSendBinary(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val webSocket: ExtensionWebSocket? = wsConnections[params.get("id").asString]
            if (webSocket == null) {
                result.addProperty("error", "WS not found")
                return result
            }
            webSocket.sendBinaryMessage(ByteArray.byteArray(*params.get("data").asString.toByteArray(Charset.defaultCharset())))
            result.addProperty("success", true)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun websocketSendText(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val webSocket: ExtensionWebSocket? = wsConnections[params.get("id").asString]
            if (webSocket == null) {
                result.addProperty("error", "WS not found")
                return result
            }
            webSocket.sendTextMessage(params.get("text").asString)
            result.addProperty("success", true)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun websocketClose(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val webSocket: ExtensionWebSocket? = wsConnections.remove(params.get("id").asString)
            if (webSocket == null) {
                result.addProperty("error", "WS not found")
                return result
            }
            webSocket.close()
            result.addProperty("success", true)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun websocketList(params: JsonObject): JsonObject {
        val result = JsonObject()
        val items = JsonArray()
        for (id: String in wsConnections.keys) items.add(JsonObject().apply { addProperty("id", id) })
        result.add("connections", items)
        result.addProperty("total", items.size())
        return result
    }

    private fun passiveIntel(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val findings = JsonArray()
            val limit: Int = if (params.has("limit")) params.get("limit").asInt else 100
            var count = 0
            for (entry in api.proxy().history()) {
                if (count >= limit) break
                val body: String = entry.response()?.bodyToString() ?: ""
                val all: String = "$body ${entry.request()}"
                var type: String? = null
                var matcher = Pattern.compile("AKIA[0-9A-Z]{16}").matcher(all)
                if (matcher.find()) {
                    type = "AWS Key: ${matcher.group()}"
                } else {
                    matcher = Pattern.compile("eyJ[a-zA-Z0-9_-]+\\.[a-zA-Z0-9_-]+\\.[a-zA-Z0-9_-]+").matcher(all)
                    if (matcher.find()) {
                        type = "JWT"
                    } else {
                        matcher = Pattern.compile("-----BEGIN.*PRIVATE KEY-----").matcher(all)
                        if (matcher.find()) {
                            type = "Private Key"
                        } else {
                            matcher = Pattern.compile("xox[baprs]-[0-9]{10,}-[0-9]{10,}-[a-zA-Z0-9]{24}").matcher(all)
                            if (matcher.find()) type = "Slack Token"
                        }
                    }
                }
                if (type != null) {
                    findings.add(
                        JsonObject().apply {
                            addProperty("type", type)
                            addProperty("url", entry.request().url())
                        },
                    )
                }
                count++
            }
            result.add("findings", findings)
            result.addProperty("total", findings.size())
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun sessionCreateRule(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            if (sessionActionRegistration != null) {
                result.addProperty("error", "Rule active")
                return result
            }
            val pattern: Pattern = Pattern.compile(params.get("find").asString)
            val replacement: String = params.get("replace").asString
            sessionActionRegistration =
                api.http().registerSessionHandlingAction(
                    object : SessionHandlingAction {
                        override fun name(): String = "MCP Rule"

                        override fun performAction(actionData: SessionHandlingActionData): ActionResult {
                            val request: String = pattern.matcher(actionData.request().toString()).replaceAll(replacement)
                            return try {
                                ActionResult.actionResult(HttpRequest.httpRequest(request))
                            } catch (
                                _: Exception,
                            ) {
                                ActionResult.actionResult(actionData.request())
                            }
                        }
                    },
                )
            result.addProperty("success", true)
            result.addProperty("rule", "s/${params.get("find").asString}/$replacement/")
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun sessionListRules(params: JsonObject): JsonObject {
        val registration: Registration? = sessionActionRegistration
        return JsonObject().apply { addProperty("active", registration != null && registration.isRegistered) }
    }

    private fun sessionRemoveRule(params: JsonObject): JsonObject {
        val registration: Registration? = sessionActionRegistration
        if (registration != null && registration.isRegistered) registration.deregister()
        sessionActionRegistration = null
        return JsonObject().apply { addProperty("success", true) }
    }

    private fun jwtDecode(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val parts: kotlin.Array<String> = Pattern.compile("\\.").split(params.get("token").asString)
            if (parts.size < 2) {
                result.addProperty("error", "Invalid JWT")
                return result
            }
            result.add(
                "header",
                JsonParser.parseString(String(Base64.getUrlDecoder().decode(parts[0]), Charset.defaultCharset())),
            )
            result.add(
                "payload",
                JsonParser.parseString(String(Base64.getUrlDecoder().decode(parts[1]), Charset.defaultCharset())),
            )
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun jwtAttack(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val parts: kotlin.Array<String> = Pattern.compile("\\.").split(params.get("token").asString)
            if ((if (params.has("attack")) params.get("attack").asString else "none") == "none") {
                val header: JsonObject =
                    JsonParser
                        .parseString(
                            String(Base64.getUrlDecoder().decode(parts[0]), Charset.defaultCharset()),
                        ).asJsonObject
                header.addProperty("alg", "none")
                result.addProperty(
                    "forged",
                    Base64.getUrlEncoder().withoutPadding().encodeToString(header.toString().toByteArray(Charset.defaultCharset())) + "." +
                        parts[1] +
                        ".",
                )
            }
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun injectionProbe(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val url: String = params.get("url").asString
            val parameter: String = params.get("param").asString
            val type: String = if (params.has("type")) params.get("type").asString else "sqli"
            var library: kotlin.Array<kotlin.Array<String>> =
                arrayOf(arrayOf("'", "' OR '1'='1", "' AND SLEEP(3)--"), arrayOf("syntax", "unclosed", "mysql", "ORA-"))
            if (type == "ssti") library = arrayOf(arrayOf("{{7*7}}", "${'$'}{7*7}", "<%=7*7%>"), arrayOf("49"))
            if (type == "lfi") library = arrayOf(arrayOf("../../etc/passwd", "....//....//etc/passwd"), arrayOf("root:", "bin:"))
            val items = JsonArray()
            for (payload: String in library[0]) {
                val fullUrl: String =
                    url + (if (url.contains("?")) "&" else "?") + parameter + "=" + java.net.URLEncoder.encode(payload, "UTF-8")
                val target = java.net.URL(fullUrl)
                val https: Boolean = "https" == target.protocol
                val service: HttpService =
                    HttpService.httpService(
                        target.host,
                        if (target.port >
                            0
                        ) {
                            target.port
                        } else if (https) {
                            443
                        } else {
                            80
                        },
                        https,
                    )
                val request: HttpRequest =
                    HttpRequest.httpRequest(
                        service,
                        "GET " + (if (target.query != null) target.path + "?" + target.query else target.path) + " HTTP/1.1\r\nHost: " +
                            target.host +
                            "\r\nConnection: close\r\n\r\n",
                    )
                val requestResponse: HttpRequestResponse = api.http().sendRequest(request)
                val response: HttpResponse? = requestResponse.response()
                val body: String = response?.bodyToString() ?: ""
                var hit = false
                for (indicator: String in library[1]) {
                    if (body.lowercase(Locale.getDefault()).contains(indicator.lowercase(Locale.getDefault()))) {
                        hit = true
                        break
                    }
                }
                items.add(
                    JsonObject().apply {
                        addProperty("payload", payload)
                        addProperty("status", response?.statusCode() ?: 0)
                        addProperty("length", body.length)
                        addProperty("indicator_match", hit)
                    },
                )
            }
            result.add("results", items)
            result.addProperty("total", items.size())
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun accessControlSweep(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val service: HttpService =
                HttpService.httpService(
                    params.get("host").asString,
                    if (params.has("port")) params.get("port").asInt else 443,
                    if (params.has("https")) params.get("https").asBoolean else true,
                )
            val items = JsonArray()
            for (authorization: String in Pattern.compile("\\|").split(params.get("auth_headers").asString)) {
                val modified: String =
                    params.get("request").asString.replace(
                        Regex("(?i)Authorization: .*\r\n"),
                        if (authorization.isEmpty()) "" else "Authorization: $authorization\r\n",
                    )
                val requestResponse: HttpRequestResponse = api.http().sendRequest(HttpRequest.httpRequest(service, modified))
                val response: HttpResponse? = requestResponse.response()
                items.add(
                    JsonObject().apply {
                        addProperty("auth", if (authorization.isEmpty()) "none" else authorization)
                        addProperty("status", response?.statusCode() ?: 0)
                        addProperty("length", response?.bodyToString()?.length ?: 0)
                    },
                )
            }
            result.add("sweeps", items)
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun raceCondition(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val service: HttpService =
                HttpService.httpService(
                    params.get("host").asString,
                    if (params.has("port")) params.get("port").asInt else 443,
                    if (params.has("https")) params.get("https").asBoolean else true,
                )
            val request: HttpRequest = HttpRequest.httpRequest(service, params.get("request").asString)
            val count: Int = if (params.has("count")) params.get("count").asInt else 10
            val futures: MutableList<Future<HttpRequestResponse>> = ArrayList()
            for (index in 0 until count) futures.add(threadPool.submit<HttpRequestResponse> { api.http().sendRequest(request) })
            val items = JsonArray()
            val lengths: MutableSet<Int> = HashSet()
            for (future: Future<HttpRequestResponse> in futures) {
                try {
                    val requestResponse: HttpRequestResponse = future.get()
                    val response: HttpResponse? = requestResponse.response()
                    items.add(
                        JsonObject().apply {
                            addProperty("status", response?.statusCode() ?: 0)
                            addProperty("length", response?.bodyToString()?.length ?: 0)
                        },
                    )
                    if (response != null) lengths.add(response.bodyToString().length)
                } catch (_: Exception) {
                }
            }
            result.add("results", items)
            result.addProperty("total", count)
            result.addProperty("unique_lengths", lengths.size)
            result.addProperty("verdict", if (lengths.size > 1) "Possible race" else "All identical")
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun inlineFuzzer(params: JsonObject): JsonObject {
        val result = JsonObject()
        try {
            val service: HttpService =
                HttpService.httpService(
                    params.get("host").asString,
                    if (params.has("port")) params.get("port").asInt else 443,
                    if (params.has("https")) params.get("https").asBoolean else true,
                )
            val template: String = params.get("template").asString
            val marker: String = if (params.has("marker")) params.get("marker").asString else "FUZZ"
            val wordlist: JsonArray = params.getAsJsonArray("wordlist")
            val futures: MutableList<Future<HttpRequestResponse>> = ArrayList()
            for (index in 0 until wordlist.size()) {
                val payload: String = wordlist[index].asString
                futures.add(
                    threadPool.submit<HttpRequestResponse> {
                        api.http().sendRequest(HttpRequest.httpRequest(service, template.replace(marker, payload)))
                    },
                )
            }
            val items = JsonArray()
            for (future: Future<HttpRequestResponse> in futures) {
                try {
                    val response: HttpResponse? = future.get().response()
                    items.add(
                        JsonObject().apply {
                            addProperty("status", response?.statusCode() ?: 0)
                            addProperty("length", response?.bodyToString()?.length ?: 0)
                        },
                    )
                } catch (_: Exception) {
                }
            }
            result.add("results", items)
            result.addProperty("total", items.size())
        } catch (exception: Exception) {
            result.addProperty("error", exception.message)
        }
        return result
    }

    private fun scopeGate(params: JsonObject): JsonObject {
        val result = JsonObject()
        when (if (params.has("action")) params.get("action").asString else "") {
            "enable" -> {
                scopeGateEnabled = true
                addAuditLog("scope:enabled")
                result.addProperty("scope_gate", true)
            }

            "disable" -> {
                scopeGateEnabled = false
                addAuditLog("scope:disabled")
                result.addProperty("scope_gate", false)
            }

            else -> {
                result.addProperty("scope_gate", scopeGateEnabled)
            }
        }
        return result
    }

    private fun privacyMode(params: JsonObject): JsonObject {
        val result = JsonObject()
        when (if (params.has("mode")) params.get("mode").asString else "") {
            "strict" -> {
                privacyStrict = true
                addAuditLog("privacy:strict")
                result.addProperty("privacy", "strict")
            }

            "off" -> {
                privacyStrict = false
                addAuditLog("privacy:off")
                result.addProperty("privacy", "off")
            }

            else -> {
                result.addProperty("privacy", if (privacyStrict) "strict" else "off")
            }
        }
        return result
    }

    private fun auditLog(params: JsonObject): JsonObject {
        val result = JsonObject()
        val limit: Int = if (params.has("limit")) params.get("limit").asInt else 50
        val items = JsonArray()
        val start: Int = max(0, auditLogEntries.size - limit)
        for (index in start until auditLogEntries.size) items.add(auditLogEntries[index])
        result.add("entries", items)
        result.addProperty("total", items.size())
        return result
    }

    private fun getToolList(): JsonArray =
        JsonArray().apply {
            toolRegistry.advertisedNames().forEach(::add)
        }

    private fun addCorsHeaders(response: Response) {
        response.addHeader("Access-Control-Allow-Origin", "http://127.0.0.1")
        response.addHeader("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        response.addHeader("Access-Control-Allow-Headers", "Content-Type, Authorization")
        response.addHeader("Vary", "Origin")
    }
}
