# modern-structured-concurrency-preview

> Use structured concurrency (preview) for related task groups

## Why It Matters

Launching related concurrent subtasks with raw `ExecutorService.submit()` calls means their lifetimes are not tied together: if one subtask fails, the others keep running unless you manually track and cancel every `Future`, and error handling is scattered across the call site. Structured concurrency treats a group of forked subtasks as a single unit of work with one lifetime, so cancellation, error propagation, and shutdown all happen together, making concurrent code as easy to reason about as a single-threaded call stack.

## Bad

```java
ExecutorService executor = Executors.newVirtualThreadPerTaskExecutor();

Future<User> userFuture = executor.submit(() -> fetchUser(userId));
Future<Order> orderFuture = executor.submit(() -> fetchOrder(orderId));

try {
    User user = userFuture.get();   // if this throws, orderFuture keeps running unmanaged
    Order order = orderFuture.get();
    return new Response(user, order);
} catch (Exception e) {
    // Must remember to cancel the other future manually - easy to forget
    userFuture.cancel(true);
    orderFuture.cancel(true);
    throw new RuntimeException(e);
} finally {
    executor.shutdown();
}
```

## Good

```java
// Preview API as of JDK 21-23 (JEP 453/462/480) - requires --enable-preview
// and may still change; confirm your project's JDK version and preview flags
// before adopting this in production code.
try (var scope = StructuredTaskScope.open(StructuredTaskScope.Joiner.awaitAllSuccessfulOrThrow())) {
    Subtask<User> userTask = scope.fork(() -> fetchUser(userId));
    Subtask<Order> orderTask = scope.fork(() -> fetchOrder(orderId));

    scope.join(); // waits for both; if either fails, the other is cancelled automatically

    return new Response(userTask.get(), orderTask.get());
} // scope close guarantees no subtask outlives this block
```

## The Core Guarantee

A `StructuredTaskScope` enforces that no forked subtask can outlive the scope: `join()` (or the scope's implicit close) waits for all subtasks to finish, and cancelling the scope (due to a failure, timeout, or shutdown policy) propagates to every subtask. This eliminates "leaked" background tasks that silently keep running after their parent operation has already failed or returned.

## API Evolution Note

Structured concurrency's API shape changed across preview iterations (JDK 21's `StructuredTaskScope.ShutdownOnFailure`/`ShutdownOnSuccess` policy classes were superseded by the `Joiner` API shown above in later previews). Always check the exact preview API for the JDK version your project targets, compile with `--enable-preview --release <N>`, and treat the API as subject to change until it is finalized.

## See Also

- [`conc-structured-concurrency`](conc-structured-concurrency.md) - Broader concurrency patterns built on structured task scopes
- [`modern-virtual-threads-jep444`](modern-virtual-threads-jep444.md) - Structured concurrency is designed around virtual-thread-per-task execution
- [`modern-scoped-values`](modern-scoped-values.md) - Scoped values are designed to compose with structured concurrency
- [`conc-completablefuture-composition`](conc-completablefuture-composition.md) - The older composition style this feature aims to replace for related tasks
