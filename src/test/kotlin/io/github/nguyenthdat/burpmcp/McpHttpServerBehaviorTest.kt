package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.core.Annotations
import burp.api.montoya.core.ByteArray
import burp.api.montoya.core.HighlightColor
import burp.api.montoya.http.Http
import burp.api.montoya.http.message.Cookie
import burp.api.montoya.http.sessions.CookieJar
import burp.api.montoya.proxy.ProxyHttpRequestResponse
import burp.api.montoya.sitemap.SiteMap
import com.google.gson.JsonObject
import com.google.gson.JsonParser
import java.lang.reflect.Method
import java.lang.reflect.Proxy
import java.net.URI
import java.net.http.HttpClient
import java.net.http.HttpRequest
import java.net.http.HttpResponse
import java.nio.file.Path
import java.time.ZonedDateTime
import java.util.Optional
import kotlin.io.path.createTempDirectory
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue
import burp.api.montoya.http.message.requests.HttpRequest as MontoyaHttpRequest
import burp.api.montoya.http.message.responses.HttpResponse as MontoyaHttpResponse
import burp.api.montoya.proxy.Proxy as MontoyaProxy

class McpHttpServerBehaviorTest {
    private val client: HttpClient = HttpClient.newHttpClient()
    private lateinit var tempHome: Path
    private var originalHome: String? = null
    private var originalToken: String? = null

    @BeforeTest
    fun configureToken() {
        originalHome = System.getProperty("user.home")
        originalToken = System.getProperty("burp.mcp.token")
        tempHome = createTempDirectory("burp-mcp-behavior-test-home")
        System.setProperty("user.home", tempHome.toString())
        System.setProperty("burp.mcp.token", "test-token")
    }

    @AfterTest
    fun restoreToken() {
        restoreProperty("user.home", originalHome)
        restoreProperty("burp.mcp.token", originalToken)
        tempHome.toFile().deleteRecursively()
    }

    @Test
    fun `proxy history returns source indices accepted by proxy detail`() {
        // Given
        val history =
            listOf(
                historyEntry("https://old.example/", "old"),
                historyEntry("https://new.example/", "new"),
            )
        val api = apiWith(proxy = fakeProxy(history))

        // When
        withServer(api) { server ->
            val listed = callTool(server, "proxy_history", "{\"limit\":1}")
            val item = listed.getAsJsonArray("items")[0].asJsonObject
            val detailed = callTool(server, "proxy_detail", "{\"index\":${item.get("index").asInt}}")

            // Then
            assertEquals("https://new.example/", item.get("url").asString)
            assertEquals("https://new.example/", detailed.get("url").asString)
        }
    }

    @Test
    fun `proxy history filtered applies highlight color`() {
        // Given
        val history =
            listOf(
                historyEntry("https://blue.example/", "blue", HighlightColor.BLUE),
                historyEntry("https://red.example/", "red", HighlightColor.RED),
            )
        val api = apiWith(proxy = fakeProxy(history))

        // When
        withServer(api) { server ->
            val result = callTool(server, "proxy_history_filtered", "{\"color\":\"red\"}")

            // Then
            assertEquals(1, result.get("matches").asInt)
            assertEquals(
                "https://red.example/",
                result
                    .getAsJsonArray("items")[0]
                    .asJsonObject
                    .get("url")
                    .asString,
            )
        }
    }

    @Test
    fun `extract from response bounds returned matches`() {
        // Given
        val api = apiWith(proxy = fakeProxy(listOf(historyEntry("https://matches.example/", "aaaaa"))))

        // When
        withServer(api) { server ->
            val result = callTool(server, "extract_from_response", "{\"index\":0,\"regex\":\"a\",\"limit\":3}")

            // Then
            assertEquals(3, result.get("total_matches").asInt)
            assertEquals(3, result.getAsJsonArray("matches").size())
            assertTrue(result.get("truncated").asBoolean)
        }
    }

    @Test
    fun `scan issue detail reports out of range index`() {
        // Given
        val siteMap = fake<SiteMap>(mapOf("issues" to { emptyList<Any>() }))
        val api = apiWith(siteMap = siteMap)

        // When
        withServer(api) { server ->
            val result = callTool(server, "scan_issue_detail", "{\"index\":0}")

            // Then
            assertEquals("Index out of range", result.get("error").asString)
        }
    }

