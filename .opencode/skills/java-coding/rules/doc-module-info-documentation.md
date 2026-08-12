# doc-module-info-documentation

> Document JPMS modules in `module-info.java`

## Why It Matters

A Java Platform Module System (JPMS) module declares its public boundary (`exports`), its dependencies (`requires`), and its service contracts (`provides`/`uses`) - but the *reasons* behind those choices (why a package is exported, why a dependency is `transitive`, why a service is provided) are invisible unless someone writes them down. Undocumented `module-info.java` files leave maintainers guessing whether a `requires` can be safely removed or an `exports` narrowed, which either causes accidental breakage or, more often, an unwillingness to ever clean the module graph up.

## Bad

```java
module com.opswat.filescanner {
    requires java.net.http;
    requires transitive com.opswat.common;
    exports com.opswat.filescanner.api;
    exports com.opswat.filescanner.detection;
    provides com.opswat.filescanner.spi.ScannerProvider
            with com.opswat.filescanner.internal.DefaultScannerProvider;
}
```

## Good

```java
/**
 * File scanning and malware detection module.
 *
 * <p>Exposes the public scanning API ({@code com.opswat.filescanner.api})
 * and detection result types ({@code com.opswat.filescanner.detection}) for
 * consumers. Internal packages such as {@code
 * com.opswat.filescanner.internal} are not exported and may change without
 * notice.
 *
 * @since 2.0
 */
module com.opswat.filescanner {
    requires java.net.http;

    // transitive: consumers of this module always need common's shared
    // types (e.g. Result<T>) on their own module path to compile against
    // our public API.
    requires transitive com.opswat.common;

    exports com.opswat.filescanner.api;
    exports com.opswat.filescanner.detection;

    // detection.internal is intentionally NOT exported - it contains
    // signature-matching internals that must stay free to change.

    // Registers the default scanner provider for ServiceLoader discovery;
    // downstream modules may supply their own via `provides` on the same
    // com.opswat.filescanner.spi.ScannerProvider contract.
    provides com.opswat.filescanner.spi.ScannerProvider
            with com.opswat.filescanner.internal.DefaultScannerProvider;
}
```

## Documenting `opens` and `uses`

Reflective access (`opens`) and service consumption (`uses`) are especially worth a comment, since both represent boundary-crossing that is easy to remove accidentally during a refactor without realizing something outside the module depends on it:

```java
module com.opswat.filescanner {
    // Opened for Jackson's reflective (de)serialization of DTOs in this
    // package only; do not open the whole module.
    opens com.opswat.filescanner.api.dto to com.fasterxml.jackson.databind;

    // Consumes third-party ScannerProvider implementations discovered via
    // ServiceLoader at startup; see ScannerRegistry for the loading logic.
    uses com.opswat.filescanner.spi.ScannerProvider;
}
```

## See Also

- [`doc-package-info`](doc-package-info.md) - Document packages with package-info.java
- [`doc-readme-module-level`](doc-readme-module-level.md) - Maintain a README per module with purpose and usage
- [`api-minimal-public-surface`](api-minimal-public-surface.md) - Keep the public API surface minimal
- [`modern-foreign-function-memory-api`](modern-foreign-function-memory-api.md) - Use the Foreign Function and Memory API safely
