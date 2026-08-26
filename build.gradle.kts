import com.google.protobuf.gradle.id
import org.jetbrains.kotlin.gradle.dsl.JvmTarget


plugins {
    kotlin("jvm") version "2.4.10"
    id("com.google.protobuf") version "0.10.0"
}


val sitegraphRulePackSha256 = "0ad7dbd9d752b914aefbc37f6958495af956156865ce1392014ec86f8f69a398"
group = "io.github.nguyenthdat.burpmcp"
version = providers.gradleProperty("version").orElse("3.0.2").get()

val grpcVersion = "1.83.1"
val protobufVersion = "4.36.0"

repositories {
    mavenCentral()
}

kotlin {
    jvmToolchain(25)
    compilerOptions {
        jvmTarget.set(JvmTarget.JVM_25)
    }
}

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(25))
    }
}

sourceSets {
    main {
        proto.srcDir("proto")
        java.srcDirs(
            layout.buildDirectory.dir("generated/sources/proto/main/java"),
            layout.buildDirectory.dir("generated/sources/proto/main/grpc"),
        )
    }
}

dependencies {
    compileOnly("net.portswigger.burp.extensions:montoya-api:2026.7")
    implementation(kotlin("stdlib"))
    implementation("io.grpc:grpc-netty-shaded:$grpcVersion")
    implementation("io.grpc:grpc-protobuf:$grpcVersion")
    implementation("com.google.protobuf:protobuf-java:$protobufVersion")
    implementation("io.grpc:grpc-stub:$grpcVersion")
    implementation("com.fasterxml.jackson.core:jackson-databind:2.22.2")
    implementation("com.fasterxml.jackson.module:jackson-module-kotlin:2.22.2")
    implementation("com.google.re2j:re2j:1.8")
    implementation("org.bouncycastle:bcpkix-jdk18on:1.85")
    compileOnly("org.apache.tomcat:annotations-api:6.0.53")

    testImplementation(kotlin("test-junit5"))
    testImplementation("net.portswigger.burp.extensions:montoya-api:2026.7")
    testImplementation("io.mockk:mockk:1.14.11")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

protobuf {
    protoc {
        artifact = "com.google.protobuf:protoc:$protobufVersion"
    }
    plugins {
        id("grpc") {
            artifact = "io.grpc:protoc-gen-grpc-java:$grpcVersion"
        }
    }
    generateProtoTasks {
        all().configureEach {
            plugins {
                id("grpc")
            }
        }
    }
}

tasks.register("printTestRuntimeClasspath") {
    dependsOn(tasks.testClasses)
    doLast {
        println(sourceSets.test.get().runtimeClasspath.asPath)
    }
}

tasks.withType<JavaCompile>().configureEach {
    options.encoding = "UTF-8"
    options.release.set(25)
}


tasks.withType<Test>().configureEach {
    useJUnitPlatform()
    javaLauncher.set(
        javaToolchains.launcherFor {
            languageVersion.set(JavaLanguageVersion.of(25))
        },
    )
}


tasks.jar {
    archiveFileName.set("burp-mcp.jar")
    duplicatesStrategy = DuplicatesStrategy.EXCLUDE
    isReproducibleFileOrder = true
    isPreserveFileTimestamps = false
    exclude("META-INF/*.SF", "META-INF/*.RSA", "META-INF/*.DSA")
    manifest {
        attributes(
            "Implementation-Title" to "Burp MCP",
            "Implementation-Version" to project.version,
        )
    }
    from({
        configurations.runtimeClasspath.get().map { dependency ->
            if (dependency.isDirectory) dependency else zipTree(dependency)
        }
    })
}
