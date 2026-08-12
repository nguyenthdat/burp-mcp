# conc-synchronized-scope

> Keep `synchronized` blocks minimal and scoped

## Why It Matters

Every statement inside a `synchronized` block is a statement no other thread can execute concurrently in that monitor, and (as of Java 21) any `synchronized` block executed by a virtual thread pins it to its carrier thread for the block's entire duration. Wide `synchronized` blocks turn concurrent code sequential, create needless contention, and — for virtual threads — can starve the whole carrier pool. Keep the locked region to the smallest set of statements that actually need mutual exclusion.

## Bad

```java
public class OrderProcessor {
    private final Map<String, Order> orders = new HashMap<>();

    public synchronized void process(Order order) {
        // Everything is inside the lock, including slow, unrelated work:
        validate(order);                 // pure CPU, doesn't need the lock
        Pricing pricing = pricingService.lookup(order); // network call!
        orders.put(order.id(), order);   // the only part that actually needs it
        notificationService.send(order); // network call, definitely doesn't need it
    }
}

// A network hiccup in pricingService or notificationService now blocks every
// other thread trying to process any order, and pins a virtual thread caller.
```

## Good

```java
public class OrderProcessor {
    private final Map<String, Order> orders = new HashMap<>();
    private final Object lock = new Object();

    public void process(Order order) {
        validate(order);                                  // outside the lock
        Pricing pricing = pricingService.lookup(order);    // outside the lock

        synchronized (lock) {
            orders.put(order.id(), order); // only the shared-state mutation is locked
        }

        notificationService.send(order); // outside the lock
    }
}
```

## Prefer a Concurrent Collection Over a Custom Lock

Often the cleanest fix is to remove the `synchronized` block entirely by using a data structure designed for concurrent access:

```java
public class OrderProcessor {
    private final ConcurrentHashMap<String, Order> orders = new ConcurrentHashMap<>();

    public void process(Order order) {
        validate(order);
        Pricing pricing = pricingService.lookup(order);
        orders.put(order.id(), order); // already thread-safe, no lock needed
        notificationService.send(order);
    }
}
```

## Never Synchronize on `this` or a Public Field in a Library Class

Locking on `this` or a publicly reachable field lets unrelated code accidentally synchronize on the same monitor and create surprise contention or deadlocks. Use a private, dedicated lock object instead, as shown above.

## See Also

- [`conc-avoid-pinning`](conc-avoid-pinning.md) - Avoid virtual-thread pinning caused by synchronized
- [`conc-concurrent-collections`](conc-concurrent-collections.md) - Use java.util.concurrent collections over manual synchronization
- [`conc-atomic-over-lock`](conc-atomic-over-lock.md) - Use atomic classes for simple counters instead of locks
