package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import com.google.gson.JsonParser
import java.lang.reflect.Proxy
import java.net.URI
import java.net.http.HttpClient
import java.net.http.HttpRequest
import java.net.http.HttpResponse
import java.nio.file.Path
import kotlin.io.path.createTempDirectory
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals

class McpHttpServerContractTest {
    private lateinit var server: McpHttpServer
    private lateinit var tempHome: Path
    private val client: HttpClient = HttpClient.newHttpClient()
    private var originalHome: String? = null
    private var originalToken: String? = null

    @BeforeTest
    fun startServer() {
        originalHome = System.getProperty("user.home")
        originalToken = System.getProperty("burp.mcp.token")
        tempHome = createTempDirectory("burp-mcp-test-home")
        System.setProperty("user.home", tempHome.toString())
        System.setProperty("burp.mcp.token", "test-token")
        val api =
            Proxy.newProxyInstance(
                MontoyaApi::class.java.classLoader,
                arrayOf(MontoyaApi::class.java),
            ) { _, method, _ ->
                throw AssertionError("Unexpected MontoyaApi call: ${method.name}")
            } as MontoyaApi
        server = McpHttpServer(api, 0)
        server.start(5_000, false)
    }

    @AfterTest
    fun stopServer() {
        server.stop()
        restoreProperty("user.home", originalHome)
        restoreProperty("burp.mcp.token", originalToken)
        tempHome.toFile().deleteRecursively()
    }

    @Test
    fun `serves the stable advertised tool contract`() {
        // Given
        val toolsRequest = authorizedRequest("/tools").GET().build()
        val healthRequest = authorizedRequest("/health").GET().build()

        // When
        val toolsResponse = client.send(toolsRequest, HttpResponse.BodyHandlers.ofString())
        val healthResponse = client.send(healthRequest, HttpResponse.BodyHandlers.ofString())

        // Then
        assertEquals(200, toolsResponse.statusCode())
        val names = JsonParser.parseString(toolsResponse.body()).asJsonArray.map { it.asString }
        assertEquals(EXPECTED_TOOL_NAMES, names)

        assertEquals(200, healthResponse.statusCode())
        val health = JsonParser.parseString(healthResponse.body()).asJsonObject
        assertEquals("ok", health.get("status").asString)
        assertEquals(names, health.getAsJsonArray("tools").map { it.asString })
    }

    @Test
    fun `returns advertised tools for an unknown dispatch name`() {
        // Given
        val request =
            authorizedRequest("/")
                .header("Content-Type", "application/json")
                .POST(HttpRequest.BodyPublishers.ofString("{\"tool\":\"missing\",\"params\":{}}"))
                .build()

        // When
        val response = client.send(request, HttpResponse.BodyHandlers.ofString())

        // Then
        assertEquals(200, response.statusCode())
        val result = JsonParser.parseString(response.body()).asJsonObject
        assertEquals("Unknown tool: missing", result.get("error").asString)
        assertEquals(80, result.getAsJsonArray("available_tools").size())
    }

    @Test
    fun `keeps legacy tools callable without advertising them`() {
        // Given
        val request =
            authorizedRequest("/")
                .header("Content-Type", "application/json")
                .POST(HttpRequest.BodyPublishers.ofString("{\"tool\":\"proxy_listeners\",\"params\":{}}"))
                .build()

        // When
        val response = client.send(request, HttpResponse.BodyHandlers.ofString())

        // Then
        assertEquals(200, response.statusCode())
        val result = JsonParser.parseString(response.body()).asJsonObject
        assertEquals(true, result.get("deprecated").asBoolean)
        val advertised =
            client
                .send(authorizedRequest("/tools").GET().build(), HttpResponse.BodyHandlers.ofString())
                .body()
        assertEquals(false, JsonParser.parseString(advertised).asJsonArray.any { it.asString == "proxy_listeners" })
    }

    private fun authorizedRequest(path: String): HttpRequest.Builder =
        HttpRequest
            .newBuilder(URI("http://127.0.0.1:${server.listeningPort}$path"))
            .header("Authorization", "Bearer test-token")

    private fun restoreProperty(
        name: String,
        value: String?,
    ) {
        if (value == null) {
            System.clearProperty(name)
        } else {
            System.setProperty(name, value)
        }
    }

    private companion object {
        val EXPECTED_TOOL_NAMES: List<String> =
            listOf(
                "proxy_history",
                "proxy_detail",
                "proxy_websocket",
                "proxy_clear",
                "proxy_history_filtered",
                "send_request",
                "send_to_repeater",
                "repeater_send",
                "repeater_modify_send",
                "send_to_intruder",
                "intruder_attack",
                "intruder_attack_async",
                "intruder_attack_wordlist",
                "intruder_pitchfork",
                "intruder_cluster_bomb",
                "intruder_battering_ram",
                "intruder_with_options",
                "sitemap",
                "target_info",
                "intercept_toggle",
                "encode",
                "decode",
                "convert_request",
                "export_request",
                "generate_csrf_poc",
                "extract_from_response",
                "payload_process",
                "scan",
                "scan_active",
                "bambda_import",
                "bcheck_import",
                "scan_results",
                "scan_issue_detail",
                "crawl",
                "get_scope",
                "add_to_scope",
                "remove_from_scope",
                "collaborator_generate",
                "collaborator_poll",
                "search_history",
                "highlight",
                "annotate",
                "compare",
                "export_config",
                "import_config",
                "set_upstream_proxy",
                "set_dns_override",
                "set_http2",
                "cookie_jar",
                "token_analysis",
                "sequencer",
                "save_project",
                "burp_version",
                "add_issue",
                "register_http_handler",
                "remove_http_handler",
                "register_proxy_rule",
                "remove_proxy_rule",
                "extensions_list",
                "log",
                "cookie_jar_set",
                "send_request_parallel",
                "websocket_create",
                "websocket_send_text",
                "websocket_send_binary",
                "websocket_close",
                "websocket_list",
                "passive_intel",
                "session_create_rule",
                "session_list_rules",
                "session_remove_rule",
                "jwt_decode",
                "jwt_attack",
                "injection_probe",
                "access_control_sweep",
                "race_condition",
                "inline_fuzzer",
                "scope_gate",
                "privacy_mode",
                "audit_log",
            )
    }
}
