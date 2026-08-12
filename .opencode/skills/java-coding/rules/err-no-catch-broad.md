# err-no-catch-broad

> Don't catch `Exception`/`Throwable` broadly

## Why It Matters

Catching `Exception` or `Throwable` swallows every failure indiscriminately — including bugs like `NullPointerException` or `ClassCastException` that should crash loudly during development, and even `Error` subtypes like `OutOfMemoryError` that a program generally cannot safely recover from. A broad catch turns "I don't know what went wrong" into "I'll pretend nothing went wrong," hiding real defects behind a generic log line.

## Bad

```java
public void processMessage(Message message) {
    try {
        validator.validate(message);
        handler.handle(message);
        repository.save(message);
    } catch (Exception e) {
        // Catches validation errors, DB errors, AND NullPointerException bugs
        // AND OutOfMemoryError-adjacent RuntimeExceptions - all treated the same
        log.error("failed to process message", e);
    }
}
```

## Good

```java
public void processMessage(Message message) {
    try {
        validator.validate(message);
        handler.handle(message);
        repository.save(message);
    } catch (ValidationException e) {
        log.warn("rejected invalid message {}: {}", message.id(), e.getMessage());
        deadLetterQueue.send(message, e);
    } catch (DataAccessException e) {
        log.error("failed to persist message {}", message.id(), e);
        throw new MessageProcessingException("could not save message " + message.id(), e);
    }
    // NullPointerException, IllegalStateException etc. propagate and surface as real bugs
}
```

## When A Broad Catch Is Justified

```java
// Top-level boundary: a request dispatcher or job runner that must never crash
// the whole process, and reports/logs unexpected failures instead of hiding them.
public void runJob(Job job) {
    try {
        job.execute();
    } catch (Exception e) {
        // Acceptable ONLY at this outermost boundary, and only because we
        // re-throw as a failure signal rather than silently continuing.
        metrics.increment("job.failed");
        log.error("job {} failed unexpectedly", job.id(), e);
        job.markFailed(e);
    }
    // Still never catch Throwable/Error here - let OutOfMemoryError etc. propagate
}
```

## See Also

- [`err-specific-catch-order`](err-specific-catch-order.md) - Catch specific exceptions before general ones
- [`err-no-empty-catch`](err-no-empty-catch.md) - Never swallow exceptions silently
- [`anti-catch-and-ignore`](anti-catch-and-ignore.md) - Don't catch and ignore exceptions
- [`err-custom-exception-hierarchy`](err-custom-exception-hierarchy.md) - Build a custom exception hierarchy
