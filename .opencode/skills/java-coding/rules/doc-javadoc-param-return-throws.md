# doc-javadoc-param-return-throws

> Document `@param`/`@return`/`@throws`

## Why It Matters

`@param`, `@return`, and `@throws` are the load-bearing tags that let a caller understand a method's contract - including failure modes - without opening the source file, and `javadoc`/IDE tooling actively warns or flags when they're missing or incomplete for public methods. Omitting `@throws` is especially costly: unchecked exceptions are invisible in the method signature, so the Javadoc is often the only place an exceptional case is ever documented at all.

## Bad

```java
/**
 * Parses a duration string.
 */
public static Duration parse(String text, ChronoUnit unit) {
    if (text == null || text.isBlank()) {
        throw new IllegalArgumentException("text must not be blank");
    }
    long amount = Long.parseLong(text);  // can throw NumberFormatException
    return Duration.of(amount, unit);
}
```

## Good

```java
/**
 * Parses a duration string as a whole number of the given time unit.
 *
 * @param text the numeric string to parse, e.g. {@code "30"}; must not be
 *     null or blank
 * @param unit the unit that {@code text} is expressed in
 * @return the parsed {@link Duration}
 * @throws IllegalArgumentException if {@code text} is null or blank
 * @throws NumberFormatException if {@code text} is not a valid {@code long}
 */
public static Duration parse(String text, ChronoUnit unit) {
    if (text == null || text.isBlank()) {
        throw new IllegalArgumentException("text must not be blank");
    }
    long amount = Long.parseLong(text);
    return Duration.of(amount, unit);
}
```

## Constructors and Void Methods

Constructors document `@param` and `@throws` but never `@return`, since they have no return value; `void` methods likewise omit `@return` entirely rather than writing something like `@return nothing`.

```java
/**
 * Creates a connection pool with the given capacity.
 *
 * @param maxConnections maximum number of concurrent connections; must be positive
 * @throws IllegalArgumentException if {@code maxConnections} is not positive
 */
public ConnectionPool(int maxConnections) {
    if (maxConnections <= 0) {
        throw new IllegalArgumentException("maxConnections must be positive");
    }
    this.maxConnections = maxConnections;
}

/**
 * Closes all connections currently held by this pool.
 *
 * @throws IOException if any connection fails to close cleanly
 */
public void closeAll() throws IOException {
    // ...
}
```

Only document `@throws` for exceptions a caller might reasonably need to handle or avoid - documenting every possible `RuntimeException` a JVM could theoretically throw (e.g., `OutOfMemoryError`) adds noise rather than value.

## See Also

- [`doc-javadoc-public-api`](doc-javadoc-public-api.md) - Document all public API with Javadoc
- [`doc-javadoc-link-tags`](doc-javadoc-link-tags.md) - Use {@link}/{@code} to cross-reference types and code
- [`err-checked-vs-unchecked`](err-checked-vs-unchecked.md) - Choose checked vs. unchecked exceptions deliberately
- [`err-exception-message-context`](err-exception-message-context.md) - Include actionable context in exception messages
