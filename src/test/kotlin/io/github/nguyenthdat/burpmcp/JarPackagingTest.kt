package io.github.nguyenthdat.burpmcp

import java.util.jar.JarFile
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

class JarPackagingTest {
    @Test
    fun `extension jar contains runtime and excludes tests`() {
        val jarPath = java.nio.file.Path.of("build/libs/burp-mcp.jar")
        assertTrue(java.nio.file.Files.exists(jarPath), "extension JAR must be built before tests run")

        JarFile(jarPath.toFile()).use { jar ->
            assertTrue(jar.entries().asSequence().none { entry ->
                entry.name.matches(Regex("META-INF/[^/]+\\.(SF|RSA|DSA)"))
            }, "fat JAR must not retain dependency signature files")
            assertNotNull(jar.getJarEntry("META-INF/extensions/burp-extension.properties"))
            assertNotNull(jar.getJarEntry("io/github/nguyenthdat/burpmcp/BurpMcpExtension.class"))
            assertNotNull(jar.getJarEntry("io/github/nguyenthdat/burpmcp/rpc/BurpRpcServer.class"))
            assertNotNull(jar.getJarEntry("io/github/nguyenthdat/burpmcp/grpc/v1/BurpServiceGrpc.class"))
            assertNotNull(jar.getJarEntry("io/grpc/netty/shaded/io/grpc/netty/NettyServerBuilder.class"))
            assertNotNull(jar.getJarEntry("com/google/protobuf/ByteString.class"))
            assertNull(jar.getJarEntry("io/github/nguyenthdat/burpmcp/BurpRpcServerTest.class"))
            assertEquals("Burp MCP", jar.manifest.mainAttributes.getValue("Implementation-Title"))
            assertEquals("3.1.0", jar.manifest.mainAttributes.getValue("Implementation-Version"))
        }
    }
}
