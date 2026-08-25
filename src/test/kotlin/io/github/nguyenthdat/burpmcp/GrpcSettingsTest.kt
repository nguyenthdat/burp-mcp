package io.github.nguyenthdat.burpmcp

import kotlin.io.path.Path
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

class GrpcSettingsTest {
    @Test
    fun `default remains plaintext IPv4 loopback`() {
        val settings = GrpcSettings()
        settings.validate()
        assertEquals(GrpcSecurityMode.LOCAL_PLAINTEXT, settings.securityMode)
        assertEquals("127.0.0.1", settings.bindAddress)
        assertEquals(9877, settings.port)
    }

    @Test
    fun `plaintext refuses a remote bind`() {
        assertFailsWith<IllegalArgumentException> {
            GrpcSettings(bindAddress = "0.0.0.0").validate()
        }
    }

    @Test
    fun `remote mTLS accepts a wildcard bind with concrete certificate identities`() {
        GrpcSettings(
            bindAddress = "0.0.0.0",
            securityMode = GrpcSecurityMode.REMOTE_MTLS,
            serverNames = listOf("burp-vm.test", "10.10.0.8"),
            tlsDirectory = Path("/tmp/burp-mcp-test-tls"),
        ).validate()
    }

    @Test
    fun `remote mTLS rejects wildcard certificate identities`() {
        assertFailsWith<IllegalArgumentException> {
            GrpcSettings(
                bindAddress = "0.0.0.0",
                securityMode = GrpcSecurityMode.REMOTE_MTLS,
                serverNames = listOf("0.0.0.0"),
                tlsDirectory = Path("/tmp/burp-mcp-test-tls"),
            ).validate()
        }
    }
}
