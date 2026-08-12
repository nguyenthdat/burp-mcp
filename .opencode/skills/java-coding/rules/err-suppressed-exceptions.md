# err-suppressed-exceptions

> Preserve suppressed exceptions from try-with-resources

## Why It Matters

When both the body of a try-with-resources block and the automatic `close()` call throw, Java attaches the `close()` failure as a *suppressed* exception on the primary one rather than discarding it — but that information is only visible if you actually log or inspect `getSuppressed()`. Code that manually catches and rethrows without preserving this chain, or that logs only `e.getMessage()`, silently throws away evidence of a second, real failure (for example, a transaction that also failed to roll back).

## Bad

```java
public void writeReport(Path path, Report report) {
    try (Writer writer = Files.newBufferedWriter(path)) {
        writer.write(report.render());
    } catch (IOException e) {
        // Only the primary exception's message is logged - if close() also failed
        // (e.g. disk full flushing the buffer), that suppressed exception is invisible here
        log.error("failed to write report: " + e.getMessage());
    }
}
```

## Good

```java
public void writeReport(Path path, Report report) {
    try (Writer writer = Files.newBufferedWriter(path)) {
        writer.write(report.render());
    } catch (IOException e) {
        log.error("failed to write report to {}", path, e); // logs full trace + suppressed exceptions
        for (Throwable suppressed : e.getSuppressed()) {
            log.error("  suppressed during resource close: {}", suppressed.toString());
        }
        throw new ReportWriteException("could not write report to " + path, e);
    }
}
```

## Why Standard Logging Frameworks Already Help

Most logging frameworks (SLF4J + Logback, java.util.logging) print suppressed exceptions automatically when you log the `Throwable` object itself (as in `log.error("...", e)`) rather than just `e.getMessage()` — this is the main reason to always pass the exception object, not a pre-formatted string, to your logger.

```java
// Bad: throws away the stack trace and any suppressed exceptions
log.error("write failed: " + e.getMessage());

// Good: the logging framework renders the full trace, causes, and suppressed exceptions
log.error("write failed", e);
```

## See Also

- [`err-try-with-resources`](err-try-with-resources.md) - Use try-with-resources for every `AutoCloseable`
- [`err-exception-chaining`](err-exception-chaining.md) - Chain causes via constructor
- [`err-finally-vs-twr`](err-finally-vs-twr.md) - Prefer try-with-resources over manual `finally` cleanup
