# conc-atomic-over-lock

> Use atomic classes for simple counters instead of locks

## Why It Matters

`synchronized` blocks and explicit `Lock`s protect arbitrary critical sections, but for a single variable being incremented, compared, or swapped, `java.util.concurrent.atomic` classes (`AtomicInteger`, `AtomicLong`, `AtomicReference`, `LongAdder`) use lock-free compare-and-swap (CAS) operations that are faster under contention, never pin virtual threads, and cannot deadlock.

## Bad

```java
public class RequestCounter {
    private long count = 0;

    public synchronized void increment() {
        count++; // Needs a lock only because ++ is a read-modify-write,
                  // not because the value itself is complex
    }

    public synchronized long get() {
        return count;
    }
}

// Every increment now contends for one monitor, and on a virtual thread this
// synchronized block risks pinning if it ever grows to include more logic.
```

## Good

```java
public class RequestCounter {
    private final AtomicLong count = new AtomicLong();

    public void increment() {
        count.incrementAndGet(); // lock-free CAS loop, no monitor, no pinning
    }

    public long get() {
        return count.get();
    }
}
```

## High-Contention Counters: Prefer `LongAdder`

`AtomicLong` still funnels all threads through CAS retries on one memory location. Under very high contention (many threads incrementing the same counter constantly), `LongAdder` stripes the counter across multiple cells internally and sums them only when read, trading a little memory and read-time cost for much better write throughput:

```java
private final LongAdder requestsHandled = new LongAdder();

public void onRequestHandled() {
    requestsHandled.increment(); // scales better than AtomicLong under heavy contention
}

public long totalHandled() {
    return requestsHandled.sum(); // only pay the summation cost when reading
}
```

## Compound Updates With `AtomicReference`

```java
private final AtomicReference<ImmutableConfig> config =
        new AtomicReference<>(ImmutableConfig.defaults());

public void updateTimeout(Duration newTimeout) {
    // updateAndGet retries atomically if another thread updates concurrently,
    // without ever taking a lock.
    config.updateAndGet(current -> current.withTimeout(newTimeout));
}
```

## When to Reach for a Lock Instead

Atomics only cover a single variable's update. If you need to keep two or more fields consistent with each other atomically, a lock (or a lock-free immutable composite swapped via a single `AtomicReference`, as above) is the right tool — do not try to compose multiple atomics and expect the combination to be atomic.

## See Also

- [`conc-synchronized-scope`](conc-synchronized-scope.md) - Keep synchronized blocks minimal and scoped
- [`conc-avoid-pinning`](conc-avoid-pinning.md) - Avoid virtual-thread pinning
- [`conc-immutable-thread-safety`](conc-immutable-thread-safety.md) - Make shared objects immutable for thread safety
