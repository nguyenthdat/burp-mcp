# conc-structured-concurrency

> Use `StructuredTaskScope` to manage related task groups

## Why It Matters

Fire-and-track concurrency with raw `ExecutorService.submit` calls and manually collected `Future`s makes it easy to leak subtasks: if one fails, the others keep running unless you remember to cancel them, and error handling is scattered across the call site. `StructuredTaskScope` (a preview API in Java 21/22, incubating under `java.util.concurrent`) ties a group of subtasks to a single block of code — when the block exits, every subtask is guaranteed to have completed, failed, or been cancelled, giving concurrent code the same clear lifetime and error propagation as sequential code.

## Bad

```java
// Manually fanning out two related fetches with no shared cancellation.
ExecutorService executor = Executors.newVirtualThreadPerTaskExecutor();

Response handle(Request request) throws Exception {
    Future<User> userFuture = executor.submit(() -> fetchUser(request.userId()));
    Future<Order> orderFuture = executor.submit(() -> fetchOrder(request.orderId()));

    // If fetchUser fails, fetchOrder keeps running to completion anyway --
    // wasted work, and no automatic cancellation of the sibling task.
    User user = userFuture.get();
    Order order = orderFuture.get();
    return new Response(user, order);
}
```

## Good

```java
// StructuredTaskScope.ShutdownOnFailure: cancel siblings as soon as one fails.
// Note: StructuredTaskScope is a preview API (JEP 480 in JDK 21-23 incubation);
// compile and run with --enable-preview.
Response handle(Request request) throws Exception {
    try (var scope = new StructuredTaskScope.ShutdownOnFailure()) {
        Supplier<User> userTask = scope.fork(() -> fetchUser(request.userId()));
        Supplier<Order> orderTask = scope.fork(() -> fetchOrder(request.orderId()));

        scope.join();           // wait for both, or until one fails
        scope.throwIfFailed();  // propagate the first failure, cancelling the rest

        return new Response(userTask.get(), orderTask.get());
    } // scope.close() guarantees no forked subtask outlives this block
}
```

## Why "Structured"

The core guarantee is that a `StructuredTaskScope` cannot be exited — normally or via exception — while any of its forked subtasks are still running. This eliminates an entire class of bugs where a request handler returns while background work from that request is still executing (and possibly still holding resources, writing to shared state, or about to throw into a `Thread`'s default uncaught-exception handler with nobody watching).

```java
// Nesting scopes composes cleanly: each level owns its own subtasks' lifetime.
try (var outer = new StructuredTaskScope.ShutdownOnFailure()) {
    Supplier<Report> report = outer.fork(() -> {
        try (var inner = new StructuredTaskScope.ShutdownOnFailure()) {
            Supplier<Data> a = inner.fork(() -> fetchA());
            Supplier<Data> b = inner.fork(() -> fetchB());
            inner.join();
            inner.throwIfFailed();
            return combine(a.get(), b.get());
        }
    });
    outer.join();
    outer.throwIfFailed();
    return report.get();
}
```

## See Also

- [`conc-executors-newVirtualThreadPerTask`](conc-executors-newVirtualThreadPerTask.md) - The executor StructuredTaskScope builds on conceptually
- [`conc-scoped-values-not-threadlocal`](conc-scoped-values-not-threadlocal.md) - Sharing immutable context into forked subtasks
- [`modern-structured-concurrency-preview`](modern-structured-concurrency-preview.md) - Broader feature overview and preview status
- [`err-exception-chaining`](err-exception-chaining.md) - Preserving cause chains when propagating subtask failures
