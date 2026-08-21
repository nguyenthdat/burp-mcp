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
            JsonParser
                .parseReader(
                    requireNotNull(
                        McpHttpServerContractTest::class.java.getResourceAsStream("/contracts/burp-tool-names.json"),
                    ) { "missing v2 Burp tool contract fixture" }.bufferedReader(),
                ).asJsonObject
                .getAsJsonArray("tools")
                .map { it.asString }
    }
}
