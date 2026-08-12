# conc-platform-threads-cpu

> Reserve platform threads for CPU-bound work

## Why It Matters

Virtual threads solve the "too many blocked threads" problem, but they do not add parallelism: a virtual thread running a tight CPU loop pins its carrier thread just as a platform thread would, and the JVM's default carrier pool is sized to the number of CPU cores. Flooding it with compute-heavy virtual threads starves everything else scheduled on those carriers, including I/O-bound virtual threads that need to unmount and remount.

## Bad

```java
// Using a virtual-thread-per-task executor for CPU-bound image processing.
try (ExecutorService executor = Executors.newVirtualThreadPerTaskExecutor()) {
    for (Image image : images) {
        executor.submit(() -> {
            // Pure CPU work: resizing, filtering, no blocking I/O at all.
            // Thousands of these compete for the small, fixed carrier pool
            // (default = number of cores) with no benefit over platform threads,
            // and they can starve genuinely I/O-bound virtual threads sharing it.
            return applyFilters(resize(image));
        });
    }
}
```

## Good

```java
// A bounded pool sized to available cores for CPU-bound work.
private static final ExecutorService CPU_POOL =
        Executors.newFixedThreadPool(Runtime.getRuntime().availableProcessors());

List<ProcessedImage> processAll(List<Image> images) throws InterruptedException {
    List<Future<ProcessedImage>> futures = new ArrayList<>();
    for (Image image : images) {
        futures.add(CPU_POOL.submit(() -> applyFilters(resize(image))));
    }
    List<ProcessedImage> results = new ArrayList<>();
    for (Future<ProcessedImage> future : futures) {
        try {
            results.add(future.get());
        } catch (ExecutionException e) {
            throw new ImageProcessingException("Failed to process image", e.getCause());
        }
    }
    return results;
}
```

## Mixed Workloads

When a task does both CPU work and I/O, run it on a virtual thread but offload the CPU-heavy portion to a bounded platform-thread pool, then continue on the virtual thread:

```java
try (ExecutorService virtual = Executors.newVirtualThreadPerTaskExecutor()) {
    virtual.submit(() -> {
        byte[] raw = httpClient.fetchBytes(url);           // I/O: fine on virtual thread
        byte[] processed = CPU_POOL.submit(() -> compress(raw)).get(); // CPU: bounded pool
        database.store(processed);                          // I/O: fine on virtual thread
    });
}
```

## Rule of Thumb

If the task's dominant cost is waiting (network, disk, locks), use virtual threads. If the dominant cost is computing (parsing, encoding, math), use a platform-thread pool sized near `availableProcessors()`.

## See Also

- [`conc-virtual-threads-io`](conc-virtual-threads-io.md) - Use virtual threads for I/O-bound concurrent tasks
- [`conc-avoid-pinning`](conc-avoid-pinning.md) - Avoid virtual-thread pinning
- [`perf-avoid-reflection-hot-path`](perf-avoid-reflection-hot-path.md) - Other hot-path performance considerations
