# conc-completablefuture-composition

> Compose async work with `CompletableFuture`

## Why It Matters

Manually coordinating multiple `Future` objects with blocking `.get()` calls loses the benefit of asynchrony and makes error handling and sequencing verbose. `CompletableFuture` provides composable combinators (`thenApply`, `thenCompose`, `thenCombine`, `allOf`, `exceptionally`) that chain dependent async steps, fan out and join independent ones, and propagate failures without nested try/catch blocks scattered across the call site.

## Bad

```java
// Blocking on each future in turn defeats the purpose of asynchronous work,
// and there is no clean way to combine failures from independent calls.
Future<User> userFuture = executor.submit(() -> userService.fetch(userId));
Future<List<Order>> ordersFuture = executor.submit(() -> orderService.fetch(userId));

User user;
List<Order> orders;
try {
    user = userFuture.get();       // blocks the calling thread
    orders = ordersFuture.get();   // blocks again, sequentially, even though
                                    // both calls were already running in parallel
} catch (ExecutionException | InterruptedException e) {
    throw new RuntimeException(e); // loses distinction between the two failures
}
Dashboard dashboard = buildDashboard(user, orders);
```

## Good

```java
CompletableFuture<User> userFuture =
        CompletableFuture.supplyAsync(() -> userService.fetch(userId), executor);
CompletableFuture<List<Order>> ordersFuture =
        CompletableFuture.supplyAsync(() -> orderService.fetch(userId), executor);

CompletableFuture<Dashboard> dashboardFuture = userFuture
        .thenCombine(ordersFuture, this::buildDashboard)
        .exceptionally(ex -> {
            log.warn("Failed to build dashboard for {}", userId, ex);
            return Dashboard.empty();
        });

// Caller decides whether to block, chain further, or return the future itself.
Dashboard dashboard = dashboardFuture.join();
```

## Sequencing Dependent Steps

```java
CompletableFuture<Receipt> checkout(Cart cart) {
    return CompletableFuture
            .supplyAsync(() -> pricingService.priceCart(cart), executor)
            .thenCompose(priced -> paymentService.chargeAsync(priced))   // depends on previous step
            .thenApply(payment -> receiptService.build(cart, payment))
            .exceptionally(ex -> Receipt.failed(cart, ex.getMessage()));
}
```

## Fanning Out Many Independent Calls

```java
CompletableFuture<List<Quote>> fetchAllQuotes(List<Supplier> suppliers) {
    List<CompletableFuture<Quote>> futures = suppliers.stream()
            .map(supplier -> CompletableFuture.supplyAsync(
                    () -> quoteService.request(supplier), executor))
            .toList();

    return CompletableFuture
            .allOf(futures.toArray(new CompletableFuture[0]))
            .thenApply(v -> futures.stream().map(CompletableFuture::join).toList());
}
```

## CompletableFuture vs Virtual Threads

With virtual threads, plenty of code that used `CompletableFuture` purely to avoid blocking a scarce platform thread can go back to simple sequential blocking calls on a virtual thread. Keep `CompletableFuture` where you genuinely need combinator-style composition (combining independent results, timeouts, fallback chains) rather than as a substitute for blocking I/O avoidance.

## See Also

- [`conc-virtual-threads-io`](conc-virtual-threads-io.md) - Use virtual threads for I/O-bound concurrent tasks
- [`conc-structured-concurrency`](conc-structured-concurrency.md) - Use StructuredTaskScope to manage related task groups
- [`err-exception-chaining`](err-exception-chaining.md) - Preserving cause chains through exceptionally/handle