    @Test
    fun `cookie jar set passes name value path and URL host in Montoya order`() {
        // Given
        var captured: List<Any?> = emptyList()
        val cookieJar =
            fake<CookieJar>(
                mapOf(
                    "setCookie" to { arguments ->
                        captured = arguments.toList()
                        null
                    },
                ),
            )
        val http = fake<Http>(mapOf("cookieJar" to { cookieJar }))
        val api = apiWith(http = http)

        // When
        withServer(api) { server ->
            val result =
                callTool(
                    server,
                    "cookie_jar_set",
                    "{\"url\":\"https://example.org/login\",\"name\":\"session\",\"value\":\"abc\"}",
                )

            // Then
            assertTrue(result.get("success").asBoolean)
            assertEquals("session", captured[0])
            assertEquals("abc", captured[1])
            assertEquals("/", captured[2])
            assertEquals("example.org", captured[3])
            assertTrue(captured[4] is ZonedDateTime)
        }
    }

    @Test
    fun `cookie jar lists cookies and filters by domain`() {
        // Given
        val expiration = ZonedDateTime.parse("2030-01-01T00:00:00Z")
        val cookieJar =
            fake<CookieJar>(
                mapOf(
                    "cookies" to {
                        listOf(
                            cookie("session", "abc", "example.org", "/", expiration),
                            cookie("other", "def", "other.org", "/", null),
                        )
                    },
                ),
            )
        val api = apiWith(http = fake<Http>(mapOf("cookieJar" to { cookieJar })))

        // When
        withServer(api) { server ->
            val result = callTool(server, "cookie_jar", "{\"domain\":\"example.org\"}")
            val item = result.getAsJsonArray("cookies")[0].asJsonObject

            // Then
            assertEquals(1, result.get("total").asInt)
            assertEquals("session", item.get("name").asString)
            assertEquals("abc", item.get("value").asString)
            assertEquals("example.org", item.get("domain").asString)
            assertEquals("/", item.get("path").asString)
            assertEquals(expiration.toString(), item.get("expiration").asString)
        }
    }

    @Test
    fun `encodes non ASCII input as UTF-8`() {
        // Given
        val api = apiWith()

        // When
        withServer(api) { server ->
            val base64 = callTool(server, "encode", "{\"input\":\"\\u2713\",\"type\":\"base64\"}")
            val hex = callTool(server, "encode", "{\"input\":\"\\u2713\",\"type\":\"hex\"}")

            // Then
            assertEquals("4pyT", base64.get("output").asString)
            assertEquals("e29c93", hex.get("output").asString)
        }
    }

    @Test
    fun `exports Python requests code when requested`() {
        // Given
        val api = apiWith()

        // When
        withServer(api) { server ->
            val result =
                callTool(
                    server,
                    "export_request",
                    "{\"request\":\"POST /qa HTTP/1.1\\r\\nHost: example.org\\r\\nX-QA: value\\r\\nContent-Length: 6\\r\\n\\r\\n\\u2713new\",\"format\":\"python\"}",
                )

            // Then
            assertTrue(result.has("python"))
            assertFalse(result.has("curl"))
            val code = result.get("python").asString
            assertTrue(code.contains("requests.request(\"POST\", \"https://example.org/qa\""))
            assertTrue(code.contains("headers={\"X-QA\":\"value\"}"))
            assertTrue(code.contains("data=\"✓new\""), code)
        }
    }

    @Test
    fun `exports curl with shell-safe literals`() {
        // Given
        val api = apiWith()

        // When
        withServer(api) { server ->
            val result =
                callTool(
                    server,
                    "export_request",
                    "{\"request\":\"POST${'$'}(touch${'$'}{IFS}/tmp/nope) /it's HTTP/1.1\\r\\nHost: example.org\\r\\nX-QA: `whoami`\\r\\nContent-Length: 11\\r\\n\\r\\nline1\\n${'$'}HOME\",\"format\":\"curl\"}",
                )

            // Then
            val expectedUrl = "https:" + "//example.org/it'\\''s"
            assertEquals(
                "curl -X 'POST${'$'}(touch${'$'}{IFS}/tmp/nope)' '$expectedUrl' -H 'X-QA: `whoami`' --data-raw 'line1\n${'$'}HOME'",
                result.get("curl").asString,
            )
        }
    }

