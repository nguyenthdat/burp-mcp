# conc-executorservice-shutdown

> Always shut down an `ExecutorService` properly

## Why It Matters

An `ExecutorService` that is never shut down keeps its threads alive indefinitely, preventing the JVM from exiting cleanly and leaking resources for the lifetime of the process. Since Java 19, `ExecutorService` implements `AutoCloseable`, so try-with-resources is now the simplest correct way to guarantee shutdown, including waiting for in-flight tasks and handling interruption.

## Bad

```java
void processAll(List<Task> tasks) {
    ExecutorService executor = Executors.newFixedThreadPool(8);
    for (Task task : tasks) {
        executor.submit(() -> process(task));
    }
    // No shutdown call at all: the pool's threads live forever, and the JVM
    // will never exit on its own (non-daemon threads keep it alive).
}

void processAllAttempt2(List<Task> tasks) {
    ExecutorService executor = Executors.newFixedThreadPool(8);
    for (Task task : tasks) {
        executor.submit(() -> process(task));
    }
    executor.shutdown();
    // BAD: shutdown() only stops accepting new tasks; it does not wait for
    // submitted ones to finish, so this method can return before work is done.
}
```

## Good

```java
void processAll(List<Task> tasks) throws InterruptedException {
    try (ExecutorService executor = Executors.newFixedThreadPool(8)) {
        for (Task task : tasks) {
            executor.submit(() -> process(task));
        }
    } // close() calls shutdown() then awaits termination, escalating to
      // shutdownNow() if interrupted -- guaranteed cleanup either way
}
```

## Pre-Java-19 Idiom (No `AutoCloseable`)

If targeting an older JDK, shut down explicitly with a bounded wait and a forceful fallback:

```java
void processAll(List<Task> tasks) throws InterruptedException {
    ExecutorService executor = Executors.newFixedThreadPool(8);
    try {
        for (Task task : tasks) {
            executor.submit(() -> process(task));
        }
    } finally {
        executor.shutdown();
        try {
            if (!executor.awaitTermination(30, TimeUnit.SECONDS)) {
                executor.shutdownNow(); // cancel in-flight tasks after the timeout
            }
        } catch (InterruptedException e) {
            executor.shutdownNow();
            Thread.currentThread().interrupt(); // restore interrupt status
            throw e;
        }
    }
}
```

## Application-Scoped Executors

For an executor meant to live for the whole application (e.g., a shared thread pool held as a static field), register a JVM shutdown hook or tie its lifecycle to your framework's container shutdown callback rather than relying on try-with-resources, since there is no single enclosing block to attach it to.

## See Also

- [`conc-executors-newVirtualThreadPerTask`](conc-executors-newVirtualThreadPerTask.md) - Use the virtual-thread-per-task executor factory
- [`err-try-with-resources`](err-try-with-resources.md) - General try-with-resources guidance
- [`conc-structured-concurrency`](conc-structured-concurrency.md) - StructuredTaskScope guarantees subtask cleanup automatically
