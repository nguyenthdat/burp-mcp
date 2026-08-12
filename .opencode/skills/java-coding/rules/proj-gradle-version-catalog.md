# proj-gradle-version-catalog

> Centralize versions in a Gradle version catalog

## Why It Matters

Scattering dependency coordinates and versions across every module's `build.gradle.kts` means bumping a library requires a find-and-replace across the repo, and typos in a group/artifact string only surface as a broken build. Gradle's version catalog (`libs.versions.toml`) centralizes every version, library, and plugin alias in one file with IDE-assisted, type-safe accessors (`libs.jackson.databind`), so a version bump is a one-line change and misspelled coordinates are caught at editing time.

## Bad

```kotlin
// orders-service/build.gradle.kts
dependencies {
    implementation("com.fasterxml.jackson.core:jackson-databind:2.16.0")
    implementation("org.slf4j:slf4j-api:2.0.13")
    testImplementation("org.junit.jupiter:junit-jupiter:5.10.2")
}

// payments-service/build.gradle.kts - versions drift silently across modules
dependencies {
    implementation("com.fasterxml.jackson.core:jackson-databind:2.15.2")  // Different version!
    implementation("org.slf4j:slf4j-api:2.0.9")                           // Different version!
    testImplementation("org.junit.jupiter:junit-jupiter:5.9.3")           // Different version!
}
```

## Good

```toml
# gradle/libs.versions.toml
[versions]
jackson = "2.16.0"
slf4j = "2.0.13"
junit = "5.10.2"

[libraries]
jackson-databind = { module = "com.fasterxml.jackson.core:jackson-databind", version.ref = "jackson" }
slf4j-api = { module = "org.slf4j:slf4j-api", version.ref = "slf4j" }
junit-jupiter = { module = "org.junit.jupiter:junit-jupiter", version.ref = "junit" }

[bundles]
logging = ["slf4j-api"]

[plugins]
spring-boot = { id = "org.springframework.boot", version = "3.3.2" }
```

```kotlin
// orders-service/build.gradle.kts - type-safe, IDE-autocompleted accessors
dependencies {
    implementation(libs.jackson.databind)
    implementation(libs.bundles.logging)
    testImplementation(libs.junit.jupiter)
}

// payments-service/build.gradle.kts - guaranteed same versions, no drift possible
dependencies {
    implementation(libs.jackson.databind)
    implementation(libs.bundles.logging)
    testImplementation(libs.junit.jupiter)
}
```

## See Also

- [`proj-dependency-management-bom`](proj-dependency-management-bom.md) - Centralize dependency versions with a BOM
- [`proj-multi-module-build`](proj-multi-module-build.md) - Split a growing codebase into multiple Maven/Gradle modules
- [`proj-maven-gradle-standard-layout`](proj-maven-gradle-standard-layout.md) - Follow the standard Maven/Gradle source layout
