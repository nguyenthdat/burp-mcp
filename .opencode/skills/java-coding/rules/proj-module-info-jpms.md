# proj-module-info-jpms

> Weigh JPMS `module-info.java` trade-offs deliberately

## Why It Matters

The Java Platform Module System enforces strong encapsulation (only exported packages are visible) and explicit dependency declarations, catching illegal internal-API usage at compile time instead of at runtime with `--illegal-access` warnings. But adopting it is not free: every dependency in the graph must be a proper module (or an automatic module with a stable name), split packages become build errors, and many libraries still ship as plain classpath JARs. Adopt `module-info.java` deliberately for libraries with a real encapsulation boundary, not reflexively for every project.

## Bad

```java
// module-info.java added without checking that every dependency is
// module-friendly, and exporting everything defeats the purpose.
module com.example.app {
    requires java.base;
    requires spring.beans;      // Automatic module name may change between releases
    requires spring.context;    // Also automatic - fragile, unversioned dependency
    exports com.example.app;
    exports com.example.app.internal;  // "internal" package exported - no encapsulation gained
}
```

## Good

```java
// module-info.java for a library with a genuine public/internal boundary
// and dependencies that are already proper modules.
module com.example.orders {
    requires transitive com.example.orders.api;  // Re-exported to consumers that need it
    requires java.sql;
    requires static com.fasterxml.jackson.databind;  // Optional at runtime (compile-time only)

    exports com.example.orders;         // Public API
    // com.example.orders.internal is intentionally not exported

    opens com.example.orders.dto to com.fasterxml.jackson.databind;  // Reflection access for JSON binding
}
```

```kotlin
// build.gradle.kts - verify module-path compatibility before committing to JPMS
java {
    modularity.inferModulePath.set(true)
}
```

## When to Skip JPMS

```java
// A typical Spring Boot web application with a dozen third-party
// dependencies that are not modularized: stick to the classpath and
// enforce encapsulation with package-private visibility and
// architecture tests (e.g. ArchUnit) instead of module-info.java.
```

## See Also

- [`proj-multi-module-build`](proj-multi-module-build.md) - Split a growing codebase into multiple Maven/Gradle modules
- [`proj-package-private-default`](proj-package-private-default.md) - Default new types/members to package-private
- [`proj-avoid-circular-package-deps`](proj-avoid-circular-package-deps.md) - Avoid circular package dependencies
