# name-constants-screaming-snake

> Use `SCREAMING_SNAKE_CASE` for `static final` constants

## Why It Matters

`SCREAMING_SNAKE_CASE` signals "this value is a compile-time or effectively-immutable constant, safe to inline and reason about globally" the instant a reader sees it, without checking the declaration. Mixing constant casing with regular field casing hides which values are meant to vary per instance and which are fixed configuration, making refactors and magic-number hunts much harder.

## Bad

```java
public class RetryPolicy {

    public static final int maxAttempts = 5;          // reads like an instance field
    public static final long DefaultTimeoutMs = 3000;  // inconsistent casing
    static final double backoffMultiplier = 1.5;       // easy to mistake for mutable state

    private int currentAttempt;

    public boolean shouldRetry() {
        return currentAttempt < maxAttempts;
    }
}
```

## Good

```java
public class RetryPolicy {

    public static final int MAX_ATTEMPTS = 5;
    public static final long DEFAULT_TIMEOUT_MS = 3000;
    static final double BACKOFF_MULTIPLIER = 1.5;

    private int currentAttempt;

    public boolean shouldRetry() {
        return currentAttempt < MAX_ATTEMPTS;
    }
}
```

## When a `static final` Is Not a Constant

Not every `static final` field is a true constant. `static final` mutable objects (collections, arrays, builders) do not represent a fixed value, only a fixed reference, and are conventionally named like regular fields rather than constants, since their contents can still change.

```java
public class Cache {

    // True constant: primitive, immutable, and semantically fixed
    private static final int MAX_ENTRIES = 1000;

    // Not a constant in spirit: the map's contents mutate at runtime
    private static final Map<String, Object> sharedCache = new ConcurrentHashMap<>();
}
```

Enum constants are the exception in the reverse direction: enum values are also written in `SCREAMING_SNAKE_CASE` even though they are not `static final` fields declared with that keyword.

```java
public enum Status {
    ACTIVE, SUSPENDED, CLOSED
}
```

## See Also

- [`name-classes-pascal`](name-classes-pascal.md) - Use PascalCase for classes, interfaces, enums, records
- [`name-methods-camel`](name-methods-camel.md) - Use camelCase for methods and fields
- [`type-enum-over-int-constants`](type-enum-over-int-constants.md) - Prefer enums over int constant groups
- [`api-immutable-by-default`](api-immutable-by-default.md) - Favor immutability in class design
