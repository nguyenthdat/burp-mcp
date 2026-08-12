# conc-scoped-values-not-threadlocal

> Prefer `ScopedValue` over unscoped `ThreadLocal` with virtual threads

## Why It Matters

`ThreadLocal` was designed for a world of pooled, long-lived platform threads; each virtual thread gets its own `ThreadLocal` storage too, but with millions of short-lived virtual threads the per-thread map overhead adds up, and mutable, unscoped `ThreadLocal` values are easy to forget to clean up, leaking data across task boundaries or pooled reuse. `ScopedValue` (preview API, JEP 446) binds an immutable value for the dynamic extent of a single call, is automatically and safely cleaned up when that call returns, and is explicitly designed to be cheap to share with child virtual threads and `StructuredTaskScope` subtasks.

## Bad

```java
// A ThreadLocal used to smuggle the current request's user context through
// deeply nested calls.
private static final ThreadLocal<UserContext> CURRENT_USER = new ThreadLocal<>();

void handleRequest(Request request) {
    CURRENT_USER.set(loadUserContext(request));
    try {
        processRequest(request);
    } finally {
        CURRENT_USER.remove(); // easy to forget; if forgotten, the next task
                                // reusing this virtual thread inherits stale data
    }
}

void processRequest(Request request) {
    // Any code deep in the call stack can *mutate* CURRENT_USER, since
    // ThreadLocal offers no immutability guarantee.
    CURRENT_USER.get().setRole("admin"); // silent, hard-to-trace state change
    auditLog(CURRENT_USER.get());
}
```

## Good

```java
// ScopedValue (preview in JDK 21+; run with --enable-preview): immutable,
// automatically bound and unbound for the extent of a single call.
private static final ScopedValue<UserContext> CURRENT_USER = ScopedValue.newInstance();

void handleRequest(Request request) {
    ScopedValue.where(CURRENT_USER, loadUserContext(request))
            .run(() -> processRequest(request));
    // CURRENT_USER is automatically unbound here -- no cleanup to forget.
}

void processRequest(Request request) {
    UserContext user = CURRENT_USER.get(); // read-only access
    auditLog(user);
    // No setter exists on ScopedValue -- mutation is structurally impossible.
}
```

## Sharing Context With Structured Concurrency

`ScopedValue` bindings are automatically visible to subtasks forked from a `StructuredTaskScope` within the same dynamic scope, which is exactly the sharing model virtual-thread fan-out needs:

```java
ScopedValue.where(CURRENT_USER, loadUserContext(request)).run(() -> {
    try (var scope = new StructuredTaskScope.ShutdownOnFailure()) {
        // Both forked tasks see the same CURRENT_USER binding automatically.
        var profile = scope.fork(() -> profileService.load(CURRENT_USER.get().id()));
        var orders = scope.fork(() -> orderService.load(CURRENT_USER.get().id()));
        scope.join();
        scope.throwIfFailed();
        render(profile.get(), orders.get());
    }
});
```

## When ThreadLocal Is Still Fine

Mutable, per-thread caches (e.g., a reusable `SimpleDateFormat`-like scratch buffer) on a small, fixed pool of platform threads are still a reasonable use of `ThreadLocal`. The concern here is specifically about using `ThreadLocal` to pass immutable request/task context across a virtual-thread-heavy call tree.

## See Also

- [`conc-structured-concurrency`](conc-structured-concurrency.md) - Use StructuredTaskScope to manage related task groups
- [`modern-scoped-values`](modern-scoped-values.md) - Broader ScopedValue feature overview
- [`conc-immutable-thread-safety`](conc-immutable-thread-safety.md) - Make shared objects immutable for thread safety
