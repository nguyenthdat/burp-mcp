# proj-dependency-management-bom

> Centralize dependency versions with a BOM

## Why It Matters

Without centralized version management, every module in a multi-module build (or every microservice in a fleet) picks its own version of a shared library, leading to version drift, incompatible transitive dependencies, and "works on my module" bugs. A Bill of Materials (BOM) publishes a single, tested set of compatible versions that every consumer imports once, so bumping a version happens in exactly one place and every module gets it consistently.

## Bad

```xml
<!-- Every module hardcodes its own version of the same libraries -->
<!-- orders-service/pom.xml -->
<dependency>
    <groupId>com.fasterxml.jackson.core</groupId>
    <artifactId>jackson-databind</artifactId>
    <version>2.15.2</version>
</dependency>

<!-- payments-service/pom.xml -->
<dependency>
    <groupId>com.fasterxml.jackson.core</groupId>
    <artifactId>jackson-databind</artifactId>
    <version>2.16.0</version>  <!-- Drifted - different behavior/bug fixes across services -->
</dependency>
```

## Good

```xml
<!-- parent/pom.xml - imports Spring Boot's BOM, pinning a consistent version set -->
<dependencyManagement>
    <dependencies>
        <dependency>
            <groupId>org.springframework.boot</groupId>
            <artifactId>spring-boot-dependencies</artifactId>
            <version>3.3.2</version>
            <type>pom</type>
            <scope>import</scope>
        </dependency>
    </dependencies>
</dependencyManagement>

<!-- orders-service/pom.xml - no version needed, resolved from the BOM -->
<dependency>
    <groupId>com.fasterxml.jackson.core</groupId>
    <artifactId>jackson-databind</artifactId>
</dependency>
```

```kotlin
// Gradle equivalent - build.gradle.kts
dependencies {
    implementation(platform("org.springframework.boot:spring-boot-dependencies:3.3.2"))
    implementation("com.fasterxml.jackson.core:jackson-databind")  // Version from the BOM
}
```

## Publishing Your Own BOM

```xml
<!-- internal-bom/pom.xml - published once, imported by every service in the fleet -->
<project>
    <groupId>com.example</groupId>
    <artifactId>internal-bom</artifactId>
    <packaging>pom</packaging>
    <dependencyManagement>
        <dependencies>
            <dependency>
                <groupId>com.example</groupId>
                <artifactId>app-api</artifactId>
                <version>4.2.0</version>
            </dependency>
        </dependencies>
    </dependencyManagement>
</project>
```

## See Also

- [`proj-gradle-version-catalog`](proj-gradle-version-catalog.md) - Centralize versions in a Gradle version catalog
- [`proj-multi-module-build`](proj-multi-module-build.md) - Split a growing codebase into multiple Maven/Gradle modules
- [`api-minimal-public-surface`](api-minimal-public-surface.md) - Keep the public API small and intentional
