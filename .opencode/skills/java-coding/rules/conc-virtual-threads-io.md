# conc-virtual-threads-io

> Use virtual threads for I/O-bound concurrent tasks

## Why It Matters

Virtual threads (JEP 444, Java 21) are cheap, JVM-managed threads that unmount from their carrier platform thread whenever they block on I/O. This lets you write simple, one-thread-per-request blocking code that scales to hundreds of thousands of concurrent connections without the memory and context-switch overhead of platform threads, and without the readability cost of reactive/async pipelines.

## Bad

```java
// Fixed-size platform thread pool for a request handler that mostly waits on I/O
ExecutorService executor = Executors.newFixedThreadPool(200);

void handleRequests(List<Request> requests) {
    for (Request request : requests) {
        executor.submit(() -> {
            // Each task blocks on a network call and a DB call.
            // With only 200 platform threads, throughput is capped by pool size,
            // not by actual I/O concurrency the underlying systems could support.
            String payload = httpClient.fetch(request.url());       // blocks
            Result result = database.query(request.toQuery());       // blocks
            respond(request, payload, result);
        });
    }
}

// Bumping pool size "fixes" throughput but each platform thread costs ~1MB
// of stack plus kernel scheduling overhead, so this doesn't scale past
// a few thousand threads.
```

## Good

```java
// One virtual thread per task; the JVM handles carrier-thread scheduling.
void handleRequests(List<Request> requests) {
    try (ExecutorService executor = Executors.newVirtualThreadPerTaskExecutor()) {
        for (Request request : requests) {
            executor.submit(() -> {
                String payload = httpClient.fetch(request.url());   // parks the
                Result result = database.query(request.toQuery());  // virtual thread,
                respond(request, payload, result);                  // carrier is freed
            });
        }
    } // executor.close() waits for all submitted tasks to finish
}
```

## When Platform Threads Are Still Right

Virtual threads help when a task spends most of its time blocked on I/O (sockets, files, JDBC drivers using blocking I/O, `Thread.sleep`). They do not help — and can hurt — CPU-bound work, since a busy virtual thread simply occupies its carrier thread like any other runnable task. See `conc-platform-threads-cpu` for that case.

## Cost Comparison

```java
// Platform thread: backed by an OS thread, ~1MB stack, expensive to create.
// Practical limit: thousands of concurrent threads.

// Virtual thread: backed by a small, heap-allocated stack that grows/shrinks,
// scheduled by the JVM onto a small pool of carrier threads (ForkJoinPool by
// default). Practical limit: millions of concurrent virtual threads.

Thread.ofVirtual().start(() -> {
    // Cheap enough to create one per logical unit of work, no pooling needed.
});
```

## See Also

- [`conc-platform-threads-cpu`](conc-platform-threads-cpu.md) - Reserve platform threads for CPU-bound work
- [`conc-executors-newVirtualThreadPerTask`](conc-executors-newVirtualThreadPerTask.md) - Use the virtual-thread-per-task executor factory
- [`conc-avoid-pinning`](conc-avoid-pinning.md) - Avoid pinning virtual threads to their carrier
- [`modern-virtual-threads-jep444`](modern-virtual-threads-jep444.md) - Background on the JEP 444 feature
