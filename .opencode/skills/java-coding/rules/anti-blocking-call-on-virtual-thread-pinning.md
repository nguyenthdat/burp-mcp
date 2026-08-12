# anti-blocking-call-on-virtual-thread-pinning

> Don't hold `synchronized` around blocking calls on virtual threads

## Why It Matters

Virtual threads normally unmount from their carrier platform thread while blocked on I/O, letting the carrier run other virtual threads. But when a blocking call happens inside a `synchronized` block or method, the virtual thread stays pinned to its carrier for the entire block - the carrier can't be reused, so under load a small pool of carrier threads gets stuck waiting and throughput collapses back to platform-thread-like scalability.

## Bad

```java
public class SessionCache {
  private final Map<String, Session> cache = new HashMap<>();

  public synchronized Session getOrLoad(String sessionId) {
    Session session = cache.get(sessionId);
    if (session == null) {
      session = database.loadSession(sessionId); // Blocking JDBC call while holding the monitor
      // The virtual thread is PINNED here - its carrier thread cannot
      // run any other virtual thread until this blocking call returns
      cache.put(sessionId, session);
    }
    return session;
  }
}

// Under 10,000 concurrent virtual threads, all funneled through this
// synchronized method, the small carrier pool saturates and the whole
// benefit of virtual threads disappears
```

## Good

```java
public class SessionCache {
  private final ConcurrentHashMap<String, Session> cache = new ConcurrentHashMap<>();

  public Session getOrLoad(String sessionId) {
    // computeIfAbsent locks only per-key internally and briefly, not the
    // whole map, and does not use `synchronized` around the blocking call
    return cache.computeIfAbsent(sessionId, id -> database.loadSession(id));
  }
}
```

```java
// If a real mutual-exclusion lock is unavoidable around blocking work,
// use ReentrantLock instead of synchronized - JEP 444 explicitly notes
// java.util.concurrent locks do NOT pin the carrier thread.
public class SessionCache {
  private final Lock lock = new ReentrantLock();
  private final Map<String, Session> cache = new HashMap<>();

  public Session getOrLoad(String sessionId) {
    lock.lock();
    try {
      return cache.computeIfAbsent(sessionId, database::loadSession);
    } finally {
      lock.unlock();
    }
  }
}
```

## Detecting Pinning

```bash
# JDK flag to log every pinning event with a stack trace
java -Djdk.tracePinnedThreads=full -jar app.jar

# Output points directly at the synchronized block causing the pin:
# Thread[#31,ForkJoinPool-1-worker-3,5,main]
#     java.base/java.lang.VirtualThread$VThreadContinuation.onPinned(VirtualThread.java:183)
#     <-- app.SessionCache.getOrLoad(SessionCache.java:12) <== monitor
```

## See Also

- [`conc-avoid-pinning`](conc-avoid-pinning.md) - The positive rule this anti-pattern violates
- [`conc-virtual-threads-io`](conc-virtual-threads-io.md) - When virtual threads are the right tool in the first place
- [`conc-atomic-over-lock`](conc-atomic-over-lock.md) - Preferring atomics/j.u.c locks over `synchronized` in virtual-thread code
- [`conc-synchronized-scope`](conc-synchronized-scope.md) - Keeping any remaining `synchronized` blocks as narrow as possible
