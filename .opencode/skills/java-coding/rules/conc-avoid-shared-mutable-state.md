# conc-avoid-shared-mutable-state

> Avoid shared mutable state; prefer immutability/confinement

## Why It Matters

Every data race, visibility bug, and heisenbug in concurrent Java traces back to mutable state reachable from more than one thread without a coordinated access strategy. Immutability removes the problem at the source — an object nothing can mutate needs no synchronization — and thread confinement (each unit of mutable state owned by exactly one thread) is the next cheapest option, especially now that virtual threads make "one thread per task" affordable.

## Bad

```java
// A shared mutable map, accessed by many concurrent request-handling threads.
public class RequestStats {
    private final Map<String, Integer> countsByEndpoint = new HashMap<>();

    public void record(String endpoint) {
        // BAD: unsynchronized read-modify-write on a plain HashMap from
        // multiple threads -- lost updates, and HashMap can even corrupt
        // its internal structure under concurrent mutation.
        Integer current = countsByEndpoint.get(endpoint);
        countsByEndpoint.put(endpoint, (current == null ? 0 : current) + 1);
    }

    public Map<String, Integer> snapshot() {
        return countsByEndpoint; // Leaks the live, still-mutable map
    }
}
```

## Good

```java
public class RequestStats {
    private final ConcurrentHashMap<String, LongAdder> countsByEndpoint =
            new ConcurrentHashMap<>();

    public void record(String endpoint) {
        countsByEndpoint.computeIfAbsent(endpoint, k -> new LongAdder()).increment();
    }

    public Map<String, Long> snapshot() {
        // Return an immutable copy; callers can never observe or cause
        // partial updates, and the internal structure stays encapsulated.
        Map<String, Long> copy = new HashMap<>();
        countsByEndpoint.forEach((endpoint, adder) -> copy.put(endpoint, adder.sum()));
        return Collections.unmodifiableMap(copy);
    }
}
```

## Confinement as an Alternative to Sharing

When state genuinely needs to be mutable and complex, prefer confining it to a single thread (or a single virtual thread per unit of work) and communicating results via immutable messages or `CompletableFuture`, rather than exposing it for concurrent mutation:

```java
// Each request gets its own mutable accumulator, confined to its own virtual
// thread; no synchronization needed because nothing else can see it.
try (ExecutorService executor = Executors.newVirtualThreadPerTaskExecutor()) {
    for (Request request : requests) {
        executor.submit(() -> {
            List<String> localAccumulator = new ArrayList<>(); // thread-confined
            processInto(request, localAccumulator);
            publish(Collections.unmodifiableList(localAccumulator)); // hand off immutably
        });
    }
}
```

## See Also

- [`conc-immutable-thread-safety`](conc-immutable-thread-safety.md) - Make shared objects immutable for thread safety
- [`conc-concurrent-collections`](conc-concurrent-collections.md) - Use java.util.concurrent collections over manual synchronization
- [`conc-atomic-over-lock`](conc-atomic-over-lock.md) - Use atomic classes for simple counters instead of locks
- [`api-immutable-by-default`](api-immutable-by-default.md) - Designing types to be immutable by default
