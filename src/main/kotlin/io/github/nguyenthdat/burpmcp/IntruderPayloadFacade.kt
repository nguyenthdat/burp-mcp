package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.core.ByteArray
import burp.api.montoya.core.Registration
import burp.api.montoya.intruder.AttackConfiguration
import burp.api.montoya.intruder.GeneratedPayload
import burp.api.montoya.intruder.IntruderInsertionPoint
import burp.api.montoya.intruder.PayloadData
import burp.api.montoya.intruder.PayloadGenerator
import burp.api.montoya.intruder.PayloadGeneratorProvider
import burp.api.montoya.intruder.PayloadProcessingResult
import burp.api.montoya.intruder.PayloadProcessor
import com.google.re2j.Pattern
import java.net.URLEncoder
import java.nio.charset.StandardCharsets
import java.security.MessageDigest
import java.util.Base64

internal const val MAX_INTRUDER_PAYLOADS = 500
private const val MAX_PAYLOAD_BYTES = 64 * 1024

internal data class PayloadProcessorRegistration(
    val id: String,
    val displayName: String,
    val operation: String,
    val registered: Boolean,
)

internal data class PayloadGeneratorRegistration(
    val id: String,
    val displayName: String,
    val payloadCount: Int,
    val maxOutputCount: Int,
    val registered: Boolean,
)

internal enum class PayloadProcessorOperation(val wireName: String) {
    PREFIX("prefix"),
    SUFFIX("suffix"),
    REGEX_REPLACE("regex_replace"),
    UPPERCASE("uppercase"),
    LOWERCASE("lowercase"),
    URL_ENCODE("url_encode"),
    BASE64_ENCODE("base64_encode"),
    BASE64_DECODE("base64_decode"),
    HEX_ENCODE("hex_encode"),
    HEX_DECODE("hex_decode"),
    SHA256("sha256"),
    SKIP_REGEX("skip_regex"),
    ;

    companion object {
        fun parse(value: String): PayloadProcessorOperation =
            entries.firstOrNull { it.wireName == value }
                ?: throw IllegalArgumentException(
                    "operation must be one of ${entries.joinToString { it.wireName }}",
                )
    }
}

internal data class PayloadProcessorSpec(
    val id: String,
    val displayName: String,
    val operation: PayloadProcessorOperation,
    val argument: String,
    val replacement: String,
) {
    fun validate() {
        require(id.isNotBlank()) { "id must not be blank" }
        require(displayName.isNotBlank()) { "display_name must not be blank" }
        require(displayName.length <= 120) { "display_name must contain at most 120 characters" }
        require(argument.toByteArray(StandardCharsets.UTF_8).size <= MAX_PAYLOAD_BYTES) { "argument must contain at most $MAX_PAYLOAD_BYTES UTF-8 bytes" }
        require(replacement.toByteArray(StandardCharsets.UTF_8).size <= MAX_PAYLOAD_BYTES) { "replacement must contain at most $MAX_PAYLOAD_BYTES UTF-8 bytes" }
        when (operation) {
            PayloadProcessorOperation.PREFIX,
            PayloadProcessorOperation.SUFFIX,
            -> require(argument.isNotEmpty()) { "argument is required for ${operation.wireName}" }
            PayloadProcessorOperation.REGEX_REPLACE -> {
                require(argument.isNotEmpty()) { "argument is required for regex_replace" }
                compilePayloadPattern(argument)
            }
            PayloadProcessorOperation.SKIP_REGEX -> {
                require(argument.isNotEmpty()) { "argument is required for skip_regex" }
                compilePayloadPattern(argument)
            }
            else -> require(argument.isEmpty() && replacement.isEmpty()) {
                "argument and replacement are not accepted for ${operation.wireName}"
            }
        }
    }
}

