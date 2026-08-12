# coll-collectors-toX

> Use the right `Collectors` (`toList`, `toMap`, `groupingBy`, `joining`)

## Why It Matters

The `Collectors` utility class has a purpose-built terminal operation for nearly every common aggregation shape. Reaching for a generic `reduce` or hand-rolled accumulation when `Collectors.groupingBy`, `toMap`, or `joining` already expresses the intent produces more code, loses built-in duplicate-key/merge handling, and is easier to get wrong (especially around mutability and merge conflicts).

## Bad

```java
// Manual grouping instead of Collectors.groupingBy
Map<OrderStatus, List<Order>> byStatus = new HashMap<>();
for (Order order : orders) {
    byStatus.computeIfAbsent(order.status(), k -> new ArrayList<>()).add(order);
}

// Manual string joining instead of Collectors.joining
StringBuilder sb = new StringBuilder();
for (int i = 0; i < names.size(); i++) {
    sb.append(names.get(i));
    if (i < names.size() - 1) {
        sb.append(", ");
    }
}
String joined = sb.toString();

// Building a Map by hand, no duplicate-key handling
Map<String, Order> latestByCustomer = new HashMap<>();
for (Order order : orders) {
    latestByCustomer.put(order.customer().id(), order); // silently overwrites
}
```

## Good

```java
// groupingBy handles the accumulation for you
Map<OrderStatus, List<Order>> byStatus = orders.stream()
    .collect(Collectors.groupingBy(Order::status));

// joining handles separators, prefix/suffix cleanly
String joined = names.stream()
    .collect(Collectors.joining(", "));

// toMap with an explicit merge function makes duplicate-key behavior intentional
Map<String, Order> latestByCustomer = orders.stream()
    .collect(Collectors.toMap(
        order -> order.customer().id(),
        Function.identity(),
        (existing, replacement) -> replacement.placedAt().isAfter(existing.placedAt())
            ? replacement
            : existing));
```

## Common Collectors Cheat Sheet

```java
// Simple list/set
List<String> names = orders.stream().map(Order::customerName).toList();
Set<OrderStatus> statuses = orders.stream().map(Order::status).collect(Collectors.toSet());

// Grouping with a downstream collector (count per group)
Map<OrderStatus, Long> countByStatus = orders.stream()
    .collect(Collectors.groupingBy(Order::status, Collectors.counting()));

// Partitioning into exactly two groups
Map<Boolean, List<Order>> partitioned = orders.stream()
    .collect(Collectors.partitioningBy(o -> o.total().compareTo(BigDecimal.valueOf(100)) > 0));

// Summing a numeric field
BigDecimal total = orders.stream()
    .collect(Collectors.reducing(BigDecimal.ZERO, Order::total, BigDecimal::add));
```

## `toList()` vs `Collectors.toList()`

Prefer `Stream.toList()` (Java 16+) for a quick immutable result; use `Collectors.toList()` only when you specifically need a mutable, modifiable `ArrayList` result or need to feed it into a collector composition API that expects a `Collector`.

## See Also

- [`coll-map-computeIfAbsent-merge`](coll-map-computeIfAbsent-merge.md) - Insert-or-update patterns outside of stream pipelines
- [`coll-stream-for-transformation`](coll-stream-for-transformation.md) - Building the pipeline that feeds into these collectors
- [`coll-avoid-side-effects-streams`](coll-avoid-side-effects-streams.md) - Why collecting beats manual accumulation
- [`coll-immutable-factories`](coll-immutable-factories.md) - Producing immutable results from a stream
