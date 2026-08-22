import com.google.protobuf.gradle.id
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    kotlin("jvm") version "2.4.10"
    id("com.google.protobuf") version "0.9.6"
}

group = "io.github.nguyenthdat.burpmcp"
version = providers.gradleProperty("version").orElse("3.0.0-alpha.1").get()

val grpcVersion = "1.73.0"
val protobufVersion = "4.31.1"

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
    }
}

dependencies {
    compileOnly("net.portswigger.burp.extensions:montoya-api:2026.7")
    implementation(kotlin("stdlib"))
    implementation("io.grpc:grpc-netty-shaded:$grpcVersion")
    implementation("io.grpc:grpc-protobuf:$grpcVersion")
    implementation("com.google.protobuf:protobuf-java:$protobufVersion")
    implementation("io.grpc:grpc-stub:$grpcVersion")
    compileOnly("org.apache.tomcat:annotations-api:6.0.53")

    testImplementation(kotlin("test-junit5"))
    testImplementation("net.portswigger.burp.extensions:montoya-api:2026.7")
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