internal data class PayloadGeneratorSpec(
    val id: String,
    val displayName: String,
    val payloads: List<String>,
    val maxOutputCount: Int,
) {
    fun validate() {
        require(id.isNotBlank()) { "id must not be blank" }
        require(displayName.isNotBlank()) { "display_name must not be blank" }
        require(displayName.length <= 120) { "display_name must contain at most 120 characters" }
        require(payloads.isNotEmpty()) { "payloads must contain at least one entry" }
        require(payloads.size <= MAX_INTRUDER_PAYLOADS) { "payloads must contain at most $MAX_INTRUDER_PAYLOADS entries" }
        require(maxOutputCount in 1..MAX_INTRUDER_PAYLOADS) { "max_output_count must be between 1 and $MAX_INTRUDER_PAYLOADS" }
        require(maxOutputCount <= payloads.size) { "max_output_count must not exceed payloads size" }
        require(payloads.all { it.toByteArray(StandardCharsets.UTF_8).size <= MAX_PAYLOAD_BYTES }) {
            "each payload must contain at most $MAX_PAYLOAD_BYTES UTF-8 bytes"
        }
    }
}

internal class IntruderPayloadFacade(
    private val api: MontoyaApi,
) : AutoCloseable {
    private data class ProcessorRecord(
        val spec: PayloadProcessorSpec,
        val registration: Registration,
    )

    private data class GeneratorRecord(
        val spec: PayloadGeneratorSpec,
        val registration: Registration,
    )

    private val processors = linkedMapOf<String, ProcessorRecord>()
    private val generators = linkedMapOf<String, GeneratorRecord>()

    @Synchronized
    fun registerProcessor(spec: PayloadProcessorSpec): PayloadProcessorRegistration {
        spec.validate()
        require(!processors.containsKey(spec.id)) { "payload processor id already exists" }
        val registration = api.intruder().registerPayloadProcessor(DeclarativePayloadProcessor(spec))
        processors[spec.id] = ProcessorRecord(spec, registration)
        return processorRegistration(spec, registration)
    }

    @Synchronized
    fun listProcessors(): List<PayloadProcessorRegistration> =
        processors.values.map { record -> processorRegistration(record.spec, record.registration) }

    @Synchronized
    fun removeProcessor(id: String): Boolean {
        require(id.isNotBlank()) { "id must not be blank" }
        val record = processors.remove(id) ?: return false
        record.registration.deregister()
        return true
    }

    @Synchronized
    fun registerGenerator(spec: PayloadGeneratorSpec): PayloadGeneratorRegistration {
        spec.validate()
        require(!generators.containsKey(spec.id)) { "payload generator id already exists" }
        val registration = api.intruder().registerPayloadGeneratorProvider(BoundedPayloadGeneratorProvider(spec))
        generators[spec.id] = GeneratorRecord(spec, registration)
        return generatorRegistration(spec, registration)
    }

    @Synchronized
    fun listGenerators(): List<PayloadGeneratorRegistration> =
        generators.values.map { record -> generatorRegistration(record.spec, record.registration) }

    @Synchronized
    fun removeGenerator(id: String): Boolean {
        require(id.isNotBlank()) { "id must not be blank" }
        val record = generators.remove(id) ?: return false
        record.registration.deregister()
        return true
    }

    override fun close() {
        synchronized(this) {
            processors.values.forEach { it.registration.deregister() }
            generators.values.forEach { it.registration.deregister() }
            processors.clear()
            generators.clear()
        }
    }

    private fun processorRegistration(spec: PayloadProcessorSpec, registration: Registration) =
        PayloadProcessorRegistration(spec.id, spec.displayName, spec.operation.wireName, registration.isRegistered)

    private fun generatorRegistration(spec: PayloadGeneratorSpec, registration: Registration) =
        PayloadGeneratorRegistration(spec.id, spec.displayName, spec.payloads.size, spec.maxOutputCount, registration.isRegistered)
}

