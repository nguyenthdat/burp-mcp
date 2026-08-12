# proj-multi-module-build

> Split a growing codebase into multiple Maven/Gradle modules

## Why It Matters

A single monolithic module forces every change - however small - to recompile and retest the entire codebase, and gives every class access to every other class regardless of intended boundaries. Splitting along real seams (API vs. implementation, service vs. shared library) shortens build/test feedback loops via incremental builds and caching, and makes unwanted dependencies (e.g. the web layer depending on internal persistence details) a build error instead of a code review nit.

## Bad

```
big-app/                          // One module, one build, everything compiled together
├── src/main/java/com/example/
│   ├── web/
│   ├── orders/
│   ├── payments/
│   ├── shared/
│   └── batch/
└── build.gradle.kts              // A single 5-minute build for any one-line change
```

## Good

```
big-app/
├── settings.gradle.kts
├── app-api/                      // Shared DTOs/interfaces, no implementation
│   └── build.gradle.kts
├── orders-service/
│   ├── build.gradle.kts          // depends on app-api only
│   └── src/main/java/...
├── payments-service/
│   ├── build.gradle.kts          // depends on app-api only
│   └── src/main/java/...
└── web/
    ├── build.gradle.kts          // depends on orders-service, payments-service
    └── src/main/java/...
```

```kotlin
// settings.gradle.kts
rootProject.name = "big-app"
include("app-api", "orders-service", "payments-service", "web")
```

```kotlin
// orders-service/build.gradle.kts
dependencies {
    implementation(project(":app-api"))
    // Note: no dependency on payments-service - the build enforces this boundary
}
```

Now changing `payments-service` only triggers a rebuild/retest of `payments-service` and its dependents (`web`), not `orders-service`.

## When to Stay Single-Module

```
// A small service or CLI tool with one deployable artifact and a handful
// of classes gains nothing from module splitting and pays real overhead
// in build configuration. See proj-flat-small-projects.
```

## See Also

- [`proj-flat-small-projects`](proj-flat-small-projects.md) - Keep small projects flat instead of over-modularizing
- [`proj-dependency-management-bom`](proj-dependency-management-bom.md) - Centralize dependency versions with a BOM
- [`proj-avoid-circular-package-deps`](proj-avoid-circular-package-deps.md) - Avoid circular package dependencies
- [`proj-module-info-jpms`](proj-module-info-jpms.md) - Weigh JPMS `module-info.java` trade-offs deliberately
