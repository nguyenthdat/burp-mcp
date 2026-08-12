# coll-removeIf-over-iterator

> Use `removeIf` instead of manual iterator removal

## Why It Matters

`Collection.removeIf` (Java 8+) expresses conditional removal as a single predicate instead of a manual `Iterator`/`while` loop, and it eliminates the classic `ConcurrentModificationException` bug caused by calling `collection.remove()` while iterating with a for-each loop instead of the iterator's own `remove()` method.

## Bad

```java
// Classic bug: modifying the list via a for-each loop throws ConcurrentModificationException
List<Order> orders = loadOrders();
for (Order order : orders) {
    if (order.isCancelled()) {
        orders.remove(order); // throws ConcurrentModificationException
    }
}

// Correct but verbose manual iterator removal
Iterator<Order> it = orders.iterator();
while (it.hasNext()) {
    Order order = it.next();
    if (order.isCancelled()) {
        it.remove();
    }
}
```

## Good

```java
// Single expression, no iterator boilerplate, no CME risk
List<Order> orders = loadOrders();
orders.removeIf(Order::isCancelled);
```

## Works Across the Collections Framework

`removeIf` is defined on `Collection`, so it applies uniformly to lists, sets, and any custom collection implementation:

```java
Set<String> tags = loadTags();
tags.removeIf(tag -> tag.isBlank());

// Combine with a compound predicate for readability
orders.removeIf(order -> order.isCancelled() || order.total().signum() == 0);
```

## Maps Need `entrySet().removeIf`

`Map` doesn't implement `Collection` directly, so conditional removal goes through its entry set view:

```java
Map<String, Integer> inventory = loadInventory();
inventory.entrySet().removeIf(entry -> entry.getValue() == 0);
```

## When to Still Filter Instead

If the goal is to produce a new filtered result rather than mutate in place - especially when the source collection is immutable or shared - prefer a stream filter into a new collection instead of `removeIf`, which mutates the receiver:

```java
List<Order> active = orders.stream()
    .filter(order -> !order.isCancelled())
    .toList();
```

## See Also

- [`coll-stream-for-transformation`](coll-stream-for-transformation.md) - Filtering into a new collection instead of mutating in place
- [`coll-map-computeIfAbsent-merge`](coll-map-computeIfAbsent-merge.md) - Other single-method Map/Collection operations that replace manual loops
- [`err-no-control-flow`](err-no-control-flow.md) - Related concern about relying on exceptions for normal control flow
- [`coll-immutable-factories`](coll-immutable-factories.md) - `removeIf` throws on immutable collections by design
