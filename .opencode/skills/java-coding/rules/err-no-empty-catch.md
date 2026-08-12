# err-no-empty-catch

> Never swallow exceptions silently

## Why It Matters

An empty `catch` block destroys evidence of a failure — the program keeps running as if nothing happened, but some operation didn't complete, some resource wasn't released, or some data wasn't saved, and there is no record anywhere that it happened. When the eventual symptom shows up (missing data, a stuck job, a corrupted state), there is no log entry connecting it back to the root cause, turning a five-minute fix into a multi-day investigation.

## Bad

```java
public void syncInventory(Warehouse warehouse) {
    try {
        inventoryClient.push(warehouse.currentStock());
    } catch (IOException e) {
        // Silently swallowed - stock sync just quietly stops working with zero trace
    }
}

public void closeQuietly(Closeable resource) {
    try {
        resource.close();
    } catch (Exception ignored) {
        // Even the "ignored" naming convention doesn't excuse losing the failure entirely
    }
}
```

## Good

```java
public void syncInventory(Warehouse warehouse) {
    try {
        inventoryClient.push(warehouse.currentStock());
    } catch (IOException e) {
        log.error("failed to sync inventory for warehouse {}", warehouse.id(), e);
        metrics.increment("inventory.sync.failure");
        retryQueue.schedule(warehouse);
    }
}

public void closeQuietly(Closeable resource) {
    try {
        resource.close();
    } catch (IOException e) {
        // Even "best effort" cleanup should leave a trace at debug level
        log.debug("failed to close resource {}, continuing", resource, e);
    }
}
```

## The Rare Legitimate No-Op

```java
try {
    Thread.sleep(retryDelay);
} catch (InterruptedException e) {
    // Restoring the interrupt flag IS the correct handling here - not empty,
    // just doesn't log, because propagating interruption is the whole point.
    Thread.currentThread().interrupt();
}
```

Even in this case, the catch block does something meaningful (restoring interrupt status); a truly empty block with no comment and no action is never acceptable.

## See Also

- [`anti-catch-and-ignore`](anti-catch-and-ignore.md) - Don't catch and ignore exceptions
- [`err-no-catch-broad`](err-no-catch-broad.md) - Don't catch `Exception`/`Throwable` broadly
- [`err-exception-message-context`](err-exception-message-context.md) - Include actionable context in exception messages