    private fun historyEntry(
        url: String,
        body: String,
        color: HighlightColor = HighlightColor.NONE,
    ): ProxyHttpRequestResponse {
        val request =
            fake<MontoyaHttpRequest>(
                mapOf(
                    "url" to { url },
                    "method" to { "GET" },
                    "toString" to { "GET $url HTTP/1.1\r\n\r\n" },
                ),
            )
        val response = response(body)
        val annotations =
            fake<Annotations>(
                mapOf(
                    "notes" to { "" },
                    "highlightColor" to { color },
                ),
            )
        return fake(
            mapOf(
                "finalRequest" to { request },
                "response" to { response },
                "annotations" to { annotations },
            ),
        )
    }

    private fun response(body: String): MontoyaHttpResponse {
        val bytes = fake<ByteArray>(mapOf("length" to { body.toByteArray().size }))
        return fake(
            mapOf(
                "bodyToString" to { body },
                "body" to { bytes },
                "statusCode" to { 200.toShort() },
                "toString" to { "HTTP/1.1 200 OK\r\n\r\n$body" },
            ),
        )
    }

    private fun cookie(
        name: String,
        value: String,
        domain: String,
        path: String,
        expiration: ZonedDateTime?,
    ): Cookie =
        fake(
            mapOf(
                "name" to { name },
                "value" to { value },
                "domain" to { domain },
                "path" to { path },
                "expiration" to { Optional.ofNullable(expiration) },
            ),
        )

    private fun fakeProxy(history: List<ProxyHttpRequestResponse>): MontoyaProxy = fake(mapOf("history" to { history }))

    private fun apiWith(
        proxy: MontoyaProxy? = null,
        http: Http? = null,
        siteMap: SiteMap? = null,
    ): MontoyaApi =
        fake(
            buildMap {
                if (proxy != null) put("proxy") { proxy }
                if (http != null) put("http") { http }
                if (siteMap != null) put("siteMap") { siteMap }
            },
        )

    private fun withServer(
        api: MontoyaApi,
        block: (McpHttpServer) -> Unit,
    ) {
        val server = McpHttpServer(api, 0)
        server.start(5_000, false)
        try {
            block(server)
        } finally {
            server.stop()
        }
    }

    private fun callTool(
        server: McpHttpServer,
        tool: String,
        params: String,
    ): JsonObject {
        val request =
            HttpRequest
                .newBuilder(URI("http://127.0.0.1:${server.listeningPort}/"))
                .header("Authorization", "Bearer test-token")
                .header("Content-Type", "application/json")
                .POST(HttpRequest.BodyPublishers.ofString("{\"tool\":\"$tool\",\"params\":$params}"))
                .build()
        val response = client.send(request, HttpResponse.BodyHandlers.ofString())
        assertEquals(200, response.statusCode())
        return JsonParser.parseString(response.body()).asJsonObject
    }

    private inline fun <reified T> fake(handlers: Map<String, (Array<out Any?>) -> Any?>): T =
        Proxy.newProxyInstance(
            T::class.java.classLoader,
            arrayOf(T::class.java),
        ) { instance, method, arguments ->
            invokeFake(instance, method, arguments ?: emptyArray(), handlers)
        } as T

    private fun invokeFake(
        instance: Any,
        method: Method,
        arguments: Array<out Any?>,
        handlers: Map<String, (Array<out Any?>) -> Any?>,
    ): Any? =
        if (handlers.containsKey(method.name)) {
            handlers.getValue(method.name).invoke(arguments)
        } else {
            when (method.name) {
                "equals" -> instance === arguments.firstOrNull()
                "hashCode" -> System.identityHashCode(instance)
                "toString" -> "Fake${method.declaringClass.simpleName}"
                else -> throw AssertionError("Unexpected ${method.declaringClass.simpleName} call: ${method.name}")
            }
        }

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
}
