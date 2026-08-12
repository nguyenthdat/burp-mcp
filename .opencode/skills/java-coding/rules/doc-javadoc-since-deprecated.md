# doc-javadoc-since-deprecated

> Use `@since` and `@deprecated` with migration guidance

## Why It Matters

`@since` tells API consumers the minimum version required for a method, which matters enormously for libraries supporting multiple release lines - without it, users have no way to know if a method they want to call is available in the version they depend on. `@deprecated` paired with the `@Deprecated` annotation is the mechanism the compiler and IDEs use to warn callers automatically, but a deprecation notice without concrete migration guidance just tells developers something is wrong without telling them how to fix it, leading to deprecated code lingering for years.

## Bad

```java
public class ConfigLoader {

    /**
     * Loads configuration from the given path.
     */
    @Deprecated
    public static Config load(String path) {  // deprecated but no reason or alternative given
        return loadLegacy(path);
    }

    public static Config loadFrom(Path path) {  // no @since - when was this added?
        // ...
        return null;
    }
}
```

## Good

```java
public class ConfigLoader {

    /**
     * Loads configuration from the given path.
     *
     * @param path filesystem path to the configuration file
     * @return the parsed {@link Config}
     * @deprecated Use {@link #loadFrom(Path)} instead, which supports
     *     {@link Path}-based resolution and symbolic links correctly.
     *     This method will be removed in version 4.0.
     * @since 1.0
     */
    @Deprecated(since = "3.2", forRemoval = true)
    public static Config load(String path) {
        return loadFrom(Path.of(path));
    }

    /**
     * Loads configuration from the given path.
     *
     * @param path filesystem path to the configuration file
     * @return the parsed {@link Config}
     * @since 3.2
     */
    public static Config loadFrom(Path path) {
        // ...
        return null;
    }
}
```

## Deprecating a Whole Type or Field

The same pairing applies to classes, interfaces, and fields, not just methods - always state what to use instead and, when known, a target removal version:

```java
/**
 * Legacy HTTP client based on {@code java.net.HttpURLConnection}.
 *
 * @deprecated Use {@link com.opswat.http.HttpClientAdapter}, which wraps
 *     the JDK's {@link java.net.http.HttpClient} and supports HTTP/2.
 *     Scheduled for removal in 5.0.
 * @since 1.0
 */
@Deprecated(since = "4.0", forRemoval = true)
public class LegacyHttpClient {
    // ...
}
```

## See Also

- [`doc-javadoc-public-api`](doc-javadoc-public-api.md) - Document all public API with Javadoc
- [`doc-javadoc-link-tags`](doc-javadoc-link-tags.md) - Use {@link}/{@code} to cross-reference types and code
- [`api-minimal-public-surface`](api-minimal-public-surface.md) - Keep the public API surface minimal