internal class DeclarativePayloadProcessor(
    private val spec: PayloadProcessorSpec,
) : PayloadProcessor {
    override fun displayName(): String = spec.displayName

    override fun processPayload(payloadData: PayloadData): PayloadProcessingResult {
        val current = payloadData.currentPayload().bytes
        val currentText = current.toString(StandardCharsets.UTF_8)
        if (spec.operation == PayloadProcessorOperation.SKIP_REGEX) {
            return if (compilePayloadPattern(spec.argument).matcher(currentText).find()) {
                PayloadProcessingResult.skipPayload()
            } else {
                PayloadProcessingResult.usePayload(ByteArray.byteArray(*current))
            }
        }
        val transformed = transformPayload(spec, current, currentText)
        require(transformed.size <= MAX_PAYLOAD_BYTES) { "processed payload exceeds $MAX_PAYLOAD_BYTES bytes" }
        return PayloadProcessingResult.usePayload(ByteArray.byteArray(*transformed))
    }
}

internal class BoundedPayloadGeneratorProvider(
    private val spec: PayloadGeneratorSpec,
) : PayloadGeneratorProvider {
    override fun displayName(): String = spec.displayName

    override fun providePayloadGenerator(attackConfiguration: AttackConfiguration): PayloadGenerator =
        BoundedPayloadGenerator(spec.payloads, spec.maxOutputCount)
}

internal class BoundedPayloadGenerator(
    private val payloads: List<String>,
    private val maxOutputCount: Int,
) : PayloadGenerator {
    private var index = 0

    override fun generatePayloadFor(insertionPoint: IntruderInsertionPoint): GeneratedPayload {
        if (index >= maxOutputCount || index >= payloads.size) return GeneratedPayload.end()
        return GeneratedPayload.payload(payloads[index++])
    }
}

internal fun transformPayload(
    spec: PayloadProcessorSpec,
    current: kotlin.ByteArray,
    currentText: String,
): kotlin.ByteArray =
    when (spec.operation) {
        PayloadProcessorOperation.PREFIX -> (spec.argument + currentText).toByteArray(StandardCharsets.UTF_8)
        PayloadProcessorOperation.SUFFIX -> (currentText + spec.argument).toByteArray(StandardCharsets.UTF_8)
        PayloadProcessorOperation.REGEX_REPLACE ->
            compilePayloadPattern(spec.argument).matcher(currentText).replaceAll(spec.replacement).toByteArray(StandardCharsets.UTF_8)
        PayloadProcessorOperation.UPPERCASE -> currentText.uppercase().toByteArray(StandardCharsets.UTF_8)
        PayloadProcessorOperation.LOWERCASE -> currentText.lowercase().toByteArray(StandardCharsets.UTF_8)
        PayloadProcessorOperation.URL_ENCODE -> URLEncoder.encode(currentText, StandardCharsets.UTF_8).replace("+", "%20").toByteArray(StandardCharsets.UTF_8)
        PayloadProcessorOperation.BASE64_ENCODE -> Base64.getEncoder().encode(current)
        PayloadProcessorOperation.BASE64_DECODE -> Base64.getDecoder().decode(currentText)
        PayloadProcessorOperation.HEX_ENCODE -> current.joinToString("") { byte -> "%02x".format(byte.toInt() and 0xff) }.toByteArray(StandardCharsets.US_ASCII)
        PayloadProcessorOperation.HEX_DECODE -> decodeHex(currentText)
        PayloadProcessorOperation.SHA256 -> MessageDigest.getInstance("SHA-256").digest(current)
            .joinToString("") { byte -> "%02x".format(byte.toInt() and 0xff) }
            .toByteArray(StandardCharsets.US_ASCII)
        PayloadProcessorOperation.SKIP_REGEX -> error("skip_regex is handled before transformation")
    }

private fun decodeHex(value: String): kotlin.ByteArray {
    require(value.length % 2 == 0) { "hex payload must contain an even number of characters" }
    return kotlin.ByteArray(value.length / 2) { index ->
        value.substring(index * 2, index * 2 + 2).toInt(16).toByte()
    }
}
private fun compilePayloadPattern(value: String): Pattern =
    try {
        Pattern.compile(value)
    } catch (exception: RuntimeException) {
        throw IllegalArgumentException("invalid RE2 regular expression: ${exception.message}", exception)
    }
