# doc-javadoc-code-samples

> Include runnable code samples in Javadoc

## Why It Matters

A worked example often communicates correct usage faster than a paragraph of prose, especially for APIs with builders, fluent chains, or non-trivial setup - readers can copy, adapt, and run the sample instead of reverse-engineering call order from method signatures alone. Since Java 18's JEP 413, the `{@snippet}` tag lets these examples be validated or even compiled as part of the build, which prevents the classic failure mode of a stale example that no longer compiles against the current API.

## Bad

```java
/**
 * Builds an HTTP request with retry and timeout configuration.
 * Use the builder to configure the request before calling build().
 */
public class RequestBuilder {
    public RequestBuilder retries(int retries) {
        // ...
        return this;
    }

    public RequestBuilder timeout(Duration timeout) {
        // ...
        return this;
    }

    public Request build() {
        // ...
        return null;
    }
}
```

## Good

```java
/**
 * Builds an HTTP request with retry and timeout configuration.
 *
 * <p>Example:
 * {@snippet :
 * Request request = new RequestBuilder()
 *         .retries(3)
 *         .timeout(Duration.ofSeconds(5))
 *         .build();
 *
 * Response response = client.send(request);
 * }
 */
public class RequestBuilder {

    /**
     * Sets the number of retry attempts on transient failure.
     *
     * @param retries maximum retry attempts; must be non-negative
     * @return this builder, for chaining
     */
    public RequestBuilder retries(int retries) {
        // ...
        return this;
    }

    public RequestBuilder timeout(Duration timeout) {
        // ...
        return this;
    }

    public Request build() {
        // ...
        return null;
    }
}
```

## Pre-JDK-18 Style with `<pre>{@code ...}`

Codebases targeting older Java releases (or that have not adopted `{@snippet}` yet) use the `<pre>{@code ...}` idiom instead, which achieves the same monospace, HTML-safe rendering without the newer tag:

```java
/**
 * Builds an HTTP request with retry and timeout configuration.
 *
 * <p>Example:
 * <pre>{@code
 * Request request = new RequestBuilder()
 *         .retries(3)
 *         .timeout(Duration.ofSeconds(5))
 *         .build();
 * }</pre>
 */
public class RequestBuilder {
    // ...
}
```

Keep samples short and focused on the common case; a Javadoc example is not the place for exhaustive edge-case coverage - that belongs in `@param`/`@throws` or the class-level "Why It Matters"-style prose.

## See Also

- [`doc-javadoc-public-api`](doc-javadoc-public-api.md) - Document all public API with Javadoc
- [`doc-javadoc-link-tags`](doc-javadoc-link-tags.md) - Use {@link}/{@code} to cross-reference types and code
- [`api-builder-complex-construction`](api-builder-complex-construction.md) - Use builders for complex construction
- [`api-fluent-method-chaining`](api-fluent-method-chaining.md) - Design fluent, chainable APIs deliberately
