# err-try-with-resources

> Use try-with-resources for every `AutoCloseable`

## Why It Matters

Manually calling `close()` in a `finally` block is easy to get subtly wrong — an exception thrown while closing one resource can mask an exception from the try block, and forgetting the `finally` entirely leaks file handles, sockets, or database connections. Try-with-resources closes resources automatically in the correct order, even on exception, and correctly attaches close-time failures as suppressed exceptions rather than losing them.

## Bad

```java
public String readFirstLine(Path path) throws IOException {
    BufferedReader reader = Files.newBufferedReader(path);
    try {
        return reader.readLine();
    } finally {
        reader.close(); // if readLine() throws AND close() throws, the close() exception wins
    }                    // and the real root cause from readLine() is lost
}

public void copy(Path source, Path destination) throws IOException {
    InputStream in = Files.newInputStream(source);
    OutputStream out = Files.newOutputStream(destination);
    in.transferTo(out);
    in.close();  // never reached if transferTo throws - leaked file handles
    out.close();
}
```

## Good

```java
public String readFirstLine(Path path) throws IOException {
    try (BufferedReader reader = Files.newBufferedReader(path)) {
        return reader.readLine();
    } // reader.close() called automatically, exceptions properly suppressed, not swallowed
}

public void copy(Path source, Path destination) throws IOException {
    try (InputStream in = Files.newInputStream(source);
         OutputStream out = Files.newOutputStream(destination)) {
        in.transferTo(out);
    } // both closed in reverse declaration order, even if transferTo throws
}
```

## Java 9+ Effectively-Final Variables

```java
InputStream in = Files.newInputStream(source);
OutputStream out = Files.newOutputStream(destination);
// Reuse existing effectively-final variables directly in the resource list
try (in; out) {
    in.transferTo(out);
}
```

## Custom AutoCloseable Resources

```java
public class ConnectionLease implements AutoCloseable {
    private final Connection connection;

    public ConnectionLease(Connection connection) {
        this.connection = connection;
    }

    @Override
    public void close() {
        pool.release(connection); // return to pool, don't actually close the socket
    }
}

try (ConnectionLease lease = pool.acquire()) {
    lease.connection().execute(query);
}
```

## See Also

- [`err-finally-vs-twr`](err-finally-vs-twr.md) - Prefer try-with-resources over manual `finally` cleanup
- [`err-suppressed-exceptions`](err-suppressed-exceptions.md) - Preserve suppressed exceptions from try-with-resources
- [`conc-executorservice-shutdown`](conc-executorservice-shutdown.md) - Shut down executor services properly
