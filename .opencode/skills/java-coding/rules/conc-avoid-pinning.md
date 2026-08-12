# conc-avoid-pinning

> Avoid virtual-thread pinning (`synchronized`, native frames)

## Why It Matters

A virtual thread that blocks inside a `synchronized` block or method, or while executing a native frame (JNI), cannot unmount from its carrier platform thread — it "pins" the carrier for the duration of the block. Since the carrier pool is small (by default, equal to the number of CPU cores), enough pinned virtual threads can exhaust it, stalling every other virtual thread in the application even though nothing looks deadlocked.

## Bad

```java
public class SessionCache {
    private final Map<String, Session> cache = new HashMap<>();

    public synchronized Session getOrLoad(String id) {
        Session session = cache.get(id);
        if (session == null) {
            // BAD: a blocking network/database call inside a synchronized
            // method. On a virtual thread, this pins the carrier for the
            // entire remote call, not just for the map access.
            session = sessionStore.loadFromDatabase(id); // blocking I/O
            cache.put(id, session);
        }
        return session;
    }
}
```

## Good

```java
public class SessionCache {
    private final ConcurrentHashMap<String, Session> cache = new ConcurrentHashMap<>();

    public Session getOrLoad(String id) {
        // No synchronized block at all: ConcurrentHashMap.computeIfAbsent
        // handles the concurrency, and the blocking call can freely unmount
        // the virtual thread from its carrier while it waits.
        return cache.computeIfAbsent(id, sessionStore::loadFromDatabase);
    }
}
```

## When You Must Use a Monitor

If you cannot avoid `synchronized` (e.g., interacting with legacy code that requires it), replace it with a `java.util.concurrent.locks.ReentrantLock`, which virtual threads can release and re-acquire around blocking calls without pinning:

```java
private final ReentrantLock lock = new ReentrantLock();

public Session getOrLoad(String id) {
    lock.lock();
    try {
        Session session = cache.get(id);
        if (session == null) {
            session = sessionStore.loadFromDatabase(id); // does not pin the carrier
            cache.put(id, session);
        }
        return session;
    } finally {
        lock.unlock();
    }
}
```

## Detecting Pinning

Run with `-Djdk.tracePinnedThreads=full` (or `short`) during development to have the JVM print a stack trace whenever a virtual thread parks while pinned, so pinning hotspots surface before they cause production incidents. Note that a short, CPU-only `synchronized` block (no blocking calls inside) is generally fine — pinning only becomes a problem when the pinned thread also blocks.

## See Also

- [`conc-synchronized-scope`](conc-synchronized-scope.md) - Keep synchronized blocks minimal and scoped
- [`conc-virtual-threads-io`](conc-virtual-threads-io.md) - Use virtual threads for I/O-bound concurrent tasks
- [`anti-blocking-call-on-virtual-thread-pinning`](anti-blocking-call-on-virtual-thread-pinning.md) - Anti-pattern reference
- [`conc-atomic-over-lock`](conc-atomic-over-lock.md) - Use atomic classes for simple counters instead of locks
