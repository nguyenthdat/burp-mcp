# modern-virtual-threads-jep444

> Adopt virtual threads (JEP 444) for scalable concurrency

## Why It Matters

Platform threads are expensive - each one reserves megabytes of stack and is a scarce OS resource, so thread-per-request servers historically had to cap concurrency at a few thousand threads with a bounded pool. Virtual threads (finalized in Java 21 via JEP 444) are cheap, JVM-managed threads that can be created in the millions and are automatically parked/unmounted from their carrier platform thread during blocking I/O, letting simple blocking-style code scale the way reactive/async code used to be required for.

## Bad

```java
// Bounded platform-thread pool caps concurrency and risks starvation under load
ExecutorService executor = Executors.newFixedThreadPool(200);

List<Future<String>> futures = new ArrayList<>();
for (String url : urls) { // e.g. 10,000 URLs to fetch
    futures.add(executor.submit(() -> fetch(url))); // blocks a scarce platform thread per call
}

for (Future<String> future : futures) {
    System.out.println(future.get());
}
executor.shutdown();
```

## Good

```java
// Virtual threads: one per task, cheap to create, unmounted while blocked on I/O
try (ExecutorService executor = Executors.newVirtualThreadPerTaskExecutor()) {
    List<Future<String>> futures = new ArrayList<>();
    for (String url : urls) {
        futures.add(executor.submit(() -> fetch(url))); // blocking fetch is fine here
    }

    for (Future<String> future : futures) {
        System.out.println(future.get());
    }
} // executor.close() waits for and shuts down all virtual threads
```

## What Changes, What Doesn't

Virtual threads use the exact same `Thread`/`ExecutorService` APIs - the code you write for blocking I/O does not change, only the executor construction does. What you get in exchange:

- Millions of virtual threads can be live simultaneously; there is no need to pool them (`newVirtualThreadPerTaskExecutor` deliberately does not reuse threads).
- Blocking calls like `Thread.sleep`, `InputStream.read`, and JDBC/HTTP client calls automatically yield the underlying carrier thread back to the scheduler.
- CPU-bound work does not benefit from virtual threads - they don't add cores, they add cheap scheduling for blocking work.

## Caution: Pinning

Certain constructs (`synchronized` blocks, some native method frames) "pin" a virtual thread to its carrier, blocking the carrier thread for the duration - defeating the scalability benefit. Prefer `java.util.concurrent.locks.ReentrantLock` over `synchronized` in code that runs on virtual threads and is expected to block frequently.

## See Also

- [`conc-virtual-threads-io`](conc-virtual-threads-io.md) - When virtual threads are (and aren't) the right tool
- [`conc-avoid-pinning`](conc-avoid-pinning.md) - Avoiding the `synchronized` pinning pitfall in detail
- [`conc-executors-newVirtualThreadPerTask`](conc-executors-newVirtualThreadPerTask.md) - Executor construction patterns for virtual threads
- [`modern-structured-concurrency-preview`](modern-structured-concurrency-preview.md) - Structuring groups of virtual-thread tasks safely
