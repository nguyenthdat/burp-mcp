package io.github.nguyenthdat.burpmcp

import org.bouncycastle.openssl.PEMParser
import org.bouncycastle.openssl.jcajce.JcaPEMKeyConverter
import java.nio.file.Files
import java.nio.file.attribute.PosixFilePermission
import java.security.cert.CertificateFactory
import java.security.cert.X509Certificate
import kotlin.io.path.createTempDirectory
import kotlin.io.path.inputStream
import kotlin.io.path.reader
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class TlsBundleManagerTest {
    @Test
    fun `generates CA server and client identities with constrained usages`() {
        val directory = createTempDirectory("burp-mcp-tls")
        val bundle = TlsBundleManager().generate(directory, listOf("burp-vm.test", "10.10.0.8"))
        val ca = certificate(bundle.caCertificate)
        val server = certificate(bundle.serverCertificate)
        val client = certificate(bundle.clientCertificate)

        assertTrue(ca.basicConstraints >= 0)
        assertEquals(listOf("1.3.6.1.5.5.7.3.1"), server.extendedKeyUsage)
        assertEquals(listOf("1.3.6.1.5.5.7.3.2"), client.extendedKeyUsage)
        assertTrue(server.subjectAlternativeNames.any { it[1] == "burp-vm.test" })
        assertTrue(server.subjectAlternativeNames.any { it[1] == "10.10.0.8" })
        server.verify(ca.publicKey)
        client.verify(ca.publicKey)
        PEMParser(bundle.clientPrivateKey.reader()).use { parser ->
            assertTrue(JcaPEMKeyConverter().getPrivateKey(parser.readObject() as org.bouncycastle.asn1.pkcs.PrivateKeyInfo).format == "PKCS#8")
        }
    }

    @Test
    fun `private keys are owner-only on POSIX filesystems`() {
        val bundle = TlsBundleManager().generate(createTempDirectory("burp-mcp-tls"), listOf("localhost"))
        runCatching { Files.getPosixFilePermissions(bundle.clientPrivateKey) }.getOrNull()?.let { permissions ->
            assertEquals(setOf(PosixFilePermission.OWNER_READ, PosixFilePermission.OWNER_WRITE), permissions)
        }
    }

    @Test
    fun `ensure regenerates when names change or certificates expire`() {
        val directory = createTempDirectory("burp-mcp-tls")
        val initial = TlsSettings(directory, listOf("first.test"))
        val generatedAt = java.time.Instant.now()
        val manager = TlsBundleManager(now = { generatedAt })
        val settings = GrpcSettings(
            securityMode = GrpcSecurityMode.REMOTE_MTLS,
            serverNames = initial.names,
            tlsDirectory = initial.directory,
        )
        manager.generate(directory, initial.names)
        val changed = settings.copy(serverNames = listOf("second.test"))
        val changedBundle = manager.ensure(changed)
        assertTrue(certificate(changedBundle.serverCertificate).subjectAlternativeNames.any { it[1] == "second.test" })
        val expiredManager = TlsBundleManager(now = { generatedAt.plus(91, java.time.temporal.ChronoUnit.DAYS) })
        val renewed = expiredManager.ensure(changed)
        assertTrue(certificate(renewed.serverCertificate).notAfter.toInstant().isAfter(generatedAt.plus(91, java.time.temporal.ChronoUnit.DAYS)))
    }

    private data class TlsSettings(val directory: java.nio.file.Path, val names: List<String>)

    private fun certificate(path: java.nio.file.Path): X509Certificate =
        path.inputStream().use {
            CertificateFactory.getInstance("X.509").generateCertificate(it) as X509Certificate
        }
}
