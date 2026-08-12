# conc-concurrent-collections

> Use `java.util.concurrent` collections over manual synchronization

## Why It Matters

Wrapping a plain `HashMap` or `ArrayList` with `Collections.synchronizedMap`/`synchronizedList`, or guarding it by hand with `synchronized`, forces every access through one lock and is easy to get subtly wrong (iteration still needs external synchronization, compound operations like check-then-act are not atomic). The `java.util.concurrent` collections (`ConcurrentHashMap`, `CopyOnWriteArrayList`, `ConcurrentLinkedQueue`, `BlockingQueue` implementations) are built for concurrent access, offer atomic compound operations, and generally scale far better under contention.

## Bad

```java
private final Map<String, Session> sessions =
        Collections.synchronizedMap(new HashMap<>());

void touch(String sessionId) {
    // Compound "check then act" is NOT atomic even though the map is
    // "synchronized" -- another thread can interleave between get and put.
    Session session = sessions.get(sessionId);
    if (session == null) {
        sessions.put(sessionId, new Session(sessionId));
    } else {
        session.recordActivity();
    }
}

void printAll() {
    // Iterating a synchronizedMap without manually locking on it is unsafe --
    // a ConcurrentModificationException or worse can occur.
    for (Session session : sessions.values()) {
        System.out.println(session);
    }
}
```

## Good

```java
private final ConcurrentHashMap<String, Session> sessions = new ConcurrentHashMap<>();

void touch(String sessionId) {
    // Atomic: computeIfAbsent + the returned reference's own thread-safe method.
    sessions.computeIfAbsent(sessionId, Session::new).recordActivity();
}

void printAll() {
    // ConcurrentHashMap's iterators are weakly consistent: safe to iterate
    // without external locking, never throws ConcurrentModificationException.
    sessions.values().forEach(System.out::println);
}
```

## Choosing the Right Concurrent Collection

```java
// Frequent reads, rare writes, small collection: CopyOnWriteArrayList.
private final CopyOnWriteArrayList<Listener> listeners = new CopyOnWriteArrayList<>();

// Producer/consumer handoff between threads: a BlockingQueue implementation.
private final BlockingQueue<Task> queue = new LinkedBlockingQueue<>(1000);

void producer(Task task) throws InterruptedException {
    queue.put(task); // blocks if full, backpressure built in
}

void consumer() throws InterruptedException {
    Task task = queue.take(); // blocks until available
    process(task);
}

// High-throughput key-value with atomic compound operations: ConcurrentHashMap.
private final ConcurrentHashMap<String, AtomicLong> hitCounts = new ConcurrentHashMap<>();
```

## See Also

- [`conc-avoid-shared-mutable-state`](conc-avoid-shared-mutable-state.md) - Avoid shared mutable state; prefer immutability/confinement
- [`conc-atomic-over-lock`](conc-atomic-over-lock.md) - Use atomic classes for simple counters instead of locks
- [`coll-choose-right-collection`](coll-choose-right-collection.md) - General collection-choice guidance
- [`coll-avoid-legacy-classes`](coll-avoid-legacy-classes.md) - Avoiding legacy synchronized collection classes like Vector and Hashtable
