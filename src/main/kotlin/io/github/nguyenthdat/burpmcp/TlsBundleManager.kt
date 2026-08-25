package io.github.nguyenthdat.burpmcp

import org.bouncycastle.asn1.x500.X500Name
import org.bouncycastle.asn1.x509.BasicConstraints
import org.bouncycastle.asn1.x509.ExtendedKeyUsage
import org.bouncycastle.asn1.x509.Extension
import org.bouncycastle.asn1.x509.GeneralName
import org.bouncycastle.asn1.x509.GeneralNames
import org.bouncycastle.asn1.x509.KeyPurposeId
import org.bouncycastle.asn1.x509.KeyUsage
import org.bouncycastle.cert.X509CertificateHolder
import org.bouncycastle.cert.jcajce.JcaX509CertificateConverter
import org.bouncycastle.cert.jcajce.JcaX509ExtensionUtils
import org.bouncycastle.cert.jcajce.JcaX509v3CertificateBuilder
import org.bouncycastle.openssl.jcajce.JcaPEMWriter
import org.bouncycastle.openssl.jcajce.JcaPKCS8Generator
import org.bouncycastle.operator.jcajce.JcaContentSignerBuilder
import java.io.StringWriter
import java.math.BigInteger
import java.nio.charset.StandardCharsets
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.StandardCopyOption
import java.nio.file.attribute.PosixFilePermission
import java.security.KeyPair
import java.security.KeyPairGenerator
import java.security.PrivateKey
import java.security.SecureRandom
import java.security.cert.X509Certificate
import java.security.spec.ECGenParameterSpec
import java.time.Instant
import java.time.temporal.ChronoUnit
import java.util.Date

internal data class TlsBundle(
    val directory: Path,
    val caCertificate: Path,
    val serverCertificate: Path,
    val serverPrivateKey: Path,
    val clientCertificate: Path,
    val clientPrivateKey: Path,
)

