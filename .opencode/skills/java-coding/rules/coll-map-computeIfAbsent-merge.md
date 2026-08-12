# coll-map-computeIfAbsent-merge

> Use `Map.computeIfAbsent`/`merge` for insert-or-update

## Why It Matters

The classic "check if a key exists, then either initialize or update" pattern requires two map lookups and is prone to race conditions under concurrent access. `computeIfAbsent`, `merge`, and `compute` perform the check-then-act as a single atomic operation (guaranteed atomic on `ConcurrentHashMap`, and simply more concise and correct on any `Map`).

## Bad

```java
// Two lookups: containsKey then get/put - also wrong under concurrent access
Map<String, List<Order>> ordersByCustomer = new HashMap<>();
for (Order order : orders) {
    String customerId = order.customer().id();
    if (!ordersByCustomer.containsKey(customerId)) {
        ordersByCustomer.put(customerId, new ArrayList<>());
    }
    ordersByCustomer.get(customerId).add(order);
}

// Manual null-check-then-initialize for a running total
Map<String, Integer> totalsByCategory = new HashMap<>();
for (Item item : items) {
    Integer current = totalsByCategory.get(item.category());
    if (current == null) {
        totalsByCategory.put(item.category(), item.quantity());
    } else {
        totalsByCategory.put(item.category(), current + item.quantity());
    }
}
```

## Good

```java
// computeIfAbsent initializes lazily, single atomic operation
Map<String, List<Order>> ordersByCustomer = new HashMap<>();
for (Order order : orders) {
    ordersByCustomer.computeIfAbsent(order.customer().id(), id -> new ArrayList<>())
        .add(order);
}

// merge combines insert-or-update in one call
Map<String, Integer> totalsByCategory = new HashMap<>();
for (Item item : items) {
    totalsByCategory.merge(item.category(), item.quantity(), Integer::sum);
}
```

## `compute` for Full Control

When the update logic needs to see both the key and the existing value (and potentially remove the entry), use `compute`:

```java
Map<String, Integer> retryCounts = new ConcurrentHashMap<>();

// Increment, but remove the entry entirely once it hits the limit
retryCounts.compute(taskId, (id, count) -> {
    int next = (count == null) ? 1 : count + 1;
    return next >= MAX_RETRIES ? null : next; // returning null removes the mapping
});
```

## Atomicity Under Concurrency

On `ConcurrentHashMap`, `computeIfAbsent`/`merge`/`compute` are guaranteed atomic per key - no other thread can observe a partially-updated state. This makes them the correct building block for concurrent caches and counters instead of external synchronization:

```java
Map<String, ExpensiveResource> cache = new ConcurrentHashMap<>();
ExpensiveResource resource = cache.computeIfAbsent(key, ExpensiveResource::loadFor);
```

## See Also

- [`coll-collectors-toX`](coll-collectors-toX.md) - `groupingBy` is the stream-pipeline equivalent of this insert-or-update pattern
- [`conc-concurrent-collections`](conc-concurrent-collections.md) - Why `ConcurrentHashMap` makes these operations safe under concurrency
- [`conc-atomic-over-lock`](conc-atomic-over-lock.md) - Preferring atomic operations over explicit locking generally
- [`coll-removeIf-over-iterator`](coll-removeIf-over-iterator.md) - Another single-call replacement for manual check-then-act loops
