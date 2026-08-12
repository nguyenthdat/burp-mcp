# coll-unmodifiable-view

> Wrap mutable collections with unmodifiable views at boundaries

## Why It Matters

When an internal mutable collection must be exposed - for example a cache or a builder's accumulated list - returning it directly lets any caller mutate internal state without going through the class's invariants. `Collections.unmodifiableList`/`unmodifiableMap`/`unmodifiableSet` (or `List.copyOf` for a true snapshot) close that hole at the API boundary while avoiding the cost of a full defensive copy on every access when a live view is acceptable.

## Bad

```java
public class OrderBook {
    private final List<Order> pendingOrders = new ArrayList<>();

    public void submit(Order order) {
        pendingOrders.add(order);
    }

    // Returns the live, mutable internal list directly
    public List<Order> getPendingOrders() {
        return pendingOrders;
    }
}

// Caller can corrupt internal state without going through submit()
OrderBook book = new OrderBook();
book.getPendingOrders().clear(); // wipes internal state from the outside
book.getPendingOrders().add(forgedOrder); // bypasses any validation in submit()
```

## Good

```java
public class OrderBook {
    private final List<Order> pendingOrders = new ArrayList<>();

    public void submit(Order order) {
        pendingOrders.add(order);
    }

    // Unmodifiable view - reflects future changes, rejects mutation attempts
    public List<Order> getPendingOrders() {
        return Collections.unmodifiableList(pendingOrders);
    }
}

// Caller sees the live data but any mutation attempt throws immediately
OrderBook book = new OrderBook();
book.getPendingOrders().add(forgedOrder); // throws UnsupportedOperationException
```

## Live View vs Snapshot

An unmodifiable *view* still reflects subsequent changes to the backing collection, which is usually desirable but can surprise callers who expect a stable snapshot. Use `List.copyOf`/`Set.copyOf`/`Map.copyOf` when the caller needs data frozen at the moment of the call:

```java
// View - caller sees updates made after this call returns
public List<Order> getPendingOrdersView() {
    return Collections.unmodifiableList(pendingOrders);
}

// Snapshot - caller gets a point-in-time copy, immune to later mutation
public List<Order> getPendingOrdersSnapshot() {
    return List.copyOf(pendingOrders);
}
```

## See Also

- [`coll-immutable-factories`](coll-immutable-factories.md) - Preferring immutable collections from the start where possible
- [`null-defensive-copy`](null-defensive-copy.md) - Defensive copying for mutable fields more broadly
- [`api-defensive-copy-mutable-args`](api-defensive-copy-mutable-args.md) - The constructor-side counterpart of this rule
- [`api-minimal-public-surface`](api-minimal-public-surface.md) - Broader principle of not leaking internal mutability
