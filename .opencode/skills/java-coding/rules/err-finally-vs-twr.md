# err-finally-vs-twr

> Prefer try-with-resources over manual `finally` cleanup

## Why It Matters

A `finally` block that manually closes several resources requires careful, often nested `try`/`catch` scaffolding to avoid leaking earlier resources if a later `close()` throws — and it is easy to write this incorrectly even when you know the pitfalls. Try-with-resources handles the correct close order, exception suppression, and null-safety for you, turning a multi-line error-prone pattern into a single, declarative resource list.

## Bad

```java
public void archiveOrders(Path source, Path archive) throws IOException {
    InputStream in = null;
    OutputStream out = null;
    try {
        in = Files.newInputStream(source);
        out = Files.newOutputStream(archive);
        in.transferTo(out);
    } finally {
        // Must null-check both, and if in.close() throws, out.close() never runs
        if (in != null) {
            in.close();
        }
        if (out != null) {
            out.close();
        }
    }
}
```

## Good

```java
public void archiveOrders(Path source, Path archive) throws IOException {
    try (InputStream in = Files.newInputStream(source);
         OutputStream out = Files.newOutputStream(archive)) {
        in.transferTo(out);
    } // both always closed, in reverse order, exceptions during close are suppressed correctly
}
```

## When `finally` Is Still The Right Tool

`finally` remains appropriate for cleanup that is not resource closing — releasing a lock acquired outside a `try`-with-resources-compatible API, restoring thread-local state, or logging regardless of outcome.

```java
Lock lock = striped.get(key);
lock.lock();
try {
    mutateSharedState(key);
} finally {
    lock.unlock(); // Lock is not AutoCloseable in java.util.concurrent.locks
}
```

## See Also

- [`err-try-with-resources`](err-try-with-resources.md) - Use try-with-resources for every `AutoCloseable`
- [`err-suppressed-exceptions`](err-suppressed-exceptions.md) - Preserve suppressed exceptions from try-with-resources
- [`conc-synchronized-scope`](conc-synchronized-scope.md) - Keep synchronized blocks minimal in scope
