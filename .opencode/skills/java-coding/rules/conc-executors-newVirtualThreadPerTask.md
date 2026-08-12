# conc-executors-newVirtualThreadPerTask

> Use `Executors.newVirtualThreadPerTaskExecutor()`

## Why It Matters

`Executors.newVirtualThreadPerTaskExecutor()` gives you the standard `ExecutorService` API while creating a brand-new virtual thread for every submitted task instead of pulling from a bounded pool. This removes the need to size, tune, or reuse a thread pool for I/O-bound work, and it composes with existing code written against `ExecutorService`, `Future`, and `invokeAll`.

## Bad

```java
// Manually creating a raw virtual thread per task, bypassing ExecutorService
// entirely, loses lifecycle management, batching, and shutdown semantics.
void fetchAll(List<String> urls) {
    List<Thread> threads = new ArrayList<>();
    for (String url : urls) {
        Thread t = Thread.ofVirtual().start(() -> httpClient.fetch(url));
        threads.add(t);
    }
    for (Thread t : threads) {
        try {
            t.join(); // No way to get a return value or propagate exceptions cleanly
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        }
    }
}

// Or worse: reusing a small fixed pool for what should be unbounded I/O fan-out.
ExecutorService executor = Executors.newFixedThreadPool(50);
```

## Good

```java
List<String> fetchAll(List<String> urls) throws InterruptedException {
    try (ExecutorService executor = Executors.newVirtualThreadPerTaskExecutor()) {
        List<Future<String>> futures = urls.stream()
                .map(url -> executor.submit(() -> httpClient.fetch(url)))
                .toList();

        List<String> results = new ArrayList<>();
        for (Future<String> future : futures) {
            try {
                results.add(future.get());
            } catch (ExecutionException e) {
                throw new FetchException("Failed to fetch URL", e.getCause());
            }
        }
        return results;
    } // try-with-resources calls close(), which shuts down and awaits termination
}
```

## Drop-In Replacement, Not a Pool

`newVirtualThreadPerTaskExecutor()` is unbounded by design — every task gets its own virtual thread immediately, there is no queueing and no fixed capacity. Do not add a `Semaphore` "just in case" unless you are protecting a genuinely limited downstream resource (e.g., a database connection pool); the executor itself does not need throttling for thread-count reasons.

```java
// Throttle the downstream resource, not the executor, when needed.
Semaphore dbConnections = new Semaphore(20);

try (ExecutorService executor = Executors.newVirtualThreadPerTaskExecutor()) {
    for (Query query : queries) {
        executor.submit(() -> {
            dbConnections.acquire();
            try {
                return database.run(query);
            } finally {
                dbConnections.release();
            }
        });
    }
}
```

## See Also

- [`conc-virtual-threads-io`](conc-virtual-threads-io.md) - Use virtual threads for I/O-bound concurrent tasks
- [`conc-executorservice-shutdown`](conc-executorservice-shutdown.md) - Always shut down an ExecutorService properly
- [`conc-structured-concurrency`](conc-structured-concurrency.md) - Use StructuredTaskScope for related task groups