internal class TlsBundleManager(
    private val random: SecureRandom = SecureRandom(),
    private val now: () -> Instant = Instant::now,
) {
    fun ensure(settings: GrpcSettings): TlsBundle {
        require(settings.securityMode == GrpcSecurityMode.REMOTE_MTLS) { "TLS bundle is only used in remote mTLS mode" }
        val bundle = paths(settings.tlsDirectory)
        if (bundle.requiredFiles().all(Files::isRegularFile) && bundle.privateFilesSecure() && bundle.matches(settings.serverNames)) return bundle
        return generate(settings.tlsDirectory, settings.serverNames)
    }

    fun generate(directory: Path, serverNames: List<String>): TlsBundle {
        require(serverNames.isNotEmpty()) { "At least one server DNS name or IP address is required" }
        Files.createDirectories(directory)
        secureDirectory(directory)

        val now = now()
        val caNotAfter = now.plus(1825, ChronoUnit.DAYS)
        val leafNotAfter = now.plus(90, ChronoUnit.DAYS)
        val caKeys = keyPair()
        val serverKeys = keyPair()
        val clientKeys = keyPair()
        val ca = createCa(caKeys, now, caNotAfter)
        val server = createLeaf(ca, caKeys.private, serverKeys, "Burp MCP server", serverNames, KeyPurposeId.id_kp_serverAuth, now, leafNotAfter)
            .also { it.verify(ca.publicKey) }
        val client = createLeaf(ca, caKeys.private, clientKeys, "Burp MCP client", emptyList(), KeyPurposeId.id_kp_clientAuth, now, leafNotAfter)
            .also { it.verify(ca.publicKey) }
        val bundle = paths(directory)

        writePem(bundle.caCertificate, ca, false)
        writePem(bundle.serverCertificate, server, false)
        writePem(bundle.serverPrivateKey, JcaPKCS8Generator(serverKeys.private, null), true)
        writePem(bundle.clientCertificate, client, false)
        writePem(bundle.clientPrivateKey, JcaPKCS8Generator(clientKeys.private, null), true)
        writeManifest(directory.resolve(MANIFEST_FILE), serverNames, caNotAfter, leafNotAfter)
        return bundle
    }

    private fun createCa(keys: KeyPair, notBefore: Instant, notAfter: Instant): X509Certificate {
        val subject = X500Name("CN=Burp MCP local CA")
        val builder = JcaX509v3CertificateBuilder(subject, serial(), Date.from(notBefore.minusSeconds(300)), Date.from(notAfter), subject, keys.public)
        val extensions = JcaX509ExtensionUtils()
        builder.addExtension(Extension.basicConstraints, true, BasicConstraints(0))
        builder.addExtension(Extension.keyUsage, true, KeyUsage(KeyUsage.keyCertSign or KeyUsage.cRLSign))
        builder.addExtension(Extension.subjectKeyIdentifier, false, extensions.createSubjectKeyIdentifier(keys.public))
        builder.addExtension(Extension.authorityKeyIdentifier, false, extensions.createAuthorityKeyIdentifier(keys.public))
        return sign(builder, keys.private)
    }

    private fun createLeaf(
        ca: X509Certificate,
        caPrivateKey: PrivateKey,
        keys: KeyPair,
        commonName: String,
        serverNames: List<String>,
        purpose: KeyPurposeId,
        notBefore: Instant,
        notAfter: Instant,
    ): X509Certificate {
        val builder = JcaX509v3CertificateBuilder(
            X509CertificateHolder(ca.encoded).subject,
            serial(),
            Date.from(notBefore.minusSeconds(300)),
            Date.from(notAfter),
            X500Name("CN=$commonName"),
            keys.public,
        )
        val extensions = JcaX509ExtensionUtils()
        builder.addExtension(Extension.basicConstraints, true, BasicConstraints(false))
        builder.addExtension(Extension.keyUsage, true, KeyUsage(KeyUsage.digitalSignature))
        builder.addExtension(Extension.extendedKeyUsage, false, ExtendedKeyUsage(purpose))
        builder.addExtension(Extension.subjectKeyIdentifier, false, extensions.createSubjectKeyIdentifier(keys.public))
        builder.addExtension(Extension.authorityKeyIdentifier, false, extensions.createAuthorityKeyIdentifier(ca))
        if (serverNames.isNotEmpty()) {
            builder.addExtension(Extension.subjectAlternativeName, false, GeneralNames(serverNames.map(::generalName).toTypedArray()))
        }
        return sign(builder, caPrivateKey)
    }

    private fun sign(builder: JcaX509v3CertificateBuilder, key: PrivateKey): X509Certificate {
        val holder = builder.build(JcaContentSignerBuilder("SHA256withRSA").build(key))
        return JcaX509CertificateConverter().getCertificate(holder).also {
            if (it.basicConstraints >= 0) it.verify(it.publicKey)
        }
    }

    private fun generalName(value: String): GeneralName =
        if (runCatching { java.net.InetAddress.getByName(value).hostAddress == value || value.contains(':') }.getOrDefault(false) || IPV4.matches(value)) {
            GeneralName(GeneralName.iPAddress, value)
        } else {
            require(HOSTNAME.matches(value)) { "Invalid certificate DNS name or IP address: $value" }
            GeneralName(GeneralName.dNSName, value)
        }

    private fun keyPair(): KeyPair = KeyPairGenerator.getInstance("RSA").apply {
        initialize(3072, random)
    }.generateKeyPair()

    private fun serial(): BigInteger = BigInteger(128, random).setBit(127).abs()

    private fun writePem(path: Path, value: Any, privateFile: Boolean) {
        val output = StringWriter()
        JcaPEMWriter(output).use { it.writeObject(value) }
        writeAtomically(path, output.toString().toByteArray(StandardCharsets.US_ASCII), privateFile)
    }

    private fun writeManifest(path: Path, names: List<String>, caExpiry: Instant, leafExpiry: Instant) {
        val body = "server_names=${names.joinToString(",")}\nca_not_after=$caExpiry\nleaf_not_after=$leafExpiry\n"
        writeAtomically(path, body.toByteArray(StandardCharsets.UTF_8), false)
    }

    private fun writeAtomically(path: Path, bytes: ByteArray, privateFile: Boolean) {
        val temp = Files.createTempFile(path.parent, ".${path.fileName}.", ".tmp")
        try {
            Files.write(temp, bytes)
            setPermissions(temp, if (privateFile) PRIVATE_FILE_PERMISSIONS else PUBLIC_FILE_PERMISSIONS)
            runCatching { Files.move(temp, path, StandardCopyOption.ATOMIC_MOVE, StandardCopyOption.REPLACE_EXISTING) }
                .getOrElse { Files.move(temp, path, StandardCopyOption.REPLACE_EXISTING) }
        } finally {
            Files.deleteIfExists(temp)
        }
    }

    private fun secureDirectory(directory: Path) = setPermissions(directory, DIRECTORY_PERMISSIONS)

    private fun setPermissions(path: Path, permissions: Set<PosixFilePermission>) {
        val fileStore = Files.getFileStore(path)
        if (fileStore.supportsFileAttributeView("posix")) {
            Files.setPosixFilePermissions(path, permissions)
            return
        }
        val file = path.toFile()
        check(file.setReadable(false, false) && file.setWritable(false, false)) { "cannot clear permissions on ${path.toAbsolutePath()}" }
        check(file.setReadable(PosixFilePermission.OWNER_READ in permissions, true)) { "cannot set owner read permission on ${path.toAbsolutePath()}" }
        check(file.setWritable(PosixFilePermission.OWNER_WRITE in permissions, true)) { "cannot set owner write permission on ${path.toAbsolutePath()}" }
        if (Files.isDirectory(path)) {
            check(file.setExecutable(PosixFilePermission.OWNER_EXECUTE in permissions, true)) { "cannot set owner execute permission on ${path.toAbsolutePath()}" }
        }
    }

    private fun paths(directory: Path) = TlsBundle(
        directory,
        directory.resolve("ca.crt"),
        directory.resolve("server.crt"),
        directory.resolve("server.key"),
        directory.resolve("client.crt"),
        directory.resolve("client.key"),
    )

    private fun TlsBundle.matches(serverNames: List<String>): Boolean {
        return runCatching {
            val ca = certificate(caCertificate)
            val server = certificate(serverCertificate)
            val client = certificate(clientCertificate)
            val at = Date.from(now())
            ca.checkValidity(at)
            server.checkValidity(at)
            client.checkValidity(at)
            server.verify(ca.publicKey)
            client.verify(ca.publicKey)
            server.subjectAlternativeNames
                .orEmpty()
                .filter { it.size == 2 && it[0] in setOf(GeneralName.dNSName, GeneralName.iPAddress) }
                .map { it[1].toString() }
                .toSet() == serverNames.toSet()
        }.getOrDefault(false)
    }

    private fun certificate(path: Path): X509Certificate =
        Files.newInputStream(path).use {
            java.security.cert.CertificateFactory.getInstance("X.509").generateCertificate(it) as X509Certificate
        }

    private fun TlsBundle.requiredFiles(): List<Path> = listOf(caCertificate, serverCertificate, serverPrivateKey, clientCertificate, clientPrivateKey)
    private fun TlsBundle.privateFilesSecure(): Boolean =
        listOf(serverPrivateKey, clientPrivateKey).all { path ->
            val fileStore = Files.getFileStore(path)
            if (fileStore.supportsFileAttributeView("posix")) {
                Files.getPosixFilePermissions(path) == PRIVATE_FILE_PERMISSIONS
            } else {
                path.toFile().canRead() && path.toFile().canWrite()
            }
        }

    private companion object {
        const val MANIFEST_FILE = "bundle.conf"
        val IPV4 = Regex("^(?:[0-9]{1,3}\\.){3}[0-9]{1,3}$")
        val HOSTNAME = Regex("^(?=.{1,253}$)(?:[A-Za-z0-9](?:[A-Za-z0-9-]{0,61}[A-Za-z0-9])?)(?:\\.(?:[A-Za-z0-9](?:[A-Za-z0-9-]{0,61}[A-Za-z0-9])?))*$")
        val DIRECTORY_PERMISSIONS = setOf(PosixFilePermission.OWNER_READ, PosixFilePermission.OWNER_WRITE, PosixFilePermission.OWNER_EXECUTE)
        val PRIVATE_FILE_PERMISSIONS = setOf(PosixFilePermission.OWNER_READ, PosixFilePermission.OWNER_WRITE)
        val PUBLIC_FILE_PERMISSIONS = setOf(PosixFilePermission.OWNER_READ, PosixFilePermission.OWNER_WRITE)
    }
}
