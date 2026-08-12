# modern-scoped-values

> Use `ScopedValue` instead of `ThreadLocal` for immutable context propagation

## Why It Matters

`ThreadLocal` was designed for a world of long-lived, pooled platform threads: its value is mutable, it must be explicitly `remove()`d to avoid leaking into a reused pooled thread, and it can silently be inherited or not inherited across thread creation in surprising ways. With virtual threads potentially numbering in the millions, per-thread mutable maps become a real memory and correctness liability. `ScopedValue` (preview in JDK 21-24 via JEP 429/446/481) models immutable, dynamically-scoped context data that is automatically and safely bound only for the duration of a specific call, with no explicit cleanup required.

## Bad

```java
private static final ThreadLocal<String> CURRENT_USER = new ThreadLocal<>();

void handleRequest(String user, Runnable task) {
    CURRENT_USER.set(user);
    try {
        task.run();
    } finally {
        CURRENT_USER.remove(); // must remember, or a pooled thread leaks the value
    }
}

void auditLog(String message) {
    System.out.println(CURRENT_USER.get() + ": " + message); // mutable, could be reassigned anywhere
}
```

## Good

```java
// Preview API as of JDK 21-24 - requires --enable-preview; confirm the exact
// package/class name and semantics for your project's targeted JDK version.
private static final ScopedValue<String> CURRENT_USER = ScopedValue.newInstance();

void handleRequest(String user, Runnable task) {
    ScopedValue.where(CURRENT_USER, user).run(task);
    // binding is automatically torn down when run() returns - no explicit cleanup
}

void auditLog(String message) {
    System.out.println(CURRENT_USER.get() + ": " + message);
}
```

## Immutability Is the Point

`ScopedValue` has no `set()` method - a value is bound once via `where(...).run(...)` or `where(...).call(...)` and cannot be reassigned within that binding's scope, which rules out an entire class of bugs where a `ThreadLocal` is mutated unexpectedly deep in a call stack:

```java
ScopedValue.where(CURRENT_USER, "alice")
        .run(() -> {
            System.out.println(CURRENT_USER.get()); // "alice"
            // No CURRENT_USER.set("bob") exists - rebind only via a nested where(...).run(...)
            ScopedValue.where(CURRENT_USER, "bob")
                    .run(() -> System.out.println(CURRENT_USER.get())); // "bob", inner scope only
            System.out.println(CURRENT_USER.get()); // back to "alice"
        });
```

Scoped values also propagate correctly and cheaply into child tasks forked from a `StructuredTaskScope`, which was a known weak point of `ThreadLocal` combined with thread pools.

## See Also

- [`conc-scoped-values-not-threadlocal`](conc-scoped-values-not-threadlocal.md) - The concurrency-focused rationale for this migration
- [`modern-structured-concurrency-preview`](modern-structured-concurrency-preview.md) - Scoped values are designed to compose with structured task scopes
- [`modern-virtual-threads-jep444`](modern-virtual-threads-jep444.md) - Why per-thread mutable state becomes costly at virtual-thread scale
- [`conc-immutable-thread-safety`](conc-immutable-thread-safety.md) - Immutability as a general thread-safety strategy
