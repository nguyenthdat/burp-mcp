package io.github.nguyenthdat.burpmcp

import burp.api.montoya.core.ByteArray
import burp.api.montoya.http.HttpService
import burp.api.montoya.http.message.requests.HttpRequest
import java.nio.charset.StandardCharsets

internal fun utf8Length(value: String): Int = value.toByteArray(StandardCharsets.UTF_8).size

internal fun shellQuote(value: String): String = "'${value.replace("'", "'\\''")}'"

internal fun httpRequestUtf8(rawRequest: String): HttpRequest = attachUtf8Body(HttpRequest.httpRequest(requestHead(rawRequest)), rawRequest)

internal fun httpRequestUtf8(
    service: HttpService,
    rawRequest: String,
): HttpRequest = attachUtf8Body(HttpRequest.httpRequest(service, requestHead(rawRequest)), rawRequest)

private fun attachUtf8Body(
    request: HttpRequest,
    rawRequest: String,
): HttpRequest {
    val separatorIndex: Int = rawRequest.indexOf("\r\n\r\n")
    if (separatorIndex < 0 || separatorIndex + 4 >= rawRequest.length) return request
    val body: String = rawRequest.substring(separatorIndex + 4)
    return request.withBody(ByteArray.byteArray(*body.toByteArray(StandardCharsets.UTF_8)))
}

private fun requestHead(rawRequest: String): String {
    val separatorIndex: Int = rawRequest.indexOf("\r\n\r\n")
    return if (separatorIndex < 0) rawRequest else rawRequest.substring(0, separatorIndex + 4)
}

internal fun replaceRequestBody(
    rawRequest: String,
    body: String,
): String {
    val bodyStart: Int = rawRequest.indexOf("\r\n\r\n")
    if (bodyStart < 0) return rawRequest
    val headers: String = rawRequest.substring(0, bodyStart)
    val contentLength = "Content-Length: ${utf8Length(body)}"
    val contentLengthPattern = Regex("(?im)^Content-Length:\\s*\\d+\\s*$")
    val updatedHeaders: String =
        if (contentLengthPattern.containsMatchIn(headers)) {
            headers.replace(contentLengthPattern, contentLength)
        } else {
            "$headers\r\n$contentLength"
        }
    return "$updatedHeaders\r\n\r\n$body"
}
